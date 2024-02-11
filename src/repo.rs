use crate::email::EmailAddress;
use crate::SYNC_REPO_NAME;
use git2::{Commit, Repository, RepositoryInitOptions, Sort, Tree};
use std::io;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RepoError {
    #[error("Git error: {0}")]
    Git2(#[from] git2::Error),
    #[error("IO error: {0}")]
    FileSystem(#[from] io::Error),
    #[error("Validation error: {0}")]
    Validation(String),
}

pub fn read_all_in_dir(dir: &Path) -> Result<Vec<Repository>, RepoError> {
    let mut repositories = Vec::new();

    if is_dir_git_repo(&dir) {
        let dir_absolute = dir.canonicalize()?;
        let dir_name = dir_absolute.file_name().unwrap();

        return if dir_name == SYNC_REPO_NAME {
            Err(RepoError::Validation(format!(
                "Cannot read {SYNC_REPO_NAME} repository as input."
            )))
        } else {
            repositories.push(Repository::open(dir)?);
            Ok(repositories)
        };
    }

    for entry in dir.read_dir()? {
        let entry = entry?;
        let entry_path = entry.path();
        let entry_name = entry.file_name();

        if !is_dir_git_repo(&entry_path) {
            continue;
        } else if entry_name == SYNC_REPO_NAME {
            continue;
        }

        repositories.push(Repository::open(&entry_path)?)
    }

    Ok(repositories)
}

pub fn get_all_commits_by_emails<'repo>(
    repository: &'repo Repository,
    emails: &[EmailAddress],
) -> Result<(u32, Vec<Commit<'repo>>), RepoError> {
    let mut revision_walker = repository.revwalk()?;

    revision_walker.set_sorting(Sort::TOPOLOGICAL | Sort::REVERSE)?;
    revision_walker.push_head()?;

    let mut revision_count: u32 = 0;
    let mut found_commits = Vec::new();

    for revision in revision_walker {
        let oid = revision?;
        let commit = repository.find_commit(oid)?;

        revision_count += 1;

        let does_email_match = {
            let commit_author = commit.author();
            let commit_author_email = commit_author.email().unwrap_or("unknown");

            emails
                .iter()
                .any(|EmailAddress(email)| email == commit_author_email)
        };

        if !does_email_match {
            continue;
        }

        found_commits.push(commit);
    }

    Ok((revision_count, found_commits))
}

pub fn read_or_create(path: &Path) -> Result<Repository, RepoError> {
    if let Ok(existing_repo) = Repository::open(path) {
        return Ok(existing_repo);
    };

    let mut options = RepositoryInitOptions::new();
    let options = options.initial_head("main");

    Ok(Repository::init_opts(path, &options)?)
}

pub fn create_empty_tree(repo: &Repository) -> Result<Tree, RepoError> {
    let tree = repo.treebuilder(None)?.write()?;
    let tree = repo.find_tree(tree)?;

    Ok(tree)
}

pub fn copy_over_commits<'repo>(
    repo: &'repo Repository,
    parents: &mut Vec<Commit<'repo>>,
    tree: &Tree,
    commits: &Vec<Commit>,
) -> Result<(), RepoError> {
    for commit in commits {
        let parents_ref: Vec<&Commit> = parents.iter().collect();

        let message = commit.message().unwrap();
        let signature = commit.author();

        let commit_id = repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &parents_ref,
        )?;

        parents.clear();
        parents.push(repo.find_commit(commit_id)?);
    }

    Ok(())
}

fn is_dir_git_repo(dir: &Path) -> bool {
    dir.join(".git").is_dir()
}
