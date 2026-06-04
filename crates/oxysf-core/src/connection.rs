//! An authenticated HTTP connection to a Salesforce org.

use std::time::Duration;

use serde::Serialize;

use crate::auth::refresh;
use crate::auth::store::{self, AuthInfo};
use crate::config::DEFAULT_API_VERSION;
use crate::error::{Result, SfError};

/// An authenticated connection to a single Salesforce org.
///
/// Wraps a blocking `reqwest` client and transparently refreshes the access
/// token once on a `401` when a refresh token is available.
pub struct Connection {
    /// The org's instance URL (no trailing slash).
    pub instance_url: String,
    /// The current OAuth access token.
    pub access_token: String,
    /// The REST API version (e.g. `62.0`).
    pub api_version: String,
    client: reqwest::blocking::Client,
    refresh_token: Option<String>,
    client_id: String,
    login_url: String,
    /// The username this connection authenticates as (for persistence).
    username: String,
}

impl Connection {
    /// Build a [`Connection`] from stored [`AuthInfo`], using `api_version`
    /// (or the default when `None`).
    pub fn from_auth(info: &AuthInfo, api_version: Option<&str>) -> Result<Connection> {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .map_err(|e| SfError::Http(e.to_string()))?;
        Ok(Connection {
            instance_url: info.instance_url.trim_end_matches('/').to_string(),
            access_token: info.access_token.clone(),
            api_version: api_version.unwrap_or(DEFAULT_API_VERSION).to_string(),
            client,
            refresh_token: info.refresh_token.clone(),
            client_id: info.client_id.clone(),
            login_url: info.login_url.clone(),
            username: info.username.clone(),
        })
    }

    /// The base path for REST data resources for this API version.
    pub fn data_path(&self) -> String {
        format!("/services/data/v{}", self.api_version)
    }

    fn url_for(&self, path: &str) -> String {
        if path.starts_with("http://") || path.starts_with("https://") {
            path.to_string()
        } else {
            format!("{}{}", self.instance_url, path)
        }
    }

    /// Perform a `GET` returning parsed JSON, refreshing the token once on 401.
    pub fn get_json(&mut self, path: &str) -> Result<serde_json::Value> {
        self.send_json(reqwest::Method::GET, path, None)
    }

    /// Perform a `POST` with a JSON body, refreshing the token once on 401.
    pub fn post_json<B: Serialize>(&mut self, path: &str, body: &B) -> Result<serde_json::Value> {
        let value = serde_json::to_value(body)?;
        self.send_json(reqwest::Method::POST, path, Some(value))
    }

    fn send_json(
        &mut self,
        method: reqwest::Method,
        path: &str,
        body: Option<serde_json::Value>,
    ) -> Result<serde_json::Value> {
        let (status, text) = self.execute(method.clone(), path, body.clone())?;

        if status == 401 && self.try_refresh()? {
            let (status2, text2) = self.execute(method, path, body)?;
            return Self::interpret(status2, text2);
        }
        Self::interpret(status, text)
    }

    /// Send one request, returning the status code and body text.
    fn execute(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<serde_json::Value>,
    ) -> Result<(u16, String)> {
        let mut req = self
            .client
            .request(method, self.url_for(path))
            .bearer_auth(&self.access_token);
        if let Some(b) = body {
            req = req.json(&b);
        }
        let resp = req.send().map_err(|e| SfError::Http(e.to_string()))?;
        let status = resp.status().as_u16();
        let text = resp.text().map_err(|e| SfError::Http(e.to_string()))?;
        Ok((status, text))
    }

    /// Map a status/body into JSON or an [`SfError`].
    fn interpret(status: u16, text: String) -> Result<serde_json::Value> {
        if (200..300).contains(&status) {
            if text.is_empty() {
                return Ok(serde_json::Value::Null);
            }
            return Ok(serde_json::from_str(&text)?);
        }
        Err(SfError::from_rest_body(status, &text))
    }

    /// Attempt a single token refresh; persist and update on success.
    fn try_refresh(&mut self) -> Result<bool> {
        let Some(refresh_token) = self.refresh_token.clone() else {
            return Ok(false);
        };
        let refreshed =
            refresh::refresh_access_token(&self.client, &self.login_url, &self.client_id, &refresh_token)?;
        self.access_token = refreshed.access_token.clone();
        if let Some(url) = &refreshed.instance_url {
            self.instance_url = url.trim_end_matches('/').to_string();
        }

        // Persist the new token (best-effort; load existing record to update).
        if let Ok(mut info) = store::load(&self.username) {
            info.access_token = refreshed.access_token;
            if let Some(url) = refreshed.instance_url {
                info.instance_url = url;
            }
            store::save(&info)?;
        }
        Ok(true)
    }
}
