mod setup;
mod sync;

use crate::cmd::sync::SyncCommandArgs;
use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum Command {
    #[command(name = "setup")]
    SetupCommand,
    #[command(name = "sync")]
    SyncCommand(SyncCommandArgs),
}

pub fn exec(cmd: Command) -> Result<()> {
    match cmd {
        Command::SetupCommand => setup::exec(),
        Command::SyncCommand(args) => sync::exec(args),
    }
}
