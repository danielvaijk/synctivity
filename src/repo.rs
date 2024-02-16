use crate::email::EmailAddress;
use crate::SYNC_REPO_NAME;
use git2::{Commit, Oid, Repository, RepositoryInitOptions, Signature, Sort, Tree};
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
    emails: &'repo Vec<EmailAddress>,
}

impl Author<'_> {
    pub fn new<'repo>(
        name: &'repo str,
        emails: &'repo Vec<EmailAddress>,
    ) -> Result<Author<'repo>, RepoError> {
        if emails.len() > 0 {
            Ok(Author { name, emails })
        } else {
            Err(RepoError::Validation(
                "An author requires at least one email address.".into(),
            ))
        }
    }

    pub fn signature_email(&self) -> &EmailAddress {
        &self.emails.get(0).unwrap()
    }
}

pub struct SyncRepo<'repo> {
    repo: Repository,
    author: &'repo Author<'repo>,
    parents: Vec<Oid>,
}

impl SyncRepo<'_> {
    pub fn read_or_create<'repo>(
        path: &'repo Path,
        author: &'repo Author,
    ) -> Result<SyncRepo<'repo>, RepoError> {
        let mut repo: Option<Repository> = None;
        let repo_path = path.join(SYNC_REPO_NAME);

        if let Ok(existing_repo) = Repository::open(&repo_path) {
            if existing_repo.head().is_ok() {
                return Err(RepoError::Validation(format!(
                    "Cannot handle existing {SYNC_REPO_NAME} repository history yet.",
                )));
            } else {
                repo = Some(existing_repo);
            };
        };

        if repo.is_none() {
            let mut options = RepositoryInitOptions::new();
            let options = options.initial_head("main");

            repo = Some(Repository::init_opts(&repo_path, &options)?);
        }

        let repo = repo.unwrap();

        // We always start from scratch since we don't handle history delta's yet,
        // so there isn't a parent commit ID to start from.
        let parents: Vec<Oid> = Vec::new();

        Ok(SyncRepo {
            repo,
            author,
            parents,
        })
    }

    pub fn copy_matching_commits(
        &mut self,
        repos_to_copy: &Vec<CopyRepo>,
    ) -> Result<(), RepoError> {
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

                if commit_iter.len() == 0 {
                    println!("Synced {} commit(s) from {}.", commit_count, repo_name);
                }
            }
        }

        Ok(())
    }

    fn copy_author_commit(&mut self, commit: &Commit) -> Result<(), RepoError> {
        let commit_id = {
            let tree = Self::create_empty_tree(&self.repo)?;

            let parents = self.get_parent_commits()?;
            let parents: Vec<&Commit> = parents.iter().collect();

            let signature = Signature::new(
                &self.author.name,
                self.author.signature_email().0.as_str(),
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

    fn get_parent_commits(&self) -> Result<Vec<Commit>, RepoError> {
        let mut commit_refs = Vec::with_capacity(self.parents.len());

        for commit_id in self.parents.iter() {
            commit_refs.push(self.repo.find_commit(*commit_id)?)
        }

        let commit_refs = commit_refs;

        Ok(commit_refs)
    }

    // Since the commits never contain any changes, we always (re)use an empty
    // tree object. There's no file/directory information to include.
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
    ) -> Result<Vec<CopyRepo<'repo>>, RepoError> {
        let mut repositories = Vec::new();

        if Self::is_dir_git_repo(&dir) {
            let dir_absolute = dir.canonicalize()?;
            let dir_name = dir_absolute.file_name().unwrap();

            return if dir_name == SYNC_REPO_NAME {
                Err(RepoError::Validation(format!(
                    "Cannot read {SYNC_REPO_NAME} repository as input."
                )))
            } else {
                repositories.push(Self::new(
                    Repository::open(dir)?,
                    String::from(dir_name.to_str().unwrap()),
                    author,
                ));

                Ok(repositories)
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

            repositories.push(Self::new(
                Repository::open(entry_path)?,
                entry_name.into_string().unwrap(),
                author,
            ));
        }

        if repositories.is_empty() {
            Err(RepoError::Validation(
                "No repositories found in the input directory".into(),
            ))
        } else {
            Ok(repositories)
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    fn get_author_commits(&self) -> Result<Vec<Commit>, RepoError> {
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

                self.author
                    .emails
                    .iter()
                    .any(|EmailAddress(email)| email == commit_author_email)
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

    fn new<'repo>(repo: Repository, name: String, author: &'repo Author) -> CopyRepo<'repo> {
        CopyRepo { repo, name, author }
    }
}
