use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub login: String,
    pub full_name: String,
    pub avatar_url: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub id: i64,
    pub name: String,
    pub full_name: String,
    pub description: String,
    pub html_url: String,
    pub clone_url: String,
    pub default_branch: String,
    pub stars_count: i64,
    pub forks_count: i64,
    pub open_issues_count: i64,
    pub updated_at: DateTime<Utc>,
    pub owner: User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub id: i64,
    pub number: i64,
    pub title: String,
    pub body: String,
    pub state: String,           // "open" or "closed"
    pub html_url: String,
    pub user: User,
    pub labels: Vec<Label>,
    pub assignees: Vec<User>,
    pub comments: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    pub id: i64,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    pub id: i64,
    pub number: i64,
    pub title: String,
    pub body: String,
    pub state: String,           // "open", "closed"
    pub merged: bool,
    pub html_url: String,
    pub user: User,
    pub labels: Vec<Label>,
    pub assignees: Vec<User>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: i64,
    pub body: String,
    pub user: User,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: i64,
    pub subject: NotificationSubject,
    pub repository: Repository,
    pub unread: bool,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSubject {
    pub title: String,
    #[serde(rename = "type")]
    pub kind: String,            // "Issue", "Pull", "Commit", etc.
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    pub id: i64,
    pub username: String,
    pub full_name: String,
    pub avatar_url: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileContent {
    pub name: String,
    pub path: String,
    #[serde(rename = "type")]
    pub kind: String,            // "file" or "dir"
    pub content: Option<String>, // base64 encoded for files
    pub encoding: Option<String>,
    pub size: i64,
}

// Request bodies

#[derive(Debug, Serialize)]
pub struct CreateIssueRequest {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<i64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignees: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct CreateCommentRequest {
    pub body: String,
}

#[derive(Debug, Serialize)]
pub struct MergePullRequest {
    #[serde(rename = "Do")]
    pub method: String,          // "merge", "rebase", or "squash"
}
