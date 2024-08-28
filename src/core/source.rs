use crate::core::TARGET_REPO_NAME;
use anyhow::{bail, Result};
use git2::{Commit, Repository, Sort};
use std::collections::HashSet;
use std::path::Path;

pub struct SourceRepo {
    repo: Repository,
    remote_url: String,
    author_email: String,
}

impl SourceRepo {
    pub fn read_all_in_dir(dir: &Path) -> Result<Vec<SourceRepo>> {
        let mut seen_remotes = HashSet::new();
        let mut repositories = Vec::new();

        if Self::is_dir_git_repo(dir) {
            let dir_absolute = dir.canonicalize()?;
            let dir_name = dir_absolute.file_name().unwrap();

            if dir_name.eq(TARGET_REPO_NAME) {
                bail!("cannot read {TARGET_REPO_NAME} repository as input")
            }

            let repo = Repository::open(dir)?;
            let repo = Self::new(repo)?;

            repositories.push(repo);

            return Ok(repositories);
        }

        for entry in dir.read_dir()? {
            let entry = entry?;
            let entry_path = entry.path();
            let entry_name = entry.file_name();

            if !Self::is_dir_git_repo(&entry_path) {
                continue;
            }

            if entry_name.eq(TARGET_REPO_NAME) {
                continue;
            }

            let repo = Repository::open(entry_path)?;
            let repo = Self::new(repo)?;

            if seen_remotes.insert(repo.remote_url.clone()) {
                repositories.push(repo);
            } else {
                println!("Ignoring duplicate repository at {}.", repo.remote_url)
            }
        }

        if repositories.is_empty() {
            bail!("no repositories found in the input directory")
        }

        Ok(repositories)
    }

    pub fn name(&self) -> &str {
        &self.remote_url
    }

    pub fn get_author_commits(&self) -> Result<Vec<Commit>> {
        let mut revision_walker = self.repo.revwalk()?;

        revision_walker.set_sorting(Sort::TOPOLOGICAL | Sort::REVERSE)?;
        revision_walker.push_head()?;

        let mut author_commits = Vec::new();

        for revision in revision_walker {
            let oid = revision?;
            let commit = self.repo.find_commit(oid)?;

            let does_email_match = {
                let commit_author = commit.author();
                let commit_author_email = commit_author.email().unwrap_or("unknown");

                self.author_email.eq(commit_author_email)
            };

            if !does_email_match {
                continue;
            }

            author_commits.push(commit);
        }

        Ok(author_commits)
    }

    fn is_dir_git_repo(dir: &Path) -> bool {
        dir.join(".git").is_dir()
    }

    fn new(repo: Repository) -> Result<SourceRepo> {
        let repo_config = repo.config()?;

        Ok(SourceRepo {
            repo,
            remote_url: repo_config.get_string("remote.origin.url")?,
            author_email: repo_config.get_string("user.email")?,
        })
    }
}
