use git2::{Oid, Repository};

// Reasons to that it might be unsafe to rebase
// 1. A commit not being rebased is a child of a commit being rebased
// 2. A branch not being rebased is pointing to a commit being rebased
// 3. A merge commit is being rebased

fn main() {
    let repo = Repository::discover("test-repo").unwrap();

    let upstream = "k";
    let branch = "main";

    let upstream = short_name_to_oid(&repo, upstream);
    let branch = short_name_to_oid(&repo, branch);

    let commits_to_rebase = get_commits_to_rebase(&repo, upstream, branch);

    for rev in commits_to_rebase {
        let commit = repo.find_commit(rev).unwrap();
        println!("{}", commit.message().unwrap());
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

fn get_commits_to_rebase(repo: &Repository, upstream: Oid, branch: Oid) -> Vec<Oid> {
    let mut revwalk = repo.revwalk().unwrap();
    revwalk.push(branch).unwrap();
    revwalk.hide(upstream).unwrap();

    revwalk.map(Result::unwrap).collect()
}
