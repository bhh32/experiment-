# Codeberg Android App — Design Document

## Overview

A native Android app for Codeberg (and other Forgejo/Gitea instances) that provides a mobile experience comparable to the GitHub Android app. Built entirely in Rust using Tauri Mobile for the Android shell and Leptos (compiled to WASM) for the reactive UI.

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Language | Rust (core + backend) |
| App Shell | Tauri Mobile (v2) — native Android WebView shell |
| UI | Leptos (Rust → WASM) — reactive frontend framework |
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

### Why Tauri Mobile + Leptos?

Tauri v2 supports Android (and iOS) as build targets. The app runs as a native Android app with:
- A Rust backend process handling all logic, networking, and state
- A Leptos frontend compiled to WASM, rendered in the native WebView
- Tauri's IPC bridge connecting the two
- Access to native Android APIs via Tauri plugins (notifications, deep links, secure storage, etc.)

Leptos provides:
- Fine-grained reactivity (signals, not virtual DOM diffing)
- Server functions / Tauri command integration
- Component-based architecture familiar to anyone who's used React/Solid
- The entire app is Rust — no context switching between languages
- `trunk` builds the WASM bundle served by Tauri's WebView

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                  UI Layer                            │
│   Leptos Components (Rust → WASM in WebView)        │
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
- Light/Dark/System theme via Leptos signals + CSS variables
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
├── Trunk.toml                              # Trunk build config for Leptos WASM
├── index.html                              # Trunk entry point (loads WASM)
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
├── src/                                    # Leptos frontend (Rust → WASM)
│   ├── lib.rs                              # Leptos app entry + mount
│   ├── app.rs                              # Root component + leptos_router
│   ├── api.rs                              # Tauri invoke wrappers (wasm-bindgen)
│   ├── styles/
│   │   ├── global.css                      # Base styles + Material tokens
│   │   ├── theme.css                       # Light/dark theme variables
│   │   └── components/                     # Per-component styles
│   ├── components/
│   │   ├── mod.rs
│   │   ├── markdown_view.rs                # Rendered markdown display
│   │   ├── diff_view.rs                    # PR diff viewer
│   │   ├── code_view.rs                    # Syntax-highlighted file viewer
│   │   ├── issue_card.rs                   # Issue list item
│   │   ├── pr_card.rs                      # PR list item
│   │   ├── repo_card.rs                    # Repo list item
│   │   ├── notification_item.rs            # Notification list item
│   │   ├── nav_bar.rs                      # Bottom navigation bar
│   │   ├── instance_switcher.rs            # Account/instance switcher
│   │   └── loading.rs                      # Loading/skeleton states
│   └── pages/
│       ├── mod.rs
│       ├── login.rs                        # Login + instance setup
│       ├── home.rs                         # Dashboard / notifications
│       ├── repo_list.rs                    # Repository listing
│       ├── repo_detail.rs                  # Repo overview (README, stats)
│       ├── file_tree.rs                    # File browser
│       ├── file_view.rs                    # Single file viewer
│       ├── issue_list.rs                   # Issue listing + filters
│       ├── issue_detail.rs                 # Issue thread
│       ├── pr_list.rs                      # PR listing + filters
│       ├── pr_detail.rs                    # PR thread + diff
│       ├── notifications.rs                # Notification center
│       └── profile.rs                      # User/org profile
└── tests/
    ├── api_client_test.rs                  # Integration tests for API client
    ├── diff_parser_test.rs                 # Diff parsing tests
    └── cache_test.rs                       # SQLite cache tests
```

## Leptos Frontend Patterns

### Tauri IPC from WASM

Leptos components call Tauri commands via `wasm-bindgen` bindings:

```rust
// src/api.rs

use serde::{de::DeserializeOwned, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

pub async fn tauri_invoke<A: Serialize, R: DeserializeOwned>(
    command: &str,
    args: &A,
) -> Result<R, String> {
    let args_js = serde_wasm_bindgen::to_value(args)
        .map_err(|e| e.to_string())?;
    let result = invoke(command, args_js).await;
    serde_wasm_bindgen::from_value(result)
        .map_err(|e| e.to_string())
}
```

### Component Example

```rust
// src/pages/repo_list.rs

use leptos::*;
use crate::api::tauri_invoke;
use crate::components::repo_card::RepoCard;

#[component]
pub fn RepoList() -> impl IntoView {
    let repos = create_resource(
        || (),
        |_| async move {
            tauri_invoke::<_, Vec<Repository>>("list_repos", &()).await
        },
    );

    view! {
        <div class="repo-list">
            <h1>"Repositories"</h1>
            <Suspense fallback=move || view! { <p>"Loading..."</p> }>
                {move || repos.get().map(|result| match result {
                    Ok(repos) => view! {
                        <For
                            each=move || repos.clone()
                            key=|repo| repo.id
                            children=move |repo| view! { <RepoCard repo=repo /> }
                        />
                    }.into_view(),
                    Err(e) => view! { <p class="error">{e}</p> }.into_view(),
                })}
            </Suspense>
        </div>
    }
}
```

### Routing

```rust
// src/app.rs

use leptos::*;
use leptos_router::*;
use crate::pages::*;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <main>
                <Routes>
                    <Route path="/login" view=Login />
                    <Route path="/" view=Home />
                    <Route path="/repos" view=RepoList />
                    <Route path="/repos/:owner/:name" view=RepoDetail />
                    <Route path="/repos/:owner/:name/issues" view=IssueList />
                    <Route path="/repos/:owner/:name/issues/:index" view=IssueDetail />
                    <Route path="/repos/:owner/:name/pulls" view=PrList />
                    <Route path="/repos/:owner/:name/pulls/:index" view=PrDetail />
                    <Route path="/repos/:owner/:name/tree/*path" view=FileTree />
                    <Route path="/repos/:owner/:name/blob/*path" view=FileView />
                    <Route path="/notifications" view=Notifications />
                    <Route path="/profile/:username" view=Profile />
                </Routes>
            </main>
            <NavBar />
        </Router>
    }
}
```

### Global State

```rust
// src/lib.rs

use leptos::*;

#[derive(Clone)]
pub struct AppState {
    pub active_instance: RwSignal<Option<Instance>>,
    pub current_user: RwSignal<Option<User>>,
    pub theme: RwSignal<Theme>,
    pub unread_count: RwSignal<u32>,
}

pub fn provide_app_state() {
    let state = AppState {
        active_instance: create_rw_signal(None),
        current_user: create_rw_signal(None),
        theme: create_rw_signal(Theme::System),
        unread_count: create_rw_signal(0),
    };
    provide_context(state);
}
```

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
4. Render as Leptos components — collapsible file cards with color-coded lines
5. Syntax highlight via syntect (in the Tauri backend, returned as HTML spans)

## Testing Strategy

| Layer | Approach |
|-------|----------|
| API Client | `#[tokio::test]` + wiremock for mock HTTP |
| Domain Logic | Standard `#[test]` unit tests |
| Diff Parser | `#[test]` with fixture files |
| Cache/DB | rusqlite in-memory database tests |
| Tauri Commands | tauri-test for command handler testing |
| Leptos Components | wasm-bindgen-test + Playwright for E2E |
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
