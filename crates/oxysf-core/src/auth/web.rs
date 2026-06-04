//! OAuth2 web-server flow with PKCE against the built-in `PlatformCLI` client.
//!
//! The flow:
//! 1. Generate a PKCE verifier + S256 challenge.
//! 2. Build the `/services/oauth2/authorize` URL.
//! 3. Start a localhost server on `127.0.0.1:1717`, open the browser.
//! 4. Capture the `code` from the redirect.
//! 5. Exchange the code (with `code_verifier`) at `/services/oauth2/token`.
//! 6. Resolve the username via the OAuth `userinfo` endpoint.

use std::collections::HashMap;
use std::time::Duration;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use rand::Rng;
use sha2::{Digest, Sha256};

use crate::auth::store::AuthInfo;
use crate::auth::{DEFAULT_LOGIN_URL, PLATFORM_CLI_CLIENT_ID};
use crate::error::{Result, SfError};

/// The localhost redirect URI registered for the CLI connected app.
pub const REDIRECT_URI: &str = "http://localhost:1717/OauthRedirect";
const REDIRECT_PORT: u16 = 1717;

/// A PKCE verifier/challenge pair.
#[derive(Debug, Clone)]
pub struct Pkce {
    /// The high-entropy verifier (43-128 chars).
    pub verifier: String,
    /// The base64url(S256(verifier)) challenge.
    pub challenge: String,
}

impl Pkce {
    /// Generate a new PKCE pair using an S256 challenge.
    pub fn generate() -> Pkce {
        // Unreserved characters allowed in a PKCE verifier per RFC 7636.
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~";
        let mut rng = rand::thread_rng();
        // 64 chars is comfortably within the 43..=128 allowed range.
        let verifier: String = (0..64)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();
        let challenge = Self::challenge_for(&verifier);
        Pkce { verifier, challenge }
    }

    /// Compute the S256 challenge for a given verifier.
    pub fn challenge_for(verifier: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(verifier.as_bytes());
        let digest = hasher.finalize();
        URL_SAFE_NO_PAD.encode(digest)
    }
}

/// Build the authorization URL for the web-server flow.
pub fn build_authorize_url(login_url: &str, client_id: &str, challenge: &str) -> String {
    let base = login_url.trim_end_matches('/');
    let query = [
        ("response_type", "code"),
        ("client_id", client_id),
        ("redirect_uri", REDIRECT_URI),
        ("code_challenge", challenge),
        ("code_challenge_method", "S256"),
        ("prompt", "login"),
    ];
    let encoded: Vec<String> = query
        .iter()
        .map(|(k, v)| format!("{}={}", k, urlencode(v)))
        .collect();
    format!("{base}/services/oauth2/authorize?{}", encoded.join("&"))
}

/// Minimal percent-encoding for query parameter values.
fn urlencode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// Run the full interactive web-server OAuth flow and return the resulting
/// [`AuthInfo`].
///
/// `login_url` defaults to [`DEFAULT_LOGIN_URL`] when `None`. This opens the
/// user's browser and blocks until the redirect is received (or it times out).
pub fn login(login_url: Option<&str>) -> Result<AuthInfo> {
    let login_url = login_url.unwrap_or(DEFAULT_LOGIN_URL).to_string();
    let client_id = PLATFORM_CLI_CLIENT_ID.to_string();
    let pkce = Pkce::generate();

    let authorize_url = build_authorize_url(&login_url, &client_id, &pkce.challenge);

    let server = tiny_http::Server::http(("127.0.0.1", REDIRECT_PORT))
        .map_err(|e| SfError::Auth(format!("failed to start local redirect server: {e}")))?;

    // Best-effort browser open; the URL is also printed for manual use.
    eprintln!("Opening browser for login. If it does not open, visit:\n{authorize_url}");
    let _ = webbrowser::open(&authorize_url);

    let code = wait_for_code(&server)?;

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(|e| SfError::Http(e.to_string()))?;

    exchange_code(&client, &login_url, &client_id, &code, &pkce.verifier)
}

/// Block on the local server until the OAuth redirect delivers a `code`.
fn wait_for_code(server: &tiny_http::Server) -> Result<String> {
    for request in server.incoming_requests() {
        let url = request.url().to_string();
        if let Some(code) = extract_query_param(&url, "code") {
            let body = "<html><body><h2>oxysf: authentication complete.</h2>\
                You may close this window and return to your terminal.</body></html>";
            let response = tiny_http::Response::from_string(body).with_header(
                tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap(),
            );
            let _ = request.respond(response);
            return Ok(code);
        }
        if let Some(err) = extract_query_param(&url, "error") {
            let _ = request.respond(tiny_http::Response::from_string("Authentication failed."));
            return Err(SfError::Auth(format!("authorization denied: {err}")));
        }
        // Ignore unrelated requests (e.g. favicon) and keep waiting.
        let _ = request.respond(tiny_http::Response::from_string("Waiting for authorization..."));
    }
    Err(SfError::Auth("redirect server closed before receiving a code".into()))
}

/// Extract a single query parameter value from a request URL/path.
pub fn extract_query_param(url: &str, key: &str) -> Option<String> {
    let query = url.split_once('?').map(|(_, q)| q).unwrap_or("");
    for pair in query.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            if k == key {
                return Some(percent_decode(v));
            }
        }
    }
    None
}

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' if i + 2 < bytes.len() => {
                let hi = (bytes[i + 1] as char).to_digit(16);
                let lo = (bytes[i + 2] as char).to_digit(16);
                if let (Some(hi), Some(lo)) = (hi, lo) {
                    out.push((hi * 16 + lo) as u8);
                    i += 3;
                    continue;
                }
                out.push(b'%');
                i += 1;
            }
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Exchange an authorization code for tokens and resolve the username.
fn exchange_code(
    client: &reqwest::blocking::Client,
    login_url: &str,
    client_id: &str,
    code: &str,
    verifier: &str,
) -> Result<AuthInfo> {
    let token_url = format!("{}/services/oauth2/token", login_url.trim_end_matches('/'));
    let mut form = HashMap::new();
    form.insert("grant_type", "authorization_code");
    form.insert("code", code);
    form.insert("client_id", client_id);
    form.insert("redirect_uri", REDIRECT_URI);
    form.insert("code_verifier", verifier);

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

    let (username, org_id) = fetch_identity(client, &instance_url, &token.access_token, token.id.as_deref());

    Ok(AuthInfo {
        username,
        instance_url,
        access_token: token.access_token,
        refresh_token: token.refresh_token,
        client_id: client_id.to_string(),
        login_url: login_url.to_string(),
        alias: None,
        org_id,
    })
}

/// Query the identity / userinfo endpoint to resolve username and org id.
///
/// Falls back gracefully when the request fails so login still succeeds.
fn fetch_identity(
    client: &reqwest::blocking::Client,
    instance_url: &str,
    access_token: &str,
    id_url: Option<&str>,
) -> (String, Option<String>) {
    let url = id_url
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("{}/services/oauth2/userinfo", instance_url.trim_end_matches('/')));

    let resp = client
        .get(&url)
        .bearer_auth(access_token)
        .send()
        .ok()
        .and_then(|r| r.json::<serde_json::Value>().ok());

    match resp {
        Some(v) => {
            let username = v
                .get("preferred_username")
                .or_else(|| v.get("username"))
                .and_then(|x| x.as_str())
                .unwrap_or("unknown")
                .to_string();
            let org_id = v
                .get("organization_id")
                .and_then(|x| x.as_str())
                .map(|s| s.to_string());
            (username, org_id)
        }
        None => ("unknown".to_string(), None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pkce_challenge_is_s256_of_verifier() {
        let pkce = Pkce::generate();
        assert!(pkce.verifier.len() >= 43 && pkce.verifier.len() <= 128);
        // Recompute the challenge and confirm it matches.
        let expected = Pkce::challenge_for(&pkce.verifier);
        assert_eq!(pkce.challenge, expected);

        // Challenge must be valid base64url (no padding) decoding to 32 bytes.
        let decoded = URL_SAFE_NO_PAD.decode(pkce.challenge.as_bytes()).unwrap();
        assert_eq!(decoded.len(), 32);
        assert!(!pkce.challenge.contains('='));
        assert!(!pkce.challenge.contains('+'));
        assert!(!pkce.challenge.contains('/'));
    }

    #[test]
    fn known_pkce_vector() {
        // RFC 7636 Appendix B test vector.
        let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        let challenge = Pkce::challenge_for(verifier);
        assert_eq!(challenge, "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM");
    }

    #[test]
    fn authorize_url_has_expected_params() {
        let url = build_authorize_url("https://login.salesforce.com", "PlatformCLI", "abc123");
        assert!(url.starts_with("https://login.salesforce.com/services/oauth2/authorize?"));
        assert!(url.contains("response_type=code"));
        assert!(url.contains("client_id=PlatformCLI"));
        assert!(url.contains("code_challenge=abc123"));
        assert!(url.contains("code_challenge_method=S256"));
        assert!(url.contains("redirect_uri=http%3A%2F%2Flocalhost%3A1717%2FOauthRedirect"));
    }

    #[test]
    fn extracts_query_params() {
        let url = "/OauthRedirect?code=abc%20123&state=x";
        assert_eq!(extract_query_param(url, "code").as_deref(), Some("abc 123"));
        assert_eq!(extract_query_param(url, "missing"), None);
    }
}
