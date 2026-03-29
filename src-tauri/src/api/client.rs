use crate::api::error::ApiError;
use crate::api::models::*;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::de::DeserializeOwned;

pub struct ForgejoClient {
    http: Client,
    base_url: String,
    token: String,
}

impl ForgejoClient {
    pub fn new(base_url: &str, token: &str) -> Self {
        Self {
            http: Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            token: token.to_string(),
        }
    }

    // Build the full API URL for a given path
    fn url(&self, path: &str) -> String {
        format!("{}/api/v1{}", self.base_url, path)
    }

    // Send a GET request and deserialize the response
    async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, ApiError> {
        let response = self.http
            .get(self.url(path))
            .header("Authorization", format!("token {}", self.token))
            .send()
            .await?;

        Self::handle_response(response).await
    }

    // Send a POST request with a JSON body
    async fn post<B: serde::Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, ApiError> {
        let response = self.http
            .post(self.url(path))
            .header("Authorization", format!("token {}", self.token))
            .json(body)
            .send()
            .await?;

        Self::handle_response(response).await
    }

    // Check status and deserialize, or return an appropriate error
    async fn handle_response<T: DeserializeOwned>(
        response: reqwest::Response,
    ) -> Result<T, ApiError> {
        let status = response.status();

        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(ApiError::Unauthorized);
        }

        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(ApiError::NotFound(response.url().to_string()));
        }

        if !status.is_success() {
            let message = response.text().await.unwrap_or_default();
            return Err(ApiError::Api {
                status: status.as_u16(),
                message,
            });
        }

        let body = response.text().await?;
        Ok(serde_json::from_str(&body)?)
    }

    // ---- User endpoints ----

    pub async fn current_user(&self) -> Result<User, ApiError> {
        self.get("/user").await
    }

    pub async fn get_user(&self, username: &str) -> Result<User, ApiError> {
        self.get(&format!("/users/{username}")).await
    }

    pub async fn list_user_orgs(&self) -> Result<Vec<Organization>, ApiError> {
        self.get("/user/orgs").await
    }

    // ---- Repository endpoints ----

    pub async fn list_repos(&self, page: i64, limit: i64) -> Result<Vec<Repository>, ApiError> {
        self.get(&format!("/user/repos?page={page}&limit={limit}")).await
    }

    pub async fn get_repo(&self, owner: &str, repo: &str) -> Result<Repository, ApiError> {
        self.get(&format!("/repos/{owner}/{repo}")).await
    }

    pub async fn search_repos(&self, query: &str) -> Result<Vec<Repository>, ApiError> {
        #[derive(serde::Deserialize)]
        struct SearchResult {
            data: Vec<Repository>,
        }
        let result: SearchResult = self.get(&format!("/repos/search?q={query}")).await?;
        Ok(result.data)
    }

    // ---- Issue endpoints ----

    pub async fn list_issues(
        &self,
        owner: &str,
        repo: &str,
        state: &str,
        page: i64,
    ) -> Result<Vec<Issue>, ApiError> {
        self.get(&format!(
            "/repos/{owner}/{repo}/issues?state={state}&page={page}&type=issues"
        )).await
    }

    pub async fn get_issue(
        &self,
        owner: &str,
        repo: &str,
        index: i64,
    ) -> Result<Issue, ApiError> {
        self.get(&format!("/repos/{owner}/{repo}/issues/{index}")).await
    }

    pub async fn create_issue(
        &self,
        owner: &str,
        repo: &str,
        body: &CreateIssueRequest,
    ) -> Result<Issue, ApiError> {
        self.post(&format!("/repos/{owner}/{repo}/issues"), body).await
    }

    pub async fn post_comment(
        &self,
        owner: &str,
        repo: &str,
        index: i64,
        body: &CreateCommentRequest,
    ) -> Result<Comment, ApiError> {
        self.post(
            &format!("/repos/{owner}/{repo}/issues/{index}/comments"),
            body,
        ).await
    }

    // ---- Pull request endpoints ----

    pub async fn list_pulls(
        &self,
        owner: &str,
        repo: &str,
        state: &str,
        page: i64,
    ) -> Result<Vec<PullRequest>, ApiError> {
        self.get(&format!(
            "/repos/{owner}/{repo}/pulls?state={state}&page={page}"
        )).await
    }

    pub async fn get_pull_diff(
        &self,
        owner: &str,
        repo: &str,
        index: i64,
    ) -> Result<String, ApiError> {
        let response = self.http
            .get(self.url(&format!("/repos/{owner}/{repo}/pulls/{index}.diff")))
            .header("Authorization", format!("token {}", self.token))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(ApiError::Api {
                status: response.status().as_u16(),
                message: response.text().await.unwrap_or_default(),
            });
        }

        Ok(response.text().await?)
    }

    pub async fn merge_pull(
        &self,
        owner: &str,
        repo: &str,
        index: i64,
        method: &str,
    ) -> Result<(), ApiError> {
        let body = MergePullRequest {
            method: method.to_string(),
        };

        let response = self.http
            .post(self.url(&format!("/repos/{owner}/{repo}/pulls/{index}/merge")))
            .header("Authorization", format!("token {}", self.token))
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(ApiError::Api {
                status: response.status().as_u16(),
                message: response.text().await.unwrap_or_default(),
            });
        }

        Ok(())
    }

    // ---- Notification endpoints ----

    pub async fn list_notifications(
        &self,
        since: Option<DateTime<Utc>>,
    ) -> Result<Vec<Notification>, ApiError> {
        let path = match since {
            Some(ts) => format!("/notifications?since={}", ts.to_rfc3339()),
            None => "/notifications".to_string(),
        };
        self.get(&path).await
    }

    pub async fn mark_notifications_read(&self) -> Result<(), ApiError> {
        let response = self.http
            .put(self.url("/notifications"))
            .header("Authorization", format!("token {}", self.token))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(ApiError::Api {
                status: response.status().as_u16(),
                message: response.text().await.unwrap_or_default(),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_construction() {
        let client = ForgejoClient::new("https://codeberg.org", "test-token");
        assert_eq!(
            client.url("/user"),
            "https://codeberg.org/api/v1/user"
        );
    }

    #[test]
    fn test_url_strips_trailing_slash() {
        let client = ForgejoClient::new("https://codeberg.org/", "test-token");
        assert_eq!(
            client.url("/repos/search"),
            "https://codeberg.org/api/v1/repos/search"
        );
    }
}
