#![cfg(test)]

use std::{path::PathBuf, str::FromStr};

use git2::{Branch, Commit, Reference, Repository, Signature};
use rand::{distributions::Alphanumeric, Rng};

use super::safe_to_rebase;

#[test]
fn simple_safe() {
    let repo = tmp_repo();

    let commit_a = commit(&repo, "HEAD", "A", &[]);

    let main = repo.head().unwrap();
    let feature = repo.branch("feature", &commit_a, false).unwrap();

    let _commit_b = commit(&repo, main.name().unwrap(), "B", &[&commit_a]);
    let _commit_c = commit(&repo, feature.get().name().unwrap(), "C", &[&commit_a]);

    match call_safe_to_rebase(&repo, &main, &feature) {
        Ok(()) => {}
        Err(_) => panic!(),
    };
}

#[test]
fn simple_unsafe() {
    let repo = tmp_repo();

    let commit_a = commit(&repo, "HEAD", "A", &[]);

    let main = repo.head().unwrap();
    let feature = repo.branch("feature", &commit_a, false).unwrap();

    let _commit_b = commit(&repo, main.name().unwrap(), "B", &[&commit_a]);
    let commit_c = commit(&repo, feature.get().name().unwrap(), "C", &[&commit_a]);

    let other_branch = repo.branch("other-branch", &commit_c, false).unwrap();

    match call_safe_to_rebase(&repo, &main, &feature) {
        Ok(()) => panic!(),
        Err(references) => {
            let names: Vec<Option<&str>> = references
                .iter()
                .map(|reference| reference.name())
                .collect();

            assert_eq!(names, [other_branch.get().name()]);
        }
    };
}

fn call_safe_to_rebase<'repo>(
    repo: &'repo Repository,
    main: &Reference,
    feature: &Branch,
) -> Result<(), Vec<Reference<'repo>>> {
    let main = repo.find_reference(main.name().unwrap()).unwrap();
    let feature = repo
        .find_branch(feature.name().unwrap().unwrap(), git2::BranchType::Local)
        .unwrap();

    safe_to_rebase(&repo, &main, &feature)
}

fn tmp_repo() -> Repository {
    let random_string: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect();

    let path = PathBuf::from_str("/tmp/safe-rebase")
        .unwrap()
        .join(random_string);

    Repository::init(path).unwrap()
}

fn commit<'repo>(
    repo: &'repo Repository,
    update_ref: &str,
    message: &str,
    parents: &[&Commit],
) -> Commit<'repo> {
    let signature = Signature::now("John Doe", "johndoe@example.com").unwrap();

    let mut index = repo.index().unwrap();
    let tree_oid = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_oid).unwrap();

    repo.find_commit(
        repo.commit(
            Some(update_ref),
            &signature,
            &signature,
            message,
            &tree,
            parents,
        )
        .unwrap(),
    )
    .unwrap()
}
