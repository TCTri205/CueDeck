//! CLI command builders for tests
//!
//! Provides pre-configured command builders with clean environments
//! to prevent log pollution and ensure consistent test execution.

use assert_cmd::Command;

/// Get a Command for the `cue` binary with clean environment
///
/// This command is pre-configured with:
/// - `RUST_LOG=error` to suppress INFO/DEBUG logs in tests
/// - Clean environment to avoid interference from user settings
///
/// # Example
///
/// ```rust
/// use cue_test_helpers::cli::cue_command;
///
/// let output = cue_command()
///     .arg("--version")
///     .assert()
///     .success();
/// ```
#[allow(deprecated)]
pub fn cue_command() -> Command {
    let mut cmd = Command::cargo_bin("cue").expect("Failed to find cue binary");
    cmd.env("RUST_LOG", "error");
    cmd.env_remove("CUE_CONFIG"); // Don't use user's config
    cmd.env_remove("CUE_WORKSPACE"); // Don't use user's workspace
    cmd
}

/// Get a Command for a specific binary with clean environment
///
/// # Arguments
///
/// * `bin_name` - Name of the binary (e.g., "cue", "cue_mcp")
///
/// # Example
///
/// ```rust
/// use cue_test_helpers::cli::command_for;
///
/// let output = command_for("cue_mcp")
///     .arg("--help")
///     .assert()
///     .success();
/// ```
#[allow(deprecated)]
pub fn command_for(bin_name: &str) -> Command {
    let mut cmd = Command::cargo_bin(bin_name)
        .unwrap_or_else(|_| panic!("Failed to find {} binary", bin_name));
    cmd.env("RUST_LOG", "error");
    cmd.env_remove("CUE_CONFIG");
    cmd.env_remove("CUE_WORKSPACE");
    cmd
}
