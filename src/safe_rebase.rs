use std::{collections::HashSet, env::current_dir, path::Path, process::Command};

use git2::{Branch, BranchType, Oid, Reference, Repository};

mod tests;

pub fn safe_rebase(
    repo_path: Option<&Path>,
    upstream: Option<&str>,
    branch: Option<&str>,
    interactive: bool,
    onto: Option<&str>,
) {
    // Get repo
    let repo = Repository::discover(repo_path.unwrap_or(&current_dir().unwrap())).unwrap();

    // Calculate upstream and branch
    let (upstream, branch) = get_upstream_and_branch(&repo, upstream, branch);

    // Check if it's safe to rebase
    let safe_to_rebase = safe_to_rebase(&repo, &upstream, &branch);

    // Perform rebase
    if safe_to_rebase {
        rebase(&repo, &upstream, &branch, interactive, onto);
    } else {
        println!("Unsafe to rebase");
    }
}

fn safe_to_rebase(repo: &Repository, upstream: &Reference, branch: &Branch) -> bool {
    // Prefetch
    prefetch(repo);

    // Find all references (excluding branch)
    let references = find_all_references(repo, branch);

    // Get all commits that will be rebased
    let commits_to_rebase = get_commits_to_rebase(repo, upstream, branch);

    // Look for commits
    !look_for_commits(repo, &references, &commits_to_rebase)
}

fn prefetch(repo: &Repository) {
    git(repo, ["fetch", "--prefetch"]);
}

fn get_upstream_and_branch<'repo>(
    repo: &'repo Repository,
    upstream: Option<&str>,
    branch: Option<&str>,
) -> (Reference<'repo>, Branch<'repo>) {
    let branch = if let Some(branch) = branch {
        repo.find_branch(branch, BranchType::Local).unwrap()
    } else {
        Branch::wrap(repo.head().unwrap())
    };

    let upstream = if let Some(upstream) = upstream {
        repo.resolve_reference_from_short_name(upstream).unwrap()
    } else {
        branch.upstream().unwrap().into_reference()
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
        .filter(|reference| !references_the_same(reference, exception.get()));

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

fn get_commits_to_rebase(repo: &Repository, upstream: &Reference, branch: &Branch) -> HashSet<Oid> {
    let mut revwalk = repo.revwalk().unwrap();
    revwalk
        .push(branch.get().peel_to_commit().unwrap().id())
        .unwrap();
    revwalk
        .hide(upstream.peel_to_commit().unwrap().id())
        .unwrap();

    revwalk.map(Result::unwrap).collect()
}

fn look_for_commits(
    repo: &Repository,
    starting_points: &[Reference],
    commits: &HashSet<Oid>,
) -> bool {
    let mut revwalk = repo.revwalk().unwrap();

    for reference in starting_points {
        revwalk
            .push(reference.peel_to_commit().unwrap().id())
            .unwrap();
    }

    revwalk
        .map(Result::unwrap)
        .any(|oid| commits.contains(&oid))
}

fn rebase(
    repo: &Repository,
    upstream: &Reference,
    branch: &Branch,
    interactive: bool,
    onto: Option<&str>,
) {
    let mut args = Vec::from(["rebase"]);

    if interactive {
        args.push("--interactive");
    }

    if let Some(onto) = onto {
        args.push("--onto");
        args.push(onto);
    }

    args.push(upstream.name().unwrap());
    args.push(branch.name().unwrap().unwrap());

    git(repo, args);
}

fn git<'a>(repo: &Repository, args: impl IntoIterator<Item = &'a str>) {
    let exit_status = Command::new("git")
        .arg("-C")
        .arg(repo.workdir().unwrap())
        .args(args)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    if !exit_status.success() {
        panic!("git exit code: {}", exit_status.code().unwrap());
    }
}
