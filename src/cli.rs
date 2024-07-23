use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Cli {
    #[arg(short = 'C')]
    pub repo_path: Option<PathBuf>,

    #[arg(short, long)]
    pub interactive: bool,

    #[arg(short = 'n', long)]
    pub dry_run: bool,

    #[arg(long)]
    pub onto: Option<String>,

    pub upstream: Option<String>,
    pub branch: Option<String>,
}
