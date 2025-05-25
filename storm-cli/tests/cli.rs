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
fn test_list_command_help() {
    let mut cmd = Command::cargo_bin("storm-cli").unwrap();
    cmd.arg("list")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("List files in an archive"));
}

#[test]
fn test_missing_archive() {
    let mut cmd = Command::cargo_bin("storm-cli").unwrap();
    cmd.arg("list").assert().failure();
}
