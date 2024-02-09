use crate::email::EmailAddress;
use git2::{Commit, Repository};
use std::io;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RepoError {
    #[error("Git error: {0}")]
    Git2(#[from] git2::Error),
    #[error("IO error: {0}")]
    FileSystem(#[from] io::Error),
}

pub fn get_repositories_in_dir(dir: &Path) -> Result<Vec<Repository>, RepoError> {
    let mut repositories = Vec::new();

    // If we are inside a Git repository already, then return that.
    if dir.join(".git").is_dir() {
        repositories.push(Repository::open(dir)?);
    }

    if !repositories.is_empty() {
        return Ok(repositories);
    }

    for entry in dir.read_dir()? {
        let entry = entry?;
        let git_path = Path::join(&entry.path(), ".git");

        // Ignore entries that do not contain a .git directory.
        if !Path::new(&git_path).is_dir() {
            continue;
        }

        repositories.push(Repository::open(entry.path())?);
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
