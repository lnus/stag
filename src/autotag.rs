use anyhow::Result;
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
};

use crate::{cmd::collect_paths, tagstore::TagStore};
use mime_guess::MimeGuess;

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
            let tags = generate_tags_from_metadata(&metadata, &path)?;
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

fn generate_tags_from_metadata(metadata: &fs::Metadata, path: &PathBuf) -> Result<Vec<String>> {
    let mut tags: HashSet<String> = HashSet::new();

    if metadata.is_dir() {
        tags.insert("directory".to_string());

        if path.join(".git").is_dir() {
            tags.insert("git".to_string());
        }
    } else if metadata.is_file() {
        tags.insert("file".to_string());

        let size = metadata.len();
        if size < 100 * 1024 {
            tags.insert("small".to_string());
        } else if size < 1024 * 1024 {
            tags.insert("medium".to_string()); // Ugly tag name lol
        } else {
            tags.insert("large".to_string());
        }

        let guess: MimeGuess = mime_guess::from_path(&path);
        for mime in guess {
            // Tag with primary type (for example, "image", "text", "application")
            tags.insert(mime.type_().as_str().to_string());

            // Tag with subtype (for example, "png", "html", "json",)
            tags.insert(mime.subtype().as_str().to_string());

            // Tag with the full MIME string (e.g., "mime:text/html").
            tags.insert(format!("mime:{}", mime));
        }
    }

    Ok(tags.into_iter().collect())
}
