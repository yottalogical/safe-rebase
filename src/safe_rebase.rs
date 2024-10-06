use std::{
    collections::{HashMap, HashSet},
    env::current_dir,
    io::{stdin, stdout, Write},
    path::Path,
    process::{Command, ExitStatus, Stdio},
};

use git2::{Branch, BranchType, Commit, Oid, Reference, Repository};

mod tests;

pub fn safe_rebase(
    repo_path: Option<&Path>,
    upstream: Option<&str>,
    branch: Option<&str>,
    interactive: bool,
    dry_run: bool,
    onto: Option<&str>,
) -> Result<(), ()> {
    // Get repo
    let repo = Repository::discover(repo_path.unwrap_or(&current_dir().unwrap())).unwrap();

    // Calculate upstream and branch
    let (upstream, branch) = get_upstream_and_branch(&repo, upstream, branch);

    // Check if it's safe to rebase
    let safe_to_rebase = safe_to_rebase(&repo, &upstream, &branch);

    // Perform rebase
    match safe_to_rebase {
        Ok(()) => {
            if dry_run {
                println!("Safe to rebase!");

                Ok(())
            } else {
                let result = rebase(&repo, &upstream, &branch, interactive, onto);

                match result {
                    Ok(_) => {
                        println!("Please run `git push --force-with-lease` soon.");

                        Ok(())
                    }
                    Err(_) => Err(()),
                }
            }
        }
        Err(references_with_commits) => {
            report_unsafe_to_rebase(&repo, &upstream, &branch, &references_with_commits);

            Err(())
        }
    }
}

fn safe_to_rebase<'repo>(
    repo: &'repo Repository,
    upstream: &Commit,
    branch: &Branch,
) -> Result<(), Vec<Reference<'repo>>> {
    // Prefetch
    prefetch(repo);

    // Find all references (excluding branch)
    let references = find_all_references(repo, branch);

    // Get all commits that will be rebased
    let commits_to_rebase = get_commits_to_rebase(repo, upstream, branch);

    // Look for commits
    look_for_commits(repo, references, upstream, &commits_to_rebase)
}

fn prefetch(repo: &Repository) {
    git(repo, ["fetch", "--prefetch", "--prune"], true).unwrap();
}

fn get_upstream_and_branch<'repo>(
    repo: &'repo Repository,
    upstream: Option<&str>,
    branch: Option<&str>,
) -> (Commit<'repo>, Branch<'repo>) {
    let branch = if let Some(branch) = branch {
        repo.find_branch(branch, BranchType::Local).unwrap()
    } else {
        Branch::wrap(repo.head().unwrap())
    };

    let upstream = if let Some(upstream) = upstream {
        match repo.resolve_reference_from_short_name(upstream) {
            Ok(upstream) => upstream.peel_to_commit().unwrap(),
            Err(_) => repo.find_commit_by_prefix(upstream).unwrap(),
        }
    } else {
        branch.upstream().unwrap().get().peel_to_commit().unwrap()
    };

    (upstream, branch)
}

fn find_all_references<'repo>(
    repo: &'repo Repository,
    exception: &Branch,
) -> Vec<Reference<'repo>> {
    let all_references = repo
        .references()
        .unwrap()
        .map(Result::unwrap)
        .filter(|reference| !references_the_same(reference, exception.get()))
        .filter(|reference| reference.name() != Some("refs/stash"));

    if let Ok(exception_upstream) = exception.upstream() {
        let exception_upstream = exception_upstream.into_reference();
        let exception_upstream_prefetch = get_prefetch_reference(repo, &exception_upstream);

        all_references
            .filter(|reference| !references_the_same(reference, &exception_upstream))
            .filter(|reference| !references_the_same(reference, &exception_upstream_prefetch))
            .collect()
    } else {
        all_references.collect()
    }
}

fn references_the_same(reference1: &Reference, reference2: &Reference) -> bool {
    match (reference1.name(), reference2.name()) {
        (Some(name1), Some(name2)) => name1 == name2,
        _ => false,
    }
}

fn get_prefetch_reference<'repo>(
    repo: &'repo Repository,
    reference: &Reference,
) -> Reference<'repo> {
    static EXPECTED_BEGINNING: &str = "refs/";
    let reference_name = reference.name().unwrap();

    assert!(reference.is_remote());
    assert_eq!(
        &reference_name[..EXPECTED_BEGINNING.len()],
        EXPECTED_BEGINNING,
    );

    repo.find_reference(&format!(
        "refs/prefetch/{}",
        &reference_name[EXPECTED_BEGINNING.len()..],
    ))
    .unwrap()
}

fn get_commits_to_rebase(repo: &Repository, upstream: &Commit, branch: &Branch) -> HashSet<Oid> {
    let mut revwalk = repo.revwalk().unwrap();
    revwalk
        .push(branch.get().peel_to_commit().unwrap().id())
        .unwrap();
    revwalk.hide(upstream.id()).unwrap();

    revwalk.map(Result::unwrap).collect()
}

fn look_for_commits<'repo>(
    repo: &'repo Repository,
    starting_points: Vec<Reference<'repo>>,
    ignore: &Commit,
    commits: &HashSet<Oid>,
) -> Result<(), Vec<Reference<'repo>>> {
    let mut revwalk = repo.revwalk().unwrap();
    revwalk.hide(ignore.id()).unwrap();
    for reference in &starting_points {
        revwalk
            .push(reference.peel_to_commit().unwrap().id())
            .unwrap();
    }

    let child_to_parent: HashMap<Oid, Vec<Oid>> = revwalk
        .map(Result::unwrap)
        .map(|oid| {
            (
                oid,
                repo.find_commit(oid)
                    .unwrap()
                    .parents()
                    .map(|parent| parent.id())
                    .collect(),
            )
        })
        .collect();

    let mut references_with_commits = Vec::new();

    for reference in starting_points {
        let mut queue = Vec::from([reference.peel_to_commit().unwrap().id()]);

        while let Some(child) = queue.pop() {
            if commits.contains(&child) {
                references_with_commits.push(reference);
                break;
            }

            if let Some(parents) = child_to_parent.get(&child) {
                queue.extend(parents);
            }
        }
    }

    if references_with_commits.is_empty() {
        Ok(())
    } else {
        Err(references_with_commits)
    }
}

fn rebase(
    repo: &Repository,
    upstream: &Commit,
    branch: &Branch,
    interactive: bool,
    onto: Option<&str>,
) -> Result<ExitStatus, ExitStatus> {
    let mut args = Vec::from(["rebase"]);

    if interactive {
        args.push("--interactive");
    }

    if let Some(onto) = onto {
        args.push("--onto");
        args.push(onto);
    }

    let upstream_id = upstream.id().to_string();
    args.push(&upstream_id);
    args.push(branch.name().unwrap().unwrap());

    git(repo, args, false)
}

fn report_unsafe_to_rebase(
    repo: &Repository,
    upstream: &Commit,
    branch: &Branch,
    references_with_commits: &[Reference],
) {
    print!("Unsafe to rebase. See why (y/n)? ");
    stdout().flush().unwrap();

    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();

    if input.trim() == "y" {
        let upstream_id = upstream.id().to_string();
        let mut args = Vec::from([
            "log",
            "--graph",
            "--oneline",
            branch.get().name().unwrap(),
            &upstream_id,
        ]);
        args.append(
            &mut references_with_commits
                .iter()
                .map(|reference| reference.name().unwrap())
                .collect(),
        );

        git(repo, args, false).unwrap();
    }
}

fn git<'a>(
    repo: &Repository,
    args: impl IntoIterator<Item = &'a str>,
    hide_output: bool,
) -> Result<ExitStatus, ExitStatus> {
    let mut command = Command::new("git");
    command.arg("-C").arg(repo.workdir().unwrap());
    command.args(args);

    if hide_output {
        command.stderr(Stdio::null());
    }

    let exit_status = command.spawn().unwrap().wait().unwrap();

    if exit_status.success() {
        Ok(exit_status)
    } else {
        Err(exit_status)
    }
}
