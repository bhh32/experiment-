//! Build the shared [`CommandContext`] from parsed global flags.

use oxysf_core::context::CommandContext;
use oxysf_core::output::OutputMode;

use crate::cli::GlobalArgs;

/// Build a [`CommandContext`] from the parsed global CLI flags.
pub fn build_context(global: &GlobalArgs) -> CommandContext {
    let output = if global.json {
        OutputMode::Json
    } else {
        OutputMode::Human
    };
    CommandContext {
        output,
        target_org: global.target_org.clone(),
        api_version: global.api_version.clone(),
        verbose: global.verbose,
    }
}
