use std::path::PathBuf;

use anyhow::{Context, Result};
use directories::ProjectDirs;
use rusqlite::{params, params_from_iter, Connection, ToSql, Transaction};

pub struct TagStore {
    conn: Connection,
}

// SQL inject me, whatever
// It's a local bundled database, why validate 5Head
mod schemas {
    pub const INIT_SQL: &str = include_str!("./sql/schema/init.sql");
}

mod queries {
    pub const REMOVE_TAGS: &str = include_str!("./sql/queries/remove_tags.sql");
    pub const LIST_TAGS: &str = include_str!("./sql/queries/list_tags.sql");
}

mod templates {
    pub const EXCLUDE_CLAUSE: &str = include_str!("./sql/templates/exclude_clause.sql");
    pub const SEARCH_QUERY: &str = include_str!("./sql/templates/search_query.sql");
}

impl TagStore {
    fn init_db(conn: &Connection) -> Result<()> {
        conn.execute_batch(schemas::INIT_SQL)
            .context("Failed to initialize the database schema")?;

        Ok(())
    }

    pub fn new() -> anyhow::Result<Self> {
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

    // NOTE: Helper / Internal functions
    fn get_or_create_tag(tx: &Transaction, tag: &str) -> Result<i64> {
        let mut stmt = tx.prepare("INSERT OR IGNORE INTO tags (name) VALUES (?1)")?;
        stmt.execute([tag])?;

        let mut stmt = tx.prepare("SELECT id FROM tags WHERE name = ?1")?;
        Ok(stmt.query_row([tag], |row| row.get(0))?)
    }

    fn get_or_create_file(tx: &Transaction, path: &PathBuf) -> Result<i64> {
        if !path.exists() {
            return Err(anyhow::anyhow!(
                "Path does not exist: {}",
                path.to_string_lossy()
            ));
        }

        let canonical_path = path.canonicalize().context("Failed to canonicalize path")?;
        let path_str = canonical_path.to_string_lossy();

        let mut stmt = tx.prepare("INSERT OR IGNORE INTO files (path) VALUES (?1)")?;
        stmt.execute([&path_str])?;

        let mut stmt = tx.prepare("SELECT id FROM files WHERE path = ?1")?;
        Ok(stmt.query_row([&path_str], |row| row.get(0))?)
    }

    // NOTE: Public API functions
    pub fn add_tags_batch(&mut self, paths: &[PathBuf], tag: &str) -> Result<()> {
        let tx = self.conn.transaction()?;
        {
            let tag_id = Self::get_or_create_tag(&tx, tag)?;

            for path in paths {
                let file_id = Self::get_or_create_file(&tx, path)?;
                tx.execute(
                    "INSERT OR IGNORE INTO file_tags (file_id, tag_id) VALUES (?1, ?2)",
                    params![file_id, tag_id],
                )?;
            }
        }

        tx.commit()?;
        Ok(())
    }

    pub fn remove_tags_batch(&mut self, paths: &[PathBuf], tag: &str) -> Result<()> {
        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare(queries::REMOVE_TAGS)?;

            for path in paths {
                let canonical_path = path.canonicalize().context("Failed to canonicalize path")?;
                stmt.execute(params![canonical_path.to_string_lossy(), tag])?;
            }
        }

        tx.commit()?;
        Ok(())
    }

    pub fn list_tagged(&self, tag: &str) -> Result<Vec<PathBuf>> {
        let mut stmt = self.conn.prepare(queries::LIST_TAGS)?;

        let paths = stmt
            .query_map([tag], |row| {
                let path_str: String = row.get(0)?;
                Ok(PathBuf::from(path_str))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(paths)
    }

    pub fn search_tags(
        &self,
        include_tags: &[&str],
        exclude_tags: &[&str],
        any: bool,
    ) -> anyhow::Result<Vec<PathBuf>> {
        if include_tags.is_empty() {
            return Ok(Vec::new());
        }

        let include_placeholders = vec!["?"; include_tags.len()].join(",");
        let exclude_placeholders = if !exclude_tags.is_empty() {
            vec!["?"; exclude_tags.len()].join(",")
        } else {
            String::new()
        };

        let having_clause = if any {
            "HAVING COUNT(DISTINCT t.name) >= 1".into()
        } else {
            format!("HAVING COUNT(DISTINCT t.name) = {}", include_tags.len())
        };

        let exclude_clause = if !exclude_tags.is_empty() {
            templates::EXCLUDE_CLAUSE.replace("{exclude_placeholders}", &exclude_placeholders)
        } else {
            String::new()
        };

        let query = templates::SEARCH_QUERY
            .replace("{include_placeholders}", &include_placeholders)
            .replace("{exclude_clause}", &exclude_clause)
            .replace("{having_clause}", &having_clause);

        let mut stmt = self.conn.prepare(&query)?;

        let mut params: Vec<&dyn ToSql> = include_tags.iter().map(|s| s as &dyn ToSql).collect();
        params.extend(exclude_tags.iter().map(|s| s as &dyn ToSql));

        let paths = stmt
            .query_map(params_from_iter(params), |row| {
                let path_str: String = row.get(0)?;
                Ok(PathBuf::from(path_str))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(paths)
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

    #[test]
    fn test_special_character_tags() -> Result<()> {
        let mut store = setup_test_db()?;
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("test_file");
        fs::write(&test_file, "test")?;

        let special_tags = vec!["태그", "标签", "🏷️", "tag with spaces", "@#$%"];
        for tag in special_tags {
            store.add_tags_batch(&[test_file.clone()], tag)?;
            let paths = store.list_tagged(tag)?;
            assert_eq!(paths.len(), 1);
        }
        Ok(())
    }

    #[test]
    fn test_symlink_path() -> Result<()> {
        let mut store = setup_test_db()?;
        let temp_dir = TempDir::new()?;

        let real_file = temp_dir.path().join("real_file");
        let symlink = temp_dir.path().join("symlink");
        fs::write(&real_file, "test")?;
        std::os::unix::fs::symlink(&real_file, &symlink)?;

        store.add_tags_batch(&[symlink.clone()], "tag")?;
        let paths = store.list_tagged("tag")?;
        assert_eq!(paths[0], real_file.canonicalize()?);

        Ok(())
    }

    #[test]
    fn test_large_tag_name() -> Result<()> {
        let mut store = setup_test_db()?;
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("test_file");
        fs::write(&test_file, "test")?;

        let large_tag = "a".repeat(10000);
        store.add_tags_batch(&[test_file], &large_tag)?;

        Ok(())
    }
}
