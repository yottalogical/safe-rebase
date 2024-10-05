use std::process::ExitCode;

use clap::Parser;
use cli::Cli;
use safe_rebase::safe_rebase;

mod cli;
mod safe_rebase;

fn main() -> ExitCode {
    let Cli {
        repo_path,
        interactive,
        dry_run,
        onto,
        upstream,
        branch,
    } = Cli::parse();

    let result = safe_rebase(
        repo_path.as_deref(),
        upstream.as_deref(),
        branch.as_deref(),
        interactive,
        dry_run,
        onto.as_deref(),
    );

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(()) => ExitCode::FAILURE,
    }
}
