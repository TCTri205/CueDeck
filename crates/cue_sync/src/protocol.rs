//! Sync protocol message types

use serde::{Deserialize, Serialize};

/// Messages exchanged between peers via relay server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SyncMessage {
    /// Initial connection handshake
    Handshake {
        peer_id: String,
        workspace_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        auth_token: Option<String>,
    },

    /// Request changes since specific version
    SyncRequest { since_version: u64 },

    /// Response with changes
    SyncResponse { changes: Vec<u8> }, // Binary automerge changes

    /// Real-time change notification
    Change {
        data: Vec<u8>, // Automerge binary patch
        timestamp: i64,
        user_id: String,
    },

    /// Periodic heartbeat to keep connection alive
    Heartbeat,

    /// Acknowledge message receipt
    Ack { msg_id: String },

    /// Error notification
    Error { code: String, message: String },
}

impl SyncMessage {
    /// Serialize message to JSON bytes
    pub fn to_bytes(&self) -> crate::Result<Vec<u8>> {
        Ok(serde_json::to_vec(self)?)
    }

    /// Deserialize message from JSON bytes
    pub fn from_bytes(bytes: &[u8]) -> crate::Result<Self> {
        Ok(serde_json::from_slice(bytes)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_serialization() {
        let msg = SyncMessage::Handshake {
            peer_id: "peer-123".to_string(),
            workspace_id: "workspace-abc".to_string(),
            auth_token: None,
        };

        let bytes = msg.to_bytes().unwrap();
        let deserialized = SyncMessage::from_bytes(&bytes).unwrap();

        match deserialized {
            SyncMessage::Handshake { peer_id, .. } => {
                assert_eq!(peer_id, "peer-123");
            }
            _ => panic!("Expected Handshake message"),
        }
    }
}
