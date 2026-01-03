//! Shared test utilities for CueDeck test suites
//!
//! This crate provides common testing utilities to eliminate code duplication
//! across test suites and ensure consistent test environments.
//!
//! # Modules
//!
//! - [`workspace`]: Test workspace initialization and setup
//! - [`cli`]: Command builders with pre-configured environments
//! - [`logging`]: Test logging configuration
//! - [`assertions`]: Domain-specific assertion helpers
//!
//! # Example
//!
//! ```rust
//! use cue_test_helpers::prelude::*;
//!
//! fn my_test() {
//!     // Create a test workspace with .cuedeck structure
//!     let workspace = init_workspace();
//!     
//!     // Use pre-configured command with RUST_LOG=error
//!     let mut cmd = cue_command()
//!         .current_dir(workspace.path())
//!         .arg("list")
//!         .assert()
//!         .success();
//! }
//! ```

pub mod workspace;
pub mod cli;
pub mod logging;
pub mod assertions;

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::workspace::{temp_dir, init_workspace, workspace_with_cards};
    pub use crate::cli::{cue_command, command_for};
    pub use crate::logging::{init_test_logging, suppress_logs};
    pub use crate::assertions::*;
}
