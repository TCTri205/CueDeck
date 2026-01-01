//! Document cache module for CueDeck
//!
//! Provides persistent caching of parsed documents using SHA256 hashing
//! for invalidation and bincode for efficient serialization.

use crate::{Document, parse_file};
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Cached document entry with hash and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedDocument {
    /// SHA256 hash of file content
    pub hash: String,
    /// Last modified time (for quick invalidation check)
    pub modified: SystemTime,
    /// Parsed document
    pub document: Document,
}

/// Document cache manager
#[derive(Debug)]
pub struct DocumentCache {
    /// Cache directory path (.cuedeck/cache/)
    cache_dir: PathBuf,
    /// In-memory cache entries
    entries: HashMap<PathBuf, CachedDocument>,
    /// Statistics
    hits: usize,
    misses: usize,
}

impl DocumentCache {
    /// Create a new cache instance
    ///
    /// # Arguments
    /// * `workspace_root` - Root directory of the workspace
    pub fn new(workspace_root: &Path) -> Result<Self> {
        let cache_dir = workspace_root.join(".cuedeck/cache");
        
        // Create cache directory if it doesn't exist
        fs::create_dir_all(&cache_dir)
            .context("Failed to create cache directory")?;
        
        Ok(Self {
            cache_dir,
            entries: HashMap::new(),
            hits: 0,
            misses: 0,
        })
    }
    
    /// Load cache from disk
    pub fn load(&mut self) -> Result<()> {
        tracing::info!("Loading cache from {:?}", self.cache_dir);
        
        let cache_file = self.cache_dir.join("documents.bin");
        
        if !cache_file.exists() {
            tracing::debug!("No cache file found, starting fresh");
            return Ok(());
        }
        
        match fs::read(&cache_file) {
            Ok(data) => {
                match bincode::deserialize::<HashMap<PathBuf, CachedDocument>>(&data) {
                    Ok(entries) => {
                        tracing::info!("Loaded {} cached documents", entries.len());
                        self.entries = entries;
                        Ok(())
                    }
                    Err(e) => {
                        tracing::warn!("Failed to deserialize cache: {}", e);
                        tracing::info!("Cache corruption detected, rebuilding...");
                        // Auto-repair: delete corrupted cache
                        let _ = fs::remove_file(&cache_file);
                        Ok(())
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to read cache file: {}", e);
                Ok(())
            }
        }
    }
    
    /// Save cache to disk
    pub fn save(&self) -> Result<()> {
        tracing::debug!("Saving cache with {} entries", self.entries.len());
        
        let cache_file = self.cache_dir.join("documents.bin");
        
        let data = bincode::serialize(&self.entries)
            .context("Failed to serialize cache")?;
        
        fs::write(&cache_file, data)
            .context("Failed to write cache file")?;
        
        tracing::info!("Cache saved successfully");
        Ok(())
    }
    
    /// Get a document from cache or parse it
    ///
    /// # Arguments
    /// * `path` - Path to the markdown file
    ///
    /// # Returns
    /// The parsed document (from cache if valid, otherwise freshly parsed)
    pub fn get_or_parse(&mut self, path: &Path) -> Result<Document> {
        // Compute current file hash and modified time
        let metadata = fs::metadata(path)
            .with_context(|| format!("Failed to read metadata for {:?}", path))?;
        
        let modified = metadata.modified()
            .context("Failed to get modified time")?;
        
        let content = fs::read(path)
            .with_context(|| format!("Failed to read file {:?}", path))?;
        
        let mut hasher = Sha256::new();
        hasher.update(&content);
        let current_hash = format!("{:x}", hasher.finalize());
        
        // Check cache
        if let Some(cached) = self.entries.get(path) {
            // Quick check: modified time
            if cached.modified == modified && cached.hash == current_hash {
                tracing::trace!("Cache HIT: {:?}", path);
                self.hits += 1;
                return Ok(cached.document.clone());
            }
        }
        
        // Cache miss - parse file
        tracing::trace!("Cache MISS: {:?}", path);
        self.misses += 1;
        
        let document = parse_file(path)?;
        
        // Update cache entry
        self.entries.insert(
            path.to_path_buf(),
            CachedDocument {
                hash: current_hash,
                modified,
                document: document.clone(),
            },
        );
        
        Ok(document)
    }
    
    /// Invalidate a specific file in the cache
    pub fn invalidate(&mut self, path: &Path) {
        if self.entries.remove(path).is_some() {
            tracing::debug!("Invalidated cache for {:?}", path);
        }
    }
    
    /// Get a cached document if it exists and is valid
    pub fn get(&self, path: &Path) -> Option<&Document> {
        self.entries.get(path).map(|entry| &entry.document)
    }
    
    /// Clear all cache entries
    pub fn clear(&mut self) {
        self.entries.clear();
        tracing::info!("Cache cleared");
    }
    
    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            entries: self.entries.len(),
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

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub entries: usize,
    pub hits: usize,
    pub misses: usize,
    pub hit_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;
    
    #[test]
    fn test_cache_new() {
        let temp = assert_fs::TempDir::new().unwrap();
        let cache = DocumentCache::new(temp.path()).unwrap();
        assert_eq!(cache.entries.len(), 0);
    }
    
    #[test]
    fn test_cache_persistence() {
        let temp = assert_fs::TempDir::new().unwrap();
        
        // Create a test markdown file
        let file = temp.child("test.md");
        file.write_str("# Test\n\nContent").unwrap();
        
        // First cache instance
        {
            let mut cache = DocumentCache::new(temp.path()).unwrap();
            cache.load().unwrap();
            
            let _doc = cache.get_or_parse(file.path()).unwrap();
            assert_eq!(cache.stats().misses, 1);
            
            cache.save().unwrap();
        }
        
        // Second cache instance - should hit cache
        {
            let mut cache = DocumentCache::new(temp.path()).unwrap();
            cache.load().unwrap();
            
            let _doc = cache.get_or_parse(file.path()).unwrap();
            assert_eq!(cache.stats().hits, 1);
        }
    }
    
    #[test]
    fn test_cache_invalidation() {
        let temp = assert_fs::TempDir::new().unwrap();
        let file = temp.child("test.md");
        file.write_str("# Test").unwrap();
        
        let mut cache = DocumentCache::new(temp.path()).unwrap();
        
        // First access
        let _doc1 = cache.get_or_parse(file.path()).unwrap();
        assert_eq!(cache.stats().misses, 1);
        
        // Modify file
        std::thread::sleep(std::time::Duration::from_millis(10));
        file.write_str("# Modified").unwrap();
        
        // Should detect change
        let _doc2 = cache.get_or_parse(file.path()).unwrap();
        assert_eq!(cache.stats().misses, 2);
    }
}
