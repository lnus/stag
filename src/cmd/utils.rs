use std::path::PathBuf;

use anyhow::Result;
use ignore::WalkBuilder;

use crate::tagstore::TagStore;

// FIX: This entire file could use some love <3

pub(crate) fn print_paths(paths: &[PathBuf]) {
    for path in paths {
        println!("{}", path.display());
    }
}

pub(crate) enum PathAction {
    Add,
    Remove,
}

/// Collects paths based on the given options (recursive, hidden).
pub fn collect_paths(paths: Vec<PathBuf>, recursive: bool, hidden: bool) -> Result<Vec<PathBuf>> {
    // FIX: Document this better
    // NOTE: Hidden flag only applies for recursive indexing
    // It doesn't really make sense if someone does ie.
    // `stag a tag .hidden` and it doesn't index.
    // Hidden is more for:
    // `stag a config .config -r --hidden`, which will now recurse
    // .config and add ALL files no matter ignore-rules
    let collected_paths: Vec<PathBuf> = if recursive {
        paths
            .iter()
            .flat_map(|path_pattern| {
                WalkBuilder::new(path_pattern)
                    .hidden(!hidden)
                    .build()
                    .filter_map(Result::ok)
                    .map(|entry| entry.path().to_path_buf())
                    .collect::<Vec<_>>()
            })
            .collect()
    } else {
        paths.into_iter().collect()
    };

    Ok(collected_paths)
}

pub(crate) fn handle_paths(
    store: &mut TagStore,
    tag: &str,
    paths: Vec<PathBuf>,
    action: PathAction,
    recursive: bool,
    hidden: bool,
) -> Result<()> {
    let paths = collect_paths(paths, recursive, hidden)?;

    match action {
        PathAction::Add => store.add_tags_batch(&paths, &tag)?,
        PathAction::Remove => store.remove_tags_batch(&paths, &tag)?,
    }

    Ok(())
}

pub(crate) fn filter_paths(paths: Vec<PathBuf>, dirs_only: bool, files_only: bool) -> Vec<PathBuf> {
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
