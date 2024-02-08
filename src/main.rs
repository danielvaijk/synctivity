use clap::{Parser, ValueHint};
use git2::Repository;
use regex::Regex;
use std::path::Path;

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
    emails: Vec<String>,
}

fn is_email_valid(email: &String) -> bool {
    match Regex::new(r"^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+$") {
        Ok(regex) => regex.is_match(email),
        Err(_) => panic!("Failed to create email validation regex."),
    }
}

fn get_repositories_in_dir(dir: &Path) -> Vec<Repository> {
    let mut repositories = Vec::new();

    // If we are inside a Git repository already, then return that.
    if dir.join(".git").is_dir() {
        match Repository::open(dir) {
            Ok(repository) => repositories.push(repository),
            Err(error) => println!("failed to open repository: {}", error),
        }

        return repositories;
    }

    for entry in dir.read_dir().expect("Failed to read input directory.") {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => panic!("Failed to process directory entry: {}", error),
        };

        let git_path = Path::join(&entry.path(), ".git");

        // Ignore entries that do not contain a .git directory.
        if !Path::new(&git_path).is_dir() {
            continue;
        }

        match Repository::open(entry.path()) {
            Ok(repository) => repositories.push(repository),
            Err(error) => println!("failed to open repository: {}", error),
        }
    }

    repositories
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

    for email in emails.iter() {
        if !is_email_valid(&email) {
            panic!("Email address '{email}' is invalid.")
        }
    }

    for repo in get_repositories_in_dir(&input_dir) {
        println!("{}", repo.head().unwrap().shorthand().unwrap())
    }
}
