use crate::email::EmailAddress;
use crate::SYNC_REPO_NAME;
use git2::{Commit, Repository, RepositoryInitOptions};
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
        let absolute_dir = dir.canonicalize()?;
        let dir_name = absolute_dir.file_name().unwrap();

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

pub fn get_commits_by_email<'repo>(
    repository: &'repo Repository,
    emails: &[EmailAddress],
) -> Result<(u32, Vec<Commit<'repo>>), RepoError> {
    let mut revision_walker = repository.revwalk()?;

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

fn is_dir_git_repo(dir: &Path) -> bool {
    dir.join(".git").is_dir()
}
