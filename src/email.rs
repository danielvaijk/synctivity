use regex::Regex;
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::string::String;

#[derive(Clone)]
pub struct EmailAddress(pub String);

#[derive(Debug)]
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

impl Display for InvalidEmailAddressError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "'{}' is not a valid email address.", self.0)
    }
}

impl Error for InvalidEmailAddressError {}
