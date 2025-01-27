mod tagstore;

use std::path::PathBuf;

use anyhow::Context;
use clap::{Parser, Subcommand};
use tagstore::TagStore;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(alias = "a")]
    Add { tag: String, path: PathBuf },
    #[command(alias = "rm")]
    Remove { tag: String, path: PathBuf },
    #[command(alias = "ls")]
    List { tag: String },
}

fn print_paths(paths: &Vec<PathBuf>) {
    for path in paths {
        println!("{}", path.display());
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let store = TagStore::new().context("Failed creating tagstore")?;

    match cli.command {
        Commands::Add { tag, path } => match store.add_tag(path, &tag) {
            Ok(()) => println!("Added tag '{}' successfully", tag),
            Err(e) => eprintln!("Failed to add tag: {:#}", e),
        },
        Commands::Remove { tag, path } => match store.remove_tag(path, &tag) {
            Ok(()) => println!("Removed tag '{}' successfully", tag),
            Err(e) => eprintln!("Failed to remove tag: {:#}", e),
        },
        Commands::List { tag } => match store.list_tagged(&tag) {
            Ok(paths) => print_paths(&paths),
            Err(e) => eprintln!("Failed to list tags: {:#}", e),
        },
    }

    Ok(())
}
