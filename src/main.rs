use clap::Parser;
use cli::Cli;
use safe_rebase::safe_rebase;

mod cli;
mod safe_rebase;

fn main() {
    let Cli {
        repo_path,
        interactive,
        onto,
        upstream,
        branch,
    } = Cli::parse();

    safe_rebase(
        repo_path.as_deref(),
        upstream.as_deref(),
        branch.as_deref(),
        interactive,
        onto.as_deref(),
    );
}
