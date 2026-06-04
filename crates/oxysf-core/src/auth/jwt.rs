//! OAuth2 JWT bearer flow.
//!
//! Builds an RS256-signed JWT asserting the identity of `username` for the
//! connected app `client_id`, then exchanges it at `/services/oauth2/token`
//! using the `jwt-bearer` grant.

use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use jsonwebtoken::{Algorithm, EncodingKey, Header};
use serde::Serialize;

use crate::auth::store::AuthInfo;
use crate::auth::DEFAULT_LOGIN_URL;
use crate::error::{Result, SfError};

/// The JWT claim set for the bearer assertion.
#[derive(Debug, Serialize)]
pub struct JwtClaims {
    /// Issuer: the connected-app consumer key (client id).
    pub iss: String,
    /// Subject: the username to authenticate as.
    pub sub: String,
    /// Audience: the login URL.
    pub aud: String,
    /// Expiration (unix seconds).
    pub exp: u64,
}

impl JwtClaims {
    /// Build claims expiring `lifetime` from `now`.
    pub fn new(client_id: &str, username: &str, login_url: &str, now: SystemTime, lifetime: Duration) -> JwtClaims {
        let exp = now
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .saturating_add(lifetime)
            .as_secs();
        JwtClaims {
            iss: client_id.to_string(),
            sub: username.to_string(),
            aud: login_url.to_string(),
            exp,
        }
    }
}

/// Sign a JWT assertion with an RS256 PEM private key (in-memory bytes).
pub fn sign_assertion(claims: &JwtClaims, pem: &[u8]) -> Result<String> {
    let key = EncodingKey::from_rsa_pem(pem)
        .map_err(|e| SfError::Auth(format!("invalid RSA private key: {e}")))?;
    let header = Header::new(Algorithm::RS256);
    jsonwebtoken::encode(&header, claims, &key)
        .map_err(|e| SfError::Auth(format!("failed to sign JWT: {e}")))
}

/// Run the JWT bearer flow and return the resulting [`AuthInfo`].
///
/// `key_path` is the path to a PEM RSA private key file. `login_url` defaults
/// to [`DEFAULT_LOGIN_URL`] when `None`.
pub fn login(
    username: &str,
    client_id: &str,
    key_path: &std::path::Path,
    login_url: Option<&str>,
) -> Result<AuthInfo> {
    let login_url = login_url.unwrap_or(DEFAULT_LOGIN_URL).to_string();
    let pem = std::fs::read(key_path)?;

    let claims = JwtClaims::new(
        client_id,
        username,
        &login_url,
        SystemTime::now(),
        Duration::from_secs(180),
    );
    let assertion = sign_assertion(&claims, &pem)?;

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(|e| SfError::Http(e.to_string()))?;

    exchange_assertion(&client, &login_url, client_id, username, &assertion)
}

/// Exchange a signed JWT assertion for an access token.
fn exchange_assertion(
    client: &reqwest::blocking::Client,
    login_url: &str,
    client_id: &str,
    username: &str,
    assertion: &str,
) -> Result<AuthInfo> {
    let token_url = format!("{}/services/oauth2/token", login_url.trim_end_matches('/'));
    let mut form = HashMap::new();
    form.insert("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer");
    form.insert("assertion", assertion);

    let resp = client
        .post(&token_url)
        .form(&form)
        .send()
        .map_err(|e| SfError::Http(e.to_string()))?;
    let status = resp.status();
    let body = resp.text().map_err(|e| SfError::Http(e.to_string()))?;
    if !status.is_success() {
        return Err(super::refresh::parse_oauth_error(status.as_u16(), &body));
    }

    let token: super::TokenResponse =
        serde_json::from_str(&body).map_err(|e| SfError::Auth(format!("invalid token response: {e}")))?;
    let instance_url = token
        .instance_url
        .clone()
        .ok_or_else(|| SfError::Auth("token response missing instance_url".into()))?;

    // JWT flow yields no refresh token; org id can be parsed from the id URL.
    let org_id = token.id.as_deref().and_then(org_id_from_identity_url);

    Ok(AuthInfo {
        username: username.to_string(),
        instance_url,
        access_token: token.access_token,
        refresh_token: None,
        client_id: client_id.to_string(),
        login_url: login_url.to_string(),
        alias: None,
        org_id,
    })
}

/// Extract the 18-char org id from an identity URL of the shape
/// `https://login.salesforce.com/id/{orgId}/{userId}`.
fn org_id_from_identity_url(id_url: &str) -> Option<String> {
    let parts: Vec<&str> = id_url.trim_end_matches('/').rsplit('/').collect();
    // parts[0] = userId, parts[1] = orgId
    parts.get(1).map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use base64::Engine as _;

    /// A throwaway 2048-bit RSA private key used only for tests.
    fn test_rsa_pem() -> Vec<u8> {
        use rsa::pkcs8::EncodePrivateKey;
        use rsa::RsaPrivateKey;
        let mut rng = rand::thread_rng();
        let key = RsaPrivateKey::new(&mut rng, 2048).expect("generate key");
        key.to_pkcs8_pem(rsa::pkcs8::LineEnding::LF)
            .unwrap()
            .as_bytes()
            .to_vec()
    }

    #[test]
    fn signs_rs256_three_segment_token() {
        let pem = test_rsa_pem();
        let claims = JwtClaims::new(
            "myConsumerKey",
            "user@example.com",
            "https://login.salesforce.com",
            SystemTime::now(),
            Duration::from_secs(180),
        );
        let token = sign_assertion(&claims, &pem).unwrap();

        let segments: Vec<&str> = token.split('.').collect();
        assert_eq!(segments.len(), 3, "JWT must have 3 segments");

        // Decode header and confirm alg=RS256.
        let header_json = URL_SAFE_NO_PAD.decode(segments[0]).unwrap();
        let header: serde_json::Value = serde_json::from_slice(&header_json).unwrap();
        assert_eq!(header["alg"], "RS256");

        // Confirm the claims round-trip.
        let claims_json = URL_SAFE_NO_PAD.decode(segments[1]).unwrap();
        let parsed: serde_json::Value = serde_json::from_slice(&claims_json).unwrap();
        assert_eq!(parsed["iss"], "myConsumerKey");
        assert_eq!(parsed["sub"], "user@example.com");
        assert_eq!(parsed["aud"], "https://login.salesforce.com");
        assert!(parsed["exp"].as_u64().unwrap() > 0);
    }

    #[test]
    fn parses_org_id_from_identity_url() {
        let id = "https://login.salesforce.com/id/00Dxx0000001gPLEAY/005xx000001Sv6dAAC";
        assert_eq!(org_id_from_identity_url(id).as_deref(), Some("00Dxx0000001gPLEAY"));
    }
}
