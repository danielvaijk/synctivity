use anyhow::Result;
use regex::Regex;
use std::str::FromStr;
use std::string::String;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EmailAddressError {
    #[error("'{0}' is not a valid email address")]
    Invalid(String),
}

#[derive(Clone)]
pub struct EmailAddress(pub String);

impl FromStr for EmailAddress {
    type Err = EmailAddressError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let regex = Regex::new(r"^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+$").unwrap();

        if regex.is_match(value) {
            Ok(EmailAddress(value.to_owned()))
        } else {
            Err(Self::Err::Invalid(value.into()))
        }
    }
}
