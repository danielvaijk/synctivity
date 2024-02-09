use regex::Regex;
use std::str::FromStr;
use std::string::String;
use thiserror::Error;

#[derive(Clone)]
pub struct EmailAddress(pub String);

#[derive(Error, Debug)]
#[error("'{0}' is not a valid email address.")]
pub struct InvalidEmailAddressError(String);

impl FromStr for EmailAddress {
    type Err = InvalidEmailAddressError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let regex = Regex::new(r"^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+$").unwrap();

        if regex.is_match(value) {
            Ok(EmailAddress(value.to_owned()))
        } else {
            Err(InvalidEmailAddressError(value.to_string()))
        }
    }
}
