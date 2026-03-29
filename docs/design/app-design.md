# Codeberg Android App — Design Document

## Overview

A native Android app for Codeberg (and other Forgejo/Gitea instances) that provides a mobile experience comparable to the GitHub Android app. Built in Rust using Tauri Mobile for the application shell and a web-based UI rendered natively on Android.

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Language | Rust (core + backend) |
| App Shell | Tauri Mobile (v2) — native Android WebView shell |
| UI | Leptos (Rust → WASM) or HTML/CSS/TypeScript frontend |
| Networking | reqwest (async HTTP client) |
| API Spec | Custom Forgejo API client crate (typed, from OpenAPI spec) |
| Auth | OAuth2 (Authorization Code + PKCE) via custom Rust implementation |
| Local Storage | SQLite via rusqlite or sqlx |
| Serialization | serde + serde_json |
| Async Runtime | tokio |
| Markdown | pulldown-cmark (Rust-native CommonMark parser) |
| Diff Parsing | Custom Rust parser for unified diff format |
| Syntax Highlighting | tree-sitter or syntect |
| Secure Storage | Android Keystore via Tauri plugin or JNI bridge |
| Build System | Cargo + Tauri CLI |
| Min SDK | 26 (Android 8.0) |
| Target SDK | 35 |

### Why Tauri Mobile?

Tauri v2 supports Android (and iOS) as build targets. The app runs as a native Android app with:
- A Rust backend process handling all logic, networking, and state
- A WebView frontend for UI rendering
- Tauri's IPC bridge connecting the two
- Access to native Android APIs via Tauri plugins (notifications, deep links, secure storage, etc.)

This gives us the full power of Rust for business logic while still having a flexible UI layer.

### Alternative: Fully Native with `android-activity`

For a pure Rust approach without WebView:
- Use the `android-activity` crate for the Android entry point
- Use `wgpu` + `egui` for GPU-rendered UI
- Full Rust stack with no web technologies
- Trade-off: less mature mobile UI ecosystem, harder to get native-feeling widgets

The Tauri approach is recommended for better UI flexibility and ecosystem maturity.

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                  UI Layer                            │
│   WebView Frontend (Leptos/WASM or HTML/TS)         │
│   Tauri IPC Commands ←→ Rust Backend                │
├─────────────────────────────────────────────────────┤
│              Tauri Rust Backend                      │
│   Command Handlers (invoke from frontend)           │
│   State Management (tauri::State)                   │
├─────────────────────────────────────────────────────┤
│                Domain Layer                          │
│   Service Traits + Implementations                  │
│   Domain Models (serde structs)                     │
├─────────────────────────────────────────────────────┤
│                Data Layer                            │
│   Remote: Forgejo API Client (reqwest)              │
│   Local: SQLite Cache (rusqlite/sqlx)               │
├─────────────────────────────────────────────────────┤
│              Core / Platform                         │
│   Auth (OAuth2 + PKCE + Token Management)           │
│   Instance Manager (multi-instance registry)        │
│   Notification Polling (tokio background tasks)     │
│   Secure Storage (Android Keystore bridge)          │
└─────────────────────────────────────────────────────┘
```

## Feature Breakdown

### Phase 1 — MVP

#### 1.1 Authentication & Instance Management
- Add/remove Forgejo instances (Codeberg as default)
- OAuth2 Authorization Code flow with PKCE
- Fallback: Personal Access Token entry
- Secure token storage (Android Keystore via Tauri plugin)
- Multi-account support

#### 1.2 Dashboard / Home
- Notification feed (unread/read, mark as read)
- Activity feed (events from followed users/repos)
- Quick actions: search, create issue/repo

#### 1.3 Repositories
- List user's repos (owned, starred, watched)
- Search repos (instance-wide)
- Repo detail view:
  - README rendering (Markdown via pulldown-cmark)
  - File tree browser
  - Branch/tag selector
  - File viewer with syntax highlighting (syntect)
  - Commit history
  - Stars/forks/watchers counts
- Create new repository
- Fork a repository

#### 1.4 Issues
- List issues (open/closed, filters: label, milestone, assignee)
- Issue detail:
  - Description (Markdown)
  - Comment thread
  - Labels, assignees, milestone display
- Create new issue
- Add comment
- Close/reopen issue
- Edit labels and assignees

#### 1.5 Pull Requests
- List PRs (open/closed/merged, filters)
- PR detail:
  - Description (Markdown)
  - Comment thread
  - Diff view (file-by-file, unified/split)
  - Commit list
  - Review status
- Merge PR (merge, rebase, squash)
- Add comment

#### 1.6 Profile
- View own profile (avatar, bio, repos, orgs)
- View other user profiles
- Organization listing and members

### Phase 2 — Enhanced

#### 2.1 Code Browsing
- Blame view
- Commit detail view (diff per file)
- Search code within a repo

#### 2.2 Releases & Tags
- List releases
- Release detail (assets, changelog)
- Download release assets

#### 2.3 CI / Forgejo Actions
- List workflow runs per repo
- View run status and logs
- Re-run failed workflows

#### 2.4 Notifications (Background)
- Tokio background task for periodic polling
- Tauri notification plugin for Android system notifications
- Notification channels:
  - Mentions
  - Review requests
  - CI failures
  - Issue/PR updates
- Notification grouping per repo
- Configurable poll interval (15, 30, 60 min)

#### 2.5 Offline Support
- Cache repos, issues, PRs in SQLite
- Offline browsing of cached content
- Queue actions (comments, status changes) for sync when online

### Phase 3 — Polish

#### 3.1 Deep Linking
- Handle `codeberg.org` URLs via Tauri deep link plugin
- Custom instance URL handling
- Android App Links verification

#### 3.2 Share & Integrations
- Share links to issues/PRs/repos
- Open in browser fallback
- Android intent filters for Codeberg URLs

#### 3.3 Theming
- Light/Dark/System theme via CSS variables + media query
- Material Design 3 styling via CSS
- Per-instance accent color

#### 3.4 Accessibility
- Semantic HTML in frontend for screen reader support
- ARIA attributes on all interactive elements
- Minimum touch targets (48dp)

## API Integration

### Base URL Pattern
```
https://{instance}/api/v1/
```

### Forgejo API Client Crate

```rust
// src-tauri/src/api/client.rs

pub struct ForgejoClient {
    http: reqwest::Client,
    base_url: String,
    token: Token,
}

pub enum Token {
    PersonalAccess(String),
    OAuth2 { access: String, refresh: String, expires_at: i64 },
}

impl ForgejoClient {
    pub async fn current_user(&self) -> Result<User> { ... }
    pub async fn list_repos(&self, params: &ListReposParams) -> Result<Vec<Repository>> { ... }
    pub async fn get_repo(&self, owner: &str, repo: &str) -> Result<Repository> { ... }
    pub async fn list_issues(&self, owner: &str, repo: &str, params: &ListIssuesParams) -> Result<Vec<Issue>> { ... }
    pub async fn create_issue(&self, owner: &str, repo: &str, body: &CreateIssue) -> Result<Issue> { ... }
    pub async fn list_pulls(&self, owner: &str, repo: &str, params: &ListPullsParams) -> Result<Vec<PullRequest>> { ... }
    pub async fn get_pull_diff(&self, owner: &str, repo: &str, index: u64) -> Result<String> { ... }
    pub async fn merge_pull(&self, owner: &str, repo: &str, index: u64, opts: &MergeOpts) -> Result<()> { ... }
    pub async fn notifications(&self, since: Option<DateTime<Utc>>) -> Result<Vec<Notification>> { ... }
    pub async fn search_repos(&self, query: &str) -> Result<Vec<Repository>> { ... }
    pub async fn get_file_contents(&self, owner: &str, repo: &str, path: &str, git_ref: Option<&str>) -> Result<FileContent> { ... }
    pub async fn list_action_runs(&self, owner: &str, repo: &str) -> Result<Vec<ActionRun>> { ... }
    // ... etc
}
```

### Key Endpoints

| Feature | Endpoint | Method |
|---------|----------|--------|
| Current user | `/user` | GET |
| User repos | `/user/repos` | GET |
| Repo detail | `/repos/{owner}/{repo}` | GET |
| Repo contents | `/repos/{owner}/{repo}/contents/{path}` | GET |
| Issues | `/repos/{owner}/{repo}/issues` | GET/POST |
| Issue comments | `/repos/{owner}/{repo}/issues/{index}/comments` | GET/POST |
| Pull requests | `/repos/{owner}/{repo}/pulls` | GET/POST |
| PR diff | `/repos/{owner}/{repo}/pulls/{index}.diff` | GET |
| PR merge | `/repos/{owner}/{repo}/pulls/{index}/merge` | POST |
| Notifications | `/notifications` | GET/PUT |
| Search repos | `/repos/search` | GET |
| Orgs | `/user/orgs` | GET |
| Actions runs | `/repos/{owner}/{repo}/actions/runs` | GET |
| Commit status | `/repos/{owner}/{repo}/statuses/{sha}` | GET/POST |

### Authentication Header
```
Authorization: token <PAT>
   — or —
Authorization: Bearer <OAuth2 token>
```

## Project Structure

```
codeberg-app/
├── Cargo.toml                              # Workspace root
├── src-tauri/
│   ├── Cargo.toml                          # Tauri app crate
│   ├── tauri.conf.json                     # Tauri config (Android target)
│   ├── gen/
│   │   └── android/                        # Generated Android project
│   ├── capabilities/
│   │   └── default.json                    # Tauri permissions
│   ├── icons/                              # App icons
│   └── src/
│       ├── main.rs                         # Tauri entry point
│       ├── commands/                        # Tauri IPC command handlers
│       │   ├── mod.rs
│       │   ├── auth.rs                     # login, logout, add_instance
│       │   ├── repos.rs                    # list_repos, get_repo, search
│       │   ├── issues.rs                   # list_issues, create_issue, comment
│       │   ├── pulls.rs                    # list_pulls, get_diff, merge
│       │   ├── notifications.rs            # list_notifications, mark_read
│       │   ├── user.rs                     # get_profile, list_orgs
│       │   └── actions.rs                  # list_runs, get_logs
│       ├── api/
│       │   ├── mod.rs
│       │   ├── client.rs                   # ForgejoClient (reqwest)
│       │   ├── models.rs                   # API response/request structs
│       │   └── error.rs                    # API error types
│       ├── auth/
│       │   ├── mod.rs
│       │   ├── oauth2.rs                   # OAuth2 PKCE flow
│       │   └── token_store.rs              # Secure token persistence
│       ├── instance/
│       │   ├── mod.rs
│       │   └── manager.rs                  # Multi-instance registry
│       ├── cache/
│       │   ├── mod.rs
│       │   ├── db.rs                       # SQLite schema + migrations
│       │   └── offline.rs                  # Offline queue
│       ├── notifications/
│       │   ├── mod.rs
│       │   └── poller.rs                   # Background polling task
│       └── util/
│           ├── mod.rs
│           ├── diff.rs                     # Unified diff parser
│           └── markdown.rs                 # pulldown-cmark wrapper
├── src/                                    # Frontend (WebView)
│   ├── index.html
│   ├── main.ts                             # Entry point
│   ├── styles/
│   │   ├── global.css                      # Base styles + Material tokens
│   │   ├── theme.css                       # Light/dark theme variables
│   │   └── components/                     # Per-component styles
│   ├── lib/
│   │   ├── api.ts                          # Tauri invoke wrappers
│   │   ├── router.ts                       # Client-side routing
│   │   └── store.ts                        # Reactive state
│   ├── components/
│   │   ├── MarkdownView.ts                 # Rendered markdown display
│   │   ├── DiffView.ts                     # PR diff viewer
│   │   ├── CodeView.ts                     # Syntax-highlighted file viewer
│   │   ├── IssueCard.ts                    # Issue list item
│   │   ├── PrCard.ts                       # PR list item
│   │   ├── RepoCard.ts                     # Repo list item
│   │   └── NotificationItem.ts             # Notification list item
│   └── pages/
│       ├── Login.ts
│       ├── Home.ts
│       ├── RepoList.ts
│       ├── RepoDetail.ts
│       ├── FileTree.ts
│       ├── FileView.ts
│       ├── IssueList.ts
│       ├── IssueDetail.ts
│       ├── PrList.ts
│       ├── PrDetail.ts
│       ├── Notifications.ts
│       └── Profile.ts
├── package.json                            # Frontend dependencies (if TS)
└── tests/
    ├── api_client_test.rs                  # Integration tests for API client
    ├── diff_parser_test.rs                 # Diff parsing tests
    └── cache_test.rs                       # SQLite cache tests
```

### Alternative: Leptos (Full Rust Frontend)

If you want the frontend in Rust too (no TypeScript), replace the `src/` frontend with Leptos:

```
src/                                        # Leptos frontend (compiles to WASM)
├── lib.rs                                  # Leptos app entry
├── app.rs                                  # Root component + router
├── api.rs                                  # Tauri invoke bindings (wasm-bindgen)
├── components/
│   ├── markdown_view.rs
│   ├── diff_view.rs
│   ├── code_view.rs
│   └── ...
└── pages/
    ├── login.rs
    ├── home.rs
    ├── repo_list.rs
    ├── repo_detail.rs
    └── ...
```

This uses `trunk` to build the WASM bundle served by Tauri's WebView.

## Multi-Instance Support

```rust
// src-tauri/src/instance/manager.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instance {
    pub id: String,              // UUID
    pub name: String,            // e.g., "Codeberg"
    pub base_url: String,        // e.g., "https://codeberg.org"
    pub token_type: TokenType,   // OAuth2 or PAT
    pub user_id: Option<i64>,
    pub username: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TokenType {
    OAuth2,
    PersonalAccessToken,
}

pub struct InstanceManager {
    instances: Vec<Instance>,
    active: Option<String>,      // Active instance ID
    db: rusqlite::Connection,
}

impl InstanceManager {
    pub fn add(&mut self, instance: Instance) -> Result<()> { ... }
    pub fn remove(&mut self, id: &str) -> Result<()> { ... }
    pub fn set_active(&mut self, id: &str) -> Result<()> { ... }
    pub fn active_instance(&self) -> Option<&Instance> { ... }
    pub fn list(&self) -> &[Instance] { ... }
}
```

Each instance gets its own `ForgejoClient` with the appropriate base URL and auth. Users switch instances from a drawer or account switcher in the UI.

## Notification Polling Strategy

Since Forgejo has no push notification infrastructure:

```rust
// src-tauri/src/notifications/poller.rs

use tokio::time::{interval, Duration};

pub struct NotificationPoller {
    interval_mins: u64,
    last_check: HashMap<String, DateTime<Utc>>,  // per instance
}

impl NotificationPoller {
    pub async fn start(&mut self, instances: Vec<(String, ForgejoClient)>) {
        let mut tick = interval(Duration::from_secs(self.interval_mins * 60));
        loop {
            tick.tick().await;
            for (instance_id, client) in &instances {
                let since = self.last_check.get(instance_id).copied();
                match client.notifications(since).await {
                    Ok(notifications) => {
                        for n in &notifications {
                            self.send_android_notification(instance_id, n);
                        }
                        self.last_check.insert(
                            instance_id.clone(),
                            Utc::now(),
                        );
                    }
                    Err(e) => log::warn!("Poll failed for {}: {}", instance_id, e),
                }
            }
        }
    }
}
```

- Uses Tauri's notification plugin to show Android system notifications
- Configurable poll interval (15, 30, 60 min)
- Respects Android battery optimization (Tauri handles lifecycle)

## Diff Rendering Strategy

PR diffs are the hardest UI component:

1. Fetch unified diff from `/repos/{owner}/{repo}/pulls/{index}.diff`
2. Parse in Rust into typed structs:

```rust
pub struct FileDiff {
    pub old_path: String,
    pub new_path: String,
    pub status: DiffStatus,     // Added, Deleted, Modified, Renamed
    pub hunks: Vec<Hunk>,
}

pub struct Hunk {
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
    pub lines: Vec<DiffLine>,
}

pub struct DiffLine {
    pub kind: LineKind,         // Context, Addition, Deletion
    pub content: String,
    pub old_lineno: Option<u32>,
    pub new_lineno: Option<u32>,
}
```

3. Send parsed structs to frontend via Tauri IPC
4. Render as collapsible file cards with color-coded lines
5. Syntax highlight via syntect (in Rust) or highlight.js (in frontend)

## Testing Strategy

| Layer | Approach |
|-------|----------|
| API Client | `#[tokio::test]` + wiremock for mock HTTP |
| Domain Logic | Standard `#[test]` unit tests |
| Diff Parser | `#[test]` with fixture files |
| Cache/DB | rusqlite in-memory database tests |
| Tauri Commands | tauri-test for command handler testing |
| Frontend | Playwright or WebDriver for E2E |
| CI | Forgejo Actions workflow running on Codeberg |

## Build & Distribution

### Build Commands
```bash
# Development
cargo tauri android dev

# Release APK
cargo tauri android build --release

# Generate signed AAB for Play Store
cargo tauri android build --release --aab
```

### Distribution
- **F-Droid**: Primary (fully open source, no proprietary dependencies)
- **Google Play Store**: Secondary, optional
- **Codeberg Releases**: Direct APK downloads
- No Google Play Services dependency — pure polling for notifications
