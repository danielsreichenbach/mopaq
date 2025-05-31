//! Integration tests for hash commands

use assert_cmd::Command;
use predicates::prelude::*;

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
        .success()
        .stdout(predicate::str::contains("0x82c45239"));
}
