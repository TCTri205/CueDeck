//! CRDT document wrapper around automerge

use automerge::{AutoCommit, transaction::Transactable, ReadDoc, ROOT};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Wrapper around automerge document for CueDeck workspace
pub struct CuedeckDocument {
    /// Automerge document (conflict-free replicated data type)
    doc: AutoCommit,
    
    /// Current version number
    version: u64,
}

/// Document content structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentEntry {
    pub content: String,
    pub frontmatter: HashMap<String, serde_json::Value>,
    pub last_modified_by: String,
    pub version: u64,
}

impl CuedeckDocument {
    /// Create a new empty document
    pub fn new() -> Self {
        Self {
            doc: AutoCommit::new(),
            version: 0,
        }
    }

    /// Initialize document structure
    pub fn initialize(&mut self, workspace_id: &str) -> crate::Result<()> {
        self.doc
            .put(ROOT, "workspace_id", workspace_id)
            .map_err(|e| crate::SyncError::CrdtError(e.to_string()))?;
        
        self.doc
            .put_object(ROOT, "documents", automerge::ObjType::Map)
            .map_err(|e| crate::SyncError::CrdtError(e.to_string()))?;
        
        self.doc
            .put_object(ROOT, "metadata", automerge::ObjType::Map)
            .map_err(|e| crate::SyncError::CrdtError(e.to_string()))?;
        
        Ok(())
    }

    /// Add or update a document
    pub fn set_document(
        &mut self,
        path: &str,
        content: &str,
        frontmatter: HashMap<String, serde_json::Value>,
        user_id: &str,
    ) -> crate::Result<()> {
        // Get the documents object (extract ObjId from tuple)
        let docs_result = self
            .doc
            .get(ROOT, "documents")
            .map_err(|e| crate::SyncError::CrdtError(e.to_string()))?;
        
        let (_value, docs_obj) = docs_result
            .ok_or_else(|| crate::SyncError::CrdtError("documents object not found".to_string()))?;

        // Create document entry object
        let doc_obj = self
            .doc
            .put_object(&docs_obj, path, automerge::ObjType::Map)
            .map_err(|e| crate::SyncError::CrdtError(e.to_string()))?;

        // Set content (text type for OT)
        self.doc
            .put(&doc_obj, "content", content)
            .map_err(|e| crate::SyncError::CrdtError(e.to_string()))?;

        // Set frontmatter (map type for LWW/Union)
        let fm_obj = self
            .doc
            .put_object(&doc_obj, "frontmatter", automerge::ObjType::Map)
            .map_err(|e| crate::SyncError::CrdtError(e.to_string()))?;

        for (key, value) in frontmatter {
            let val_str = serde_json::to_string(&value)?;
            self.doc
                .put(&fm_obj, key, val_str)
                .map_err(|e| crate::SyncError::CrdtError(e.to_string()))?;
        }

        // Set metadata
        self.doc
            .put(&doc_obj, "last_modified_by", user_id)
            .map_err(|e| crate::SyncError::CrdtError(e.to_string()))?;

        self.version += 1;
        self.doc
            .put(&doc_obj, "version", self.version)
            .map_err(|e| crate::SyncError::CrdtError(e.to_string()))?;

        Ok(())
    }

    /// Get current version number
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Get automerge changes since version
    pub fn get_changes_since(&self, _version: u64) -> Vec<u8> {
        // TODO: Implement proper change extraction
        // For now, return empty (will be implemented with automerge APIs)
        vec![]
    }

    /// Apply remote changes
    pub fn apply_changes(&mut self, _changes: &[u8]) -> crate::Result<()> {
        // TODO: Implement change application
        // automerge will handle conflict resolution automatically
        Ok(())
    }

    /// Save document state to binary
    pub fn save(&mut self) -> Vec<u8> {
        self.doc.save()
    }

    /// Load document from binary
    pub fn load(data: &[u8]) -> crate::Result<Self> {
        let doc = AutoCommit::load(data)
            .map_err(|e| crate::SyncError::CrdtError(e.to_string()))?;
        
        Ok(Self { doc, version: 0 }) // TODO: Extract version from doc
    }
}

impl Default for CuedeckDocument {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_creation() {
        let mut doc = CuedeckDocument::new();
        doc.initialize("workspace-123").unwrap();

        let mut frontmatter = HashMap::new();
        frontmatter.insert("title".to_string(), serde_json::json!("Test Doc"));
        frontmatter.insert("priority".to_string(), serde_json::json!("high"));

        doc.set_document(
            "test.md",
            "# Test Content",
            frontmatter,
            "user-123",
        )
        .unwrap();

        assert!(doc.version() > 0);
    }
}
