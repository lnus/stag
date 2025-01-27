mod tagstore;

use std::path::PathBuf;

use anyhow::Context;
use clap::{Parser, Subcommand};
use ignore::WalkBuilder;
use tagstore::{SearchMode, TagStore};

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
    #[command(alias = "s")]
    Search {
        #[clap(required = true, num_args = 1..)]
        tags: Vec<String>,
        #[clap(long, default_value = "any")]
        mode: String,
    },
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
            // TODO: .hidden(false), I think it's good to respect gitignore? Could have this
            // configable behaviour
            WalkBuilder::new(path_pattern)
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
        Commands::Search { tags, mode } => {
            let mode = match mode.as_str() {
                "all" => SearchMode::All,
                "any" => SearchMode::Any,
                _ => return Err(anyhow::anyhow!("Invalid search mode. Use 'all' or 'any'")),
            };

            let tag_refs: Vec<&str> = tags.iter().map(|s| s.as_str()).collect();
            if let Ok(paths) = store.search_tags(&tag_refs, mode) {
                print_paths(&paths);
            }
        }
    }

    Ok(())
}
