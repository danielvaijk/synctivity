#![deny(clippy::all)]

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
    #[arg(short = 'n', long, required = true, value_hint = ValueHint::Other)]
    author_name: String,

    /// The commit signature email address(es) to match commits for.
    /// The first email address will also be used to sign synced commits.
    #[arg(short = 'e', long, required = true, value_delimiter = ',', value_hint = ValueHint::EmailAddress)]
    author_emails: Vec<EmailAddress>,
}

const SYNC_REPO_NAME: &str = "synctivity";

fn main() -> Result<(), Box<dyn Error>> {
    let Arguments {
        input_dir,
        output_dir,
        author_name,
        author_emails,
    } = Arguments::parse();

    if !input_dir.is_dir() {
        return Err("Input directory is invalid".into());
    }

    if !output_dir.is_dir() {
        return Err("Output directory is invalid".into());
    }

    let author = Author::new(&author_name, &author_emails)?;
    let mut sync_repo = SyncRepo::read_or_create(&output_dir, &author)?;
    let repos_to_copy = CopyRepo::read_all_in_dir(&input_dir, &author)?;

    sync_repo.copy_matching_commits(&repos_to_copy)?;

    Ok(())
}
