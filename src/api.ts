/**
 * Codeberg API client - wraps the Gitea/Forgejo REST API v1.
 */

const DEFAULT_BASE_URL = "https://codeberg.org";

export interface CodebergClientOptions {
  token: string;
  baseUrl?: string;
}

export class CodebergClient {
  private token: string;
  private baseUrl: string;

  constructor(opts: CodebergClientOptions) {
    this.token = opts.token;
    this.baseUrl = (opts.baseUrl ?? DEFAULT_BASE_URL).replace(/\/+$/, "");
  }

  private async request<T>(
    method: string,
    path: string,
    body?: unknown,
    query?: Record<string, string | number | boolean | undefined>,
  ): Promise<T> {
    const url = new URL(`/api/v1${path}`, this.baseUrl);
    if (query) {
      for (const [k, v] of Object.entries(query)) {
        if (v !== undefined) url.searchParams.set(k, String(v));
      }
    }

    const headers: Record<string, string> = {
      Authorization: `token ${this.token}`,
      Accept: "application/json",
    };
    if (body !== undefined) {
      headers["Content-Type"] = "application/json";
    }

    const res = await fetch(url.toString(), {
      method,
      headers,
      body: body !== undefined ? JSON.stringify(body) : undefined,
    });

    if (!res.ok) {
      const text = await res.text().catch(() => "");
      throw new Error(
        `Codeberg API ${method} ${path} responded ${res.status}: ${text}`,
      );
    }

    if (res.status === 204) return undefined as T;
    return (await res.json()) as T;
  }

  // ── User ────────────────────────────────────────────────────────────
  getAuthenticatedUser() {
    return this.request<any>("GET", "/user");
  }

  getUser(username: string) {
    return this.request<any>("GET", `/users/${encodeURIComponent(username)}`);
  }

  // ── Repositories ────────────────────────────────────────────────────
  listMyRepos(opts?: { page?: number; limit?: number }) {
    return this.request<any[]>("GET", "/user/repos", undefined, opts);
  }

  listUserRepos(
    owner: string,
    opts?: { page?: number; limit?: number },
  ) {
    return this.request<any[]>(
      "GET",
      `/users/${encodeURIComponent(owner)}/repos`,
      undefined,
      opts,
    );
  }

  listOrgRepos(
    org: string,
    opts?: { page?: number; limit?: number },
  ) {
    return this.request<any[]>(
      "GET",
      `/orgs/${encodeURIComponent(org)}/repos`,
      undefined,
      opts,
    );
  }

  getRepo(owner: string, repo: string) {
    return this.request<any>(
      "GET",
      `/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}`,
    );
  }

  createRepo(body: {
    name: string;
    description?: string;
    private?: boolean;
    auto_init?: boolean;
    default_branch?: string;
    license?: string;
    gitignores?: string;
  }) {
    return this.request<any>("POST", "/user/repos", body);
  }

  forkRepo(owner: string, repo: string, body?: { name?: string; organization?: string }) {
    return this.request<any>(
      "POST",
      `/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/forks`,
      body ?? {},
    );
  }

  // ── Branches ────────────────────────────────────────────────────────
  listBranches(
    owner: string,
    repo: string,
    opts?: { page?: number; limit?: number },
  ) {
    return this.request<any[]>(
      "GET",
      `/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/branches`,
      undefined,
      opts,
    );
  }

  createBranch(owner: string, repo: string, body: { new_branch_name: string; old_branch_name?: string }) {
    return this.request<any>(
      "POST",
      `/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/branches`,
      body,
    );
  }

  // ── File contents ───────────────────────────────────────────────────
  getFileContents(
    owner: string,
    repo: string,
    filepath: string,
    ref?: string,
  ) {
    return this.request<any>(
      "GET",
      `/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/contents/${filepath}`,
      undefined,
      ref ? { ref } : undefined,
    );
  }

  createOrUpdateFile(
    owner: string,
    repo: string,
    filepath: string,
    body: {
      content: string; // base64 encoded
      message: string;
      branch?: string;
      sha?: string; // required for update
      new_branch?: string;
    },
  ) {
    return this.request<any>(
      "PUT",
      `/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/contents/${filepath}`,
      body,
    );
  }

  deleteFile(
    owner: string,
    repo: string,
    filepath: string,
    body: {
      message: string;
      sha: string;
      branch?: string;
      new_branch?: string;
    },
  ) {
    return this.request<any>(
      "DELETE",
      `/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/contents/${filepath}`,
      body,
    );
  }

  // ── Issues ──────────────────────────────────────────────────────────
  listIssues(
    owner: string,
    repo: string,
    opts?: {
      state?: string;
      labels?: string;
      page?: number;
      limit?: number;
      type?: string;
      milestones?: string;
      assignee?: string;
    },
  ) {
    return this.request<any[]>(
      "GET",
      `/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/issues`,
      undefined,
      opts as Record<string, string | number | boolean | undefined>,
    );
  }

  getIssue(owner: string, repo: string, index: number) {
    return this.request<any>(
      "GET",
      `/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/issues/${index}`,
    );
  }

  createIssue(
    owner: string,
    repo: string,
    body: {
      title: string;
      body?: string;
      assignees?: string[];
      labels?: number[];
      milestone?: number;
    },
  ) {
    return this.request<any>(
      "POST",
      `/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/issues`,
      body,
    );
  }

  updateIssue(
    owner: string,
    repo: string,
    index: number,
    body: {
      title?: string;
      body?: string;
      state?: string;
      assignees?: string[];
      labels?: number[];
      milestone?: number;
    },
  ) {
    return this.request<any>(
      "PATCH",
      `/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/issues/${index}`,
      body,
    );
  }

  // ── Issue Comments ──────────────────────────────────────────────────
  listIssueComments(
    owner: string,
    repo: string,
    index: number,
    opts?: { page?: number; limit?: number },
  ) {
    return this.request<any[]>(
      "GET",
      `/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/issues/${index}/comments`,
      undefined,
      opts,
    );
  }

  createIssueComment(
    owner: string,
    repo: string,
    index: number,
    body: { body: string },
  ) {
    return this.request<any>(
      "POST",
      `/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/issues/${index}/comments`,
      body,
    );
  }

  // ── Labels ──────────────────────────────────────────────────────────
  listLabels(
    owner: string,
    repo: string,
    opts?: { page?: number; limit?: number },
  ) {
    return this.request<any[]>(
      "GET",
      `/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/labels`,
      undefined,
      opts,
    );
  }

  createLabel(
    owner: string,
    repo: string,
    body: { name: string; color: string; description?: string },
  ) {
    return this.request<any>(
      "POST",
      `/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/labels`,
      body,
    );
  }

  // ── Milestones ──────────────────────────────────────────────────────
  listMilestones(
    owner: string,
    repo: string,
    opts?: { state?: string; page?: number; limit?: number },
  ) {
    return this.request<any[]>(
      "GET",
      `/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/milestones`,
      undefined,
      opts as Record<string, string | number | boolean | undefined>,
    );
  }

  // ── Pull Requests ──────────────────────────────────────────────────
  listPullRequests(
    owner: string,
    repo: string,
    opts?: {
      state?: string;
      sort?: string;
      labels?: string;
      page?: number;
      limit?: number;
    },
  ) {
    return this.request<any[]>(
      "GET",
      `/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/pulls`,
      undefined,
      opts as Record<string, string | number | boolean | undefined>,
    );
  }

  getPullRequest(owner: string, repo: string, index: number) {
    return this.request<any>(
      "GET",
      `/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/pulls/${index}`,
    );
  }

  createPullRequest(
    owner: string,
    repo: string,
    body: {
      title: string;
      body?: string;
      head: string;
      base: string;
      assignees?: string[];
      labels?: number[];
      milestone?: number;
    },
  ) {
    return this.request<any>(
      "POST",
      `/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/pulls`,
      body,
    );
  }

  updatePullRequest(
    owner: string,
    repo: string,
    index: number,
    body: {
      title?: string;
      body?: string;
      state?: string;
      assignees?: string[];
      labels?: number[];
      milestone?: number;
      base?: string;
    },
  ) {
    return this.request<any>(
      "PATCH",
      `/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/pulls/${index}`,
      body,
    );
  }

  listPullRequestFiles(
    owner: string,
    repo: string,
    index: number,
    opts?: { page?: number; limit?: number },
  ) {
    return this.request<any[]>(
      "GET",
      `/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/pulls/${index}/files`,
      undefined,
      opts,
    );
  }

  mergePullRequest(
    owner: string,
    repo: string,
    index: number,
    body: {
      Do: string; // merge, rebase, rebase-merge, squash
      merge_message_field?: string;
      delete_branch_after_merge?: boolean;
    },
  ) {
    return this.request<any>(
      "POST",
      `/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/pulls/${index}/merge`,
      body,
    );
  }

  // ── PR Reviews ──────────────────────────────────────────────────────
  listPullRequestReviews(owner: string, repo: string, index: number) {
    return this.request<any[]>(
      "GET",
      `/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/pulls/${index}/reviews`,
    );
  }

  createPullRequestReview(
    owner: string,
    repo: string,
    index: number,
    body: {
      body?: string;
      event: string; // APPROVE, REQUEST_CHANGES, COMMENT
      comments?: Array<{
        path: string;
        body: string;
        new_position?: number;
        old_position?: number;
      }>;
    },
  ) {
    return this.request<any>(
      "POST",
      `/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/pulls/${index}/reviews`,
      body,
    );
  }

  // ── Search ──────────────────────────────────────────────────────────
  searchRepos(opts: {
    q: string;
    page?: number;
    limit?: number;
    sort?: string;
    order?: string;
  }) {
    return this.request<{ ok: boolean; data: any[] }>(
      "GET",
      "/repos/search",
      undefined,
      opts as Record<string, string | number | boolean | undefined>,
    );
  }

  searchIssues(opts: {
    q: string;
    state?: string;
    labels?: string;
    type?: string;
    page?: number;
    limit?: number;
  }) {
    return this.request<any[]>(
      "GET",
      "/repos/search",
      undefined,
      // Gitea doesn't have a dedicated issue search endpoint,
      // but we can use the issues endpoint with query params
      opts as Record<string, string | number | boolean | undefined>,
    );
  }

  // ── Releases ────────────────────────────────────────────────────────
  listReleases(
    owner: string,
    repo: string,
    opts?: { page?: number; limit?: number },
  ) {
    return this.request<any[]>(
      "GET",
      `/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/releases`,
      undefined,
      opts,
    );
  }

  // ── Commits ─────────────────────────────────────────────────────────
  listCommits(
    owner: string,
    repo: string,
    opts?: { sha?: string; page?: number; limit?: number },
  ) {
    return this.request<any[]>(
      "GET",
      `/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/git/commits`,
      undefined,
      opts as Record<string, string | number | boolean | undefined>,
    );
  }

  // ── Notifications ───────────────────────────────────────────────────
  listNotifications(opts?: {
    all?: boolean;
    page?: number;
    limit?: number;
    status_types?: string;
    subject_type?: string;
  }) {
    return this.request<any[]>(
      "GET",
      "/notifications",
      undefined,
      opts as Record<string, string | number | boolean | undefined>,
    );
  }

  // ── Organizations ───────────────────────────────────────────────────
  listMyOrgs(opts?: { page?: number; limit?: number }) {
    return this.request<any[]>("GET", "/user/orgs", undefined, opts);
  }

  getOrg(org: string) {
    return this.request<any>("GET", `/orgs/${encodeURIComponent(org)}`);
  }
}
