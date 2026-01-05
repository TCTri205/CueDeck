//! # CueDeck Sync Engine
//!
//! CRDT-based peer-to-peer synchronization for team collaboration.
//!
//! ## Architecture
//!
//! - **CRDT**: automerge-rs for conflict-free replicated data types
//! - **Transport**: WebSocket for P2P communication
//! - **Offline Support**: Pending changes queue with automatic resync
//! - **Security**: Workspace isolation, rate limiting, optional auth
//!
//! ## Usage
//!
//! ```rust,no_run
//! use cue_sync::{SyncEngine, SyncConfig};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = SyncConfig {
//!         relay_url: "ws://localhost:8080".to_string(),
//!         workspace_id: "my-workspace".to_string(),
//!         user_id: "user-123".to_string(),
//!         ..Default::default()
//!     };
//!     
//!     let mut engine = SyncEngine::new(config).await?;
//!     engine.start().await?;
//!     
//!     Ok(())
//! }
//! ```

pub mod config;
pub mod doc;
pub mod engine;
pub mod offline;
pub mod protocol;
pub mod resolver;

pub use config::SyncConfig;
pub use doc::CuedeckDocument;
pub use engine::SyncEngine;
pub use offline::OfflineSyncManager;
pub use protocol::SyncMessage;
pub use resolver::{ConflictResolver, ConflictStrategy};

/// Common result type for sync operations
pub type Result<T> = std::result::Result<T, SyncError>;

/// Errors that can occur during sync operations
#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("CRDT operation failed: {0}")]
    CrdtError(String),

    #[error("WebSocket error: {0}")]
    WebSocketError(Box<tokio_tungstenite::tungstenite::Error>),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Offline sync error: {0}")]
    OfflineError(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Room full (max peers reached)")]
    RoomFull,

    #[error("Invalid message: {0}")]
    InvalidMessage(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(#[from] anyhow::Error),
}

impl From<tokio_tungstenite::tungstenite::Error> for SyncError {
    fn from(e: tokio_tungstenite::tungstenite::Error) -> Self {
        SyncError::WebSocketError(Box::new(e))
    }
}
