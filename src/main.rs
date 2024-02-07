use clap::{Parser,ValueHint};
use std::path::Path;
use regex::Regex;

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

fn main() {
    let Arguments {in_dir, out_dir, emails} = Arguments::parse();

    if !Path::new(&in_dir).is_dir() {
        panic!("Input directory is invalid.");
    }

    if !Path::new(&out_dir).is_dir() {
        panic!("Output directory is invalid.");
    }

    for email in emails.iter() {
        if !is_email_valid(&email) {
            panic!("Email address '{email}' is invalid.")
        }
    }

    println!("Input directory: {}", in_dir);
    println!("Output directory: {}", out_dir);
    println!("Emails: {:?}", emails);
}
