use clap::{Parser, ValueHint};
use email::EmailAddress;
use std::path::Path;

mod email;
mod repo;

#[derive(Parser)]
struct Arguments {
    /// A path containing the source repositories.
    #[arg(short, long = "input-dir", default_value = ".", value_hint = ValueHint::DirPath)]
    in_dir: String,

    /// A path to the output repository.
    #[arg(short, long = "output-dir", default_value = ".", value_hint = ValueHint::DirPath)]
    out_dir: String,

    /// Your commit signature email addresses.
    #[arg(short, long, required = true, value_delimiter = ',', value_hint = ValueHint::EmailAddress)]
    emails: Vec<EmailAddress>,
}

fn main() {
    let arguments = Arguments::parse();

    let emails = arguments.emails;
    let input_dir = Path::new(&arguments.in_dir);
    let output_dir = Path::new(&arguments.out_dir);

    if !input_dir.is_dir() {
        panic!("Input directory is invalid.");
    }

    if !output_dir.is_dir() {
        panic!("Output directory is invalid.");
    }

    let repos = match repo::get_repositories_in_dir(&input_dir) {
        Ok(repos) => repos,
        Err(error) => panic!("{}", error),
    };

    for repo in repos {
        let commits = repo::get_commits_by_email(&repo, &emails);
        let (revision_count, found_commits) = match commits {
            Ok(result) => result,
            Err(error) => panic!("{}", error),
        };

        println!("revision count total: {}", revision_count);
        println!("revision count matched: {}", found_commits.len());
    }
}
