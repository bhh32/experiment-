//! Command-line interface definition (clap derive).

use clap::{Args, Parser, Subcommand};

/// oxy-sf: a fast native reimplementation of common Salesforce CLI commands.
#[derive(Debug, Parser)]
#[command(name = "oxysf", version, about, long_about = None)]
pub struct Cli {
    /// The topic to run.
    #[command(subcommand)]
    pub topic: Topic,

    #[command(flatten)]
    pub global: GlobalArgs,
}

/// Global flags available on every command.
#[derive(Debug, Clone, Args)]
pub struct GlobalArgs {
    /// Emit machine-readable JSON output.
    #[arg(long, global = true)]
    pub json: bool,

    /// The username or alias of the target org.
    #[arg(short = 'o', long, global = true)]
    pub target_org: Option<String>,

    /// Override the Salesforce REST API version (e.g. 62.0).
    #[arg(long, global = true)]
    pub api_version: Option<String>,

    /// Enable verbose output.
    #[arg(short = 'v', long, global = true)]
    pub verbose: bool,
}

/// Top-level command topics.
#[derive(Debug, Subcommand)]
pub enum Topic {
    /// Manage and authenticate Salesforce orgs.
    #[command(subcommand)]
    Org(oxysf_org::OrgCommand),
    /// Query and manipulate org data.
    #[command(subcommand)]
    Data(oxysf_data::DataCommand),
}
