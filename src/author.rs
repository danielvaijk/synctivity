use crate::email::EmailAddress;
use crate::error::RepoError;

pub struct Author<'author> {
    pub name: &'author str,
    pub emails: &'author Vec<EmailAddress>,
}

impl Author<'_> {
    pub fn new<'author>(
        name: &'author str,
        emails: &'author Vec<EmailAddress>,
    ) -> Result<Author<'author>, RepoError> {
        if !emails.is_empty() {
            Ok(Author { name, emails })
        } else {
            Err(RepoError::Validation(
                "An author requires at least one email address.".into(),
            ))
        }
    }

    pub fn signature_email(&self) -> &EmailAddress {
        self.emails.first().unwrap()
    }
}
