//! Offline sync manager

use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

/// Manages pending changes when offline
pub struct OfflineSyncManager {
    /// Queue of pending changes
    pending_changes: VecDeque<Vec<u8>>,

    /// Directory for persisting pending changes
    pending_dir: PathBuf,

    /// Last successful sync timestamp
    pub last_sync: SystemTime,

    /// Maximum offline duration before full resync
    max_offline_duration: Duration,
}

impl OfflineSyncManager {
    pub fn new(pending_dir: PathBuf, max_offline_duration: Duration) -> Self {
        Self {
            pending_changes: VecDeque::new(),
            pending_dir,
            last_sync: SystemTime::now(),
            max_offline_duration,
        }
    }

    /// Add a change to pending queue
    pub fn add_pending_change(&mut self, change: Vec<u8>) -> crate::Result<()> {
        self.pending_changes.push_back(change.clone());

        // Persist to disk
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        
        let file_path = self.pending_dir.join(format!("change_{}.bin", timestamp));
        fs::create_dir_all(&self.pending_dir)?;
        fs::write(&file_path, &change)?;

        tracing::debug!("Added pending change: {:?}", file_path);
        Ok(())
    }

    /// Check if full resync is needed
    pub fn needs_full_resync(&self) -> bool {
        let offline_duration = SystemTime::now()
            .duration_since(self.last_sync)
            .unwrap_or(Duration::ZERO);

        offline_duration > self.max_offline_duration
    }

    /// Get all pending changes
    pub fn get_pending_changes(&self) -> Vec<Vec<u8>> {
        self.pending_changes.iter().cloned().collect()
    }

    /// Clear pending changes after successful sync
    pub fn clear_pending(&mut self) -> crate::Result<()> {
        self.pending_changes.clear();

        // Remove persisted files
        if self.pending_dir.exists() {
            fs::remove_dir_all(&self.pending_dir)?;
        }

        self.last_sync = SystemTime::now();
        tracing::info!("Cleared pending changes");
        Ok(())
    }

    /// Load pending changes from disk
    pub fn load_pending_from_disk(&mut self) -> crate::Result<()> {
        if !self.pending_dir.exists() {
            return Ok(());
        }

        let entries = fs::read_dir(&self.pending_dir)?;
        let mut files: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .and_then(|s| s.to_str())
                    .map(|s| s == "bin")
                    .unwrap_or(false)
            })
            .collect();

        // Sort by filename (timestamp)
        files.sort_by_key(|e| e.path());

        for entry in files {
            let data = fs::read(entry.path())?;
            self.pending_changes.push_back(data);
        }

        tracing::info!("Loaded {} pending changes from disk", self.pending_changes.len());
        Ok(())
    }

    /// Compress pending changes into single changeset
    pub fn compress_pending(&self) -> crate::Result<Vec<u8>> {
        // For now, concatenate all changes
        // TODO: Use proper automerge compression
        let mut combined = Vec::new();
        for change in &self.pending_changes {
            combined.extend_from_slice(change);
        }
        Ok(combined)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_offline_sync_manager() {
        let temp_dir = std::env::temp_dir().join("cue_sync_test");
        let _ = fs::remove_dir_all(&temp_dir); // Clean up

        let mut manager = OfflineSyncManager::new(
            temp_dir.clone(),
            Duration::from_secs(7 * 24 * 60 * 60),
        );

        // Add pending changes
        manager.add_pending_change(vec![1, 2, 3]).unwrap();
        manager.add_pending_change(vec![4, 5, 6]).unwrap();

        assert_eq!(manager.get_pending_changes().len(), 2);
        assert!(!manager.needs_full_resync());

        // Clear pending
        manager.clear_pending().unwrap();
        assert_eq!(manager.get_pending_changes().len(), 0);

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_needs_full_resync() {
        let temp_dir = std::env::temp_dir().join("cue_sync_test2");
        let _ = fs::remove_dir_all(&temp_dir);

        let mut manager = OfflineSyncManager::new(
            temp_dir.clone(),
            Duration::from_secs(1), // 1 second for test
        );

        assert!(!manager.needs_full_resync());

        // Simulate offline for 2 seconds
        manager.last_sync = SystemTime::now() - Duration::from_secs(2);
        assert!(manager.needs_full_resync());

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
