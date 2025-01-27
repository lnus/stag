mod tagstore;

use std::path::PathBuf;

use anyhow::Context;
use clap::{Parser, Subcommand};
use ignore::WalkBuilder;
use tagstore::TagStore;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(alias = "a")]
    Add {
        tag: String,
        #[clap(required = true, num_args = 1..)]
        paths: Vec<PathBuf>,
    },
    #[command(alias = "rm")]
    Remove {
        tag: String,
        #[clap(required = true, num_args = 1..)]
        paths: Vec<PathBuf>,
    },
    #[command(alias = "ls")]
    List { tag: String },
}

fn print_paths(paths: &[PathBuf]) {
    for path in paths {
        println!("{}", path.display());
    }
}

enum PathAction {
    Add,
    Remove,
}

fn handle_paths(
    store: &mut TagStore,
    tag: &str,
    paths: Vec<PathBuf>,
    action: PathAction,
) -> anyhow::Result<()> {
    let paths: Vec<_> = paths
        .iter()
        .flat_map(|path_pattern| {
            WalkBuilder::new(path_pattern)
                .hidden(false)
                .build()
                .filter_map(Result::ok)
                .map(|entry| entry.path().to_path_buf())
                .collect::<Vec<_>>()
        })
        .collect();

    match action {
        PathAction::Add => store.add_tags_batch(&paths, &tag)?,
        PathAction::Remove => store.remove_tags_batch(&paths, &tag)?,
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let mut store = TagStore::new().context("Failed creating tagstore")?;

    match cli.command {
        Commands::Add { tag, paths } => {
            handle_paths(&mut store, &tag, paths.clone(), PathAction::Add)?
        }
        Commands::Remove { tag, paths } => {
            handle_paths(&mut store, &tag, paths.clone(), PathAction::Remove)?
        }
        Commands::List { tag } => {
            if let Ok(paths) = store.list_tagged(&tag) {
                print_paths(&paths);
            }
        }
    }

    Ok(())
}
