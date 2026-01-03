//! Embedding cache module for CueDeck
//!
//! Provides persistent caching of document embeddings using SHA256 hashing
//! for invalidation and LRU eviction for memory management.

use crate::embeddings::EmbeddingModel;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Cached embedding entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedEmbedding {
    /// The embedding vector (384 dimensions for MiniLM-L6-v2)
    pub embedding: Vec<f32>,
    /// When this embedding was created
    pub created_at: SystemTime,
    /// Access count for LRU tracking
    pub access_count: u64,
    /// Last access time for LRU
    pub last_accessed: SystemTime,
}

/// Embedding cache manager with LRU eviction
#[derive(Debug)]
pub struct EmbeddingCache {
    /// Cache directory path (.cuedeck/cache/)
    cache_dir: PathBuf,
    /// In-memory cache: document_hash -> embedding
    entries: HashMap<String, CachedEmbedding>,
    /// Maximum entries (configurable, default 1000)
    max_entries: usize,
    /// Statistics
    hits: usize,
    misses: usize,
    /// Dirty flag for persistence
    dirty: bool,
}

impl EmbeddingCache {
    /// Create a new embedding cache instance
    ///
    /// # Arguments
    /// * `workspace_root` - Root directory of the workspace
    /// * `max_entries` - Maximum number of cached embeddings (LRU eviction)
    pub fn new(workspace_root: &Path, max_entries: usize) -> Result<Self> {
        let cache_dir = workspace_root.join(".cuedeck/cache");

        // Create cache directory if it doesn't exist
        fs::create_dir_all(&cache_dir).context("Failed to create cache directory")?;

        Ok(Self {
            cache_dir,
            entries: HashMap::new(),
            max_entries,
            hits: 0,
            misses: 0,
            dirty: false,
        })
    }

    /// Load cache from disk
    pub fn load(&mut self) -> Result<()> {
        tracing::debug!("Loading embedding cache from {:?}", self.cache_dir);

        let cache_file = self.cache_dir.join("embeddings.bin");

        if !cache_file.exists() {
            tracing::debug!("No embedding cache found, starting fresh");
            return Ok(());
        }

        match fs::read(&cache_file) {
            Ok(data) => {
                match bincode::deserialize::<HashMap<String, CachedEmbedding>>(&data) {
                    Ok(entries) => {
                        tracing::info!("Loaded {} cached embeddings", entries.len());
                        self.entries = entries;
                        Ok(())
                    }
                    Err(e) => {
                        tracing::warn!("Failed to deserialize embedding cache: {}", e);
                        tracing::info!("Cache corruption detected, rebuilding...");
                        // Auto-repair: delete corrupted cache
                        let _ = fs::remove_file(&cache_file);
                        Ok(())
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to read embedding cache file: {}", e);
                Ok(())
            }
        }
    }

    /// Save cache to disk
    pub fn save(&self) -> Result<()> {
        if !self.dirty {
            tracing::trace!("Embedding cache not dirty, skipping save");
            return Ok(());
        }

        tracing::debug!("Saving embedding cache with {} entries", self.entries.len());

        let cache_file = self.cache_dir.join("embeddings.bin");

        let data = bincode::serialize(&self.entries).context("Failed to serialize embedding cache")?;

        fs::write(&cache_file, data).context("Failed to write embedding cache file")?;

        tracing::info!("Embedding cache saved successfully");
        Ok(())
    }

    /// Get embedding from cache or compute it
    ///
    /// # Arguments
    /// * `doc_hash` - SHA256 hash of the document content
    /// * `content` - Document content (only used if cache miss)
    ///
    /// # Returns
    /// The embedding vector (384 dimensions)
    pub fn get_or_compute(&mut self, doc_hash: &str, content: &str) -> Result<Vec<f32>> {
        // Check cache first
        if let Some(cached) = self.entries.get_mut(doc_hash) {
            tracing::trace!("Embedding cache HIT: {}", &doc_hash[..8]);
            self.hits += 1;
            
            // Update LRU metadata
            cached.access_count += 1;
            cached.last_accessed = SystemTime::now();
            self.dirty = true;
            
            return Ok(cached.embedding.clone());
        }

        // Cache miss - compute embedding
        tracing::trace!("Embedding cache MISS: {}", &doc_hash[..8]);
        self.misses += 1;

        // Truncate very long content to avoid embedding overhead
        // Use char_indices to ensure we don't split in the middle of a UTF-8 character
        let content_sample = if content.len() > 5000 {
            // Find the last safe char boundary before 5000 bytes
            let safe_end = content
                .char_indices()
                .take_while(|(idx, _)| *idx < 5000)
                .last()
                .map(|(idx, ch)| idx + ch.len_utf8())
                .unwrap_or(0);
            &content[..safe_end]
        } else {
            content
        };

        let embedding = EmbeddingModel::embed(content_sample)?;

        // Evict if at capacity
        if self.entries.len() >= self.max_entries {
            self.evict_lru();
        }

        // Insert new entry
        let now = SystemTime::now();
        self.entries.insert(
            doc_hash.to_string(),
            CachedEmbedding {
                embedding: embedding.clone(),
                created_at: now,
                access_count: 1,
                last_accessed: now,
            },
        );
        self.dirty = true;

        Ok(embedding)
    }

    /// Evict least recently used entry
    fn evict_lru(&mut self) {
        if self.entries.is_empty() {
            return;
        }

        // Find LRU entry (oldest last_accessed time)
        let lru_key = self
            .entries
            .iter()
            .min_by_key(|(_, v)| v.last_accessed)
            .map(|(k, _)| k.clone());

        if let Some(key) = lru_key {
            tracing::debug!("Evicting LRU embedding: {}", &key[..8]);
            self.entries.remove(&key);
        }
    }

    /// Invalidate a specific hash in the cache
    pub fn invalidate(&mut self, doc_hash: &str) {
        if self.entries.remove(doc_hash).is_some() {
            tracing::debug!("Invalidated embedding for: {}", &doc_hash[..8]);
            self.dirty = true;
        }
    }

    /// Check if a hash is cached
    pub fn contains(&self, doc_hash: &str) -> bool {
        self.entries.contains_key(doc_hash)
    }

    /// Clear all cache entries
    pub fn clear(&mut self) {
        self.entries.clear();
        self.dirty = true;
        tracing::info!("Embedding cache cleared");
    }

    /// Get cache statistics
    pub fn stats(&self) -> EmbeddingCacheStats {
        EmbeddingCacheStats {
            entries: self.entries.len(),
            max_entries: self.max_entries,
            hits: self.hits,
            misses: self.misses,
            hit_rate: if self.hits + self.misses > 0 {
                (self.hits as f64) / ((self.hits + self.misses) as f64)
            } else {
                0.0
            },
        }
    }
}

/// Embedding cache statistics
#[derive(Debug, Clone)]
pub struct EmbeddingCacheStats {
    pub entries: usize,
    pub max_entries: usize,
    pub hits: usize,
    pub misses: usize,
    pub hit_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_new() {
        let temp = assert_fs::TempDir::new().unwrap();
        let cache = EmbeddingCache::new(temp.path(), 100).unwrap();
        assert_eq!(cache.entries.len(), 0);
        assert_eq!(cache.max_entries, 100);
    }

    #[test]
    fn test_cache_hit_miss() {
        let temp = assert_fs::TempDir::new().unwrap();
        let mut cache = EmbeddingCache::new(temp.path(), 100).unwrap();

        // First access = miss
        let hash = "abc123def456789012345678901234567890123456789012345678901234";
        let embedding = cache.get_or_compute(hash, "test content").unwrap();
        assert_eq!(cache.stats().misses, 1);
        assert_eq!(cache.stats().hits, 0);

        // Second access = hit
        let embedding2 = cache.get_or_compute(hash, "test content").unwrap();
        assert_eq!(cache.stats().hits, 1);
        assert_eq!(embedding, embedding2);
    }

    #[test]
    fn test_cache_persistence() {
        let temp = assert_fs::TempDir::new().unwrap();
        let hash = "abc123def456789012345678901234567890123456789012345678901234";

        // First instance - populate cache
        {
            let mut cache = EmbeddingCache::new(temp.path(), 100).unwrap();
            cache.load().unwrap();
            let _ = cache.get_or_compute(hash, "test content").unwrap();
            cache.save().unwrap();
        }

        // Second instance - should load from disk
        {
            let mut cache = EmbeddingCache::new(temp.path(), 100).unwrap();
            cache.load().unwrap();
            assert!(cache.contains(hash));
            
            // Access should be a hit
            let _ = cache.get_or_compute(hash, "test content").unwrap();
            assert_eq!(cache.stats().hits, 1);
            assert_eq!(cache.stats().misses, 0);
        }
    }

    #[test]
    fn test_lru_eviction() {
        let temp = assert_fs::TempDir::new().unwrap();
        let mut cache = EmbeddingCache::new(temp.path(), 3).unwrap(); // Max 3 entries

        // Insert 3 entries
        for i in 0..3 {
            let hash = format!("{:064}", i); // 64 char hash
            let _ = cache.get_or_compute(&hash, &format!("content {}", i)).unwrap();
        }
        assert_eq!(cache.entries.len(), 3);

        // Access first entry to make it recently used
        let first_hash = format!("{:064}", 0);
        let _ = cache.get_or_compute(&first_hash, "content 0").unwrap();

        // Insert 4th entry - should evict entry 1 (oldest unused)
        let new_hash = format!("{:064}", 99);
        let _ = cache.get_or_compute(&new_hash, "content new").unwrap();
        
        assert_eq!(cache.entries.len(), 3);
        assert!(cache.contains(&first_hash)); // Still there (recently used)
        assert!(cache.contains(&new_hash));   // New entry
        // Entry 1 should be evicted (oldest)
    }

    #[test]
    fn test_invalidation() {
        let temp = assert_fs::TempDir::new().unwrap();
        let mut cache = EmbeddingCache::new(temp.path(), 100).unwrap();
        
        let hash = "abc123def456789012345678901234567890123456789012345678901234";
        let _ = cache.get_or_compute(hash, "test content").unwrap();
        assert!(cache.contains(hash));
        
        cache.invalidate(hash);
        assert!(!cache.contains(hash));
    }
}
