use std::path::PathBuf;

use anyhow::Context;
use directories::ProjectDirs;
use rusqlite::Connection;

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
        let proj_dirs = ProjectDirs::from("com", "retag", "retag")
            .ok_or_else(|| anyhow::anyhow!("Could not determine project directories"))?;

        let data_dir = proj_dirs.data_dir();
        std::fs::create_dir_all(data_dir).context("Failed the create data directory")?;

        let db_path = data_dir.join("tags.db");
        let conn = Connection::open(db_path)?;
        Self::init_db(&conn)?;
        Ok(Self { conn })
    }

    #[allow(dead_code)]
    pub fn add_tag(&self, path: PathBuf, tag: &str) -> anyhow::Result<()> {
        let path_str = path.canonicalize()?.to_string_lossy().to_string();

        self.conn
            .execute(
                "INSERT OR IGNORE INTO files (path) VALUES (?1)",
                [&path_str],
            )
            .context("Failed inserting into table 'files'")?;

        self.conn
            .execute(
                "INSERT OR IGNORE INTO tags (file_path, tag) VALUES (?1, ?2)",
                [&path_str, tag],
            )
            .context("Failed inserting into table 'tags'")?;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn remove_tag(&self, path: PathBuf, tag: &str) -> anyhow::Result<()> {
        let path_str = path.canonicalize()?.to_string_lossy().to_string();

        self.conn
            .execute(
                "DELETE FROM tags WHERE file_path = ?1 AND tag = ?2",
                [&path_str, tag],
            )
            .context("Failed to remove tag")?;

        // Clean up from files table if no more tag remains for the file
        self.conn
            .execute(
                "DELETE FROM files WHERE path = ?1 
                 AND NOT EXISTS (SELECT 1 FROM tags WHERE file_path = ?1)",
                [&path_str],
            )
            .context("Failed to clean up files table")?;

        Ok(())
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
        let tx = self.conn.unchecked_transaction()?;
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
        let store = setup_test_db()?;
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("test_file");
        fs::write(&test_file, "test content")?;

        store.add_tag(test_file.clone(), "test_tag")?;
        let paths = store.list_tagged("test_tag")?;

        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], test_file.canonicalize()?);
        Ok(())
    }

    #[test]
    fn test_multiple_tags_same_file() -> anyhow::Result<()> {
        let store = setup_test_db()?;
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("test_file");
        fs::write(&test_file, "test content")?;

        store.add_tag(test_file.clone(), "tag1")?;
        store.add_tag(test_file.clone(), "tag2")?;

        let paths1 = store.list_tagged("tag1")?;
        let paths2 = store.list_tagged("tag2")?;

        assert_eq!(paths1.len(), 1);
        assert_eq!(paths2.len(), 1);
        assert_eq!(paths1[0], paths2[0]);
        Ok(())
    }

    #[test]
    fn test_same_tag_multiple_files() -> anyhow::Result<()> {
        let store = setup_test_db()?;
        let temp_dir = TempDir::new()?;
        let file1 = temp_dir.path().join("file1");
        let file2 = temp_dir.path().join("file2");
        fs::write(&file1, "test content")?;
        fs::write(&file2, "test content")?;

        store.add_tag(file1.clone(), "shared_tag")?;
        store.add_tag(file2.clone(), "shared_tag")?;

        let paths = store.list_tagged("shared_tag")?;
        assert_eq!(paths.len(), 2);
        assert!(paths.contains(&file1.canonicalize()?));
        assert!(paths.contains(&file2.canonicalize()?));
        Ok(())
    }

    #[test]
    fn test_remove_tag() -> anyhow::Result<()> {
        let store = setup_test_db()?;
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("test_file");
        fs::write(&test_file, "test content")?;

        store.add_tag(test_file.clone(), "test_tag")?;
        store.remove_tag(test_file, "test_tag")?;

        let paths = store.list_tagged("test_tag")?;
        assert!(paths.is_empty());
        Ok(())
    }

    #[test]
    fn test_remove_nonexistent_tag() -> anyhow::Result<()> {
        let store = setup_test_db()?;
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("test_file");
        fs::write(&test_file, "test content")?;

        // Should not error when removing non-existent tag
        store.remove_tag(test_file, "nonexistent_tag")?;
        Ok(())
    }

    #[test]
    fn test_invalid_path() {
        let store = setup_test_db().unwrap();
        let result = store.add_tag(PathBuf::from("/definitely/not/a/real/path"), "tag");
        assert!(result.is_err());
    }

    #[test]
    fn test_cleanup_after_last_tag_removed() -> anyhow::Result<()> {
        let store = setup_test_db()?;
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("test_file");
        fs::write(&test_file, "test content")?;

        store.add_tag(test_file.clone(), "tag1")?;
        store.add_tag(test_file.clone(), "tag2")?;

        store.remove_tag(test_file.clone(), "tag1")?;

        // File should still exist in files table
        let paths = store.list_tagged("tag2")?;
        assert_eq!(paths.len(), 1);

        store.remove_tag(test_file.clone(), "tag2")?;

        // File should be cleaned up
        let paths = store.list_tagged("tag2")?;
        assert!(paths.is_empty());
        Ok(())
    }
}
