use std::{
    collections::VecDeque,
    fmt,
    io::Read,
    process::{Command, Stdio},
    time::Duration,
};

use regex::RegexBuilder;
use serde::{Deserialize, Serialize};

// ─── Error Handling ──────────────────────────────────────────────────────────

/// Unified error type for all Tailscale CLI operations.
#[derive(Debug, Clone)]
pub enum TailscaleError {
    /// The `tailscale` binary was not found on $PATH.
    NotInstalled,
    /// The tailscale daemon is not running.
    DaemonNotRunning,
    /// The operator permission has not been set for the current user.
    OperatorNotSet,
    /// A CLI command failed with an error message.
    CommandFailed(String),
    /// Failed to parse CLI output.
    ParseError(String),
}

impl fmt::Display for TailscaleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TailscaleError::NotInstalled => {
                write!(f, "Tailscale is not installed. Please install it first.")
            }
            TailscaleError::DaemonNotRunning => {
                write!(f, "Tailscale daemon is not running. Start it with: sudo systemctl start tailscaled")
            }
            TailscaleError::OperatorNotSet => {
                write!(f, "Tailscale operator not set. Run: sudo tailscale set --operator=$USER")
            }
            TailscaleError::CommandFailed(msg) => write!(f, "Command failed: {msg}"),
            TailscaleError::ParseError(msg) => write!(f, "Parse error: {msg}"),
        }
    }
}

pub type TsResult<T> = Result<T, TailscaleError>;

/// Run a tailscale command and return its stdout as a String.
/// Handles common failure modes gracefully.
fn run_tailscale(args: &[&str]) -> TsResult<String> {
    let output = Command::new("tailscale")
        .args(args)
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                TailscaleError::NotInstalled
            } else {
                TailscaleError::CommandFailed(e.to_string())
            }
        })?;

    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        if stderr.contains("is not running") || stderr.contains("connection refused") {
            return Err(TailscaleError::DaemonNotRunning);
        }
        if stderr.contains("not an operator") || stderr.contains("permission denied") {
            return Err(TailscaleError::OperatorNotSet);
        }
        return Err(TailscaleError::CommandFailed(stderr));
    }

    String::from_utf8(output.stdout)
        .map_err(|e| TailscaleError::ParseError(e.to_string()))
}

/// Run a tailscale command, piping stdout through grep for a pattern.
fn run_tailscale_grep(ts_args: &[&str], grep_pattern: &str) -> TsResult<String> {
    let ts_cmd = Command::new("tailscale")
        .args(ts_args)
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                TailscaleError::NotInstalled
            } else {
                TailscaleError::CommandFailed(e.to_string())
            }
        })?;

    let ts_stdout = ts_cmd
        .stdout
        .ok_or_else(|| TailscaleError::CommandFailed("Failed to capture stdout".into()))?;

    let grep_output = Command::new("grep")
        .arg(grep_pattern)
        .stdin(ts_stdout)
        .output()
        .map_err(|e| TailscaleError::CommandFailed(format!("grep failed: {e}")))?;

    String::from_utf8(grep_output.stdout)
        .map_err(|e| TailscaleError::ParseError(e.to_string()))
}

// ─── Startup Checks ─────────────────────────────────────────────────────────

/// Check if tailscale is installed and the daemon is running.
pub fn check_tailscale_available() -> TsResult<()> {
    run_tailscale(&["version"])?;
    Ok(())
}

/// Check if the operator permission is set for the current user.
pub fn check_operator_set() -> TsResult<bool> {
    let prefs = run_tailscale(&["debug", "prefs"])?;
    let has_operator = prefs
        .lines()
        .any(|line| line.contains("OperatorUser") && !line.contains("\"\""));
    Ok(has_operator)
}

// ─── Device Info Structures ──────────────────────────────────────────────────

/// Detailed information about a device on the tailnet.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeviceInfo {
    pub name: String,
    pub dns_name: String,
    pub tailscale_ip: String,
    pub os: String,
    pub online: bool,
    pub is_exit_node: bool,
    pub tags: Vec<String>,
    pub relay: String,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub last_seen: String,
    pub is_self: bool,
}

/// Result of a ping operation.
#[derive(Debug, Clone)]
pub struct PingResult {
    pub target: String,
    pub latency_ms: f64,
    pub via: String,
}

/// Information about a Tailscale Serve entry.
#[derive(Debug, Clone, Default)]
pub struct ServeEntry {
    pub protocol: String,
    pub local_addr: String,
    pub path: String,
    pub funnel: bool,
}

/// Information about a subnet route.
#[derive(Debug, Clone)]
pub struct SubnetRoute {
    pub cidr: String,
    pub advertised: bool,
    pub approved: bool,
}

/// Tailnet lock status information.
#[derive(Debug, Clone)]
pub struct LockStatus {
    pub enabled: bool,
    pub node_signed: bool,
    pub pending_signatures: u32,
}

/// Transfer history entry.
#[derive(Debug, Clone)]
pub struct TransferRecord {
    pub file_name: String,
    pub device: String,
    pub direction: TransferDirection,
    pub timestamp: String,
    pub success: bool,
}

#[derive(Debug, Clone)]
pub enum TransferDirection {
    Sent,
    Received,
}

impl fmt::Display for TransferDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransferDirection::Sent => write!(f, "Sent"),
            TransferDirection::Received => write!(f, "Received"),
        }
    }
}

// ─── Status Queries (all now return TsResult) ────────────────────────────────

/// Get the IPv4 address assigned to this computer.
pub fn get_tailscale_ip() -> TsResult<String> {
    let ip = run_tailscale(&["ip", "-4"])?;
    Ok(ip.trim().to_string())
}

/// Get the IPv6 address assigned to this computer.
pub fn get_tailscale_ipv6() -> TsResult<String> {
    let ip = run_tailscale(&["ip", "-6"])?;
    Ok(ip.trim().to_string())
}

/// Get Tailscale's connection status.
pub fn get_tailscale_con_status() -> TsResult<bool> {
    let output = run_tailscale_grep(&["debug", "prefs"], "WantRunning")?;
    Ok(output.contains("true"))
}

/// Get the current status of SSH enablement.
pub fn get_tailscale_ssh_status() -> TsResult<bool> {
    let output = run_tailscale_grep(&["debug", "prefs"], "RunSSH")?;
    Ok(output.contains("true"))
}

/// Get the current status of accept-routes enablement.
pub fn get_tailscale_routes_status() -> TsResult<bool> {
    let output = run_tailscale_grep(&["debug", "prefs"], "RouteAll")?;
    Ok(output.contains("true"))
}

/// Get MagicDNS status.
pub fn get_magic_dns_status() -> TsResult<bool> {
    let output = run_tailscale_grep(&["debug", "prefs"], "CorpDNS")?;
    Ok(output.contains("true"))
}

/// Get the device list (simple names for Tail Drop dropdown).
pub fn get_tailscale_devices() -> TsResult<Vec<String>> {
    let out = run_tailscale(&["status"])?;

    let reg = RegexBuilder::new(r#"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}"#)
        .build()
        .map_err(|e| TailscaleError::ParseError(e.to_string()))?;

    let mut status_output: VecDeque<String> = out
        .lines()
        .filter(|line| reg.is_match(line))
        .filter_map(|line| line.split_whitespace().nth(1).map(|s| s.to_string()))
        .collect();

    // Pop this system's device name out of the VecDeque
    status_output.pop_front();
    // Add Select as the first element
    status_output.push_front("Select".to_string());

    Ok(status_output.into())
}

/// Get detailed information about all devices on the tailnet.
pub fn get_device_details() -> TsResult<Vec<DeviceInfo>> {
    let json_str = run_tailscale(&["status", "--json"])?;

    let json: serde_json::Value = serde_json::from_str(&json_str)
        .map_err(|e| TailscaleError::ParseError(format!("JSON parse error: {e}")))?;

    let mut devices = Vec::new();

    // Parse self node
    if let Some(self_node) = json.get("Self") {
        if let Some(dev) = parse_peer_node(self_node, true) {
            devices.push(dev);
        }
    }

    // Parse peer nodes
    if let Some(peers) = json.get("Peer").and_then(|p| p.as_object()) {
        for (_key, peer) in peers {
            if let Some(dev) = parse_peer_node(peer, false) {
                devices.push(dev);
            }
        }
    }

    Ok(devices)
}

fn parse_peer_node(node: &serde_json::Value, is_self: bool) -> Option<DeviceInfo> {
    let dns_name = node
        .get("DNSName")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim_end_matches('.')
        .to_string();

    let name = dns_name.split('.').next().unwrap_or("").to_string();

    let tailscale_ip = node
        .get("TailscaleIPs")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let os = node
        .get("OS")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let online = node
        .get("Online")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let is_exit_node = node
        .get("ExitNode")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let tags: Vec<String> = node
        .get("Tags")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    let relay = node
        .get("Relay")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let rx_bytes = node
        .get("RxBytes")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    let tx_bytes = node
        .get("TxBytes")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    let last_seen = node
        .get("LastSeen")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    if name.is_empty() {
        return None;
    }

    Some(DeviceInfo {
        name,
        dns_name,
        tailscale_ip,
        os,
        online: if is_self { true } else { online },
        is_exit_node,
        tags,
        relay,
        rx_bytes,
        tx_bytes,
        last_seen,
        is_self,
    })
}

/// Get the status of whether or not the host is an exit node.
pub fn get_is_exit_node() -> TsResult<bool> {
    let output = run_tailscale(&["debug", "prefs"])?;
    let adv_rts: String = output
        .lines()
        .filter(|line| line.to_lowercase().contains("advertiseroutes"))
        .flat_map(|line| line.chars())
        .collect();

    Ok(!adv_rts.contains("null") && !adv_rts.is_empty())
}

/// Get the list of accounts the device is registered on.
pub fn get_acct_list() -> TsResult<Vec<String>> {
    let accts_str = run_tailscale(&["switch", "--list"])?;

    let tailnets: Vec<String> = accts_str
        .lines()
        .filter(|line| !line.to_lowercase().starts_with("id"))
        .map(|line| line.to_string())
        .collect();

    let mut ret_accts = Vec::new();

    for acct in tailnets {
        let parts: Vec<String> = acct
            .split_whitespace()
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.to_string())
            .collect();

        if parts.len() > 1 {
            ret_accts.push(parts[1].clone());
        }
    }

    Ok(ret_accts)
}

/// Get which account the device is currently logged into.
pub fn get_current_acct() -> TsResult<String> {
    let output = run_tailscale(&["status", "--json"])?;

    let name = output
        .lines()
        .filter(|line| line.trim().starts_with("\"Name\""))
        .filter_map(|line| {
            line.trim()
                .split_whitespace()
                .last()
                .map(|s| s.replace('"', "").replace(',', "").trim().to_string())
        })
        .next()
        .unwrap_or_else(|| "Unknown".to_string());

    Ok(name)
}

/// Get available exit nodes.
pub fn get_avail_exit_nodes() -> TsResult<Vec<String>> {
    let exit_node_list_string = run_tailscale(&["exit-node", "list"])?;

    if exit_node_list_string.is_empty() {
        return Ok(vec!["None".to_string()]);
    }

    let fq_hostname_reg = RegexBuilder::new(r#"\w.\w.ts.net"#)
        .build()
        .map_err(|e| TailscaleError::ParseError(e.to_string()))?;

    let mut exit_node_list: Vec<String> = vec!["None".to_string()];

    let mut exit_node_map: Vec<String> = exit_node_list_string
        .lines()
        .filter(|line| fq_hostname_reg.is_match(line))
        .filter_map(|hostname| {
            hostname
                .split_whitespace()
                .nth(1)
                .and_then(|fqdn| fqdn.split('.').next())
                .map(|name| name.to_string())
        })
        .collect();

    exit_node_list.append(&mut exit_node_map);
    Ok(exit_node_list)
}

// ─── Control Functions ───────────────────────────────────────────────────────

/// Set the Tailscale connection up/down.
pub fn tailscale_int_up(up: bool) -> TsResult<bool> {
    if up {
        run_tailscale(&["up"])?;
        Ok(true)
    } else {
        run_tailscale(&["down"])?;
        Ok(false)
    }
}

/// Toggle SSH on/off.
pub fn set_ssh(ssh: bool) -> TsResult<bool> {
    if ssh {
        run_tailscale(&["set", "--ssh"])?;
    } else {
        run_tailscale(&["set", "--ssh=false"])?;
    }
    Ok(ssh)
}

/// Toggle accept-routes on/off.
pub fn set_routes(accept_routes: bool) -> TsResult<bool> {
    if accept_routes {
        run_tailscale(&["set", "--accept-routes"])?;
    } else {
        run_tailscale(&["set", "--accept-routes=false"])?;
    }
    Ok(accept_routes)
}

/// Toggle MagicDNS on/off.
pub fn set_magic_dns(enabled: bool) -> TsResult<bool> {
    if enabled {
        run_tailscale(&["set", "--accept-dns"])?;
    } else {
        run_tailscale(&["set", "--accept-dns=false"])?;
    }
    Ok(enabled)
}

/// Make current host an exit node.
pub fn enable_exit_node(is_exit_node: bool) -> TsResult<()> {
    run_tailscale(&["set", &format!("--advertise-exit-node={is_exit_node}")])?;
    tailscale_int_up(true)?;
    Ok(())
}

/// Add/remove exit node's access to the host's local LAN.
pub fn exit_node_allow_lan_access(is_allowed: bool) -> TsResult<String> {
    run_tailscale(&[
        "set",
        &format!("--exit-node-allow-lan-access={is_allowed}"),
    ])?;
    Ok(format!(
        "Exit node LAN access {}.",
        if is_allowed { "enabled" } else { "disabled" }
    ))
}

/// Set selected exit node.
pub fn set_exit_node(exit_node: String) -> TsResult<bool> {
    run_tailscale(&["set", &format!("--exit-node={exit_node}")])?;
    Ok(exit_node.is_empty())
}

/// Switch accounts.
pub fn switch_accounts(acct_name: String) -> TsResult<bool> {
    let output = run_tailscale(&["switch", &acct_name])?;
    Ok(output.to_lowercase().contains("success"))
}

/// Open login in browser for a new account.
pub fn login_new_account() -> TsResult<()> {
    run_tailscale(&["login"])?;
    Ok(())
}

// ─── Ping ────────────────────────────────────────────────────────────────────

/// Ping a device and return latency info.
pub async fn ping_device(target: String) -> TsResult<PingResult> {
    let output = tokio::process::Command::new("tailscale")
        .args(["ping", "--c", "1", &target])
        .output()
        .await
        .map_err(|e| TailscaleError::CommandFailed(e.to_string()))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    // Parse: "pong from device (100.x.y.z) via DERP(nyc) in 25ms"
    // or:    "pong from device (100.x.y.z) via 1.2.3.4:41641 in 5ms"
    let latency_ms = stdout
        .lines()
        .find(|l| l.starts_with("pong"))
        .and_then(|line| {
            line.rsplit("in ")
                .next()
                .and_then(|s| s.trim_end_matches("ms").trim().parse::<f64>().ok())
        })
        .unwrap_or(0.0);

    let via = stdout
        .lines()
        .find(|l| l.starts_with("pong"))
        .and_then(|line| {
            line.split("via ")
                .nth(1)
                .and_then(|s| s.split(" in ").next())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "unknown".to_string());

    Ok(PingResult {
        target,
        latency_ms,
        via,
    })
}

// ─── Tail Drop (File Transfer) ──────────────────────────────────────────────

/// Send files through Tail Drop (async, non-blocking).
pub async fn tailscale_send(file_paths: Vec<Option<String>>, target: &str) -> Option<String> {
    let mut errors = Vec::new();

    for path in file_paths.iter() {
        match path {
            Some(p) => {
                let result = tokio::process::Command::new("tailscale")
                    .args(["file", "cp", p, &format!("{target}:")])
                    .output()
                    .await;

                match result {
                    Ok(output) => {
                        if !output.status.success() {
                            let err = String::from_utf8_lossy(&output.stderr).to_string();
                            errors.push(format!("{p}: {err}"));
                        }
                    }
                    Err(e) => {
                        errors.push(format!("{p}: {e}"));
                    }
                }
            }
            None => {
                return Some(String::from(
                    "Something went wrong sending the file!\nPossible bad file path!",
                ));
            }
        };
    }

    if !errors.is_empty() {
        return Some(format!(
            "Errors sending file(s):\n{}",
            errors.join("\n")
        ));
    }

    None
}

/// Receive files through Tail Drop (async, non-blocking).
/// Uses a configurable download directory.
pub async fn tailscale_receive(download_dir: Option<String>) -> String {
    let download_path = match download_dir {
        Some(dir) => dir,
        None => {
            // Default to ~/Downloads/
            let whoami_result = tokio::process::Command::new("whoami")
                .output()
                .await;

            match whoami_result {
                Ok(output) => {
                    let username = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    format!("/home/{username}/Downloads/")
                }
                Err(e) => {
                    return format!("Failed to determine user: {e}");
                }
            }
        }
    };

    let rx_result = tokio::process::Command::new("tailscale")
        .args(["file", "get", &download_path])
        .output()
        .await;

    match rx_result {
        Ok(output) => {
            if output.stderr.is_empty() {
                format!("Received file(s) in {download_path}")
            } else {
                String::from_utf8_lossy(&output.stderr).to_string()
            }
        }
        Err(e) => format!("Failed to receive files: {e}"),
    }
}

/// Wait for a specified number of seconds (async, non-blocking).
/// Fixed: uses tokio::time::sleep instead of thread::sleep.
pub async fn clear_status(wait_time: u64) -> Option<String> {
    tokio::time::sleep(Duration::from_secs(wait_time)).await;
    None
}

// ─── Tailscale Serve & Funnel ────────────────────────────────────────────────

/// Get the current Tailscale Serve configuration.
pub fn get_serve_status() -> TsResult<Vec<ServeEntry>> {
    let output = run_tailscale(&["serve", "status"])?;

    let mut entries = Vec::new();

    // Parse output lines like:
    //   https://hostname.tailnet.ts.net (Funnel on)
    //   |-- /         proxy http://127.0.0.1:3000
    //   |-- /api      proxy http://127.0.0.1:8080
    let mut current_funnel = false;

    for line in output.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("https://") || trimmed.starts_with("http://") {
            current_funnel = trimmed.contains("Funnel on");
        } else if trimmed.starts_with("|--") {
            let parts: Vec<&str> = trimmed.splitn(4, char::is_whitespace).collect();
            if parts.len() >= 4 {
                let path = parts[1].to_string();
                let protocol = parts[2].to_string();
                let local_addr = parts[3].to_string();

                entries.push(ServeEntry {
                    protocol,
                    local_addr,
                    path,
                    funnel: current_funnel,
                });
            }
        }
    }

    Ok(entries)
}

/// Add a new serve entry: expose a local port on a given path.
pub fn add_serve(port: u16, path: &str) -> TsResult<String> {
    let local = format!("http://127.0.0.1:{port}");
    let serve_path = if path.is_empty() { "/" } else { path };
    run_tailscale(&["serve", "--bg", "--set-path", serve_path, &local])
}

/// Remove a serve entry for a given path.
pub fn remove_serve(path: &str) -> TsResult<String> {
    let serve_path = if path.is_empty() { "/" } else { path };
    run_tailscale(&["serve", "--bg", "--remove", serve_path])
}

/// Toggle Funnel on or off for a given port.
pub fn toggle_funnel(port: u16, enable: bool) -> TsResult<String> {
    if enable {
        let local = format!("http://127.0.0.1:{port}");
        run_tailscale(&["funnel", "--bg", &local])
    } else {
        run_tailscale(&["funnel", "--bg", "off"])
    }
}

// ─── Subnet Router Management ───────────────────────────────────────────────

/// Get the currently advertised subnet routes.
pub fn get_advertised_routes() -> TsResult<Vec<String>> {
    let prefs = run_tailscale(&["debug", "prefs"])?;

    let routes: Vec<String> = prefs
        .lines()
        .filter(|line| line.to_lowercase().contains("advertiseroutes"))
        .filter_map(|line| {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 2 {
                let route_part = parts[1..].join(":").trim().to_string();
                if route_part.contains("null") || route_part.is_empty() {
                    None
                } else {
                    Some(route_part.trim_matches(|c| c == '[' || c == ']' || c == '"').to_string())
                }
            } else {
                None
            }
        })
        .flat_map(|s| {
            s.split(',')
                .map(|r| r.trim().trim_matches('"').to_string())
                .filter(|r| !r.is_empty())
                .collect::<Vec<String>>()
        })
        .collect();

    Ok(routes)
}

/// Set the advertised subnet routes (replaces all existing).
pub fn set_advertised_routes(routes: &[String]) -> TsResult<()> {
    let routes_str = routes.join(",");
    if routes_str.is_empty() {
        run_tailscale(&["set", "--advertise-routes="])?;
    } else {
        run_tailscale(&["set", &format!("--advertise-routes={routes_str}")])?;
    }
    Ok(())
}

// ─── Tailnet Lock ────────────────────────────────────────────────────────────

/// Get the current tailnet lock status.
pub fn get_lock_status() -> TsResult<LockStatus> {
    let output = run_tailscale(&["lock", "status"]);

    match output {
        Ok(status_str) => {
            let enabled = status_str.contains("Tailnet lock is enabled");
            let node_signed = status_str.contains("This node has been signed");
            let pending = status_str
                .lines()
                .find(|l| l.contains("pending"))
                .and_then(|l| {
                    l.split_whitespace()
                        .next()
                        .and_then(|n| n.parse::<u32>().ok())
                })
                .unwrap_or(0);

            Ok(LockStatus {
                enabled,
                node_signed,
                pending_signatures: pending,
            })
        }
        Err(_) => {
            // lock command may not be available on all tailscale versions
            Ok(LockStatus {
                enabled: false,
                node_signed: false,
                pending_signatures: 0,
            })
        }
    }
}

/// Sign a node for tailnet lock.
pub fn lock_sign_node(node_key: &str) -> TsResult<String> {
    run_tailscale(&["lock", "sign", node_key])
}

// ─── Clipboard Utility ──────────────────────────────────────────────────────

/// Copy text to the system clipboard (Wayland).
pub fn copy_to_clipboard(text: &str) -> Result<(), String> {
    let mut child = Command::new("wl-copy")
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to run wl-copy: {e}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        stdin
            .write_all(text.as_bytes())
            .map_err(|e| format!("Failed to write to clipboard: {e}"))?;
    }

    child
        .wait()
        .map_err(|e| format!("wl-copy failed: {e}"))?;

    Ok(())
}

// ─── Utility Functions ───────────────────────────────────────────────────────

/// Format bytes into a human-readable string.
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}
