//! The `org` command topic for oxy-sf: login, logout, list, and display.

#![forbid(unsafe_code)]

use clap::{Args, Subcommand};
use oxysf_core::auth::store::{self, AuthInfo};
use oxysf_core::auth::{jwt, web};
use oxysf_core::context::CommandContext;
use oxysf_core::error::{Result, SfError};
use oxysf_core::output::{print_success, render_table};
use serde::Serialize;

/// The `org` topic command tree.
#[derive(Debug, Subcommand)]
pub enum OrgCommand {
    /// Authenticate with a Salesforce org.
    #[command(subcommand)]
    Login(LoginCommand),
    /// Remove org credentials stored locally.
    Logout(LogoutArgs),
    /// List locally authenticated orgs.
    List(ListArgs),
    /// Display details about the target org.
    Display(DisplayArgs),
}

/// Login subcommands.
#[derive(Debug, Subcommand)]
pub enum LoginCommand {
    /// Log in using the OAuth web-server (browser) flow with PKCE.
    Web(WebArgs),
    /// Log in using the JWT bearer flow.
    Jwt(JwtArgs),
}

/// Arguments for `org login web`.
#[derive(Debug, Args)]
pub struct WebArgs {
    /// The login URL (defaults to https://login.salesforce.com).
    #[arg(long)]
    pub instance_url: Option<String>,
    /// Set an alias for the authenticated org.
    #[arg(short = 'a', long)]
    pub alias: Option<String>,
    /// Set the newly authenticated org as the default target org.
    #[arg(long)]
    pub set_default: bool,
}

/// Arguments for `org login jwt`.
#[derive(Debug, Args)]
pub struct JwtArgs {
    /// The username to authenticate as.
    #[arg(long)]
    pub username: String,
    /// The connected app consumer key (client id).
    #[arg(long)]
    pub client_id: String,
    /// Path to the RSA private key (PEM) used to sign the JWT.
    #[arg(long)]
    pub jwt_key_file: String,
    /// The login URL (defaults to https://login.salesforce.com).
    #[arg(long)]
    pub instance_url: Option<String>,
    /// Set an alias for the authenticated org.
    #[arg(short = 'a', long)]
    pub alias: Option<String>,
    /// Set the newly authenticated org as the default target org.
    #[arg(long)]
    pub set_default: bool,
}

/// Arguments for `org logout`.
#[derive(Debug, Args)]
pub struct LogoutArgs {
    /// Remove all locally stored orgs.
    #[arg(long)]
    pub all: bool,
}

/// Arguments for `org list`.
#[derive(Debug, Args)]
pub struct ListArgs {}

/// Arguments for `org display`.
#[derive(Debug, Args)]
pub struct DisplayArgs {}

/// Dispatch an `org` subcommand.
pub fn run(ctx: &CommandContext, cmd: &OrgCommand) -> Result<()> {
    match cmd {
        OrgCommand::Login(LoginCommand::Web(args)) => login_web(ctx, args),
        OrgCommand::Login(LoginCommand::Jwt(args)) => login_jwt(ctx, args),
        OrgCommand::Logout(args) => logout(ctx, args),
        OrgCommand::List(_) => list(ctx),
        OrgCommand::Display(_) => display(ctx),
    }
}

/// A trimmed view of an authenticated org used in success output.
#[derive(Debug, Serialize)]
struct OrgSummary {
    username: String,
    #[serde(rename = "instanceUrl")]
    instance_url: String,
    #[serde(rename = "orgId", skip_serializing_if = "Option::is_none")]
    org_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    alias: Option<String>,
}

impl From<&AuthInfo> for OrgSummary {
    fn from(info: &AuthInfo) -> Self {
        OrgSummary {
            username: info.username.clone(),
            instance_url: info.instance_url.clone(),
            org_id: info.org_id.clone(),
            alias: info.alias.clone(),
        }
    }
}

fn login_web(ctx: &CommandContext, args: &WebArgs) -> Result<()> {
    let mut info = web::login(args.instance_url.as_deref())?;
    info.alias = args.alias.clone();
    store::save(&info)?;
    if args.set_default {
        let key = info.alias.clone().unwrap_or_else(|| info.username.clone());
        store::set_default_target_org(Some(&key))?;
    }
    let summary = OrgSummary::from(&info);
    print_success(ctx.output, summary, vec![], render_login);
    Ok(())
}

fn login_jwt(ctx: &CommandContext, args: &JwtArgs) -> Result<()> {
    let mut info = jwt::login(
        &args.username,
        &args.client_id,
        std::path::Path::new(&args.jwt_key_file),
        args.instance_url.as_deref(),
    )?;
    info.alias = args.alias.clone();
    store::save(&info)?;
    if args.set_default {
        let key = info.alias.clone().unwrap_or_else(|| info.username.clone());
        store::set_default_target_org(Some(&key))?;
    }
    let summary = OrgSummary::from(&info);
    print_success(ctx.output, summary, vec![], render_login);
    Ok(())
}

fn render_login(s: &OrgSummary) {
    println!("Successfully authorized {}", s.username);
    println!("  Instance URL: {}", s.instance_url);
    if let Some(org_id) = &s.org_id {
        println!("  Org Id:       {org_id}");
    }
    if let Some(alias) = &s.alias {
        println!("  Alias:        {alias}");
    }
}

#[derive(Debug, Serialize)]
struct LogoutResult {
    #[serde(rename = "removedUsernames")]
    removed: Vec<String>,
}

fn logout(ctx: &CommandContext, args: &LogoutArgs) -> Result<()> {
    let mut removed = Vec::new();
    if args.all {
        for info in store::list() {
            if store::remove(&info.username)? {
                removed.push(info.username);
            }
        }
    } else {
        let org = ctx.resolve_target_org()?;
        if store::remove(&org)? {
            removed.push(org);
        } else {
            return Err(SfError::NotAuthenticated(org));
        }
    }
    let result = LogoutResult { removed };
    print_success(ctx.output, result, vec![], |r| {
        if r.removed.is_empty() {
            println!("No orgs were removed.");
        } else {
            println!("Logged out of: {}", r.removed.join(", "));
        }
    });
    Ok(())
}

#[derive(Debug, Serialize)]
struct OrgListEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    alias: Option<String>,
    username: String,
    #[serde(rename = "orgId", skip_serializing_if = "Option::is_none")]
    org_id: Option<String>,
    #[serde(rename = "instanceUrl")]
    instance_url: String,
    #[serde(rename = "connectedStatus")]
    connected_status: String,
}

fn list(ctx: &CommandContext) -> Result<()> {
    let default = store::default_target_org()?;
    let entries: Vec<OrgListEntry> = store::list()
        .into_iter()
        .map(|info| {
            let is_default = default.as_deref() == Some(&info.username)
                || (info.alias.is_some() && default.as_deref() == info.alias.as_deref());
            OrgListEntry {
                alias: info.alias,
                username: info.username,
                org_id: info.org_id,
                instance_url: info.instance_url,
                connected_status: if is_default {
                    "Connected (default)".to_string()
                } else {
                    "Connected".to_string()
                },
            }
        })
        .collect();

    print_success(ctx.output, entries, vec![], |entries| {
        if entries.is_empty() {
            println!("No authenticated orgs found. Run `oxysf org login web` to add one.");
            return;
        }
        let rows: Vec<Vec<String>> = entries
            .iter()
            .map(|e| {
                vec![
                    e.alias.clone().unwrap_or_default(),
                    e.username.clone(),
                    e.org_id.clone().unwrap_or_default(),
                    e.instance_url.clone(),
                    e.connected_status.clone(),
                ]
            })
            .collect();
        let table = render_table(
            &["Alias", "Username", "Org Id", "Instance Url", "Status"],
            &rows,
        );
        println!("{table}");
    });
    Ok(())
}

#[derive(Debug, Serialize)]
struct DisplayResult {
    username: String,
    #[serde(rename = "instanceUrl")]
    instance_url: String,
    #[serde(rename = "loginUrl")]
    login_url: String,
    #[serde(rename = "clientId")]
    client_id: String,
    #[serde(rename = "orgId", skip_serializing_if = "Option::is_none")]
    org_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    alias: Option<String>,
    #[serde(rename = "accessToken")]
    access_token: String,
}

fn mask(token: &str) -> String {
    if token.len() <= 4 {
        return "****".to_string();
    }
    let tail = &token[token.len() - 4..];
    format!("****{tail}")
}

fn display(ctx: &CommandContext) -> Result<()> {
    let info = ctx.load_target_auth()?;
    let access_token = if ctx.verbose {
        info.access_token.clone()
    } else {
        mask(&info.access_token)
    };
    let result = DisplayResult {
        username: info.username,
        instance_url: info.instance_url,
        login_url: info.login_url,
        client_id: info.client_id,
        org_id: info.org_id,
        alias: info.alias,
        access_token,
    };

    print_success(ctx.output, result, vec![], |r| {
        let rows = vec![
            vec!["Username".to_string(), r.username.clone()],
            vec!["Alias".to_string(), r.alias.clone().unwrap_or_default()],
            vec!["Org Id".to_string(), r.org_id.clone().unwrap_or_default()],
            vec!["Instance Url".to_string(), r.instance_url.clone()],
            vec!["Login Url".to_string(), r.login_url.clone()],
            vec!["Client Id".to_string(), r.client_id.clone()],
            vec!["Access Token".to_string(), r.access_token.clone()],
        ];
        let table = render_table(&["Key", "Value"], &rows);
        println!("{table}");
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn masks_token() {
        assert_eq!(mask("00Dxx00000ABCDEF"), "****CDEF");
        assert_eq!(mask("abc"), "****");
    }
}
