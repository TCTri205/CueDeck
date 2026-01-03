//! Common types and errors for CueDeck
//!
//! This crate provides shared data structures used across all CueDeck components.

pub mod sanitizer;
pub mod telemetry;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

/// Core error types for CueDeck operations
#[derive(Error, Debug)]
pub enum CueError {
    #[error("File not found: {path}")]
    FileNotFound { path: String },

    #[error("Cycle detected in dependency graph")]
    CycleDetected,

    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),

    #[error("Dependency not found: {0}")]
    DependencyNotFound(String),

    #[error("Token limit exceeded: {current} > {limit}")]
    TokenLimit { current: usize, limit: usize },

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Invalid configuration: {0}")]
    ConfigError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Cache is stale")]
    StaleCache,

    #[error("Resource locked by PID {pid}")]
    Locked { pid: u32 },

    #[error("Rate limit exceeded: {current}/{limit} in {window}s")]
    RateLimit {
        current: usize,
        limit: usize,
        window: u64,
    },

    #[error("Invalid input: {0}")]
    ValidationError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// Standard metadata for Cue Cards
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CardMetadata {
    /// Title of the card/document
    pub title: String,

    /// Status of the task (todo, active, done, archived)
    #[serde(default = "default_status")]
    pub status: String,

    /// Assignee (GitHub username or similar)
    #[serde(default)]
    pub assignee: Option<String>,

    /// Priority (low, medium, high, critical)
    #[serde(default = "default_priority")]
    pub priority: String,

    /// Tags/Labels
    #[serde(default)]
    pub tags: Option<Vec<String>>,

    /// Creation timestamp (ISO 8601)
    #[serde(default)]
    pub created: Option<String>,

    /// Last updated timestamp (ISO 8601)
    /// Automatically set by update_task() or manual edits
    #[serde(default)]
    pub updated: Option<String>,

    /// Task IDs this task depends on
    #[serde(default)]
    pub depends_on: Option<Vec<String>>,
}

fn default_status() -> String {
    "todo".to_string()
}
fn default_priority() -> String {
    "medium".to_string()
}

/// Represents a markdown document with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// Path to the document (workspace-relative or absolute)
    pub path: PathBuf,

    /// Parsed YAML frontmatter (typed)
    pub frontmatter: Option<CardMetadata>,

    /// SHA256 hash of content
    pub hash: String,

    /// Estimated token count
    pub tokens: usize,

    /// Parsed anchors (headings)
    pub anchors: Vec<Anchor>,

    /// Outgoing links (dependencies) detected in the file
    #[serde(default)]
    pub links: Vec<String>,
}

/// Represents a heading/anchor within a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anchor {
    /// URL-safe slug (e.g., "login-flow")
    pub slug: String,

    /// Original header text
    pub header: String,

    /// Heading level (1-6)
    pub level: u8,

    /// Start line number (1-indexed)
    pub start_line: usize,

    /// End line number (inclusive, 1-indexed)
    pub end_line: usize,
}

/// Represents a task dependency relationship
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskDependency {
    /// ID of the task that has the dependency
    pub from_id: String,
    /// ID of the task that is depended upon
    pub to_id: String,
}

/// Result type alias
/// Result type alias
pub type Result<T> = std::result::Result<T, CueError>;

/// Exit code constants per CLI_REFERENCE.md
pub const EXIT_SUCCESS: i32 = 0;
pub const EXIT_ERROR: i32 = 1;
pub const EXIT_USAGE: i32 = 2;
pub const EXIT_CONFIG_ERROR: i32 = 101;
pub const EXIT_TERMINATED: i32 = 130;
