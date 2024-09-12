use crate::config::Settings;
use crate::core::TargetRepo;
use anyhow::Result;
use std::fs;

pub fn exec() -> Result<()> {
    let settings = Settings::new()?;

    if !settings.get_base_dir().exists() {
        fs::create_dir(&settings.get_base_dir())?;
        TargetRepo::create(&settings)?;
    }

    Ok(())
}
