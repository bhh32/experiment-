//! Tailscale LocalAPI client.
//!
//! Communicates with the `tailscaled` daemon via its Unix socket at
//! `/var/run/tailscale/tailscaled.sock` using HTTP-over-UDS.
//!
//! This replaces all `Command::new("tailscale")` calls, making the applet:
//! - Flatpak-safe (no CLI binary needed, just socket access)
//! - More robust (structured JSON instead of regex-parsed CLI text)
//! - Faster (no process spawn overhead per query)

use std::fmt;

use hyper_util::rt::TokioIo;
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::Request;
use serde::{Deserialize, Serialize};
use tokio::net::UnixStream;

/// Default path to the tailscaled Unix socket.
const DEFAULT_SOCKET_PATH: &str = "/var/run/tailscale/tailscaled.sock";

/// The host header value expected by tailscaled.
const LOCAL_API_HOST: &str = "local-tailscaled.sock";

// ─── Error Types ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum TailscaleError {
    /// The tailscaled socket was not found — daemon likely not running.
    SocketNotFound,
    /// Connection to the socket was refused.
    ConnectionRefused(String),
    /// The HTTP request failed.
    RequestFailed(String),
    /// The response could not be parsed.
    ParseError(String),
    /// The API returned an error status code.
    ApiError(u16, String),
    /// Operator permission not set.
    OperatorNotSet,
}

impl fmt::Display for TailscaleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TailscaleError::SocketNotFound => write!(
                f,
                "Tailscale daemon not found. Is tailscaled running?\n\
                 Socket not found at {DEFAULT_SOCKET_PATH}\n\
                 Start it with: sudo systemctl start tailscaled"
            ),
            TailscaleError::ConnectionRefused(e) => {
                write!(f, "Cannot connect to tailscaled: {e}")
            }
            TailscaleError::RequestFailed(e) => write!(f, "Request failed: {e}"),
            TailscaleError::ParseError(e) => write!(f, "Parse error: {e}"),
            TailscaleError::ApiError(code, body) => {
                write!(f, "API error (HTTP {code}): {body}")
            }
            TailscaleError::OperatorNotSet => write!(
                f,
                "Tailscale operator not set for your user.\n\
                 Run: sudo tailscale set --operator=$USER"
            ),
        }
    }
}

pub type TsResult<T> = Result<T, TailscaleError>;

// ─── API Client ──────────────────────────────────────────────────────────────

/// A client for the Tailscale LocalAPI over Unix socket.
#[derive(Clone)]
pub struct TailscaleClient {
    socket_path: String,
}

impl TailscaleClient {
    /// Create a new client using the default socket path.
    pub fn new() -> Self {
        Self {
            socket_path: DEFAULT_SOCKET_PATH.to_string(),
        }
    }

    /// Check if the tailscaled socket exists.
    pub fn is_available(&self) -> bool {
        std::path::Path::new(&self.socket_path).exists()
    }

    /// Send a GET request to the LocalAPI.
    async fn get(&self, path: &str) -> TsResult<String> {
        self.request("GET", path, None).await
    }

    /// Send a POST request to the LocalAPI.
    async fn post(&self, path: &str, body: Option<String>) -> TsResult<String> {
        self.request("POST", path, body).await
    }

    /// Send a PATCH request to the LocalAPI.
    async fn patch(&self, path: &str, body: Option<String>) -> TsResult<String> {
        self.request("PATCH", path, body).await
    }

    /// Core HTTP request over Unix socket.
    async fn request(
        &self,
        method: &str,
        path: &str,
        body: Option<String>,
    ) -> TsResult<String> {
        // Check socket exists first
        if !std::path::Path::new(&self.socket_path).exists() {
            return Err(TailscaleError::SocketNotFound);
        }

        // Connect to the Unix socket
        let stream = UnixStream::connect(&self.socket_path)
            .await
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    TailscaleError::OperatorNotSet
                } else {
                    TailscaleError::ConnectionRefused(e.to_string())
                }
            })?;

        let io = TokioIo::new(stream);

        // Create the HTTP connection
        let (mut sender, conn) = hyper::client::conn::http1::handshake(io)
            .await
            .map_err(|e| TailscaleError::RequestFailed(e.to_string()))?;

        // Spawn the connection handler
        tokio::spawn(async move {
            if let Err(e) = conn.await {
                eprintln!("LocalAPI connection error: {e}");
            }
        });

        // Build the request
        let uri = format!("http://{LOCAL_API_HOST}{path}");
        let req_body = match body {
            Some(b) => Full::new(Bytes::from(b)),
            None => Full::new(Bytes::new()),
        };

        let req = Request::builder()
            .method(method)
            .uri(&uri)
            .header("Host", LOCAL_API_HOST)
            .body(req_body)
            .map_err(|e| TailscaleError::RequestFailed(e.to_string()))?;

        // Send the request
        let response = sender
            .send_request(req)
            .await
            .map_err(|e| TailscaleError::RequestFailed(e.to_string()))?;

        let status = response.status().as_u16();

        // Read the response body
        let body_bytes = response
            .into_body()
            .collect()
            .await
            .map_err(|e| TailscaleError::RequestFailed(e.to_string()))?
            .to_bytes();

        let body_str = String::from_utf8_lossy(&body_bytes).to_string();

        if status == 403 {
            return Err(TailscaleError::OperatorNotSet);
        }

        if status >= 400 {
            return Err(TailscaleError::ApiError(status, body_str));
        }

        Ok(body_str)
    }

    // ─── Status & Info Queries ───────────────────────────────────────────

    /// Get the full tailscale status (equivalent to `tailscale status --json`).
    pub async fn status(&self) -> TsResult<Status> {
        let body = self.get("/localapi/v0/status").await?;
        serde_json::from_str(&body)
            .map_err(|e| TailscaleError::ParseError(format!("status: {e}")))
    }

    /// Get current preferences.
    pub async fn prefs(&self) -> TsResult<Prefs> {
        let body = self.get("/localapi/v0/prefs").await?;
        serde_json::from_str(&body)
            .map_err(|e| TailscaleError::ParseError(format!("prefs: {e}")))
    }

    /// Get available profiles (accounts).
    pub async fn profiles(&self) -> TsResult<Vec<Profile>> {
        let body = self.get("/localapi/v0/profiles/").await?;
        serde_json::from_str(&body)
            .map_err(|e| TailscaleError::ParseError(format!("profiles: {e}")))
    }

    /// Get the current profile.
    pub async fn current_profile(&self) -> TsResult<Profile> {
        let body = self.get("/localapi/v0/profiles/current").await?;
        serde_json::from_str(&body)
            .map_err(|e| TailscaleError::ParseError(format!("current profile: {e}")))
    }

    // ─── Mutation Operations ─────────────────────────────────────────────

    /// Update preferences (equivalent to `tailscale set ...`).
    /// Accepts a partial Prefs JSON and merges it.
    pub async fn set_prefs(&self, prefs: &PrefsUpdate) -> TsResult<Prefs> {
        let body = serde_json::to_string(prefs)
            .map_err(|e| TailscaleError::ParseError(e.to_string()))?;
        let response = self.patch("/localapi/v0/prefs", Some(body)).await?;
        serde_json::from_str(&response)
            .map_err(|e| TailscaleError::ParseError(format!("set_prefs response: {e}")))
    }

    /// Interactive login (opens browser).
    pub async fn login_interactive(&self) -> TsResult<()> {
        self.post("/localapi/v0/login-interactive", None).await?;
        Ok(())
    }

    /// Switch to a different profile/account.
    pub async fn switch_profile(&self, profile_id: &str) -> TsResult<()> {
        self.post(
            &format!("/localapi/v0/profiles/{profile_id}"),
            None,
        )
        .await?;
        Ok(())
    }

    /// Ping a peer.
    pub async fn ping(&self, ip: &str, ping_type: &str) -> TsResult<PingResult> {
        let body = self
            .post(
                &format!("/localapi/v0/ping?ip={ip}&type={ping_type}"),
                None,
            )
            .await?;
        serde_json::from_str(&body)
            .map_err(|e| TailscaleError::ParseError(format!("ping: {e}")))
    }

    /// Send a file via Tail Drop (PUT file content).
    pub async fn file_put(
        &self,
        peer_id: &str,
        filename: &str,
        content: Vec<u8>,
    ) -> TsResult<()> {
        let path = format!(
            "/localapi/v0/file-put/{peer_id}/{filename}"
        );

        // For file-put we need to send raw bytes, so use the low-level request
        if !std::path::Path::new(&self.socket_path).exists() {
            return Err(TailscaleError::SocketNotFound);
        }

        let stream = UnixStream::connect(&self.socket_path)
            .await
            .map_err(|e| TailscaleError::ConnectionRefused(e.to_string()))?;

        let io = TokioIo::new(stream);

        let (mut sender, conn) = hyper::client::conn::http1::handshake(io)
            .await
            .map_err(|e| TailscaleError::RequestFailed(e.to_string()))?;

        tokio::spawn(async move {
            if let Err(e) = conn.await {
                eprintln!("file-put connection error: {e}");
            }
        });

        let uri = format!("http://{LOCAL_API_HOST}{path}");
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Host", LOCAL_API_HOST)
            .header("Content-Length", content.len())
            .body(Full::new(Bytes::from(content)))
            .map_err(|e| TailscaleError::RequestFailed(e.to_string()))?;

        let response = sender
            .send_request(req)
            .await
            .map_err(|e| TailscaleError::RequestFailed(e.to_string()))?;

        let status = response.status().as_u16();
        if status >= 400 {
            let body_bytes = response
                .into_body()
                .collect()
                .await
                .map_err(|e| TailscaleError::RequestFailed(e.to_string()))?
                .to_bytes();
            let body_str = String::from_utf8_lossy(&body_bytes).to_string();
            return Err(TailscaleError::ApiError(status, body_str));
        }

        Ok(())
    }

    /// Retrieve waiting files from Tail Drop inbox.
    pub async fn waiting_files(&self) -> TsResult<Vec<WaitingFile>> {
        let body = self.get("/localapi/v0/files/").await?;
        serde_json::from_str(&body).map_err(|e| {
            // Empty response means no files
            if body.is_empty() || body == "null" {
                return TailscaleError::ParseError("no files".into());
            }
            TailscaleError::ParseError(format!("files: {e}"))
        })
    }

    /// Download a specific file from Tail Drop inbox.
    pub async fn file_get(&self, filename: &str) -> TsResult<Vec<u8>> {
        // For file-get we need raw bytes back
        if !std::path::Path::new(&self.socket_path).exists() {
            return Err(TailscaleError::SocketNotFound);
        }

        let stream = UnixStream::connect(&self.socket_path)
            .await
            .map_err(|e| TailscaleError::ConnectionRefused(e.to_string()))?;

        let io = TokioIo::new(stream);

        let (mut sender, conn) = hyper::client::conn::http1::handshake(io)
            .await
            .map_err(|e| TailscaleError::RequestFailed(e.to_string()))?;

        tokio::spawn(async move {
            if let Err(e) = conn.await {
                eprintln!("file-get connection error: {e}");
            }
        });

        let uri = format!(
            "http://{LOCAL_API_HOST}/localapi/v0/files/{filename}"
        );
        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("Host", LOCAL_API_HOST)
            .body(Full::new(Bytes::new()))
            .map_err(|e| TailscaleError::RequestFailed(e.to_string()))?;

        let response = sender
            .send_request(req)
            .await
            .map_err(|e| TailscaleError::RequestFailed(e.to_string()))?;

        let status = response.status().as_u16();
        let body_bytes = response
            .into_body()
            .collect()
            .await
            .map_err(|e| TailscaleError::RequestFailed(e.to_string()))?
            .to_bytes();

        if status >= 400 {
            return Err(TailscaleError::ApiError(
                status,
                String::from_utf8_lossy(&body_bytes).to_string(),
            ));
        }

        Ok(body_bytes.to_vec())
    }

    /// Delete a file from the Tail Drop inbox (after downloading).
    pub async fn file_delete(&self, filename: &str) -> TsResult<()> {
        self.request(
            "DELETE",
            &format!("/localapi/v0/files/{filename}"),
            None,
        )
        .await?;
        Ok(())
    }

    /// Set the exit node to use. Pass empty string to clear.
    pub async fn set_exit_node(&self, node_ip: &str) -> TsResult<Prefs> {
        let prefs = PrefsUpdate {
            exit_node_ip: Some(node_ip.to_string()),
            ..Default::default()
        };
        self.set_prefs(&prefs).await
    }

    /// Enable/disable this host as an exit node.
    pub async fn set_advertise_exit_node(&self, advertise: bool) -> TsResult<Prefs> {
        let prefs = if advertise {
            PrefsUpdate {
                advertise_routes: Some(vec!["0.0.0.0/0".to_string(), "::/0".to_string()]),
                ..Default::default()
            }
        } else {
            PrefsUpdate {
                advertise_routes: Some(vec![]),
                ..Default::default()
            }
        };
        self.set_prefs(&prefs).await
    }

    /// Set SSH enabled/disabled.
    pub async fn set_ssh(&self, enabled: bool) -> TsResult<Prefs> {
        let prefs = PrefsUpdate {
            run_ssh: Some(enabled),
            ..Default::default()
        };
        self.set_prefs(&prefs).await
    }

    /// Set accept-routes enabled/disabled.
    pub async fn set_accept_routes(&self, accept: bool) -> TsResult<Prefs> {
        let prefs = PrefsUpdate {
            route_all: Some(accept),
            ..Default::default()
        };
        self.set_prefs(&prefs).await
    }

    /// Set accept-dns (MagicDNS) enabled/disabled.
    pub async fn set_accept_dns(&self, accept: bool) -> TsResult<Prefs> {
        let prefs = PrefsUpdate {
            corp_dns: Some(accept),
            ..Default::default()
        };
        self.set_prefs(&prefs).await
    }

    /// Set exit-node-allow-lan-access.
    pub async fn set_exit_node_allow_lan(&self, allow: bool) -> TsResult<Prefs> {
        let prefs = PrefsUpdate {
            exit_node_allow_lan_access: Some(allow),
            ..Default::default()
        };
        self.set_prefs(&prefs).await
    }

    /// Set advertised routes (subnet router).
    pub async fn set_advertise_routes(&self, routes: Vec<String>) -> TsResult<Prefs> {
        let prefs = PrefsUpdate {
            advertise_routes: Some(routes),
            ..Default::default()
        };
        self.set_prefs(&prefs).await
    }

    /// Connect (set WantRunning = true).
    pub async fn connect(&self) -> TsResult<Prefs> {
        let prefs = PrefsUpdate {
            want_running: Some(true),
            ..Default::default()
        };
        self.set_prefs(&prefs).await
    }

    /// Disconnect (set WantRunning = false).
    pub async fn disconnect(&self) -> TsResult<Prefs> {
        let prefs = PrefsUpdate {
            want_running: Some(false),
            ..Default::default()
        };
        self.set_prefs(&prefs).await
    }

}

impl Default for TailscaleClient {
    fn default() -> Self {
        Self::new()
    }
}

// ─── API Response Types ──────────────────────────────────────────────────────

/// Full status response from `/localapi/v0/status`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct Status {
    /// Version of the tailscale backend.
    #[serde(default)]
    pub version: String,

    /// Whether the backend is running.
    #[serde(default)]
    pub backend_state: String,

    /// This node's info.
    #[serde(rename = "Self")]
    pub self_node: Option<PeerStatus>,

    /// Map of peer node key → peer status.
    #[serde(default)]
    pub peer: std::collections::HashMap<String, PeerStatus>,

    /// The current tailnet name.
    #[serde(default)]
    pub current_tailnet: Option<TailnetStatus>,

    /// MagicDNS suffix for the tailnet.
    #[serde(default)]
    pub magic_dns_suffix: String,
}

/// Status of a peer or self node.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct PeerStatus {
    /// Stable node ID.
    #[serde(rename = "ID", default)]
    pub id: String,

    /// Public key of the node.
    #[serde(default)]
    pub public_key: String,

    /// Hostname.
    #[serde(default)]
    pub host_name: String,

    /// DNS name (FQDN).
    #[serde(rename = "DNSName", default)]
    pub dns_name: String,

    /// OS of the peer.
    #[serde(rename = "OS", default)]
    pub os: String,

    /// Tailscale IP addresses.
    #[serde(rename = "TailscaleIPs", default)]
    pub tailscale_ips: Vec<String>,

    /// Whether the peer is online.
    #[serde(default)]
    pub online: bool,

    /// Whether this node is an exit node.
    #[serde(default)]
    pub exit_node: bool,

    /// Whether this is the currently used exit node.
    #[serde(default)]
    pub exit_node_option: bool,

    /// Tags assigned to this node.
    #[serde(default)]
    pub tags: Option<Vec<String>>,

    /// DERP relay region.
    #[serde(default)]
    pub relay: String,

    /// Bytes received from this peer.
    #[serde(default)]
    pub rx_bytes: u64,

    /// Bytes sent to this peer.
    #[serde(default)]
    pub tx_bytes: u64,

    /// When the peer was last seen.
    #[serde(default)]
    pub last_seen: String,

    /// Whether this peer offers exit node capability.
    #[serde(default)]
    pub exit_node_option_enabled: bool,

    /// Whether this is a Mullvad exit node.
    #[serde(default)]
    pub is_mullvad: bool,
}

/// Tailnet info.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct TailnetStatus {
    /// Name of the tailnet.
    #[serde(default)]
    pub name: String,

    /// MagicDNS suffix.
    #[serde(rename = "MagicDNSSuffix", default)]
    pub magic_dns_suffix: String,

    /// Whether MagicDNS is enabled.
    #[serde(rename = "MagicDNSEnabled", default)]
    pub magic_dns_enabled: bool,
}

/// Preferences from `/localapi/v0/prefs`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct Prefs {
    /// Whether the user wants tailscale running.
    #[serde(default)]
    pub want_running: bool,

    /// Whether SSH is enabled.
    #[serde(default)]
    pub run_ssh: bool,

    /// Whether to accept routes from other nodes.
    #[serde(default)]
    pub route_all: bool,

    /// Whether to accept DNS from the tailnet.
    #[serde(default)]
    pub corp_dns: bool,

    /// Exit node IP being used.
    #[serde(rename = "ExitNodeIP", default)]
    pub exit_node_ip: String,

    /// Whether to allow LAN access when using an exit node.
    #[serde(default)]
    pub exit_node_allow_lan_access: bool,

    /// Routes being advertised.
    #[serde(default)]
    pub advertise_routes: Option<Vec<String>>,

    /// Hostname of this node.
    #[serde(default)]
    pub hostname: String,

    /// Operator user.
    #[serde(default)]
    pub operator_user: String,
}

/// Partial prefs update for PATCH /localapi/v0/prefs.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct PrefsUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub want_running: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_ssh: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub route_all: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub corp_dns: Option<bool>,

    #[serde(rename = "ExitNodeIP", skip_serializing_if = "Option::is_none")]
    pub exit_node_ip: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_node_allow_lan_access: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub advertise_routes: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,
}

/// Profile (account) info.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct Profile {
    /// Profile ID.
    #[serde(rename = "ID", default)]
    pub id: String,

    /// Display name of the profile/account.
    #[serde(default)]
    pub name: String,

    /// The tailnet name.
    #[serde(default)]
    pub network_profile: Option<NetworkProfile>,

    /// Whether this is the current profile.
    #[serde(default)]
    pub current_profile: bool,
}

/// Network profile associated with a Profile.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct NetworkProfile {
    /// MagicDNS name of the tailnet.
    #[serde(rename = "MagicDNSName", default)]
    pub magic_dns_name: String,

    /// Domain name.
    #[serde(default)]
    pub domain_name: String,
}

/// A file waiting in the Tail Drop inbox.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct WaitingFile {
    pub name: String,
    pub size: u64,
}

/// Ping result from `/localapi/v0/ping`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct PingResult {
    /// The type of ping (TSMP, disco, ICMP).
    #[serde(rename = "Type", default)]
    pub ping_type: String,

    /// The IP that was pinged.
    #[serde(rename = "IP", default)]
    pub ip: String,

    /// Node IP pinged.
    #[serde(default)]
    pub node_ip: String,

    /// Name of the node.
    #[serde(default)]
    pub node_name: String,

    /// Latency in seconds.
    #[serde(default)]
    pub latency_seconds: f64,

    /// The endpoint used.
    #[serde(default)]
    pub endpoint: String,

    /// Whether the connection is direct (not relayed).
    #[serde(default)]
    pub is_direct: bool,

    /// Error message, if any.
    #[serde(default)]
    pub err: String,
}
