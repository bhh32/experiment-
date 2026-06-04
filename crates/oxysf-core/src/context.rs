//! Shared command context passed from the binary into topic crates.
//!
//! The binary owns clap parsing and builds a [`CommandContext`] from the global
//! flags; topic crates receive it and use it to resolve orgs and render output.

use crate::auth::store::{self, AuthInfo};
use crate::config::{self};
use crate::connection::Connection;
use crate::error::{Result, SfError};
use crate::output::OutputMode;

/// The resolved global options for a single command invocation.
#[derive(Debug, Clone)]
pub struct CommandContext {
    /// How output should be rendered.
    pub output: OutputMode,
    /// The explicit `--target-org` value, if provided.
    pub target_org: Option<String>,
    /// The explicit `--api-version` value, if provided.
    pub api_version: Option<String>,
    /// Whether verbose output was requested.
    pub verbose: bool,
}

impl CommandContext {
    /// Resolve the target org username/alias for this invocation.
    pub fn resolve_target_org(&self) -> Result<String> {
        config::resolve_target_org(self.target_org.as_deref())
    }

    /// Load the [`AuthInfo`] for the resolved target org.
    pub fn load_target_auth(&self) -> Result<AuthInfo> {
        let org = self.resolve_target_org()?;
        store::load(&org).map_err(|_| SfError::NotAuthenticated(org))
    }

    /// Build an authenticated [`Connection`] for the resolved target org.
    pub fn connection(&self) -> Result<Connection> {
        let info = self.load_target_auth()?;
        Connection::from_auth(&info, self.api_version.as_deref())
    }
}
