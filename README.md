# Codeberg MCP Server

An [MCP (Model Context Protocol)](https://modelcontextprotocol.io) server that connects AI assistants like Claude to [Codeberg](https://codeberg.org) — the free, open-source Git forge powered by Forgejo/Gitea.

## Features

**Repositories** — list, create, fork, get details
**Branches** — list, create
**File Contents** — read, create/update, delete files directly in repos
**Issues** — list, get, create, update
**Issue Comments** — list and create comments
**Labels & Milestones** — list and create
**Pull Requests** — list, get, create, update, merge
**PR Reviews** — list and submit reviews with inline comments
**Search** — search repositories across Codeberg
**Releases** — list releases for any repo
**Commits** — list commit history
**Notifications** — view your Codeberg notifications
**Organizations** — list and inspect orgs
**Users** — look up user profiles

## Setup

### 1. Get a Codeberg API Token

1. Go to **Codeberg → Settings → Applications**
2. Generate a new token with the scopes you need
3. Copy the token

### 2. Install

```bash
npm install
npm run build
```

### 3. Configure for Claude Code

Add to your Claude Code MCP settings (`~/.claude/settings.json` or project `.claude/settings.json`):

```json
{
  "mcpServers": {
    "codeberg": {
      "command": "node",
      "args": ["/path/to/codeberg-mcp-server/dist/index.js"],
      "env": {
        "CODEBERG_TOKEN": "your-token-here"
      }
    }
  }
}
```

### 4. Configure for Claude Desktop

Add to your Claude Desktop config (`~/Library/Application Support/Claude/claude_desktop_config.json` on macOS):

```json
{
  "mcpServers": {
    "codeberg": {
      "command": "node",
      "args": ["/path/to/codeberg-mcp-server/dist/index.js"],
      "env": {
        "CODEBERG_TOKEN": "your-token-here"
      }
    }
  }
}
```

## Environment Variables

| Variable | Required | Default | Description |
|---|---|---|---|
| `CODEBERG_TOKEN` | Yes | — | Codeberg API token |
| `CODEBERG_URL` | No | `https://codeberg.org` | Base URL (for self-hosted Gitea/Forgejo instances) |

## Available Tools

### User
- `get_authenticated_user` — Get your profile
- `get_user` — Look up any user

### Repositories
- `list_my_repos` — Your repositories
- `list_user_repos` — A user's public repos
- `list_org_repos` — An org's repos
- `get_repo` — Repository details
- `create_repo` — Create a new repo
- `fork_repo` — Fork a repo

### Branches
- `list_branches` — List branches
- `create_branch` — Create a branch

### File Contents
- `get_file_contents` — Read a file or directory listing
- `create_or_update_file` — Create or update a file (with commit)
- `delete_file` — Delete a file (with commit)

### Issues
- `list_issues` — List issues (filter by state, labels, assignee, milestone)
- `get_issue` — Get issue details
- `create_issue` — Create an issue
- `update_issue` — Update an issue (title, body, state, assignees, labels)

### Issue Comments
- `list_issue_comments` — List comments on an issue
- `create_issue_comment` — Add a comment

### Labels & Milestones
- `list_labels` / `create_label`
- `list_milestones`

### Pull Requests
- `list_pull_requests` — List PRs (filter by state, sort, labels)
- `get_pull_request` — PR details
- `create_pull_request` — Open a new PR
- `update_pull_request` — Update a PR
- `list_pull_request_files` — Files changed in a PR
- `merge_pull_request` — Merge a PR (merge, rebase, squash)

### PR Reviews
- `list_pull_request_reviews` — List reviews
- `create_pull_request_review` — Submit a review (approve, request changes, comment)

### Search
- `search_repos` — Search repositories

### Other
- `list_releases` — Repository releases
- `list_commits` — Commit history
- `list_notifications` — Your notifications
- `list_my_orgs` / `get_org` — Organizations

## Self-Hosted Gitea/Forgejo

This server works with any Gitea or Forgejo instance. Set `CODEBERG_URL` to your instance URL:

```json
{
  "env": {
    "CODEBERG_TOKEN": "your-token",
    "CODEBERG_URL": "https://git.example.com"
  }
}
```

## License

MIT
