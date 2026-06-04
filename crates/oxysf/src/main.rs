//! The `oxysf` binary: parse CLI args, build context, dispatch to topics.
//!
//! oxy-sf is an independent project and is not affiliated with or endorsed by
//! Salesforce.

#![forbid(unsafe_code)]

mod cli;
mod context;

use clap::Parser;
use oxysf_core::error::Result;
use oxysf_core::output::print_error;

use cli::{Cli, Topic};

fn main() {
    let cli = Cli::parse();
    let ctx = context::build_context(&cli.global);

    let result: Result<()> = match &cli.topic {
        Topic::Org(cmd) => oxysf_org::run(&ctx, cmd),
        Topic::Data(cmd) => oxysf_data::run(&ctx, cmd),
    };

    if let Err(err) = result {
        print_error(ctx.output, &err);
        std::process::exit(err.exit_code());
    }
}
