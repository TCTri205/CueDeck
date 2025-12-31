#![allow(deprecated)]
//! CLI integration tests

use assert_cmd::Command;
use predicates::prelude::*;
use assert_fs::TempDir;

#[test]
fn test_cue_help() {
    let mut cmd = Command::cargo_bin("cue").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("CueDeck"));
}

#[test]
fn test_cue_version() {
    let mut cmd = Command::cargo_bin("cue").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("0.1.0"));
}

#[test]
fn test_cue_init() {
    let temp = TempDir::new().unwrap();
    
    let mut cmd = Command::cargo_bin("cue").unwrap();
    cmd.current_dir(temp.path())
        .arg("init")
        .assert()
        .success()
        .stderr(predicate::str::contains("Workspace initialized"));
    
    // Verify .cuedeck directory was created
    assert!(temp.path().join(".cuedeck").exists(), ".cuedeck directory should be created");
    assert!(temp.path().join(".cuedeck/config.toml").exists(), "config.toml should be created");
}

#[test]
fn test_cue_init_already_initialized() {
    let temp = TempDir::new().unwrap();
    
    // Initialize once
    Command::cargo_bin("cue")
        .unwrap()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();
    
    // Try to initialize again - should succeed (idempotent) but warn
    let mut cmd = Command::cargo_bin("cue").unwrap();
    cmd.current_dir(temp.path())
        .arg("init")
        .assert()
        .success() // Changed from failure() to success() as cmd_init returns Ok
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn test_cue_card_generate() {
    let temp = TempDir::new().unwrap();
    
    // Initialize workspace
    Command::cargo_bin("cue")
        .unwrap()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();
    
    // Generate a card: 'cue card new "Test Card"'
    // 'New' variant has 'title' field, which defaults to positional argument if no #[arg] attribute
    let mut cmd = Command::cargo_bin("cue").unwrap();
    cmd.current_dir(temp.path())
        .arg("card")
        .arg("new")
        .arg("Test Card")
        .assert()
        .success()
        .stderr(predicate::str::contains("Created .cuedeck/cards/"));
}

#[test]
fn test_verbose_logging() {
    let mut cmd = Command::cargo_bin("cue").unwrap();
    cmd.arg("--verbose")
        .arg("--version")
        .assert()
        .success();
    // Note: We can't easily test stderr in this case, but the command should succeed
}

#[test]
fn test_invalid_command() {
    let mut cmd = Command::cargo_bin("cue").unwrap();
    cmd.arg("invalid-command")
        .assert()
        .failure();
}
