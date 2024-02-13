use crate::repo::{Author, CopyRepo, SyncRepo};
use clap::{Parser, ValueHint};
use email::EmailAddress;
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
    let author = Author::new(&author_name, &author_email, &match_emails);

    let sync_repo = output_dir.join(SYNC_REPO_NAME);
    let sync_repo = SyncRepo::read_or_create_repo_from_path(&sync_repo)?;

    let repos_to_copy = match CopyRepo::read_all_in_dir(&input_dir, &author)? {
        Some(repos_to_sync) => repos_to_sync,
        None => return Err("No repositories found in the input directory".into()),
    };

    let mut sync_repo = SyncRepo::from(&sync_repo, &author)?;
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

            sync_repo.copy_commit(&commit)?;

            if commit_iter.len() == 0 {
                println!("Synced {} commit(s) from {}.", commit_count, repo_name);
            }
        }
    }

    Ok(())
}
