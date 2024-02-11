use clap::{Parser, ValueHint};
use email::EmailAddress;
use git2::{Commit, Signature};
use std::path::PathBuf;

mod email;
mod repo;

#[derive(Parser)]
struct Arguments {
    /// A path containing the source repositories.
    #[arg(short, long, default_value = ".", value_hint = ValueHint::DirPath)]
    input_dir: PathBuf,

    /// A path to the output repository.
    #[arg(short, long, default_value = ".", value_hint = ValueHint::DirPath)]
    output_dir: PathBuf,

    /// Your commit signature email address(es).
    #[arg(short, long, required = true, value_delimiter = ',', value_hint = ValueHint::EmailAddress)]
    emails: Vec<EmailAddress>,
}

const SYNC_REPO_NAME: &str = "synctivity";

fn main() {
    let Arguments {
        emails,
        input_dir,
        output_dir,
    } = Arguments::parse();

    if !input_dir.is_dir() {
        panic!("Input directory is invalid.");
    }

    if !output_dir.is_dir() {
        panic!("Output directory is invalid.");
    }

    let sync_repo_path = output_dir.join(SYNC_REPO_NAME);
    let sync_repo = match repo::read_or_create(&sync_repo_path) {
        Ok(sync_repo) => sync_repo,
        Err(error) => panic!("Failed to create {} repository: {}", SYNC_REPO_NAME, error),
    };

    if sync_repo.head().is_ok() {
        panic!(
            "Cannot handle existing {} repository history yet.",
            SYNC_REPO_NAME
        );
    }

    let repos = match repo::read_all_in_dir(&input_dir) {
        Ok(repos) => repos,
        Err(error) => panic!("Failed to read input repositories: {}", error),
    };

    if repos.is_empty() {
        panic!("Could not find any repositories in the input directory.");
    }

    for repo in repos {
        let commits = repo::get_commits_by_email(&repo, &emails);
        let (revision_count, found_commits) = match commits {
            Ok(result) => result,
            Err(error) => panic!("Failed to read commits from repository: {}", error),
        };

        println!("revision count total: {}", revision_count);
        println!("revision count matched: {}", found_commits.len());
    }
}
