use crate::repo::{Author, SyncRepo};
use clap::{Parser, ValueHint};
use email::EmailAddress;
use git2::Commit;
use std::error::Error;
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

    /// The name to sign the sync commits with.
    #[arg(short = 'n', long, required = true, value_delimiter = ',', value_hint = ValueHint::Other)]
    author_name: String,

    /// The email address to sign the sync commits with.
    #[arg(short = 'e', long, required = true, value_delimiter = ',', value_hint = ValueHint::EmailAddress)]
    author_email: EmailAddress,

    /// The commit signature email address(es) to match commits for.
    #[arg(short, long, required = true, value_delimiter = ',', value_hint = ValueHint::EmailAddress)]
    match_emails: Vec<EmailAddress>,
}

const SYNC_REPO_NAME: &str = "synctivity";

fn main() -> Result<(), Box<dyn Error>> {
    let Arguments {
        input_dir,
        output_dir,
        author_name,
        author_email,
        match_emails,
    } = Arguments::parse();

    if !input_dir.is_dir() {
        return Err("Input directory is invalid".into());
    }

    if !output_dir.is_dir() {
        return Err("Output directory is invalid".into());
    }

    let EmailAddress(author_email) = author_email;
    let author = Author::new(&author_name, &author_email);

    let sync_repo = output_dir.join(SYNC_REPO_NAME);
    let sync_repo = repo::read_or_create_repo(&sync_repo)?;

    if sync_repo.head().is_ok() {
        return Err(
            format!("Cannot handle existing {SYNC_REPO_NAME} repository history yet.",).into(),
        );
    }

    let sync_repo = SyncRepo::new(&author, &sync_repo)?;
    let repos_to_sync = match repo::read_all_in_dir(&input_dir)? {
        Some(repos_to_sync) => repos_to_sync,
        None => return Err("No repositories found in the input directory".into()),
    };

    // We always start from scratch since we don't handle history delta's yet,
    // so there isn't a parent commit ID to start from.
    let mut parents: Vec<Commit> = Vec::new();

    for repo in repos_to_sync {
        let author_commits = repo::get_author_commits(&repo, &match_emails);
        let (revision_count, found_commits) = author_commits?;

        if found_commits.is_empty() {
            println!("Found 0 commits by author.");
            continue;
        }

        match sync_repo.copy_over_commits(&mut parents, &found_commits) {
            Ok(_) => println!(
                "Copied {} commit(s) out of {}.",
                revision_count,
                found_commits.len()
            ),
            Err(error) => return Err(Box::new(error)),
        }
    }

    Ok(())
}
