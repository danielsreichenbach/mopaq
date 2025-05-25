//! Integration tests for hash debug commands

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_debug_hash_help() {
    let mut cmd = Command::cargo_bin("storm-cli").unwrap();
    cmd.arg("debug")
        .arg("hash")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Generate hash values"));
}

#[test]
fn test_debug_hash_all() {
    let mut cmd = Command::cargo_bin("storm-cli").unwrap();
    cmd.arg("debug")
        .arg("hash")
        .arg("test.txt")
        .arg("--all")
        .assert()
        .success()
        .stdout(predicate::str::contains("TABLE_OFFSET"))
        .stdout(predicate::str::contains("NAME_A"))
        .stdout(predicate::str::contains("NAME_B"))
        .stdout(predicate::str::contains("FILE_KEY"))
        .stdout(predicate::str::contains("KEY2_MIX"));
}

#[test]
fn test_debug_hash_specific_type() {
    let mut cmd = Command::cargo_bin("storm-cli").unwrap();
    cmd.arg("debug")
        .arg("hash")
        .arg("test.txt")
        .arg("--hash-type")
        .arg("file-key")
        .assert()
        .success()
        .stdout(predicate::str::contains("FILE_KEY"));
}

#[test]
fn test_debug_hash_jenkins() {
    let mut cmd = Command::cargo_bin("storm-cli").unwrap();
    cmd.arg("debug")
        .arg("hash")
        .arg("test.txt")
        .arg("--jenkins")
        .assert()
        .success()
        .stdout(predicate::str::contains("Jenkins hash"));
}

#[test]
fn test_debug_hash_compare() {
    let mut cmd = Command::cargo_bin("storm-cli").unwrap();
    cmd.arg("debug")
        .arg("hash-compare")
        .arg("file1.txt")
        .arg("file2.txt")
        .assert()
        .success()
        .stdout(predicate::str::contains("Comparing hash values"))
        .stdout(predicate::str::contains("MPQ Hash comparison"))
        .stdout(predicate::str::contains("Jenkins hash comparison"));
}

#[test]
fn test_debug_hash_listfile() {
    let mut cmd = Command::cargo_bin("storm-cli").unwrap();
    cmd.arg("debug")
        .arg("hash")
        .arg("(listfile)")
        .arg("--all")
        .assert()
        .success()
        // Check for known hash values
        .stdout(predicate::str::contains("0x5F3DE859"))
        .stdout(predicate::str::contains("0xFD657910"))
        .stdout(predicate::str::contains("0x4E9B98A7"));
}

#[test]
fn test_debug_hash_invalid_type() {
    let mut cmd = Command::cargo_bin("storm-cli").unwrap();
    cmd.arg("debug")
        .arg("hash")
        .arg("test.txt")
        .arg("--hash-type")
        .arg("invalid")
        .assert()
        .success()
        .stdout(predicate::str::contains("Invalid hash type"));
}
