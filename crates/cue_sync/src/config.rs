//! Configuration for sync engine

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

/// Configuration for the sync engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    /// WebSocket relay server URL
    pub relay_url: String,

    /// Workspace ID (acts as room identifier)
    pub workspace_id: String,

    /// User ID (from ~/.cuedeck/user.toml)
    pub user_id: String,

    /// User display name
    pub user_name: String,

    /// Optional authentication token for self-hosted relays
    pub auth_token: Option<String>,

    /// Directory for pending changes when offline
    pub pending_dir: PathBuf,

    /// Maximum offline duration before full resync (default: 7 days)
    pub max_offline_duration: Duration,

    /// Auto-sync interval (default: 30 seconds)
    pub sync_interval: Duration,

    /// Enable end-to-end encryption
    pub encryption_enabled: bool,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            relay_url: "ws://localhost:8080".to_string(),
            workspace_id: String::new(),
            user_id: String::new(),
            user_name: "CueDeck User".to_string(),
            auth_token: None,
            pending_dir: PathBuf::from(".cuedeck/sync/pending"),
            max_offline_duration: Duration::from_secs(7 * 24 * 60 * 60), // 7 days
            sync_interval: Duration::from_secs(30),
            encryption_enabled: true,
        }
    }
}

impl SyncConfig {
    /// Load config from TOML file
    pub fn from_toml(path: impl Into<PathBuf>) -> anyhow::Result<Self> {
        let path = path.into();
        let content = std::fs::read_to_string(&path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }

    /// Validate configuration
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.workspace_id.is_empty() {
            anyhow::bail!("workspace_id cannot be empty");
        }
        if self.user_id.is_empty() {
            anyhow::bail!("user_id cannot be empty");
        }
        if !self.relay_url.starts_with("ws://") && !self.relay_url.starts_with("wss://") {
            anyhow::bail!("relay_url must start with ws:// or wss://");
        }
        Ok(())
    }
}
