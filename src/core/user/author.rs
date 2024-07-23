use super::email::EmailAddress;
use anyhow::Result;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthorError {
    #[error("an author requires at least one email address")]
    EmailRequired,
}

pub struct Author<'author> {
    pub name: &'author str,
    pub emails: &'author Vec<EmailAddress>,
}

impl Author<'_> {
    pub fn new<'author>(
        name: &'author str,
        emails: &'author Vec<EmailAddress>,
    ) -> Result<Author<'author>, AuthorError> {
        if !emails.is_empty() {
            Ok(Author { name, emails })
        } else {
            Err(AuthorError::EmailRequired)
        }
    }

    pub fn signature_email(&self) -> &EmailAddress {
        self.emails.first().unwrap()
    }
}
