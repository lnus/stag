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
        #[clap(short, long)]
        recursive: bool,
    },
    #[command(alias = "rm")]
    Remove {
        tag: String,
        #[clap(required = true, num_args = 1..)]
        paths: Vec<PathBuf>,
        #[clap(short, long)]
        recursive: bool,
    },
    #[command(alias = "ls")]
    List { tag: String },
    #[command(alias = "s")]
    Search {
        #[clap(required = true, num_args = 1..)]
        tags: Vec<String>,
        #[clap(long)]
        any: bool,
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
    recursive: bool,
) -> anyhow::Result<()> {
    // TODO: Consider hidden(false)
    // I think it's good to respect gitignore?
    // Could have this as a config flag
    let paths: Vec<_> = if recursive {
        paths
            .iter()
            .flat_map(|path_pattern| {
                WalkBuilder::new(path_pattern)
                    .build()
                    .filter_map(Result::ok)
                    .map(|entry| entry.path().to_path_buf())
                    .collect::<Vec<_>>()
            })
            .collect()
    } else {
        paths.into_iter().collect()
    };

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
        Commands::Add {
            tag,
            paths,
            recursive,
        } => handle_paths(&mut store, &tag, paths.clone(), PathAction::Add, recursive)?,
        Commands::Remove {
            tag,
            paths,
            recursive,
        } => handle_paths(
            &mut store,
            &tag,
            paths.clone(),
            PathAction::Remove,
            recursive,
        )?,
        Commands::List { tag } => {
            if let Ok(paths) = store.list_tagged(&tag) {
                print_paths(&paths);
            }
        }
        Commands::Search { tags, any } => {
            let mode = if any {
                SearchMode::Any
            } else {
                SearchMode::All
            };
            let tag_refs: Vec<&str> = tags.iter().map(|s| s.as_str()).collect();
            if let Ok(paths) = store.search_tags(&tag_refs, mode) {
                print_paths(&paths);
            }
        }
    }

    Ok(())
}
