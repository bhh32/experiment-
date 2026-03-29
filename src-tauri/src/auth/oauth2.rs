use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthConfig {
    pub client_id: String,
    pub redirect_uri: String,
    pub authorize_url: String, // {base_url}/login/oauth/authorize
    pub token_url: String,     // {base_url}/login/oauth/access_token
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthToken {
    pub access_token: String,
    pub token_type: String,
    pub refresh_token: Option<String>,
    pub expires_in: Option<i64>,
}

impl OAuthConfig {
    pub fn for_instance(base_url: &str, client_id: &str) -> Self {
        let base = base_url.trim_end_matches('/');
        Self {
            client_id: client_id.to_string(),
            redirect_uri: "codeberg-app://oauth/callback".to_string(),
            authorize_url: format!("{base}/login/oauth/authorize"),
            token_url: format!("{base}/login/oauth/access_token"),
        }
    }

    // Build the authorization URL with PKCE challenge
    pub fn auth_url(&self, state: &str, code_challenge: &str) -> String {
        format!(
            "{}?client_id={}&redirect_uri={}&response_type=code&state={}&code_challenge={}&code_challenge_method=S256",
            self.authorize_url,
            self.client_id,
            self.redirect_uri,
            state,
            code_challenge,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oauth_config_urls() {
        let config = OAuthConfig::for_instance("https://codeberg.org", "my-app");
        assert_eq!(
            config.authorize_url,
            "https://codeberg.org/login/oauth/authorize"
        );
        assert_eq!(
            config.token_url,
            "https://codeberg.org/login/oauth/access_token"
        );
    }

    #[test]
    fn test_auth_url_contains_pkce() {
        let config = OAuthConfig::for_instance("https://codeberg.org", "my-app");
        let url = config.auth_url("random-state", "challenge123");
        assert!(url.contains("code_challenge=challenge123"));
        assert!(url.contains("code_challenge_method=S256"));
    }
}
