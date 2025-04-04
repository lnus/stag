mod cmd;
mod handlers;
mod utils;

use anyhow::Result;

pub use cmd::*;
pub use utils::collect_paths;

pub trait Run {
    fn run(&self) -> Result<()>;
}

impl Run for Commands {
    fn run(&self) -> Result<()> {
        match self {
            Commands::Add(cmd) => cmd.run(),
            Commands::Remove(cmd) => cmd.run(),
            Commands::List(cmd) => cmd.run(),
            Commands::Search(cmd) => cmd.run(),
            Commands::Autotag(cmd) => cmd.run(),
            Commands::Inspect(cmd) => cmd.run(),
        }
    }
}
