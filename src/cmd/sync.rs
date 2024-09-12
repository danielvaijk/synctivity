use crate::config::Settings;
use crate::core::{SourceRepo, TargetRepo};
use anyhow::{bail, Result};
use clap::Parser;
use clap::ValueHint;
use std::path::PathBuf;

#[derive(Parser)]
pub struct SyncCommandArgs {
    /// A path containing the source repositories.
    #[arg(short, long, default_value = ".", value_hint = ValueHint::DirPath)]
    input_dir: PathBuf,
}

pub fn exec(args: SyncCommandArgs) -> Result<()> {
    let SyncCommandArgs { input_dir } = args;

    if !input_dir.is_dir() {
        bail!("input directory is invalid");
    }

    let settings = Settings::new()?;
    let mut target_repo = TargetRepo::load(&settings)?;
    let source_repos = SourceRepo::read_all_in_dir(&settings, &input_dir)?;

    target_repo.copy_matching_commits(&source_repos)
}
