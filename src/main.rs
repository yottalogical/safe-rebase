use std::collections::HashMap;

use git2::{Commit, Oid, Repository};

// Reasons to that it might be unsafe to rebase
// 1. A commit not being rebased is a child of a commit being rebased
//      (but don't worry about commits that don't belong to any reference)
// 2. A branch not being rebased is pointing to a commit being rebased
// 3. A merge commit is being rebased
//      (this one might be okay, since the rebase command drops merge commits by default)

fn main() {
    let repo = Repository::discover("test-repo").unwrap();

    let upstream = "k";
    let branch = "main";

    let upstream = short_name_to_oid(&repo, upstream);
    let branch = short_name_to_oid(&repo, branch);

    let commits_to_rebase = get_commits_to_rebase(&repo, upstream, branch);

    let commits_to_children = get_commits_to_children(&repo);

    println!("commits_to_rebase:");
    for rev in commits_to_rebase {
        let commit = repo.find_commit(rev).unwrap();
        println!("{}", commit.message().unwrap().trim());
    }
    println!("");

    println!("commits_to_children:");
    for (parent, children) in commits_to_children {
        let parent = repo.find_commit(parent).unwrap();

        println!(
            "{}: {:?}",
            parent.message().unwrap().trim(),
            children
                .iter()
                .map(|commit| commit.message().unwrap().trim())
                .collect::<Vec<_>>(),
        );
    }
}

fn short_name_to_oid(repo: &Repository, branch: &str) -> Oid {
    repo.resolve_reference_from_short_name(branch)
        .unwrap()
        .resolve()
        .unwrap()
        .target()
        .unwrap()
}

fn get_commits_to_children(repo: &Repository) -> HashMap<Oid, Vec<Commit>> {
    let mut revwalk = repo.revwalk().unwrap();

    for reference in repo.references().unwrap() {
        revwalk
            .push(reference.unwrap().peel_to_commit().unwrap().id())
            .unwrap();
    }

    let mut commits_to_children: HashMap<Oid, Vec<Commit>> = HashMap::new();

    for oid in revwalk {
        let commit = repo.find_commit(oid.unwrap()).unwrap();
        commits_to_children.entry(commit.id()).or_default();

        for parent in commit.parents() {
            commits_to_children
                .entry(parent.id())
                .or_default()
                .push(commit.clone());
        }
    }

    commits_to_children
}

fn get_commits_to_rebase(repo: &Repository, upstream: Oid, branch: Oid) -> Vec<Oid> {
    let mut revwalk = repo.revwalk().unwrap();
    revwalk.push(branch).unwrap();
    revwalk.hide(upstream).unwrap();

    revwalk.map(Result::unwrap).collect()
}
