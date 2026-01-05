//! Sync engine orchestration

use crate::{
    config::SyncConfig, doc::CuedeckDocument, offline::OfflineSyncManager,
    protocol::SyncMessage,
};
use std::collections::HashMap;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use futures_util::{SinkExt, StreamExt};

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// Peer connection information
pub struct PeerConnection {
    pub id: String,
    pub last_sync: std::time::SystemTime,
}

/// Main sync engine
pub struct SyncEngine {
    config: SyncConfig,
    document: CuedeckDocument,
    offline_manager: OfflineSyncManager,
    ws_stream: Option<WsStream>,
    #[allow(dead_code)] // Will be used for multi-peer features
    peers: HashMap<String, PeerConnection>,
}

impl SyncEngine {
    /// Create new sync engine
    pub async fn new(config: SyncConfig) -> crate::Result<Self> {
        config.validate()?;

        let mut document = CuedeckDocument::new();
        document.initialize(&config.workspace_id)?;

        let offline_manager = OfflineSyncManager::new(
            config.pending_dir.clone(),
            config.max_offline_duration,
        );

        Ok(Self {
            config,
            document,
            offline_manager,
            ws_stream: None,
            peers: HashMap::new(),
        })
    }

    /// Connect to relay server
    pub async fn connect(&mut self) -> crate::Result<()> {
        tracing::info!("Connecting to relay: {}", self.config.relay_url);

        let (ws_stream, _) = connect_async(&self.config.relay_url).await?;
        self.ws_stream = Some(ws_stream);

        // Send handshake
        self.send_message(SyncMessage::Handshake {
            peer_id: self.config.user_id.clone(),
            workspace_id: self.config.workspace_id.clone(),
            auth_token: self.config.auth_token.clone(),
        })
        .await?;

        tracing::info!("Connected successfully");
        Ok(())
    }

    /// Start sync loop
    pub async fn start(&mut self) -> crate::Result<()> {
        self.connect().await?;

        // Load pending changes if resuming after offline
        self.offline_manager.load_pending_from_disk()?;

        // Check if full resync needed
        if self.offline_manager.needs_full_resync() {
            tracing::warn!("Offline >7 days, performing full resync");
            self.full_sync().await?;
        } else if !self.offline_manager.get_pending_changes().is_empty() {
            tracing::info!("Syncing {} pending changes", 
                self.offline_manager.get_pending_changes().len());
            self.incremental_sync().await?;
        }

        // Main event loop
        self.event_loop().await
    }

    /// Event loop for handling sync messages
    async fn event_loop(&mut self) -> crate::Result<()> {
        loop {
            // Take ownership of ws_stream temporarily to avoid borrow conflict
            let mut ws_stream = self.ws_stream.take()
                .ok_or_else(|| crate::SyncError::CrdtError("Not connected".to_string()))?;

            // Receive next message
            let message_result = ws_stream.next().await;
            
            // Put ws_stream back
            self.ws_stream = Some(ws_stream);

            match message_result {
                Some(Ok(msg)) => {
                    if let tokio_tungstenite::tungstenite::Message::Text(text) = msg {
                        let sync_msg = SyncMessage::from_bytes(text.as_bytes())?;
                        self.handle_message(sync_msg).await?;
                    }
                }
                Some(Err(e)) => {
                    tracing::error!("WebSocket error: {}", e);
                    return Err(crate::SyncError::WebSocketError(Box::new(e)));
                }
                None => {
                    tracing::info!("WebSocket connection closed");
                    break;
                }
            }
        }

        Ok(())
    }

    /// Handle incoming sync message
    async fn handle_message(&mut self, message: SyncMessage) -> crate::Result<()> {
        match message {
            SyncMessage::Change { data, user_id, .. } => {
                tracing::debug!("Received change from {}", user_id);
                self.document.apply_changes(&data)?;
            }

            SyncMessage::SyncRequest { since_version } => {
                tracing::debug!("Sync request since version {}", since_version);
                let changes = self.document.get_changes_since(since_version);
                self.send_message(SyncMessage::SyncResponse { changes })
                    .await?;
            }

            SyncMessage::Heartbeat => {
                // Respond with heartbeat
                self.send_message(SyncMessage::Heartbeat).await?;
            }

            _ => {}
        }

        Ok(())
    }

    /// Send message to relay
    async fn send_message(&mut self, message: SyncMessage) -> crate::Result<()> {
        let ws_stream = self
            .ws_stream
            .as_mut()
            .ok_or_else(|| crate::SyncError::CrdtError("Not connected".to_string()))?;

        let bytes = message.to_bytes()?;
        let text = String::from_utf8_lossy(&bytes).to_string();

        ws_stream
            .send(tokio_tungstenite::tungstenite::Message::Text(text))
            .await?;

        Ok(())
    }

    /// Perform full resync (download entire state)
    async fn full_sync(&mut self) -> crate::Result<()> {
        tracing::info!("Performing full resync");
        // TODO: Implement full document download
        Ok(())
    }

    /// Perform incremental sync (send pending changes)
    async fn incremental_sync(&mut self) -> crate::Result<()> {
        tracing::info!("Performing incremental sync");
        let compressed = self.offline_manager.compress_pending()?;

        self.send_message(SyncMessage::Change {
            data: compressed,
            timestamp: chrono::Utc::now().timestamp(),
            user_id: self.config.user_id.clone(),
        })
        .await?;

        self.offline_manager.clear_pending()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sync_engine_creation() {
        let config = SyncConfig {
            workspace_id: "test-workspace".to_string(),
            user_id: "test-user".to_string(),
            ..Default::default()
        };

        let engine = SyncEngine::new(config).await.unwrap();
        assert_eq!(engine.peers.len(), 0);
    }
}
