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
    List {
        tag: String,
        #[clap(long)]
        dirs: bool,
        #[clap(long)]
        files: bool,
    },
    #[command(alias = "s")]
    Search {
        #[clap(required = true, num_args = 1..)]
        tags: Vec<String>,
        #[clap(long)]
        any: bool,
        #[clap(long)]
        dirs: bool,
        #[clap(long)]
        files: bool,
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

// NOTE: Maybe this should be moved into the SQL layer?
// However this function is pretty ripping fast O(n) so whatever?
fn filter_paths(paths: Vec<PathBuf>, dirs_only: bool, files_only: bool) -> Vec<PathBuf> {
    if !dirs_only && !files_only {
        return paths;
    }

    paths
        .into_iter()
        .filter(|p| {
            if dirs_only {
                p.is_dir()
            } else if files_only {
                p.is_file()
            } else {
                true
            }
        })
        .collect()
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
        Commands::List { tag, dirs, files } => {
            if dirs && files {
                return Err(anyhow::anyhow!("Cannot specify both --dirs and --files"));
            }

            if let Ok(paths) = store.list_tagged(&tag) {
                print_paths(&filter_paths(paths, dirs, files));
            }
        }
        Commands::Search {
            tags,
            any,
            dirs,
            files,
        } => {
            if dirs && files {
                return Err(anyhow::anyhow!("Cannot specify both --dirs and --files"));
            }

            let mode = if any {
                SearchMode::Any
            } else {
                SearchMode::All
            };
            let tag_refs: Vec<&str> = tags.iter().map(|s| s.as_str()).collect();
            if let Ok(paths) = store.search_tags(&tag_refs, mode) {
                print_paths(&filter_paths(paths, dirs, files));
            }
        }
    }

    Ok(())
}
