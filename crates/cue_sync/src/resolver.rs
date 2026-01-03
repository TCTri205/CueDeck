//! Conflict resolution strategies

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Conflict resolution strategies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictStrategy {
    /// Operational Transformation (for text content)
    /// automerge handles this automatically
    OperationalTransform,

    /// Last-Write-Wins (for scalar values)
    /// Keep the change with the latest timestamp
    LastWriteWins,

    /// Union Merge (for arrays/sets)
    /// Combine all values from all changes
    UnionMerge,

    /// Tombstone (for deletions)
    /// Mark as deleted but keep for recovery period
    Tombstone,
}

/// Represents a conflict between concurrent changes
#[derive(Debug, Clone)]
pub struct Conflict {
    pub field_type: FieldType,
    pub changes: Vec<Change>,
}

/// Type of field being modified
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldType {
    /// Markdown content (uses OT)
    Content,

    /// Scalar metadata field (uses LWW)
    MetadataScalar,

    /// Array metadata field (uses Union)
    MetadataArray,

    /// File deletion (uses Tombstone)
    Deletion,
}

/// A single change made by a peer
#[derive(Debug, Clone)]
pub struct Change {
    pub timestamp: i64,
    pub user_id: String,
    pub value: serde_json::Value,
}

/// Result of conflict resolution
#[derive(Debug, Clone)]
pub enum Resolution {
    /// Auto-merged by automerge (OT)
    AutoMerged,

    /// Keep specific change (LWW)
    KeepChange(Change),

    /// Union of all values (Union)
    Union(Vec<serde_json::Value>),

    /// Mark as tombstone (Deletion)
    Tombstone {
        expires_at: std::time::SystemTime,
    },
}

/// Conflict resolver
pub struct ConflictResolver {
    tombstone_ttl_days: u64,
}

impl ConflictResolver {
    pub fn new(tombstone_ttl_days: u64) -> Self {
        Self {
            tombstone_ttl_days,
        }
    }

    /// Resolve a conflict based on field type
    pub fn resolve(&self, conflict: &Conflict) -> crate::Result<Resolution> {
        match conflict.field_type {
            FieldType::Content => {
                // OT is handled automatically by automerge
                Ok(Resolution::AutoMerged)
            }

            FieldType::MetadataScalar => {
                // Last-Write-Wins: Keep change with latest timestamp
                let latest = conflict
                    .changes
                    .iter()
                    .max_by_key(|c| c.timestamp)
                    .ok_or_else(|| {
                        crate::SyncError::CrdtError("No changes to resolve".to_string())
                    })?;

                Ok(Resolution::KeepChange(latest.clone()))
            }

            FieldType::MetadataArray => {
                // Union: Combine all unique values
                let mut union: HashSet<String> = HashSet::new();
                for change in &conflict.changes {
                    if let serde_json::Value::Array(arr) = &change.value {
                        for val in arr {
                            if let Some(s) = val.as_str() {
                                union.insert(s.to_string());
                            }
                        }
                    }
                }

                let values: Vec<serde_json::Value> = union
                    .into_iter()
                    .map(serde_json::Value::String)
                    .collect();

                Ok(Resolution::Union(values))
            }

            FieldType::Deletion => {
                // Tombstone: Mark for deletion with TTL
                let expires_at = std::time::SystemTime::now()
                    + std::time::Duration::from_secs(self.tombstone_ttl_days * 24 * 60 * 60);

                Ok(Resolution::Tombstone { expires_at })
            }
        }
    }
}

impl Default for ConflictResolver {
    fn default() -> Self {
        Self::new(7) // 7 days TTL by default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_last_write_wins() {
        let resolver = ConflictResolver::default();

        let conflict = Conflict {
            field_type: FieldType::MetadataScalar,
            changes: vec![
                Change {
                    timestamp: 100,
                    user_id: "user_a".to_string(),
                    value: serde_json::json!("old"),
                },
                Change {
                    timestamp: 200,
                    user_id: "user_b".to_string(),
                    value: serde_json::json!("new"),
                },
            ],
        };

        let resolution = resolver.resolve(&conflict).unwrap();
        
        match resolution {
            Resolution::KeepChange(change) => {
                assert_eq!(change.timestamp, 200);
                assert_eq!(change.value, serde_json::json!("new"));
            }
            _ => panic!("Expected KeepChange resolution"),
        }
    }

    #[test]
    fn test_union_merge() {
        let resolver = ConflictResolver::default();

        let conflict = Conflict {
            field_type: FieldType::MetadataArray,
            changes: vec![
                Change {
                    timestamp: 100,
                    user_id: "user_a".to_string(),
                    value: serde_json::json!(["tag1", "tag2"]),
                },
                Change {
                    timestamp: 200,
                    user_id: "user_b".to_string(),
                    value: serde_json::json!(["tag2", "tag3"]),
                },
            ],
        };

        let resolution = resolver.resolve(&conflict).unwrap();
        
        match resolution {
            Resolution::Union(values) => {
                assert_eq!(values.len(), 3); // tag1, tag2, tag3 (unique)
            }
            _ => panic!("Expected Union resolution"),
        }
    }

    #[test]
    fn test_tombstone() {
        let resolver = ConflictResolver::default();

        let conflict = Conflict {
            field_type: FieldType::Deletion,
            changes: vec![],
        };

        let resolution = resolver.resolve(&conflict).unwrap();
        
        match resolution {
            Resolution::Tombstone { expires_at } => {
                assert!(expires_at > std::time::SystemTime::now());
            }
            _ => panic!("Expected Tombstone resolution"),
        }
    }
}
