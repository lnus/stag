use assert_cmd::Command;
use predicates::prelude::*;
use serial_test::serial;
use tempfile::TempDir;

// NOTE: I can make this work in parallel, but for now
// serial_tests crate works fine.
fn with_test_env<F, R>(test: F) -> R
where
    F: FnOnce() -> R,
{
    let temp_dir =
        TempDir::new().expect("Failed to create temporary directory for test environment");
    let db_path = temp_dir.path().join("test.db");
    let old_value = std::env::var("STAG_DB_PATH").ok();
    std::env::set_var("STAG_DB_PATH", db_path.clone());

    let result = test();

    match old_value {
        Some(val) => std::env::set_var("STAG_DB_PATH", val),
        None => std::env::remove_var("STAG_DB_PATH"),
    }

    result
}

fn normalize_path(path: &std::path::Path) -> anyhow::Result<String> {
    Ok(path
        .canonicalize()
        .map_err(|e| anyhow::anyhow!("Failed to canonicalize path: {}", e))?
        .to_string_lossy()
        .to_string())
}

// NOTE: This test is kinda silly but good sanity check I guess
// I wrote it out to bootstrap the test_env functions and noramlize_path
#[test]
#[serial]
fn test_add_and_list() -> anyhow::Result<()> {
    with_test_env(|| {
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("test.txt");
        std::fs::write(&test_file, "test content")?;

        let normalized_path = normalize_path(&test_file)?;

        Command::cargo_bin("stag")?
            .args(["a", "rust", &normalized_path])
            .assert()
            .success();

        Command::cargo_bin("stag")?
            .args(["ls", "rust"])
            .assert()
            .success()
            .stdout(predicates::str::contains(&normalized_path));

        Ok(())
    })
}

#[test]
#[serial]
fn test_recursive_tagging() -> anyhow::Result<()> {
    with_test_env(|| {
        let temp_dir = TempDir::new()?;
        let proj_dir = temp_dir.path().join("project");
        std::fs::create_dir(&proj_dir)?;
        std::fs::write(proj_dir.join("main.rs"), "fn main() {}")?;
        std::fs::write(proj_dir.join("lib.rs"), "pub fn lib() {}")?;

        let normalized_dir = normalize_path(&proj_dir)?;

        Command::cargo_bin("stag")?
            .args(["a", "rust", &normalized_dir, "-r"])
            .assert()
            .success();

        Command::cargo_bin("stag")?
            .args(["ls", "rust", "--files"])
            .assert()
            .success()
            .stdout(predicates::str::contains("main.rs"))
            .stdout(predicates::str::contains("lib.rs"));

        Ok(())
    })
}

#[test]
#[serial]
fn test_tag_combinations() -> anyhow::Result<()> {
    with_test_env(|| {
        let temp_dir = TempDir::new()?;
        let proj1 = temp_dir.path().join("rust-proj");
        let proj2 = temp_dir.path().join("py-proj");
        std::fs::create_dir(&proj1)?;
        std::fs::create_dir(&proj2)?;

        let rust_path = normalize_path(&proj1)?;
        let py_path = normalize_path(&proj2)?;

        // Tag both with 'proj'
        Command::cargo_bin("stag")?
            .args(["a", "proj", &rust_path, &py_path])
            .assert()
            .success();

        // Tag rust-proj with 'rust'
        Command::cargo_bin("stag")?
            .args(["a", "rust", &rust_path])
            .assert()
            .success();

        // Test AND search
        Command::cargo_bin("stag")?
            .args(["s", "proj", "rust"])
            .assert()
            .success()
            .stdout(predicates::str::contains(&rust_path))
            .stdout(predicates::str::contains(&py_path).not());

        // Test OR search
        Command::cargo_bin("stag")?
            .args(["s", "rust", "py", "--any"])
            .assert()
            .success()
            .stdout(predicates::str::contains(&rust_path));

        // Test exclusion
        Command::cargo_bin("stag")?
            .args(["s", "proj", "-e", "rust"])
            .assert()
            .success()
            .stdout(predicates::str::contains(&py_path))
            .stdout(predicates::str::contains(&rust_path).not());

        Ok(())
    })
}

#[test]
#[serial]
fn test_tag_removal() -> anyhow::Result<()> {
    with_test_env(|| {
        let temp_dir = TempDir::new()?;
        let test_dir = temp_dir.path().join("test-dir");
        std::fs::create_dir(&test_dir)?;

        let normalized_path = normalize_path(&test_dir)?;

        // Add tags
        Command::cargo_bin("stag")?
            .args(["a", "proj", &normalized_path])
            .assert()
            .success();

        Command::cargo_bin("stag")?
            .args(["a", "rust", &normalized_path])
            .assert()
            .success();

        // Remove one tag
        Command::cargo_bin("stag")?
            .args(["rm", "rust", &normalized_path])
            .assert()
            .success();

        // Verify tag was removed
        Command::cargo_bin("stag")?
            .args(["ls", "rust"])
            .assert()
            .success()
            .stdout(predicates::str::contains(&normalized_path).not());

        // Verify other tag remains
        Command::cargo_bin("stag")?
            .args(["ls", "proj"])
            .assert()
            .success()
            .stdout(predicates::str::contains(&normalized_path));

        Ok(())
    })
}

#[test]
#[serial]
fn test_dir_file_filtering() -> anyhow::Result<()> {
    with_test_env(|| {
        let temp_dir = TempDir::new()?;
        let test_dir = temp_dir.path().join("test-dir");
        std::fs::create_dir(&test_dir)?;
        let test_file = temp_dir.path().join("test.txt");
        std::fs::write(&test_file, "content")?;

        let dir_path = normalize_path(&test_dir)?;
        let file_path = normalize_path(&test_file)?;

        // Tag both
        Command::cargo_bin("stag")?
            .args(["a", "test", &dir_path, &file_path])
            .assert()
            .success();

        // Test --dirs filter
        Command::cargo_bin("stag")?
            .args(["ls", "test", "--dirs"])
            .assert()
            .success()
            .stdout(predicates::str::contains(&dir_path))
            .stdout(predicates::str::contains(&file_path).not());

        // Test --files filter
        Command::cargo_bin("stag")?
            .args(["ls", "test", "--files"])
            .assert()
            .success()
            .stdout(predicates::str::contains(&file_path))
            .stdout(predicates::str::contains(&dir_path).not());

        Ok(())
    })
}

#[test]
#[serial]
fn test_invalid_flag_combinations() -> anyhow::Result<()> {
    with_test_env(|| {
        // Test --dirs and --files together (should fail)
        Command::cargo_bin("stag")?
            .args(["ls", "test", "--dirs", "--files"])
            .assert()
            .failure()
            .stderr(predicates::str::contains("Cannot specify both"));

        Command::cargo_bin("stag")?
            .args(["s", "test", "--dirs", "--files"])
            .assert()
            .failure()
            .stderr(predicates::str::contains("Cannot specify both"));

        Ok(())
    })
}

#[test]
#[serial]
fn test_nonexistent_paths() -> anyhow::Result<()> {
    with_test_env(|| {
        Command::cargo_bin("stag")?
            .args(["a", "test", "/path/that/does/not/exist"])
            .assert()
            .failure();

        Ok(())
    })
}

#[test]
#[serial]
fn test_empty_tag_results() -> anyhow::Result<()> {
    with_test_env(|| {
        // Search for non-existent tag should succeed but return empty
        Command::cargo_bin("stag")?
            .args(["ls", "nonexistenttag"])
            .assert()
            .success()
            .stdout(predicates::str::is_empty());

        Command::cargo_bin("stag")?
            .args(["s", "nonexistenttag"])
            .assert()
            .success()
            .stdout(predicates::str::is_empty());

        Ok(())
    })
}

#[test]
#[serial]
fn test_complex_search_combinations() -> anyhow::Result<()> {
    with_test_env(|| {
        let temp_dir = TempDir::new()?;
        let file1 = temp_dir.path().join("file1");
        let file2 = temp_dir.path().join("file2");
        std::fs::write(&file1, "content")?;
        std::fs::write(&file2, "content")?;

        let path1 = normalize_path(&file1)?;
        let path2 = normalize_path(&file2)?;

        // Setup: file1 has tags [a, b, c], file2 has tags [a, b, d]
        for tag in ["a", "b", "c"] {
            Command::cargo_bin("stag")?
                .args(["a", tag, &path1])
                .assert()
                .success();
        }
        for tag in ["a", "b", "d"] {
            Command::cargo_bin("stag")?
                .args(["a", tag, &path2])
                .assert()
                .success();
        }

        // Test complex combinations
        // AND search with exclusion
        Command::cargo_bin("stag")?
            .args(["s", "a", "b", "-e", "c"])
            .assert()
            .success()
            .stdout(predicates::str::contains(&path2))
            .stdout(predicates::str::contains(&path1).not());

        // OR search with exclusion
        Command::cargo_bin("stag")?
            .args(["s", "c", "d", "--any", "-e", "b"])
            .assert()
            .success()
            .stdout(predicates::str::is_empty());

        Ok(())
    })
}

#[test]
#[serial]
fn test_hidden_files_handling() -> anyhow::Result<()> {
    // NOTE: This test looks weird, see note in `main.rs` about hidden
    with_test_env(|| {
        let temp_dir = TempDir::new()?;

        let hidden_file = temp_dir.path().join(".hidden.txt");
        let hidden_dir = temp_dir.path().join(".hidden_dir");
        let normal_file_1 = temp_dir.path().join("normal2.txt");
        let normal_file_2 = hidden_dir.as_path().join("normal2.txt");

        std::fs::write(&hidden_file, "hidden content")?; // root
        std::fs::write(&normal_file_1, "normal content")?; // root
        std::fs::create_dir(&hidden_dir)?; // root
        std::fs::write(&normal_file_2, "normal content")?; // inside hidden dir

        let hidden_file_path = normalize_path(&hidden_file)?;
        let hidden_dir_path = normalize_path(&hidden_dir)?;
        let normal_file_1_path = normalize_path(&normal_file_1)?;
        let normal_file_2_path = normalize_path(&normal_file_2)?;

        // Should ignore hidden files
        Command::cargo_bin("stag")?
            .args(["a", "test", &temp_dir.path().to_string_lossy(), "-r"])
            .assert()
            .success();

        Command::cargo_bin("stag")?
            .args(["ls", "test"])
            .assert()
            .success()
            .stdout(predicates::str::contains(&hidden_file_path).not())
            .stdout(predicates::str::contains(&hidden_dir_path).not())
            .stdout(predicates::str::contains(&normal_file_2_path).not())
            .stdout(predicates::str::contains(&normal_file_1_path));

        // Should recurse and find hidden files
        Command::cargo_bin("stag")?
            .args([
                "a",
                "hidden",
                &temp_dir.path().to_string_lossy(),
                "-r",
                "--hidden",
            ])
            .assert()
            .success();

        Command::cargo_bin("stag")?
            .args(["ls", "hidden"])
            .assert()
            .success()
            .stdout(predicates::str::contains(&hidden_file_path))
            .stdout(predicates::str::contains(&hidden_dir_path))
            .stdout(predicates::str::contains(&normal_file_2_path))
            .stdout(predicates::str::contains(&normal_file_1_path));

        Ok(())
    })
}
