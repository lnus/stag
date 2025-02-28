use anyhow::{Context, Result};
use clap::Parser;
use cmd::{Cli, Run};

mod autotag;
mod cmd;
mod tagstore;

fn main() -> Result<()> {
    let cli = Cli::parse();
    cli.command.run().context("Failed to execute command")
}
