//! Configuration management for CueDeck
//!
//! This crate handles loading and validating `.cuedeck/config.toml`

use cue_common::{CueError, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Project root path (set programmatically, not in TOML)
    #[serde(skip)]
    pub root: PathBuf,

    /// Core settings
    #[serde(default)]
    pub core: CoreConfig,

    /// Parser settings
    #[serde(default)]
    pub parser: ParserConfig,

    /// Security settings
    #[serde(default)]
    pub security: SecurityConfig,

    /// MCP settings
    #[serde(default)]
    pub mcp: McpConfig,

    /// Author settings
    #[serde(default)]
    pub author: AuthorConfig,

    /// Watcher settings
    #[serde(default)]
    pub watcher: WatcherConfig,

    /// Cache settings
    #[serde(default)]
    pub cache: CacheConfig,

    /// Search settings (Phase 5: Hybrid Search)
    #[serde(default)]
    pub search: SearchConfig,

    // Keep old budgets field for backward compatibility
    #[serde(default, skip_serializing)]
    pub budgets: TokenBudgets,
}

/// Core configuration ([core])
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreConfig {
    #[serde(default = "default_token_limit")]
    pub token_limit: usize,

    #[serde(default = "default_hash_algo")]
    pub hash_algo: String,
}

fn default_token_limit() -> usize {
    32_000
}
fn default_hash_algo() -> String {
    "sha256".to_string()
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            token_limit: default_token_limit(),
            hash_algo: default_hash_algo(),
        }
    }
}

/// Parser configuration ([parser])
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParserConfig {
    #[serde(default = "default_ignore_patterns")]
    pub ignore_patterns: Vec<String>,

    #[serde(default = "default_anchor_levels")]
    pub anchor_levels: Vec<u8>,
}

fn default_ignore_patterns() -> Vec<String> {
    vec![
        "target/".to_string(),
        "node_modules/".to_string(),
        ".git/".to_string(),
    ]
}

fn default_anchor_levels() -> Vec<u8> {
    vec![1, 2, 3]
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            ignore_patterns: default_ignore_patterns(),
            anchor_levels: default_anchor_levels(),
        }
    }
}

/// Security configuration ([security])
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    #[serde(default = "default_secret_patterns")]
    pub secret_patterns: Vec<String>,

    #[serde(default)]
    pub extra_patterns: Vec<String>,
}

fn default_secret_patterns() -> Vec<String> {
    vec!["sk-.*".to_string(), "ghp_.*".to_string()]
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            secret_patterns: default_secret_patterns(),
            extra_patterns: vec![],
        }
    }
}

/// MCP configuration ([mcp])
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    #[serde(default = "default_search_limit")]
    pub search_limit: usize,
}

fn default_search_limit() -> usize {
    10
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            search_limit: default_search_limit(),
        }
    }
}

/// Author configuration ([author])
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthorConfig {
    #[serde(default)]
    pub name: String,

    #[serde(default)]
    pub email: String,
}

/// Watcher configuration ([watcher])
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatcherConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,

    #[serde(default = "default_debounce_ms")]
    pub debounce_ms: u64,

    #[serde(default = "default_watcher_ignore")]
    pub ignore_patterns: Vec<String>,
}

fn default_debounce_ms() -> u64 {
    500
}
fn default_watcher_ignore() -> Vec<String> {
    vec![".git/".to_string(), ".cache/".to_string()]
}

impl Default for WatcherConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            debounce_ms: default_debounce_ms(),
            ignore_patterns: default_watcher_ignore(),
        }
    }
}

/// Cache configuration ([cache])
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    #[serde(default = "default_cache_mode")]
    pub cache_mode: String,

    #[serde(default = "default_memory_limit")]
    pub memory_limit_mb: usize,

    /// Legacy enabled field for backward compat
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_cache_mode() -> String {
    "lazy".to_string()
}
fn default_memory_limit() -> usize {
    512
}
fn default_true() -> bool {
    true
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            cache_mode: default_cache_mode(),
            memory_limit_mb: default_memory_limit(),
            enabled: true,
        }
    }
}

/// Search configuration ([search]) - Phase 5: Hybrid Search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    /// Weight for semantic search (0.0-1.0), default 0.7
    #[serde(default = "default_semantic_weight")]
    pub semantic_weight: f32,

    /// Weight for keyword search (0.0-1.0), default 0.3
    #[serde(default = "default_keyword_weight")]
    pub keyword_weight: f32,

    /// Maximum entries in embedding cache (LRU eviction)
    #[serde(default = "default_embedding_cache_max")]
    pub embedding_cache_max_entries: usize,

    /// Default search mode: "hybrid", "keyword", or "semantic"
    #[serde(default = "default_search_mode")]
    pub default_mode: String,
}

fn default_semantic_weight() -> f32 {
    0.7
}
fn default_keyword_weight() -> f32 {
    0.3
}
fn default_embedding_cache_max() -> usize {
    1000
}
fn default_search_mode() -> String {
    "hybrid".to_string()
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            semantic_weight: default_semantic_weight(),
            keyword_weight: default_keyword_weight(),
            embedding_cache_max_entries: default_embedding_cache_max(),
            default_mode: default_search_mode(),
        }
    }
}

/// Token budget configuration (legacy, for backward compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBudgets {
    #[serde(default = "default_feature_budget")]
    pub feature: usize,

    #[serde(default = "default_bugfix_budget")]
    pub bugfix: usize,

    #[serde(default = "default_refactor_budget")]
    pub refactor: usize,
}

fn default_feature_budget() -> usize {
    6000
}
fn default_bugfix_budget() -> usize {
    4000
}
fn default_refactor_budget() -> usize {
    5000
}

impl Default for TokenBudgets {
    fn default() -> Self {
        Self {
            feature: default_feature_budget(),
            bugfix: default_bugfix_budget(),
            refactor: default_refactor_budget(),
        }
    }
}

impl Config {
    /// Load configuration from workspace root
    pub fn load(workspace_root: &Path) -> Result<Self> {
        let config_path = workspace_root.join(".cuedeck/config.toml");

        if !config_path.exists() {
            // Return default config
            return Ok(Self {
                root: workspace_root.to_path_buf(),
                core: CoreConfig::default(),
                parser: ParserConfig::default(),
                security: SecurityConfig::default(),
                mcp: McpConfig::default(),
                author: AuthorConfig::default(),
                watcher: WatcherConfig::default(),
                cache: CacheConfig::default(),
                search: SearchConfig::default(),
                budgets: TokenBudgets::default(),
            });
        }

        let content = std::fs::read_to_string(&config_path)
            .map_err(|e| CueError::ConfigError(format!("Failed to read config: {}", e)))?;

        let mut config: Config = toml::from_str(&content)
            .map_err(|e| CueError::ConfigError(format!("Failed to parse config: {}", e)))?;

        config.root = workspace_root.to_path_buf();
        Ok(config)
    }
}
