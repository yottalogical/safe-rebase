use git2::{AnnotatedCommit, Oid, RebaseOperationType, RebaseOptions, Repository};

// Reasons to that it might be unsafe to rebase
// 1. A commit not being rebased is a child of a commit being rebased
// 2. A branch not being rebased is pointing to a commit being rebased
// 3. A merge commit is being rebased

fn main() {
    let repo = Repository::discover(concat!(env!("CARGO_MANIFEST_DIR"), "/test-repo")).unwrap();

    let upstream = "k";
    let branch = "main";

    let upstream = short_name_to_annotated_commit(&repo, upstream);
    let branch = short_name_to_annotated_commit(&repo, branch);

    let commits_to_rebase = get_commits_to_rebase(&repo, &upstream, &branch, &upstream);

    for rev in commits_to_rebase {
        let commit = repo.find_commit(rev).unwrap();
        println!("{}", commit.message().unwrap());
    }
}

fn short_name_to_annotated_commit<'a>(repo: &'a Repository, branch: &str) -> AnnotatedCommit<'a> {
    repo.reference_to_annotated_commit(&repo.resolve_reference_from_short_name(branch).unwrap())
        .unwrap()
}

fn get_commits_to_rebase(
    repo: &Repository,
    upstream: &AnnotatedCommit,
    branch: &AnnotatedCommit,
    onto: &AnnotatedCommit,
) -> Vec<Oid> {
    repo.rebase(
        Some(branch),
        Some(upstream),
        Some(onto),
        Some(&mut RebaseOptions::new().inmemory(true)),
    )
    .unwrap()
    .map(Result::unwrap)
    .filter_map(|operation| match operation.kind().unwrap() {
        RebaseOperationType::Pick
        | RebaseOperationType::Reword
        | RebaseOperationType::Edit
        | RebaseOperationType::Squash
        | RebaseOperationType::Fixup => Some(operation.id()),
        RebaseOperationType::Exec => None,
    })
    .collect()
}
