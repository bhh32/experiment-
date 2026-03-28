# Codeberg Android App — Design Document

## Overview

A native Android app for Codeberg (and other Forgejo/Gitea instances) that provides a mobile experience comparable to the GitHub Android app.

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Language | Kotlin |
| UI Framework | Jetpack Compose + Material 3 |
| Architecture | MVVM + Clean Architecture |
| Networking | Retrofit + OkHttp |
| API Spec | Generated client from Forgejo OpenAPI/Swagger spec |
| Auth | OAuth2 (Authorization Code + PKCE) via AppAuth library |
| Local Storage | Room (SQLite) for offline caching |
| Image Loading | Coil |
| Dependency Injection | Hilt |
| Navigation | Jetpack Navigation Compose |
| Notifications | Polling service + WorkManager (no native push from Forgejo) |
| Markdown | Custom renderer (Markwon or CommonMark + Compose) |
| Diff Rendering | Custom composable with syntax highlighting |
| Build System | Gradle (Kotlin DSL) |
| Min SDK | 26 (Android 8.0) |
| Target SDK | 35 |

## Architecture

```
┌─────────────────────────────────────────────────┐
│                  UI Layer                        │
│         Jetpack Compose Screens                  │
│         ViewModels (per feature)                 │
├─────────────────────────────────────────────────┤
│                Domain Layer                      │
│         Use Cases / Interactors                  │
│         Repository Interfaces                    │
│         Domain Models                            │
├─────────────────────────────────────────────────┤
│                Data Layer                        │
│   Remote: Retrofit API Client (Forgejo REST)     │
│   Local: Room Database (offline cache)           │
│   Repository Implementations                     │
├─────────────────────────────────────────────────┤
│              Core / Common                       │
│   Auth (OAuth2 + Token Management)               │
│   Instance Management (multi-instance)           │
│   Networking (OkHttp interceptors)               │
│   Notifications (WorkManager polling)            │
└─────────────────────────────────────────────────┘
```

## Feature Breakdown

### Phase 1 — MVP

#### 1.1 Authentication & Instance Management
- Add/remove Forgejo instances (Codeberg as default)
- OAuth2 Authorization Code flow with PKCE
- Fallback: Personal Access Token entry
- Secure token storage (EncryptedSharedPreferences)
- Multi-account support

#### 1.2 Dashboard / Home
- Notification feed (unread/read, mark as read)
- Activity feed (events from followed users/repos)
- Quick actions: search, create issue/repo

#### 1.3 Repositories
- List user's repos (owned, starred, watched)
- Search repos (instance-wide)
- Repo detail view:
  - README rendering (Markdown)
  - File tree browser
  - Branch/tag selector
  - File viewer with syntax highlighting
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
- WorkManager periodic polling (configurable interval: 5-60 min)
- Android notification channels:
  - Mentions
  - Review requests
  - CI failures
  - Issue/PR updates
- Notification grouping per repo

#### 2.5 Offline Support
- Cache repos, issues, PRs in Room database
- Offline browsing of cached content
- Queue actions (comments, status changes) for sync when online

### Phase 3 — Polish

#### 3.1 Deep Linking
- Handle `codeberg.org` URLs (repos, issues, PRs, users)
- Custom instance URL handling

#### 3.2 Widgets
- Home screen widget: notification count, repo shortcuts

#### 3.3 Share & Integrations
- Share links to issues/PRs/repos
- Open in browser fallback
- Intent filters for Codeberg URLs

#### 3.4 Theming
- Material You / Dynamic Color support
- Light/Dark/System theme
- Per-instance accent color

#### 3.5 Accessibility
- Full TalkBack support
- Content descriptions on all interactive elements
- Minimum touch targets (48dp)

## API Integration

### Base URL Pattern
```
https://{instance}/api/v1/
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

### Client Generation
Generate the API client from the Forgejo OpenAPI spec:
```
https://{instance}/swagger.json
```
Use **OpenAPI Generator** with the `kotlin` generator targeting Retrofit.

## Project Structure

```
app/
├── build.gradle.kts
├── src/main/
│   ├── AndroidManifest.xml
│   └── kotlin/com/example/codebergapp/
│       ├── App.kt                          # Application class
│       ├── MainActivity.kt                 # Single activity host
│       ├── navigation/
│       │   └── AppNavGraph.kt              # Navigation routes
│       ├── core/
│       │   ├── auth/
│       │   │   ├── AuthManager.kt          # OAuth2 + token management
│       │   │   └── TokenStore.kt           # EncryptedSharedPreferences
│       │   ├── instance/
│       │   │   ├── InstanceManager.kt      # Multi-instance registry
│       │   │   └── Instance.kt             # Data class
│       │   ├── network/
│       │   │   ├── ApiClientFactory.kt     # Per-instance Retrofit
│       │   │   └── AuthInterceptor.kt      # Token injection
│       │   └── di/
│       │       └── AppModule.kt            # Hilt modules
│       ├── data/
│       │   ├── remote/
│       │   │   └── api/                    # Generated Retrofit interfaces
│       │   ├── local/
│       │   │   ├── AppDatabase.kt          # Room database
│       │   │   └── dao/                    # DAOs per entity
│       │   └── repository/                 # Repository implementations
│       ├── domain/
│       │   ├── model/                      # Domain models
│       │   └── usecase/                    # Use cases
│       └── ui/
│           ├── theme/
│           │   └── Theme.kt               # Material 3 theme
│           ├── common/
│           │   ├── MarkdownRenderer.kt     # Compose Markdown
│           │   ├── DiffViewer.kt           # PR diff composable
│           │   └── SyntaxHighlighter.kt    # Code viewer
│           ├── auth/
│           │   ├── LoginScreen.kt
│           │   └── LoginViewModel.kt
│           ├── home/
│           │   ├── HomeScreen.kt
│           │   └── HomeViewModel.kt
│           ├── repo/
│           │   ├── RepoListScreen.kt
│           │   ├── RepoDetailScreen.kt
│           │   ├── FileTreeScreen.kt
│           │   ├── FileViewScreen.kt
│           │   └── RepoViewModel.kt
│           ├── issue/
│           │   ├── IssueListScreen.kt
│           │   ├── IssueDetailScreen.kt
│           │   └── IssueViewModel.kt
│           ├── pullrequest/
│           │   ├── PrListScreen.kt
│           │   ├── PrDetailScreen.kt
│           │   ├── DiffScreen.kt
│           │   └── PrViewModel.kt
│           ├── notification/
│           │   ├── NotificationListScreen.kt
│           │   └── NotificationViewModel.kt
│           └── profile/
│               ├── ProfileScreen.kt
│               └── ProfileViewModel.kt
├── src/main/res/
│   ├── values/
│   │   ├── strings.xml
│   │   ├── themes.xml
│   │   └── colors.xml
│   └── drawable/                           # Icons, logos
└── src/test/                               # Unit tests
```

## Multi-Instance Support

Unlike GitHub (single host), this app must support arbitrary Forgejo/Gitea instances:

```kotlin
data class Instance(
    val id: String,           // UUID
    val name: String,         // e.g., "Codeberg"
    val baseUrl: String,      // e.g., "https://codeberg.org"
    val tokenType: TokenType, // OAUTH2 or PAT
    val token: String,        // encrypted
    val userId: Long?,
    val username: String?,
    val avatarUrl: String?,
)
```

Each instance gets its own Retrofit client with the appropriate base URL and auth interceptor. Users can switch between instances from a drawer or account switcher.

## Notification Polling Strategy

Since Forgejo has no push notification infrastructure:

1. **WorkManager** schedules periodic work (minimum 15 min on Android)
2. Worker calls `GET /notifications?since={lastCheck}` for each active instance
3. New notifications are shown via Android NotificationManager
4. User-configurable poll interval (15, 30, 60 min)
5. Battery optimization: respect Doze mode, use expedited work only when charging

## Diff Rendering Strategy

PR diffs are the hardest UI component:

1. Fetch unified diff from `/repos/{owner}/{repo}/pulls/{index}.diff`
2. Parse into file-level hunks
3. Render as a lazy list of file cards, each expandable to show hunks
4. Color-code additions (green) and deletions (red)
5. Syntax highlight changed lines using a lightweight lexer
6. Support inline commenting (POST to PR review API)

## Testing Strategy

| Layer | Approach |
|-------|----------|
| ViewModels | JUnit + MockK + Turbine (Flow testing) |
| Use Cases | JUnit + MockK |
| Repositories | JUnit + MockWebServer (API) + Room in-memory DB |
| UI | Compose UI testing (AndroidComposeTestRule) |
| Integration | Espresso for critical flows (login, create issue) |
| CI | Forgejo Actions workflow running on Codeberg |

## Distribution

- **F-Droid**: Primary distribution (open source, no Google dependencies required)
- **Google Play Store**: Secondary, optional
- **GitHub/Codeberg Releases**: Direct APK downloads
- Consider a flavor without Google Play Services (replace FCM polling with pure WorkManager)
