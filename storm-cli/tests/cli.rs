//! Integration tests for storm-cli

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("storm-cli").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Command-line tool for working with MPQ archives",
        ));
}

#[test]
fn test_cli_version() {
    let mut cmd = Command::cargo_bin("storm-cli").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("storm-cli"));
}

#[test]
fn test_file_list_command_help() {
    let mut cmd = Command::cargo_bin("storm-cli").unwrap();
    cmd.arg("file")
        .arg("list")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("List files in an archive"));
}

#[test]
fn test_missing_archive() {
    let mut cmd = Command::cargo_bin("storm-cli").unwrap();
    cmd.arg("file").arg("list").assert().failure();
}

#[test]
fn test_archive_info_help() {
    let mut cmd = Command::cargo_bin("storm-cli").unwrap();
    cmd.arg("archive")
        .arg("info")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Show detailed archive information",
        ));
}

#[test]
fn test_archive_info_missing_archive() {
    let mut cmd = Command::cargo_bin("storm-cli").unwrap();
    cmd.arg("archive").arg("info").assert().failure();
}

#[test]
fn test_hash_generate_help() {
    let mut cmd = Command::cargo_bin("storm-cli").unwrap();
    cmd.arg("hash")
        .arg("generate")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Generate hash values"));
}

#[test]
fn test_hash_generate_all() {
    let mut cmd = Command::cargo_bin("storm-cli").unwrap();
    cmd.arg("hash")
        .arg("generate")
        .arg("test.txt")
        .arg("--all")
        .assert()
        .success()
        .stdout(predicate::str::contains("Table offset"))
        .stdout(predicate::str::contains("Name A"))
        .stdout(predicate::str::contains("Name B"))
        .stdout(predicate::str::contains("File key"))
        .stdout(predicate::str::contains("Key2 mix"));
}

#[test]
fn test_hash_generate_specific_type() {
    let mut cmd = Command::cargo_bin("storm-cli").unwrap();
    cmd.arg("hash")
        .arg("generate")
        .arg("test.txt")
        .arg("--hash-type")
        .arg("file-key")
        .assert()
        .success();
}

#[test]
fn test_hash_jenkins() {
    let mut cmd = Command::cargo_bin("storm-cli").unwrap();
    cmd.arg("hash")
        .arg("jenkins")
        .arg("test.txt")
        .assert()
        .success()
        .stdout(predicate::str::contains("Jenkins hash"));
}

#[test]
fn test_hash_compare() {
    let mut cmd = Command::cargo_bin("storm-cli").unwrap();
    cmd.arg("hash")
        .arg("compare")
        .arg("file1.txt")
        .arg("file2.txt")
        .assert()
        .success()
        .stdout(predicate::str::contains("Hash comparison"));
}

#[test]
fn test_hash_generate_listfile() {
    let mut cmd = Command::cargo_bin("storm-cli").unwrap();
    cmd.arg("hash")
        .arg("generate")
        .arg("(listfile)")
        .arg("--all")
        .assert()
        .success()
        .stdout(predicate::str::contains("Table offset"));
}

#[test]
fn test_hash_generate_invalid_type() {
    let mut cmd = Command::cargo_bin("storm-cli").unwrap();
    cmd.arg("hash")
        .arg("generate")
        .arg("test.txt")
        .arg("--hash-type")
        .arg("invalid")
        .assert()
        .failure();
}
