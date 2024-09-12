use anyhow::Result;
use std::env;
use std::path::PathBuf;

pub struct Settings {
    base_dir: PathBuf,
}

impl Settings {
    pub fn new() -> Result<Self> {
        let home_dir = {
            if cfg!(windows) {
                PathBuf::from(env::var("USERPROFILE")?)
            } else {
                PathBuf::from(env::var("HOME")?)
            }
        };

        Ok(Self {
            base_dir: home_dir.join(".synctivity"),
        })
    }

    pub fn get_base_dir(&self) -> &PathBuf {
        &self.base_dir
    }
}
