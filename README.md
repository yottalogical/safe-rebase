# safe-rebase

A tool for automatically detecting if a Git rebase is safe.

The official Git documentation outlines a single rule you must follow in order for rebases to be safe:

> [Do not rebase commits that exist outside your repository and that people may have based work on.](https://git-scm.com/book/en/v2/Git-Branching-Rebasing#_rebase_peril)

This tool will scan your entire repository looking for any work based on the commits you are about to rebase. If it
doesn't find any, it will start a rebase like normal. If it does find some, it will warn you and show you where in the
commit graph it can be found.

Warning: It's impossible for this tool to perfectly detect whether a rebase if safe. It's possible that new work based
on those commits exists on other machines, but hasn't been pushed yet, or perhaps someone will base work on them later
before they pull your rebased branch. I recommend you continue to communicate with your team about rebases as you
normally would, and use this tool as an additional safety check.

## Usage

To use safe-rebase, run a rebase command with all the same arguments and options as you normally would, but replace `git
rebase` with `safe-rebase`. Example:

```sh
# Normal
git rebase main

# With safe-rebase
safe-rebase main
```

safe-rebase currently supports the following options:

- `-C`: [Repository path](https://git-scm.com/docs/git#Documentation/git.txt--Cpath)
- `-i`, `--interactive`: [Interactive](https://git-scm.com/docs/git-rebase#Documentation/git-rebase.txt--i)
- `-n`, `--dry-run`: Detect if it's safe to rebase without starting a rebase
- `--onto`: [New base](https://git-scm.com/docs/git-rebase#Documentation/git-rebase.txt---ontonewbase)
- `--autostash`: [Autostash](https://git-scm.com/docs/git-rebase#Documentation/git-rebase.txt---autostash)
- `-h`, `--help`: Print help for safe-rebase
- `-V`, `--version`: Print version of safe-rebase

## Installation

1. Install the [Rust toolchain](https://rust-lang.org/tools/install/) if you haven't already
2. Run `cargo install --git https://github.com/yottalogical/safe-rebase.git`

(Cloning this repository manually is not necessary; Cargo will handle it for you)
