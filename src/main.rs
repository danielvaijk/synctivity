use clap::{Parser, ValueHint};
use git2::{Commit, Repository};
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

fn get_commits_by_email<'repo>(
    repository: &'repo Repository,
    emails: &[String],
) -> (u32, Vec<Commit<'repo>>) {
    let mut walker = match repository.revwalk() {
        Ok(value) => value,
        Err(error) => panic!("failed to create revision walker: {}", error),
    };

    if let Err(error) = walker.push_head() {
        panic!(
            "failed to add HEAD commit to revision walker for traversal: {}",
            error
        );
    }

    let mut revision_count: u32 = 0;
    let mut found_commits = Vec::new();

    for revision in walker {
        let oid = match revision {
            Ok(value) => value,
            Err(error) => panic!("failed to get OID from revision: {}", error),
        };

        let commit = match repository.find_commit(oid) {
            Ok(value) => value,
            Err(error) => panic!("failed to find commit with OID {}: {}", oid, error),
        };

        revision_count += 1;

        let does_email_match = {
            let commit_author = commit.author();
            let commit_author_email = commit_author.email().unwrap_or("unknown");

            emails.iter().any(|email| email == commit_author_email)
        };

        if !does_email_match {
            continue;
        }

        found_commits.push(commit);
    }

    (revision_count, found_commits)
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
        let (revision_count, found_commits) = get_commits_by_email(&repo, &emails);

        println!("revision count total: {revision_count}");
        println!("revision count matched: {}", found_commits.len());
    }
}
