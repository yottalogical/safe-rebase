use std::{collections::HashSet, process::Command};

use git2::{Branch, BranchType, Oid, Reference, Repository};

fn main() {
    let upstream = "k";
    let branch = "main";

    // Get repo
    let repo = Repository::discover("test-repo").unwrap();

    // Prefetch
    prefetch(&repo);

    // Calculate upstream and branch
    let upstream = repo.resolve_reference_from_short_name(upstream).unwrap();
    let branch = repo.find_branch(branch, BranchType::Local).unwrap();

    // Find all references (excluding branch)
    let references = find_all_references(&repo, &branch);

    // Get all commits that will be rebased
    let commits_to_rebase = get_commits_to_rebase(&repo, &upstream, &branch);

    // Look for commits
    let unsafe_to_rebase = look_for_commits(&repo, &references, &commits_to_rebase);

    // Perform rebase
    if unsafe_to_rebase {
        println!("Unsafe to rebase");
    } else {
        println!(
            "git -C {} rebase {} {}",
            repo.path().to_str().unwrap(),
            upstream.name().unwrap(),
            branch.name().unwrap().unwrap(),
        );
    }
}

fn prefetch(repo: &Repository) {
    let exit_status = Command::new("git")
        .arg("-C")
        .arg(repo.path())
        .arg("fetch")
        .arg("--prefetch")
        .current_dir("/Users/adam/Documents/Git.nosync/safe-to-rebase")
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    if !exit_status.success() {
        panic!("git exit status: {}", exit_status.code().unwrap());
    }
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

fn get_commits_to_rebase<'repo>(
    repo: &'repo Repository,
    upstream: &Reference,
    branch: &Branch,
) -> HashSet<Oid> {
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
