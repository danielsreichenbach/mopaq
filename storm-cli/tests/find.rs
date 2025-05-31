//! Integration tests for find command

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_find_command_help() {
    let mut cmd = Command::cargo_bin("storm-cli").unwrap();
    cmd.arg("file")
        .arg("find")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Find files in an archive"));
}

#[test]
fn test_find_missing_args() {
    let mut cmd = Command::cargo_bin("storm-cli").unwrap();
    cmd.arg("file").arg("find").assert().failure();
}

#[test]
fn test_find_nonexistent_archive() {
    let mut cmd = Command::cargo_bin("storm-cli").unwrap();
    cmd.arg("file")
        .arg("find")
        .arg("nonexistent.mpq")
        .arg("file.txt")
        .assert()
        .failure()
        .stderr(predicate::str::contains("I/O error"));
}
