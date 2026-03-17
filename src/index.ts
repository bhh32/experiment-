#!/usr/bin/env node

/**
 * Codeberg MCP Server
 *
 * An MCP server that provides tools for interacting with Codeberg
 * (Gitea/Forgejo API) — repos, issues, pull requests, files, and more.
 *
 * Configuration via environment variables:
 *   CODEBERG_TOKEN  – API token (required)
 *   CODEBERG_URL    – Base URL (default: https://codeberg.org)
 */

import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { z } from "zod";
import { CodebergClient } from "./api.js";

// ── Bootstrap ─────────────────────────────────────────────────────────
const token = process.env.CODEBERG_TOKEN;
if (!token) {
  console.error("Error: CODEBERG_TOKEN environment variable is required.");
  process.exit(1);
}

const client = new CodebergClient({
  token,
  baseUrl: process.env.CODEBERG_URL,
});

const server = new McpServer({
  name: "codeberg",
  version: "1.0.0",
});

// ── Helpers ───────────────────────────────────────────────────────────
function json(data: unknown): { content: Array<{ type: "text"; text: string }> } {
  return { content: [{ type: "text", text: JSON.stringify(data, null, 2) }] };
}

const ownerRepo = {
  owner: z.string().describe("Repository owner (user or org)"),
  repo: z.string().describe("Repository name"),
};

const pagination = {
  page: z.number().optional().describe("Page number (default 1)"),
  limit: z.number().optional().describe("Items per page (default 20, max 50)"),
};

// ═══════════════════════════════════════════════════════════════════════
// USER TOOLS
// ═══════════════════════════════════════════════════════════════════════

server.tool(
  "get_authenticated_user",
  "Get the currently authenticated Codeberg user",
  {},
  async () => json(await client.getAuthenticatedUser()),
);

server.tool(
  "get_user",
  "Get a Codeberg user's public profile",
  { username: z.string().describe("Username to look up") },
  async ({ username }) => json(await client.getUser(username)),
);

// ═══════════════════════════════════════════════════════════════════════
// REPOSITORY TOOLS
// ═══════════════════════════════════════════════════════════════════════

server.tool(
  "list_my_repos",
  "List repositories owned by the authenticated user",
  pagination,
  async (args) => json(await client.listMyRepos(args)),
);

server.tool(
  "list_user_repos",
  "List public repositories for a given user",
  {
    owner: z.string().describe("Username"),
    ...pagination,
  },
  async ({ owner, ...rest }) => json(await client.listUserRepos(owner, rest)),
);

server.tool(
  "list_org_repos",
  "List repositories belonging to an organization",
  {
    org: z.string().describe("Organization name"),
    ...pagination,
  },
  async ({ org, ...rest }) => json(await client.listOrgRepos(org, rest)),
);

server.tool(
  "get_repo",
  "Get detailed information about a repository",
  ownerRepo,
  async ({ owner, repo }) => json(await client.getRepo(owner, repo)),
);

server.tool(
  "create_repo",
  "Create a new repository for the authenticated user",
  {
    name: z.string().describe("Repository name"),
    description: z.string().optional().describe("Repository description"),
    private: z.boolean().optional().describe("Whether repo is private"),
    auto_init: z.boolean().optional().describe("Initialize with README"),
    default_branch: z.string().optional().describe("Default branch name"),
    license: z.string().optional().describe("License template (e.g. MIT)"),
    gitignores: z.string().optional().describe("Gitignore template(s)"),
  },
  async (args) => json(await client.createRepo(args)),
);

server.tool(
  "fork_repo",
  "Fork a repository",
  {
    ...ownerRepo,
    name: z.string().optional().describe("Name for the fork"),
    organization: z.string().optional().describe("Fork to this org instead of your account"),
  },
  async ({ owner, repo, ...rest }) =>
    json(await client.forkRepo(owner, repo, rest)),
);

// ═══════════════════════════════════════════════════════════════════════
// BRANCH TOOLS
// ═══════════════════════════════════════════════════════════════════════

server.tool(
  "list_branches",
  "List branches of a repository",
  { ...ownerRepo, ...pagination },
  async ({ owner, repo, ...rest }) =>
    json(await client.listBranches(owner, repo, rest)),
);

server.tool(
  "create_branch",
  "Create a new branch in a repository",
  {
    ...ownerRepo,
    new_branch_name: z.string().describe("Name for the new branch"),
    old_branch_name: z.string().optional().describe("Branch to create from (default: repo default branch)"),
  },
  async ({ owner, repo, ...rest }) =>
    json(await client.createBranch(owner, repo, rest)),
);

// ═══════════════════════════════════════════════════════════════════════
// FILE CONTENT TOOLS
// ═══════════════════════════════════════════════════════════════════════

server.tool(
  "get_file_contents",
  "Get the contents of a file or directory in a repository",
  {
    ...ownerRepo,
    path: z.string().describe("Path to file or directory"),
    ref: z.string().optional().describe("Branch, tag, or commit SHA"),
  },
  async ({ owner, repo, path, ref }) => {
    const result = await client.getFileContents(owner, repo, path, ref);
    // Decode base64 content for files
    if (result.content && result.encoding === "base64") {
      const decoded = Buffer.from(result.content, "base64").toString("utf-8");
      return {
        content: [
          { type: "text" as const, text: `File: ${result.path}\nSize: ${result.size} bytes\n\n${decoded}` },
        ],
      };
    }
    return json(result);
  },
);

server.tool(
  "create_or_update_file",
  "Create or update a file in a repository",
  {
    ...ownerRepo,
    path: z.string().describe("File path in the repository"),
    content: z.string().describe("File content (will be base64 encoded automatically)"),
    message: z.string().describe("Commit message"),
    branch: z.string().optional().describe("Branch to commit to"),
    sha: z.string().optional().describe("SHA of file being replaced (required for updates)"),
    new_branch: z.string().optional().describe("Create this new branch for the commit"),
  },
  async ({ owner, repo, path, content, ...rest }) => {
    const encoded = Buffer.from(content).toString("base64");
    return json(
      await client.createOrUpdateFile(owner, repo, path, {
        ...rest,
        content: encoded,
      }),
    );
  },
);

server.tool(
  "delete_file",
  "Delete a file from a repository",
  {
    ...ownerRepo,
    path: z.string().describe("File path to delete"),
    message: z.string().describe("Commit message"),
    sha: z.string().describe("SHA of the file to delete"),
    branch: z.string().optional().describe("Branch to delete from"),
    new_branch: z.string().optional().describe("Create this new branch for the deletion"),
  },
  async ({ owner, repo, path, ...rest }) =>
    json(await client.deleteFile(owner, repo, path, rest)),
);

// ═══════════════════════════════════════════════════════════════════════
// ISSUE TOOLS
// ═══════════════════════════════════════════════════════════════════════

server.tool(
  "list_issues",
  "List issues in a repository",
  {
    ...ownerRepo,
    state: z.enum(["open", "closed", "all"]).optional().describe("Filter by state"),
    labels: z.string().optional().describe("Comma-separated label names"),
    assignee: z.string().optional().describe("Filter by assignee username"),
    type: z.enum(["issues", "pulls"]).optional().describe("Filter: issues only or pulls only"),
    milestones: z.string().optional().describe("Comma-separated milestone names"),
    ...pagination,
  },
  async ({ owner, repo, ...rest }) =>
    json(await client.listIssues(owner, repo, rest)),
);

server.tool(
  "get_issue",
  "Get details of a specific issue",
  {
    ...ownerRepo,
    issue_number: z.number().describe("Issue number"),
  },
  async ({ owner, repo, issue_number }) =>
    json(await client.getIssue(owner, repo, issue_number)),
);

server.tool(
  "create_issue",
  "Create a new issue in a repository",
  {
    ...ownerRepo,
    title: z.string().describe("Issue title"),
    body: z.string().optional().describe("Issue body (markdown)"),
    assignees: z.array(z.string()).optional().describe("Usernames to assign"),
    labels: z.array(z.number()).optional().describe("Label IDs to apply"),
    milestone: z.number().optional().describe("Milestone ID"),
  },
  async ({ owner, repo, ...rest }) =>
    json(await client.createIssue(owner, repo, rest)),
);

server.tool(
  "update_issue",
  "Update an existing issue",
  {
    ...ownerRepo,
    issue_number: z.number().describe("Issue number"),
    title: z.string().optional().describe("New title"),
    body: z.string().optional().describe("New body"),
    state: z.enum(["open", "closed"]).optional().describe("New state"),
    assignees: z.array(z.string()).optional().describe("New assignees"),
    labels: z.array(z.number()).optional().describe("New label IDs"),
    milestone: z.number().optional().describe("New milestone ID"),
  },
  async ({ owner, repo, issue_number, ...rest }) =>
    json(await client.updateIssue(owner, repo, issue_number, rest)),
);

// ═══════════════════════════════════════════════════════════════════════
// ISSUE COMMENT TOOLS
// ═══════════════════════════════════════════════════════════════════════

server.tool(
  "list_issue_comments",
  "List comments on an issue",
  {
    ...ownerRepo,
    issue_number: z.number().describe("Issue number"),
    ...pagination,
  },
  async ({ owner, repo, issue_number, ...rest }) =>
    json(await client.listIssueComments(owner, repo, issue_number, rest)),
);

server.tool(
  "create_issue_comment",
  "Add a comment to an issue",
  {
    ...ownerRepo,
    issue_number: z.number().describe("Issue number"),
    body: z.string().describe("Comment body (markdown)"),
  },
  async ({ owner, repo, issue_number, body }) =>
    json(await client.createIssueComment(owner, repo, issue_number, { body })),
);

// ═══════════════════════════════════════════════════════════════════════
// LABEL TOOLS
// ═══════════════════════════════════════════════════════════════════════

server.tool(
  "list_labels",
  "List labels in a repository",
  { ...ownerRepo, ...pagination },
  async ({ owner, repo, ...rest }) =>
    json(await client.listLabels(owner, repo, rest)),
);

server.tool(
  "create_label",
  "Create a label in a repository",
  {
    ...ownerRepo,
    name: z.string().describe("Label name"),
    color: z.string().describe("Label color hex (e.g. #ff0000)"),
    description: z.string().optional().describe("Label description"),
  },
  async ({ owner, repo, ...rest }) =>
    json(await client.createLabel(owner, repo, rest)),
);

// ═══════════════════════════════════════════════════════════════════════
// MILESTONE TOOLS
// ═══════════════════════════════════════════════════════════════════════

server.tool(
  "list_milestones",
  "List milestones in a repository",
  {
    ...ownerRepo,
    state: z.enum(["open", "closed", "all"]).optional().describe("Filter by state"),
    ...pagination,
  },
  async ({ owner, repo, ...rest }) =>
    json(await client.listMilestones(owner, repo, rest)),
);

// ═══════════════════════════════════════════════════════════════════════
// PULL REQUEST TOOLS
// ═══════════════════════════════════════════════════════════════════════

server.tool(
  "list_pull_requests",
  "List pull requests in a repository",
  {
    ...ownerRepo,
    state: z.enum(["open", "closed", "all"]).optional().describe("Filter by state"),
    sort: z.string().optional().describe("Sort field (oldest, recentupdate, leastupdate, mostcomment, leastcomment, priority)"),
    labels: z.string().optional().describe("Comma-separated label IDs"),
    ...pagination,
  },
  async ({ owner, repo, ...rest }) =>
    json(await client.listPullRequests(owner, repo, rest)),
);

server.tool(
  "get_pull_request",
  "Get details of a specific pull request",
  {
    ...ownerRepo,
    pull_number: z.number().describe("Pull request number"),
  },
  async ({ owner, repo, pull_number }) =>
    json(await client.getPullRequest(owner, repo, pull_number)),
);

server.tool(
  "create_pull_request",
  "Create a new pull request",
  {
    ...ownerRepo,
    title: z.string().describe("PR title"),
    body: z.string().optional().describe("PR body (markdown)"),
    head: z.string().describe("Source branch (or fork:branch)"),
    base: z.string().describe("Target branch"),
    assignees: z.array(z.string()).optional().describe("Assignee usernames"),
    labels: z.array(z.number()).optional().describe("Label IDs"),
    milestone: z.number().optional().describe("Milestone ID"),
  },
  async ({ owner, repo, ...rest }) =>
    json(await client.createPullRequest(owner, repo, rest)),
);

server.tool(
  "update_pull_request",
  "Update an existing pull request",
  {
    ...ownerRepo,
    pull_number: z.number().describe("Pull request number"),
    title: z.string().optional().describe("New title"),
    body: z.string().optional().describe("New body"),
    state: z.enum(["open", "closed"]).optional().describe("New state"),
    assignees: z.array(z.string()).optional().describe("New assignees"),
    labels: z.array(z.number()).optional().describe("New label IDs"),
    milestone: z.number().optional().describe("New milestone ID"),
    base: z.string().optional().describe("New base branch"),
  },
  async ({ owner, repo, pull_number, ...rest }) =>
    json(await client.updatePullRequest(owner, repo, pull_number, rest)),
);

server.tool(
  "list_pull_request_files",
  "List files changed in a pull request",
  {
    ...ownerRepo,
    pull_number: z.number().describe("Pull request number"),
    ...pagination,
  },
  async ({ owner, repo, pull_number, ...rest }) =>
    json(await client.listPullRequestFiles(owner, repo, pull_number, rest)),
);

server.tool(
  "merge_pull_request",
  "Merge a pull request",
  {
    ...ownerRepo,
    pull_number: z.number().describe("Pull request number"),
    merge_method: z.enum(["merge", "rebase", "rebase-merge", "squash"]).describe("Merge method"),
    merge_message: z.string().optional().describe("Custom merge commit message"),
    delete_branch: z.boolean().optional().describe("Delete branch after merge"),
  },
  async ({ owner, repo, pull_number, merge_method, merge_message, delete_branch }) =>
    json(
      await client.mergePullRequest(owner, repo, pull_number, {
        Do: merge_method,
        merge_message_field: merge_message,
        delete_branch_after_merge: delete_branch,
      }),
    ),
);

// ═══════════════════════════════════════════════════════════════════════
// PR REVIEW TOOLS
// ═══════════════════════════════════════════════════════════════════════

server.tool(
  "list_pull_request_reviews",
  "List reviews on a pull request",
  {
    ...ownerRepo,
    pull_number: z.number().describe("Pull request number"),
  },
  async ({ owner, repo, pull_number }) =>
    json(await client.listPullRequestReviews(owner, repo, pull_number)),
);

server.tool(
  "create_pull_request_review",
  "Submit a review on a pull request",
  {
    ...ownerRepo,
    pull_number: z.number().describe("Pull request number"),
    body: z.string().optional().describe("Review body"),
    event: z.enum(["APPROVE", "REQUEST_CHANGES", "COMMENT"]).describe("Review action"),
    comments: z
      .array(
        z.object({
          path: z.string().describe("File path"),
          body: z.string().describe("Comment body"),
          new_position: z.number().optional().describe("Line in new file"),
          old_position: z.number().optional().describe("Line in old file"),
        }),
      )
      .optional()
      .describe("Inline review comments"),
  },
  async ({ owner, repo, pull_number, ...rest }) =>
    json(await client.createPullRequestReview(owner, repo, pull_number, rest)),
);

// ═══════════════════════════════════════════════════════════════════════
// SEARCH TOOLS
// ═══════════════════════════════════════════════════════════════════════

server.tool(
  "search_repos",
  "Search for repositories on Codeberg",
  {
    q: z.string().describe("Search query"),
    sort: z.string().optional().describe("Sort by (alpha, created, updated, size, id, stars, forks)"),
    order: z.enum(["asc", "desc"]).optional().describe("Sort order"),
    ...pagination,
  },
  async (args) => json(await client.searchRepos(args)),
);

// ═══════════════════════════════════════════════════════════════════════
// RELEASES TOOLS
// ═══════════════════════════════════════════════════════════════════════

server.tool(
  "list_releases",
  "List releases for a repository",
  { ...ownerRepo, ...pagination },
  async ({ owner, repo, ...rest }) =>
    json(await client.listReleases(owner, repo, rest)),
);

// ═══════════════════════════════════════════════════════════════════════
// COMMIT TOOLS
// ═══════════════════════════════════════════════════════════════════════

server.tool(
  "list_commits",
  "List commits in a repository",
  {
    ...ownerRepo,
    sha: z.string().optional().describe("Branch name or commit SHA to list from"),
    ...pagination,
  },
  async ({ owner, repo, ...rest }) =>
    json(await client.listCommits(owner, repo, rest)),
);

// ═══════════════════════════════════════════════════════════════════════
// NOTIFICATION TOOLS
// ═══════════════════════════════════════════════════════════════════════

server.tool(
  "list_notifications",
  "List notifications for the authenticated user",
  {
    all: z.boolean().optional().describe("Show all notifications including read"),
    status_types: z.string().optional().describe("Filter by status (unread, read, pinned)"),
    subject_type: z.string().optional().describe("Filter by type (issue, pull, commit, repository)"),
    ...pagination,
  },
  async (args) => json(await client.listNotifications(args)),
);

// ═══════════════════════════════════════════════════════════════════════
// ORGANIZATION TOOLS
// ═══════════════════════════════════════════════════════════════════════

server.tool(
  "list_my_orgs",
  "List organizations the authenticated user belongs to",
  pagination,
  async (args) => json(await client.listMyOrgs(args)),
);

server.tool(
  "get_org",
  "Get details about an organization",
  { org: z.string().describe("Organization name") },
  async ({ org }) => json(await client.getOrg(org)),
);

// ── Start server ──────────────────────────────────────────────────────
async function main() {
  const transport = new StdioServerTransport();
  await server.connect(transport);
  console.error("Codeberg MCP server running on stdio");
}

main().catch((err) => {
  console.error("Fatal error:", err);
  process.exit(1);
});
