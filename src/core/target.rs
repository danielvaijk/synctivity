use crate::core::{SourceRepo, TARGET_REPO_NAME};
use anyhow::{bail, Result};
use git2::{Commit, Oid, Repository, RepositoryInitOptions, Signature, Tree};
use std::path::Path;

pub struct TargetRepo {
    repo: Repository,
    parents: Vec<Oid>,
}

impl TargetRepo {
    pub fn read_or_create(path: &Path) -> Result<TargetRepo> {
        let mut repo: Option<Repository> = None;
        let repo_path = path.join(TARGET_REPO_NAME);

        if let Ok(existing_repo) = Repository::open(&repo_path) {
            if existing_repo.head().is_ok() {
                bail!("Cannot handle existing {TARGET_REPO_NAME} repository history yet.",);
            }

            repo = Some(existing_repo);
        };

        if repo.is_none() {
            let mut options = RepositoryInitOptions::new();
            let options = options.initial_head("main");

            repo = Some(Repository::init_opts(&repo_path, options)?);
        }

        let repo = repo.unwrap();

        // We always start from scratch since we don't handle history delta's yet,
        // so there isn't a parent commit ID to start from.
        let parents: Vec<Oid> = Vec::new();

        Ok(TargetRepo { repo, parents })
    }

    pub fn copy_matching_commits(&mut self, repos_to_copy: &Vec<SourceRepo>) -> Result<()> {
        let mut commit_iters = Vec::with_capacity(repos_to_copy.len());

        for repo in repos_to_copy.iter() {
            let commits = repo.get_author_commits()?;
            let result = (repo.name(), commits.len(), commits.into_iter());

            commit_iters.push(result)
        }

        while commit_iters.iter().any(|(.., iter)| iter.len() > 0) {
            for item in commit_iters.iter_mut() {
                let (repo_name, commit_count, commit_iter) = item;
                let commit = commit_iter.next().unwrap();

                self.copy_author_commit(&commit)?;

                if commit_iter.len().eq(&0) {
                    println!("Synced {} commit(s) from {}.", commit_count, repo_name);
                }
            }
        }

        Ok(())
    }

    fn copy_author_commit(&mut self, commit: &Commit) -> Result<()> {
        let commit_id = {
            let tree = Self::create_empty_tree(&self.repo)?;

            let parents = self.get_parent_commits()?;
            let parents: Vec<&Commit> = parents.iter().collect();

            let commit_author = commit.author();

            let signature = Signature::new(
                commit_author.name().unwrap(),
                commit_author.email().unwrap(),
                &commit.author().when(),
            )?;

            self.repo.commit(
                Some("HEAD"),
                &signature,
                &signature,
                commit.message().unwrap(),
                &tree,
                &parents,
            )?
        };

        self.parents.clear();
        self.parents.push(commit_id);

        Ok(())
    }

    fn get_parent_commits(&self) -> Result<Vec<Commit>> {
        let mut commit_refs = Vec::with_capacity(self.parents.len());

        for commit_id in self.parents.iter() {
            commit_refs.push(self.repo.find_commit(*commit_id)?)
        }

        Ok(commit_refs)
    }

    // Since the commits never contain any changes, we always (re)use an empty
    // tree object. There's no file/directory information to include.
    fn create_empty_tree(repo: &Repository) -> Result<Tree> {
        let tree = repo.treebuilder(None)?.write()?;
        let tree = repo.find_tree(tree)?;

        Ok(tree)
    }
}
