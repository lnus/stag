use anyhow::Result;
use std::{collections::HashMap, fs, path::PathBuf};

use crate::{cmd::collect_paths, tagstore::TagStore};

pub fn autotag_paths(
    store: &mut TagStore,
    paths: Vec<PathBuf>,
    recursive: bool,
    hidden: bool,
) -> Result<()> {
    let paths = collect_paths(paths, recursive, hidden)?;

    let mut tag_map: HashMap<String, Vec<PathBuf>> = HashMap::new();

    for path in paths {
        if let Ok(metadata) = fs::metadata(&path) {
            let tags = generate_tags_from_metadata(&metadata, &path);
            for tag in tags {
                tag_map.entry(tag).or_default().push(path.clone())
            }
        }
    }

    for (tag, paths) in tag_map {
        store.add_tags_batch(&paths, &tag)?;
    }

    Ok(())
}

fn generate_tags_from_metadata(metadata: &fs::Metadata, _path: &PathBuf) -> Vec<String> {
    let mut tags = Vec::new();

    if metadata.is_dir() {
        tags.push("directory".to_string());
    } else if metadata.is_file() {
        tags.push("file".to_string());
    }

    tags
}
