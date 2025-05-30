//! Integration tests for create command

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_create_command_help() {
    let mut cmd = Command::cargo_bin("storm-cli").unwrap();
    cmd.arg("create")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Create a new MPQ archive"));
}

#[test]
fn test_create_simple_archive() {
    let temp_dir = TempDir::new().unwrap();
    let source_dir = temp_dir.path().join("source");
    let archive_path = temp_dir.path().join("test.mpq");

    // Create test files
    fs::create_dir_all(&source_dir).unwrap();
    fs::write(source_dir.join("file1.txt"), "Hello, MPQ!").unwrap();
    fs::write(source_dir.join("file2.txt"), "Another file").unwrap();

    // Create archive
    let mut cmd = Command::cargo_bin("storm-cli").unwrap();
    cmd.arg("create")
        .arg(archive_path.to_str().unwrap())
        .arg(source_dir.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Archive created successfully!"));

    // Verify archive exists
    assert!(archive_path.exists());

    // List contents
    let mut cmd = Command::cargo_bin("storm-cli").unwrap();
    cmd.arg("list")
        .arg(archive_path.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("file1.txt"))
        .stdout(predicate::str::contains("file2.txt"));
}

#[test]
fn test_create_with_subdirectories() {
    let temp_dir = TempDir::new().unwrap();
    let source_dir = temp_dir.path().join("source");
    let archive_path = temp_dir.path().join("test.mpq");

    // Create nested structure
    let sub_dir = source_dir.join("subdir");
    fs::create_dir_all(&sub_dir).unwrap();
    fs::write(source_dir.join("root.txt"), "Root file").unwrap();
    fs::write(sub_dir.join("nested.txt"), "Nested file").unwrap();

    // Create archive
    let mut cmd = Command::cargo_bin("storm-cli").unwrap();
    cmd.arg("create")
        .arg(archive_path.to_str().unwrap())
        .arg(source_dir.to_str().unwrap())
        .assert()
        .success();

    // Verify files with correct paths
    let mut cmd = Command::cargo_bin("storm-cli").unwrap();
    cmd.arg("list")
        .arg(archive_path.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("root.txt"))
        .stdout(predicate::str::contains("subdir\\nested.txt"));
}

#[test]
fn test_create_no_listfile() {
    let temp_dir = TempDir::new().unwrap();
    let source_dir = temp_dir.path().join("source");
    let archive_path = temp_dir.path().join("test.mpq");

    fs::create_dir_all(&source_dir).unwrap();
    fs::write(source_dir.join("test.txt"), "Test").unwrap();

    // Create archive without listfile
    let mut cmd = Command::cargo_bin("storm-cli").unwrap();
    cmd.arg("create")
        .arg(archive_path.to_str().unwrap())
        .arg(source_dir.to_str().unwrap())
        .arg("--no-listfile")
        .assert()
        .success();

    // Try to list - should show warning about no listfile
    let mut cmd = Command::cargo_bin("storm-cli").unwrap();
    cmd.arg("list")
        .arg(archive_path.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("No (listfile) found"));
}
