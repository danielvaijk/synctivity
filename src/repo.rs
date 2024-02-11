use crate::email::EmailAddress;
use crate::SYNC_REPO_NAME;
use git2::{Commit, Repository, RepositoryInitOptions, Signature, Sort, Tree};
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

pub struct Author<'repo> {
    name: &'repo str,
    email: &'repo str,
    email_aliases: &'repo Vec<EmailAddress>,
}

impl Author<'_> {
    pub fn new<'repo>(
        name: &'repo str,
        email: &'repo str,
        email_aliases: &'repo Vec<EmailAddress>,
    ) -> Author<'repo> {
        Author {
            name,
            email,
            email_aliases,
        }
    }
}

pub struct SyncRepo<'repo> {
    repo: &'repo Repository,
    author: &'repo Author<'repo>,
    tree: Tree<'repo>,
}

impl SyncRepo<'_> {
    pub fn from<'repo>(
        repo: &'repo Repository,
        author: &'repo Author,
    ) -> Result<SyncRepo<'repo>, RepoError> {
        // Since the commits never contain any changes, we always (re)use an empty
        // tree object. There's no file/directory information to include.
        let tree = Self::create_empty_tree(&repo)?;

        Ok(SyncRepo { repo, tree, author })
    }

    pub fn read_or_create_repo_from_path(path: &Path) -> Result<Repository, RepoError> {
        if let Ok(repo) = Repository::open(path) {
            return if repo.head().is_ok() {
                Err(RepoError::Validation(format!(
                    "Cannot handle existing {SYNC_REPO_NAME} repository history yet.",
                )))
            } else {
                Ok(repo)
            };
        };

        let mut options = RepositoryInitOptions::new();
        let options = options.initial_head("main");

        Ok(Repository::init_opts(path, &options)?)
    }

    pub fn copy_over_commits<'repo>(
        &'repo self,
        parents: &mut Vec<Commit<'repo>>,
        commits: &Vec<Commit>,
    ) -> Result<(), RepoError> {
        let SyncRepo { author, repo, tree } = self;

        for commit in commits {
            let parents_ref: Vec<&Commit> = parents.iter().collect();
            let signature = Signature::new(&author.name, &author.email, &commit.author().when())?;

            let commit_id = repo.commit(
                Some("HEAD"),
                &signature,
                &signature,
                commit.message().unwrap(),
                &tree,
                &parents_ref,
            )?;

            parents.clear();
            parents.push(repo.find_commit(commit_id)?);
        }

        Ok(())
    }

    fn create_empty_tree(repo: &Repository) -> Result<Tree, RepoError> {
        let tree = repo.treebuilder(None)?.write()?;
        let tree = repo.find_tree(tree)?;

        Ok(tree)
    }
}

pub struct CopyRepo<'repo> {
    repo: Repository,
    name: String,
    author: &'repo Author<'repo>,
}

impl CopyRepo<'_> {
    pub fn read_all_in_dir<'repo>(
        dir: &'repo Path,
        author: &'repo Author,
    ) -> Result<Option<Vec<CopyRepo<'repo>>>, RepoError> {
        let mut repositories = Vec::new();

        if Self::is_dir_git_repo(&dir) {
            let dir_absolute = dir.canonicalize()?;
            let dir_name = dir_absolute.file_name().unwrap();

            return if dir_name == SYNC_REPO_NAME {
                Err(RepoError::Validation(format!(
                    "Cannot read {SYNC_REPO_NAME} repository as input."
                )))
            } else {
                repositories.push(Self::from(
                    Repository::open(dir)?,
                    String::from(dir_name.to_str().unwrap()),
                    author,
                ));

                Ok(Some(repositories))
            };
        }

        for entry in dir.read_dir()? {
            let entry = entry?;
            let entry_path = entry.path();
            let entry_name = entry.file_name();

            if !Self::is_dir_git_repo(&entry_path) {
                continue;
            } else if entry_name == SYNC_REPO_NAME {
                continue;
            }

            repositories.push(Self::from(
                Repository::open(entry_path)?,
                entry_name.into_string().unwrap(),
                author,
            ));
        }

        if repositories.is_empty() {
            Ok(None)
        } else {
            Ok(Some(repositories))
        }
    }

    pub fn get_author_commits(&self) -> Result<(u32, Vec<Commit>), RepoError> {
        let mut revision_walker = self.repo.revwalk()?;

        revision_walker.set_sorting(Sort::TOPOLOGICAL | Sort::REVERSE)?;
        revision_walker.push_head()?;

        let mut total_count: u32 = 0;
        let mut found_commits = Vec::new();

        for revision in revision_walker {
            let oid = revision?;
            let commit = self.repo.find_commit(oid)?;

            total_count += 1;

            let does_email_match = {
                let commit_author = commit.author();
                let commit_author_email = commit_author.email().unwrap_or("unknown");

                self.author
                    .email_aliases
                    .iter()
                    .any(|EmailAddress(email)| email == commit_author_email)
            };

            if !does_email_match {
                continue;
            }

            found_commits.push(commit);
        }

        Ok((total_count, found_commits))
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    fn is_dir_git_repo(dir: &Path) -> bool {
        dir.join(".git").is_dir()
    }

    fn from<'repo>(repo: Repository, name: String, author: &'repo Author) -> CopyRepo<'repo> {
        CopyRepo { repo, name, author }
    }
}
