//! On-disk persistence of authenticated org credentials.
//!
//! # Security / compatibility note
//!
//! The real `sf` CLI stores auth files under `~/.sf/` **encrypted** (keyed via
//! the OS keychain). To avoid clobbering or corrupting those real files, oxy-sf
//! writes its own files under a `~/.sf/oxysf/` subdirectory. For now these are
//! plaintext JSON created with `0600` permissions on Unix. Matching `sf`'s
//! on-disk encryption for a true drop-in experience is a documented follow-up.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::config::sf_dir;
use crate::error::{Result, SfError};

/// Environment variable that overrides the auth storage directory (used in
/// tests so they never touch the real `~/.sf`).
pub const STORE_DIR_ENV: &str = "OXYSF_AUTH_DIR";

/// Credentials and metadata for a single authenticated org.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthInfo {
    /// The org username (e.g. `user@example.com`).
    pub username: String,
    /// The org's instance URL (e.g. `https://na1.salesforce.com`).
    pub instance_url: String,
    /// The OAuth access token.
    pub access_token: String,
    /// The OAuth refresh token, when available (web flow).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    /// The OAuth client id (connected app) used to obtain the tokens.
    pub client_id: String,
    /// The login URL used for auth (e.g. `https://login.salesforce.com`).
    pub login_url: String,
    /// An optional human-friendly alias for the org.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    /// The org id, when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub org_id: Option<String>,
}

/// Return the directory where oxy-sf stores its auth files.
pub fn store_dir() -> PathBuf {
    if let Ok(dir) = std::env::var(STORE_DIR_ENV) {
        return PathBuf::from(dir);
    }
    sf_dir().join("oxysf")
}

fn file_for(dir: &Path, username: &str) -> PathBuf {
    dir.join(format!("{username}.json"))
}

/// Persist an [`AuthInfo`] to disk (one JSON file per org).
pub fn save(info: &AuthInfo) -> Result<()> {
    let dir = store_dir();
    std::fs::create_dir_all(&dir)?;
    let path = file_for(&dir, &info.username);
    let json = serde_json::to_string_pretty(info)?;
    write_secure(&path, json.as_bytes())?;
    Ok(())
}

/// Write bytes to `path`, creating the file with `0600` perms on Unix.
fn write_secure(path: &Path, bytes: &[u8]) -> Result<()> {
    #[cfg(unix)]
    {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(path)?;
        f.write_all(bytes)?;
    }
    #[cfg(not(unix))]
    {
        std::fs::write(path, bytes)?;
    }
    Ok(())
}

/// Load an [`AuthInfo`] by username or alias.
pub fn load(username_or_alias: &str) -> Result<AuthInfo> {
    let dir = store_dir();

    // Direct match on username.
    let direct = file_for(&dir, username_or_alias);
    if direct.exists() {
        return read_info(&direct);
    }

    // Otherwise scan for a matching alias.
    for info in list() {
        if info.alias.as_deref() == Some(username_or_alias) {
            return Ok(info);
        }
    }

    Err(SfError::NotAuthenticated(username_or_alias.to_string()))
}

fn read_info(path: &Path) -> Result<AuthInfo> {
    let contents = std::fs::read_to_string(path)?;
    let info: AuthInfo = serde_json::from_str(&contents)?;
    Ok(info)
}

/// List all stored orgs. Unreadable files are silently skipped.
pub fn list() -> Vec<AuthInfo> {
    let dir = store_dir();
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        // Skip our internal config file.
        if path.file_name().and_then(|n| n.to_str()) == Some("config.json") {
            continue;
        }
        if let Ok(info) = read_info(&path) {
            out.push(info);
        }
    }
    out.sort_by(|a, b| a.username.cmp(&b.username));
    out
}

/// Remove a stored org by username (or alias). Returns `true` if removed.
pub fn remove(username_or_alias: &str) -> Result<bool> {
    let dir = store_dir();
    let direct = file_for(&dir, username_or_alias);
    if direct.exists() {
        std::fs::remove_file(&direct)?;
        return Ok(true);
    }
    // Try alias.
    for info in list() {
        if info.alias.as_deref() == Some(username_or_alias) {
            let path = file_for(&dir, &info.username);
            std::fs::remove_file(&path)?;
            return Ok(true);
        }
    }
    Ok(false)
}

/// The small config file used to record the default target org.
#[derive(Debug, Default, Serialize, Deserialize)]
struct StoreConfig {
    #[serde(default, rename = "target-org", skip_serializing_if = "Option::is_none")]
    target_org: Option<String>,
}

fn config_path() -> PathBuf {
    store_dir().join("config.json")
}

/// Return the configured default target org alias/username, if any.
pub fn default_target_org() -> Result<Option<String>> {
    let path = config_path();
    if !path.exists() {
        return Ok(None);
    }
    let contents = std::fs::read_to_string(&path)?;
    let cfg: StoreConfig = serde_json::from_str(&contents).unwrap_or_default();
    Ok(cfg.target_org)
}

/// Set (or clear) the default target org alias/username.
pub fn set_default_target_org(value: Option<&str>) -> Result<()> {
    let dir = store_dir();
    std::fs::create_dir_all(&dir)?;
    let cfg = StoreConfig {
        target_org: value.map(|s| s.to_string()),
    };
    let json = serde_json::to_string_pretty(&cfg)?;
    write_secure(&config_path(), json.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(username: &str) -> AuthInfo {
        AuthInfo {
            username: username.into(),
            instance_url: "https://example.my.salesforce.com".into(),
            access_token: "00Dxx!token".into(),
            refresh_token: Some("refresh".into()),
            client_id: "PlatformCLI".into(),
            login_url: "https://login.salesforce.com".into(),
            alias: Some("dev".into()),
            org_id: Some("00D000000000000EAA".into()),
        }
    }

    #[test]
    fn save_load_remove_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        // Scope the env override to this test.
        std::env::set_var(STORE_DIR_ENV, dir.path());

        let info = sample("user@example.com");
        save(&info).unwrap();

        let loaded = load("user@example.com").unwrap();
        assert_eq!(loaded, info);

        // Load by alias too.
        let by_alias = load("dev").unwrap();
        assert_eq!(by_alias.username, "user@example.com");

        assert_eq!(list().len(), 1);

        assert!(remove("user@example.com").unwrap());
        assert!(load("user@example.com").is_err());
        assert_eq!(list().len(), 0);

        std::env::remove_var(STORE_DIR_ENV);
    }

    #[cfg(unix)]
    #[test]
    fn saved_file_is_0600() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        std::env::set_var(STORE_DIR_ENV, dir.path());

        let info = sample("perm@example.com");
        save(&info).unwrap();
        let path = file_for(&store_dir(), "perm@example.com");
        let mode = std::fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);

        std::env::remove_var(STORE_DIR_ENV);
    }
}
