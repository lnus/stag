use crate::{autotag::autotag_paths, tagstore::TagStore};
use anyhow::{anyhow, Result};

use super::{
    utils::{filter_paths, handle_paths, print_paths, PathAction},
    Add, Autotag, Inspect, List, Remove, Search,
};

impl Add {
    pub fn run(&self) -> Result<()> {
        let mut store = TagStore::new()?;
        handle_paths(
            &mut store,
            &self.tag,
            self.paths.clone(),
            PathAction::Add,
            self.recursive,
            self.hidden,
        )
    }
}

impl Remove {
    pub fn run(&self) -> Result<()> {
        let mut store = TagStore::new()?;
        handle_paths(
            &mut store,
            &self.tag,
            self.paths.clone(),
            PathAction::Remove,
            self.recursive,
            self.hidden,
        )
    }
}

impl List {
    pub fn run(&self) -> Result<()> {
        if self.dirs && self.files {
            return Err(anyhow!("Cannot specify both --dirs and --files"));
        };

        let store = TagStore::new()?;

        if let Ok(paths) = store.list_tagged(&self.tag) {
            print_paths(&filter_paths(paths, self.dirs, self.files));
        }

        Ok(())
    }
}

impl Search {
    pub fn run(&self) -> Result<()> {
        if self.dirs && self.files {
            return Err(anyhow!("Cannot specify both --dirs and --files"));
        };

        let store = TagStore::new()?;

        let include_tags: Vec<&str> = self.tags.iter().map(|s| s.as_str()).collect();
        let exclude_tags: Vec<&str> = self.exclude.iter().map(|s| s.as_str()).collect();

        if let Ok(paths) = store.search_tags(&include_tags, &exclude_tags, self.any) {
            print_paths(&filter_paths(paths, self.dirs, self.files));
        }

        Ok(())
    }
}

impl Autotag {
    pub fn run(&self) -> Result<()> {
        let mut store = TagStore::new()?;

        autotag_paths(
            &mut store,
            self.paths.clone(),
            self.preview, // TODO: Possibly remove? Just messing with something
            self.recursive,
            self.hidden,
        )?;

        Ok(())
    }
}

impl Inspect {
    pub fn run(&self) -> Result<()> {
        let store = TagStore::new()?;

        for path in &self.paths {
            if let Ok(tags) = store.get_file_tags(path) {
                if self.verbose {
                    print!("{}: ", path.display());
                }

                println!("{}", tags.join(", "));
            }
        }

        Ok(())
    }
}
