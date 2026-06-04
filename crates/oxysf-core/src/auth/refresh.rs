//! OAuth2 refresh-token grant: exchange a refresh token for a new access token.

use std::collections::HashMap;

use crate::error::{Result, SfError};

/// The result of a successful token refresh.
#[derive(Debug, Clone)]
pub struct RefreshedToken {
    /// The new access token.
    pub access_token: String,
    /// The instance URL returned by the token endpoint, if present.
    pub instance_url: Option<String>,
}

/// Perform a `refresh_token` grant against `login_url`.
///
/// `login_url` is the org's authorization server base (e.g.
/// `https://login.salesforce.com` or the org instance URL). On success the new
/// access token is returned; the caller is responsible for persisting it.
pub fn refresh_access_token(
    client: &reqwest::blocking::Client,
    login_url: &str,
    client_id: &str,
    refresh_token: &str,
) -> Result<RefreshedToken> {
    let token_url = format!("{}/services/oauth2/token", login_url.trim_end_matches('/'));

    let mut form = HashMap::new();
    form.insert("grant_type", "refresh_token");
    form.insert("client_id", client_id);
    form.insert("refresh_token", refresh_token);

    let resp = client
        .post(&token_url)
        .form(&form)
        .send()
        .map_err(|e| SfError::Http(e.to_string()))?;

    let status = resp.status();
    let body = resp.text().map_err(|e| SfError::Http(e.to_string()))?;

    if !status.is_success() {
        return Err(parse_oauth_error(status.as_u16(), &body));
    }

    let token: super::TokenResponse =
        serde_json::from_str(&body).map_err(|e| SfError::Auth(format!("invalid token response: {e}")))?;

    Ok(RefreshedToken {
        access_token: token.access_token,
        instance_url: token.instance_url,
    })
}

/// Parse an OAuth error body shaped like `{"error":..,"error_description":..}`.
pub(crate) fn parse_oauth_error(status: u16, body: &str) -> SfError {
    #[derive(serde::Deserialize)]
    struct OauthError {
        #[serde(default)]
        error: String,
        #[serde(default)]
        error_description: String,
    }

    if let Ok(e) = serde_json::from_str::<OauthError>(body) {
        if !e.error.is_empty() || !e.error_description.is_empty() {
            return SfError::Auth(format!("{}: {}", e.error, e.error_description));
        }
    }
    SfError::Http(format!("HTTP {status}: {body}"))
}
