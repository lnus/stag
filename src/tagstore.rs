use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use anyhow::Context;
use directories::ProjectDirs;
use rusqlite::{params_from_iter, Connection};

pub struct TagStore {
    conn: Connection,
}

impl TagStore {
    fn init_db(conn: &Connection) -> anyhow::Result<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS files (path TEXT PRIMARY KEY)",
            (),
        )
        .context("Failed creating table 'files'")?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS tags (
                file_path TEXT,
                tag TEXT,
                PRIMARY KEY (file_path, tag),
                FOREIGN KEY(file_path) REFERENCES files(path)
            )",
            (),
        )
        .context("Failed creating table 'tags'")?;

        Ok(())
    }

    pub fn new() -> anyhow::Result<Self> {
        // NOTE: This is primarily for integration testing.
        // But you can also use it outside of it.
        // Not fully supported yet but it should work fine.
        let conn = if let Ok(path) = std::env::var("STAG_DB_PATH") {
            Connection::open(path)?
        } else {
            let proj_dirs = ProjectDirs::from("com", "stag", "stag")
                .ok_or_else(|| anyhow::anyhow!("Could not determine project directories"))?;

            let data_dir = proj_dirs.data_dir();
            std::fs::create_dir_all(data_dir).context("Failed the create data directory")?;

            Connection::open(data_dir.join("tags.db"))?
        };

        Self::init_db(&conn)?;
        Ok(Self { conn })
    }

    // TODO: This is so unreadable and ugly
    // Maybe we should separate some queries out into SQL files...
    pub fn search_tags(
        &self,
        tags: &[&str],
        excluded: &[&str],
        any: bool,
    ) -> anyhow::Result<Vec<PathBuf>> {
        if tags.is_empty() {
            return Ok(Vec::new());
        }

        let query = if any {
            let placeholders = (1..=tags.len())
                .map(|i| format!("?{}", i))
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "SELECT DISTINCT files.path FROM files JOIN tags \
                    ON files.path = tags.file_path \
                    WHERE tags.tag IN ({})",
                placeholders
            )
        } else {
            let conditions = (0..tags.len())
                .map(|i| {
                    format!(
                        "EXISTS (SELECT 1 FROM tags t{} \
                            WHERE t{}.file_path = files.path \
                            AND t{}.tag = ?{})",
                        i,
                        i,
                        i,
                        i + 1
                    )
                })
                .collect::<Vec<_>>()
                .join(" AND ");
            format!("SELECT DISTINCT path FROM files WHERE {}", conditions)
        };

        let mut stmt = self.conn.prepare(&query)?;
        let paths = stmt
            .query_map(params_from_iter(tags), |row| {
                Ok(PathBuf::from(row.get::<_, String>(0)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        // FIXME: This is giga-fugly filtering
        // But as long as it works, I'm kind of happy.
        // This does n SQL queries for tags...
        // Using -exclude is a performance loss now
        let filtered_paths = paths
            .into_iter()
            .filter(|path| {
                let path_tags = self.get_tags(path).unwrap_or_default();
                !excluded
                    .iter()
                    .any(|&exclude_tag| path_tags.contains(exclude_tag))
            })
            .collect();

        Ok(filtered_paths)
    }

    fn get_tags(&self, path: &Path) -> anyhow::Result<HashSet<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT tag FROM tags WHERE file_path = ?")?;
        let tags = stmt
            .query_map([path.to_string_lossy().as_ref()], |row| {
                row.get::<_, String>(0)
            })?
            .filter_map(Result::ok)
            .collect();
        Ok(tags)
    }

    pub fn list_tagged(&self, tag: &str) -> anyhow::Result<Vec<PathBuf>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT files.path FROM files
                JOIN tags ON files.path = tags.file_path
                WHERE tags.tag = ?1",
            )
            .context("Failed to prepare list query")?;

        let paths = stmt
            .query_map([tag], |row| Ok(PathBuf::from(row.get::<_, String>(0)?)))?
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to collect paths from query")?;

        Ok(paths)
    }

    pub fn add_tags_batch(&mut self, paths: &[PathBuf], tag: &str) -> anyhow::Result<()> {
        let tx = self.conn.transaction()?;
        {
            let mut files_stmt = tx.prepare("INSERT OR IGNORE INTO files (path) VALUES (?1)")?;

            let mut tags_stmt =
                tx.prepare("INSERT OR IGNORE INTO tags (file_path, tag) VALUES (?1, ?2)")?;

            for path in paths {
                let path_str = path.canonicalize()?.to_string_lossy().to_string();

                files_stmt
                    .execute([&path_str])
                    .context("Failed inserting into table 'files'")?;

                tags_stmt
                    .execute([&path_str, tag])
                    .context("Failed inserting into table 'tags'")?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn remove_tags_batch(&mut self, paths: &[PathBuf], tag: &str) -> anyhow::Result<()> {
        let tx = self.conn.transaction()?;
        {
            // Prepare statements once for reuse
            let mut tags_stmt = tx.prepare("DELETE FROM tags WHERE file_path = ?1 AND tag = ?2")?;

            let mut cleanup_stmt = tx.prepare(
                "DELETE FROM files WHERE path = ?1 
         AND NOT EXISTS (SELECT 1 FROM tags WHERE file_path = ?1)",
            )?;

            for path in paths {
                let path_str = path.canonicalize()?.to_string_lossy().to_string();

                tags_stmt
                    .execute([&path_str, tag])
                    .context("Failed to remove tag")?;

                cleanup_stmt
                    .execute([&path_str])
                    .context("Failed to clean up files table")?;
            }
        }
        tx.commit()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_test_db() -> anyhow::Result<TagStore> {
        let conn = Connection::open_in_memory()?;
        TagStore::init_db(&conn)?;
        Ok(TagStore { conn })
    }

    #[test]
    fn test_add_and_list_tag() -> anyhow::Result<()> {
        let mut store = setup_test_db()?;
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("test_file");
        fs::write(&test_file, "test content")?;

        store.add_tags_batch(&[test_file.clone()], "test_tag")?;
        let paths = store.list_tagged("test_tag")?;

        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], test_file.canonicalize()?);
        Ok(())
    }

    #[test]
    fn test_multiple_tags_same_file() -> anyhow::Result<()> {
        let mut store = setup_test_db()?;
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("test_file");
        fs::write(&test_file, "test content")?;

        store.add_tags_batch(&[test_file.clone()], "tag1")?;
        store.add_tags_batch(&[test_file.clone()], "tag2")?;

        let paths1 = store.list_tagged("tag1")?;
        let paths2 = store.list_tagged("tag2")?;

        assert_eq!(paths1.len(), 1);
        assert_eq!(paths2.len(), 1);
        assert_eq!(paths1[0], paths2[0]);
        Ok(())
    }

    #[test]
    fn test_same_tag_multiple_files() -> anyhow::Result<()> {
        let mut store = setup_test_db()?;
        let temp_dir = TempDir::new()?;
        let file1 = temp_dir.path().join("file1");
        let file2 = temp_dir.path().join("file2");
        fs::write(&file1, "test content")?;
        fs::write(&file2, "test content")?;

        store.add_tags_batch(&[file1.clone()], "shared_tag")?;
        store.add_tags_batch(&[file2.clone()], "shared_tag")?;

        let paths = store.list_tagged("shared_tag")?;
        assert_eq!(paths.len(), 2);
        assert!(paths.contains(&file1.canonicalize()?));
        assert!(paths.contains(&file2.canonicalize()?));
        Ok(())
    }

    #[test]
    fn test_remove_tag() -> anyhow::Result<()> {
        let mut store = setup_test_db()?;
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("test_file");
        fs::write(&test_file, "test content")?;

        store.add_tags_batch(&[test_file.clone()], "test_tag")?;
        store.remove_tags_batch(&[test_file.clone()], "test_tag")?;

        let paths = store.list_tagged("test_tag")?;
        assert!(paths.is_empty());
        Ok(())
    }

    #[test]
    fn test_remove_nonexistent_tag() -> anyhow::Result<()> {
        let mut store = setup_test_db()?;
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("test_file");
        fs::write(&test_file, "test content")?;

        // Should not error when removing non-existent tag
        store.remove_tags_batch(&[test_file], "nonexistent_tag")?;
        Ok(())
    }

    #[test]
    fn test_invalid_path() {
        let mut store = setup_test_db().unwrap();
        let result = store.add_tags_batch(&[PathBuf::from("/definitely/not/a/real/path")], "tag");
        assert!(result.is_err());
    }

    #[test]
    fn test_cleanup_after_last_tag_removed() -> anyhow::Result<()> {
        let mut store = setup_test_db()?;
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("test_file");
        fs::write(&test_file, "test content")?;

        store.add_tags_batch(&[test_file.clone()], "tag1")?;
        store.add_tags_batch(&[test_file.clone()], "tag2")?;

        store.remove_tags_batch(&[test_file.clone()], "tag1")?;

        // File should still exist in files table
        let paths = store.list_tagged("tag2")?;
        assert_eq!(paths.len(), 1);

        store.remove_tags_batch(&[test_file.clone()], "tag2")?;

        // File should be cleaned up
        let paths = store.list_tagged("tag2")?;
        assert!(paths.is_empty());
        Ok(())
    }

    #[test]
    fn test_batch_operations() -> anyhow::Result<()> {
        let mut store = setup_test_db()?;
        let temp_dir = TempDir::new()?;
        let files: Vec<PathBuf> = (0..5)
            .map(|i| {
                let path = temp_dir.path().join(format!("file{}", i));
                fs::write(&path, "test").unwrap();
                path
            })
            .collect();

        // Test batch add
        store.add_tags_batch(&files, "batch_tag")?;
        let tagged = store.list_tagged("batch_tag")?;
        assert_eq!(tagged.len(), 5);

        // Test batch remove
        store.remove_tags_batch(&files[0..2], "batch_tag")?;
        let remaining = store.list_tagged("batch_tag")?;
        assert_eq!(remaining.len(), 3);

        Ok(())
    }

    #[test]
    fn test_batch_transaction_rollback() -> anyhow::Result<()> {
        let mut store = setup_test_db()?;
        let temp_dir = TempDir::new()?;
        let valid_file = temp_dir.path().join("valid");
        fs::write(&valid_file, "test")?;

        let paths = vec![valid_file, PathBuf::from("/nonexistent/path")];

        // Should fail and rollback due to invalid path
        assert!(store.add_tags_batch(&paths, "tag").is_err());

        // Verify nothing was committed
        let tagged = store.list_tagged("tag")?;
        assert!(tagged.is_empty());

        Ok(())
    }

    #[test]
    fn test_search_tags_any() -> anyhow::Result<()> {
        let mut store = setup_test_db()?;
        let temp_dir = TempDir::new()?;

        let files: Vec<PathBuf> = (0..3)
            .map(|i| {
                let path = temp_dir.path().join(format!("file{}", i));
                fs::write(&path, "test").unwrap();
                path
            })
            .collect();

        store.add_tags_batch(&[files[0].clone()], "tag1")?;
        store.add_tags_batch(&[files[1].clone()], "tag2")?;
        store.add_tags_batch(&[files[2].clone()], "tag3")?;
        store.add_tags_batch(&files[0..2], "common")?;

        // Test OR search
        let results = store.search_tags(&["tag1", "tag2"], &[], true)?;
        assert_eq!(results.len(), 2);
        assert!(results.contains(&files[0].canonicalize()?));
        assert!(results.contains(&files[1].canonicalize()?));

        Ok(())
    }

    #[test]
    fn test_search_tags_all() -> anyhow::Result<()> {
        let mut store = setup_test_db()?;
        let temp_dir = TempDir::new()?;

        let file1 = temp_dir.path().join("file1");
        fs::write(&file1, "test")?;

        // File with both tags
        store.add_tags_batch(&[file1.clone()], "tag1")?;
        store.add_tags_batch(&[file1.clone()], "tag2")?;

        // Test AND search
        let results = store.search_tags(&["tag1", "tag2"], &[], false)?;
        assert_eq!(results.len(), 1);
        assert!(results.contains(&file1.canonicalize()?));

        // Should return empty when searching for non-existent combination
        let results = store.search_tags(&["tag1", "nonexistent"], &[], false)?;
        assert!(results.is_empty());

        Ok(())
    }

    #[test]
    fn test_search_tags_with_exclusions() -> anyhow::Result<()> {
        let mut store = setup_test_db()?;
        let temp_dir = TempDir::new()?;

        let files: Vec<PathBuf> = (0..2)
            .map(|i| {
                let path = temp_dir.path().join(format!("file{}", i));
                fs::write(&path, "test").unwrap();
                path
            })
            .collect();

        store.add_tags_batch(&files, "include")?;
        store.add_tags_batch(&[files[1].clone()], "exclude")?;

        let results = store.search_tags(&["include"], &["exclude"], true)?;
        assert_eq!(results.len(), 1);
        assert!(results.contains(&files[0].canonicalize()?));

        Ok(())
    }

    #[test]
    fn test_search_tags_empty() -> anyhow::Result<()> {
        let store = setup_test_db()?;

        // Empty searches should return empty results
        let results = store.search_tags(&[], &[], true)?;
        assert!(results.is_empty());

        let results = store.search_tags(&[], &[], false)?;
        assert!(results.is_empty());

        Ok(())
    }
}
