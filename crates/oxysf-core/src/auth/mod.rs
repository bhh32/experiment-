//! Authentication flows and on-disk credential storage.
//!
//! This module groups the OAuth web-server (PKCE) flow, the JWT bearer flow,
//! the refresh-token grant, and the local credential store.

pub mod jwt;
pub mod refresh;
pub mod store;
pub mod web;

pub use store::AuthInfo;

/// The built-in Salesforce CLI connected-app client id used for the web flow.
///
/// This is the public `PlatformCLI` client id shipped by Salesforce for the
/// official CLI's OAuth web-server flow.
pub const PLATFORM_CLI_CLIENT_ID: &str = "PlatformCLI";

/// The default Salesforce login URL (production / Developer Edition).
pub const DEFAULT_LOGIN_URL: &str = "https://login.salesforce.com";

/// The shape of a successful OAuth token response from Salesforce.
#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct TokenResponse {
    pub access_token: String,
    #[serde(default)]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub instance_url: Option<String>,
    #[serde(default)]
    pub id: Option<String>,
}
