use anyhow::Result;
use git2::Config;

pub struct Author {
    name: String,
    email: String,
}

impl Author {
    pub fn new(git_config: Config) -> Result<Author> {
        let name = git_config.get_string("user.name")?;
        let email = git_config.get_string("user.email")?;

        Ok(Author { name, email })
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_email(&self) -> &str {
        &self.email
    }
}
