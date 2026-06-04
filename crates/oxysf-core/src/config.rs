//! Configuration: locating `~/.sf`, reading `sfdx-project.json`, resolving the
//! target org and API version.

use std::path::PathBuf;

use serde::Deserialize;

use crate::auth::store;
use crate::error::{Result, SfError};

/// The default Salesforce REST API version used when none is configured.
pub const DEFAULT_API_VERSION: &str = "62.0";

/// Return the `~/.sf` directory (Salesforce CLI's config home).
pub fn sf_dir() -> PathBuf {
    if let Some(home) = dirs::home_dir() {
        home.join(".sf")
    } else {
        // Fall back to the current directory's `.sf` if the home dir is unknown.
        PathBuf::from(".sf")
    }
}

/// A single package directory entry from `sfdx-project.json`.
#[derive(Debug, Clone, Deserialize)]
pub struct PackageDirectory {
    /// The relative path of the package directory.
    pub path: String,
    /// Whether this is the default package directory.
    #[serde(default)]
    pub default: bool,
}

/// The fields of `sfdx-project.json` that oxy-sf cares about.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct SfdxProject {
    /// The API version configured for source operations.
    #[serde(default, rename = "sourceApiVersion")]
    pub source_api_version: Option<String>,
    /// The configured package directories.
    #[serde(default, rename = "packageDirectories")]
    pub package_directories: Vec<PackageDirectory>,
}

impl SfdxProject {
    /// Load `sfdx-project.json` from the given directory, if present.
    ///
    /// Returns `Ok(None)` when the file does not exist.
    pub fn load_from(dir: &std::path::Path) -> Result<Option<SfdxProject>> {
        let path = dir.join("sfdx-project.json");
        if !path.exists() {
            return Ok(None);
        }
        let contents = std::fs::read_to_string(&path)?;
        let project: SfdxProject = serde_json::from_str(&contents)
            .map_err(|e| SfError::Config(format!("invalid sfdx-project.json: {e}")))?;
        Ok(Some(project))
    }
}

/// Resolve the API version to use, given an optional explicit override.
///
/// Resolution order: explicit `--api-version` > project `sourceApiVersion` >
/// [`DEFAULT_API_VERSION`].
pub fn resolve_api_version(explicit: Option<&str>, project: Option<&SfdxProject>) -> String {
    if let Some(v) = explicit {
        return v.to_string();
    }
    if let Some(p) = project {
        if let Some(v) = &p.source_api_version {
            return v.clone();
        }
    }
    DEFAULT_API_VERSION.to_string()
}

/// Resolve the target org username/alias.
///
/// Resolution order: explicit `--target-org` > the stored default alias
/// (`target-org`) > error [`SfError::NoTargetOrg`].
pub fn resolve_target_org(explicit: Option<&str>) -> Result<String> {
    if let Some(o) = explicit {
        return Ok(o.to_string());
    }
    if let Some(default) = store::default_target_org()? {
        return Ok(default);
    }
    Err(SfError::NoTargetOrg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sf_dir_ends_with_dot_sf() {
        assert!(sf_dir().ends_with(".sf"));
    }

    #[test]
    fn api_version_prefers_explicit() {
        let proj = SfdxProject {
            source_api_version: Some("58.0".into()),
            ..Default::default()
        };
        assert_eq!(resolve_api_version(Some("60.0"), Some(&proj)), "60.0");
        assert_eq!(resolve_api_version(None, Some(&proj)), "58.0");
        assert_eq!(resolve_api_version(None, None), DEFAULT_API_VERSION);
    }

    #[test]
    fn explicit_target_org_wins() {
        assert_eq!(resolve_target_org(Some("me@x.com")).unwrap(), "me@x.com");
    }

    #[test]
    fn parses_sfdx_project() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("sfdx-project.json"),
            r#"{"sourceApiVersion":"59.0","packageDirectories":[{"path":"force-app","default":true}]}"#,
        )
        .unwrap();
        let proj = SfdxProject::load_from(dir.path()).unwrap().unwrap();
        assert_eq!(proj.source_api_version.as_deref(), Some("59.0"));
        assert_eq!(proj.package_directories.len(), 1);
        assert!(proj.package_directories[0].default);
    }
}
