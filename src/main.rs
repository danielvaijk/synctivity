#![deny(clippy::all)]

use crate::cli::Cli;
use anyhow::Result;
use clap::Parser;

mod cli;
mod cmd;
mod config;
mod core;

fn main() -> Result<()> {
    cmd::exec(Cli::parse().command)
}
