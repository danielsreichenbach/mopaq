//! Integration tests for shell completion

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_completion_command_exists() {
    let mut cmd = Command::cargo_bin("storm-cli").unwrap();
    cmd.arg("completion")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Generate completion scripts"));
}

#[test]
fn test_generate_bash_completion() {
    let mut cmd = Command::cargo_bin("storm-cli").unwrap();
    cmd.arg("completion")
        .arg("bash")
        .assert()
        .success()
        .stdout(predicate::str::contains("_storm-cli()"))
        .stdout(predicate::str::contains("complete -F"));
}

#[test]
fn test_generate_zsh_completion() {
    let mut cmd = Command::cargo_bin("storm-cli").unwrap();
    cmd.arg("completion")
        .arg("zsh")
        .assert()
        .success()
        .stdout(predicate::str::contains("#compdef storm-cli"));
}

#[test]
fn test_generate_fish_completion() {
    let mut cmd = Command::cargo_bin("storm-cli").unwrap();
    cmd.arg("completion")
        .arg("fish")
        .assert()
        .success()
        .stdout(predicate::str::contains("complete -c storm-cli"));
}

#[test]
fn test_generate_powershell_completion() {
    let mut cmd = Command::cargo_bin("storm-cli").unwrap();
    cmd.arg("completion")
        .arg("powershell")
        .assert()
        .success()
        .stdout(predicate::str::contains("storm-cli"));
}
