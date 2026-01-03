/// Parse a single markdown file into a Document
///
/// Use this for parallel processing or when cache management is handled externally.
pub fn parse_file(path: &Path) -> Result<Document> {
    let _content = fs::read_to_string(path).with_context(|| format!("Failed to read file {:?}", path))?;
    
    // Parse using internal logic
    // Note: We use the existing logic but just moved to a pub function
    // For now we just call the private one if we can, or just reimplement/expose it.
    Ok(crate::parse_file(path)?)
}

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use crate::Document;

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
    pub(crate) entries: HashMap<PathBuf, CachedDocument>,
    /// In-memory hash cache for fast validation (Phase 7 optimization)
    hash_cache: HashMap<PathBuf, String>,
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
        fs::create_dir_all(&cache_dir).context("Failed to create cache directory")?;

        Ok(Self {
            cache_dir,
            entries: HashMap::new(),
            hash_cache: HashMap::new(),
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
                        
                        // Rebuild hash_cache from entries for fast validation (Phase 7 optimization)
                        self.hash_cache = entries
                            .iter()
                            .map(|(path, cached)| (path.clone(), cached.hash.clone()))
                            .collect();
                        
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

        let data = bincode::serialize(&self.entries).context("Failed to serialize cache")?;

        fs::write(&cache_file, data).context("Failed to write cache file")?;

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
        // Fast path: Check in-memory hash cache first
        let metadata = fs::metadata(path)
            .with_context(|| format!("Failed to read metadata for {:?}", path))?;
        let modified = metadata.modified().context("Failed to get modified time")?;

        // Check if we have a cached entry with matching modified time
        if let Some(cached) = self.entries.get(path) {
            if cached.modified == modified {
                // Check in-memory hash cache to avoid re-hashing
                if let Some(cached_hash) = self.hash_cache.get(path) {
                    if cached_hash == &cached.hash {
                        tracing::trace!("Cache HIT (fast path): {:?}", path);
                        self.hits += 1;
                        return Ok(cached.document.clone());
                    }
                }
            }
        }

        // Slower path: Compute hash if modified time changed or hash not in cache
        let content = fs::read(path).with_context(|| format!("Failed to read file {:?}", path))?;
        let mut hasher = Sha256::new();
        hasher.update(&content);
        let current_hash = format!("{:x}", hasher.finalize());

        // Update in-memory hash cache
        self.hash_cache.insert(path.to_path_buf(), current_hash.clone());

        // Check if document is still cached with this hash
        if let Some(cached) = self.entries.get(path) {
            if cached.hash == current_hash {
                tracing::trace!("Cache HIT (hash match): {:?}", path);
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
            self.hash_cache.remove(path);  // Also remove from hash cache
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
        self.hash_cache.clear();  // Also clear hash cache
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

    /// Check if a file needs to be updated (Phase 7.x: Parallel Parsing)
    ///
    /// # Arguments
    /// * `path` - Path to the file
    /// * `current_hash` - Current SHA256 hash of the file
    /// * `modified` - Current modification time
    ///
    /// # Returns
    /// `true` if file needs parsing, `false` if cached version is valid
    pub fn needs_update(&self, path: &Path, current_hash: &str, modified: SystemTime) -> bool {
        match self.entries.get(path) {
            Some(cached) => {
                // Check if modified time or hash changed
                cached.modified != modified || cached.hash != current_hash
            }
            None => true, // Not in cache, needs parsing
        }
    }

    /// Insert a parsed document into the cache (Phase 7.x: Parallel Parsing)
    ///
    /// # Arguments
    /// * `path` - Path to the file
    /// * `document` - Parsed document
    /// * `hash` - SHA256 hash of the file content
    /// * `modified` - File modification time
    pub fn insert(&mut self, path: PathBuf, document: Document, hash: String, modified: SystemTime) {
        self.hash_cache.insert(path.clone(), hash.clone());
        self.entries.insert(
            path,
            CachedDocument {
                hash,
                modified,
                document,
            },
        );
    }

    /// Batch insert multiple documents (Phase 7.x: Parallel Parsing)
    ///
    /// This is more efficient than calling `insert` multiple times as it
    /// avoids repeated allocations.
    ///
    /// # Arguments
    /// * `documents` - Iterator of (path, document, hash, modified) tuples
    pub fn insert_batch<I>(&mut self, documents: I)
    where
        I: IntoIterator<Item = (PathBuf, Document, String, SystemTime)>,
    {
        for (path, document, hash, modified) in documents {
            self.insert(path, document, hash, modified);
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
    use std::fs;
    use std::time::SystemTime;
    use sha2::{Sha256, Digest};
    
    // Use the crate-level parse_file
    use crate::parse_file; 

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

    #[test]
    fn test_needs_update() {
        let temp = assert_fs::TempDir::new().unwrap();
        let file_path = temp.child("test_needs_update.md");
        file_path.write_str("# Original Content").unwrap();

        let mut cache = DocumentCache::new(temp.path()).unwrap();

        // Helper to get file hash and modified time
        let get_file_info = |path: &Path| -> (String, SystemTime) {
            let content = fs::read(path).unwrap();
            let mut hasher = Sha256::new();
            hasher.update(&content);
            let hash = format!("{:x}", hasher.finalize());
            let modified = fs::metadata(path).unwrap().modified().unwrap();
            (hash, modified)
        };

        let (initial_hash, initial_modified) = get_file_info(file_path.path());

        // 1. Not in cache, should need update
        assert!(cache.needs_update(file_path.path(), &initial_hash, initial_modified));

        // Insert into cache
        let document = parse_file(file_path.path()).unwrap();
        cache.insert(file_path.path().to_path_buf(), document, initial_hash.clone(), initial_modified);

        // 2. In cache, same hash and modified time, should NOT need update
        assert!(!cache.needs_update(file_path.path(), &initial_hash, initial_modified));

        // Modify file content
        std::thread::sleep(std::time::Duration::from_millis(10)); // Ensure modified time changes
        file_path.write_str("# Updated Content").unwrap();
        let (updated_hash, updated_modified) = get_file_info(file_path.path());

        // 3. In cache, but content changed (different hash), should need update
        assert!(cache.needs_update(file_path.path(), &updated_hash, updated_modified));

        // 4. In cache, but modified time changed, should need update
        assert!(cache.needs_update(file_path.path(), &initial_hash, updated_modified));
    }

    #[test]
    fn test_insert() {
        let temp = assert_fs::TempDir::new().unwrap();
        let file_path = temp.child("test_insert.md");
        file_path.write_str("# Hello").unwrap();

        let mut cache = DocumentCache::new(temp.path()).unwrap();

        let content = fs::read(file_path.path()).unwrap();
        let mut hasher = Sha256::new();
        hasher.update(&content);
        let hash = format!("{:x}", hasher.finalize());
        let modified = fs::metadata(file_path.path()).unwrap().modified().unwrap();
        let document = parse_file(file_path.path()).unwrap();

        assert_eq!(cache.entries.len(), 0);
        assert_eq!(cache.hash_cache.len(), 0);

        cache.insert(file_path.path().to_path_buf(), document.clone(), hash.clone(), modified);

        assert_eq!(cache.entries.len(), 1);
        assert_eq!(cache.hash_cache.len(), 1);
        assert!(cache.entries.contains_key(file_path.path()));
        // verify hash instead of content
        assert_eq!(cache.get(file_path.path()).unwrap().hash, hash);
        assert_eq!(cache.hash_cache.get(file_path.path()).unwrap(), &hash);
    }

    #[test]
    fn test_insert_batch() {
        let temp = assert_fs::TempDir::new().unwrap();
        let mut cache = DocumentCache::new(temp.path()).unwrap();

        // Create test files
        let files: Vec<_> = (0..5)
            .map(|i| {
                let file = temp.child(format!("test_{}.md", i));
                file.write_str(&format!("# Test {}", i)).unwrap();
                file
            })
            .collect();

        // Parse files and prepare batch
        let batch: Vec<_> = files
            .iter()
            .map(|file| {
                let doc = parse_file(file.path()).unwrap();
                let content = fs::read(file.path()).unwrap();
                let mut hasher = Sha256::new();
                hasher.update(&content);
                let hash = format!("{:x}", hasher.finalize());
                let metadata = fs::metadata(file.path()).unwrap();
                let modified = metadata.modified().unwrap();
                (file.to_path_buf(), doc, hash, modified)
            })
            .collect();

        assert_eq!(cache.entries.len(), 0);

        // Batch insert
        cache.insert_batch(batch);

        // Verify all inserted
        assert_eq!(cache.entries.len(), 5);
        assert_eq!(cache.hash_cache.len(), 5);
        for file in &files {
            assert!(cache.get(file.path()).is_some());
        }

        // Verify content of one of the documents
        let doc1_cached = cache.get(files[0].path()).unwrap();
        let doc1_original = parse_file(files[0].path()).unwrap();
        assert_eq!(doc1_cached.hash, doc1_original.hash);
    }

    #[test]
    fn test_cache_stats() {
        // Phase 7.4: Verify cache hit/miss tracking and hit rate calculation
        let temp = assert_fs::TempDir::new().unwrap();
        let mut cache = DocumentCache::new(temp.path()).unwrap();

        // Create test files
        let files: Vec<_> = (0..10)
            .map(|i| {
                let file = temp.child(format!("stats_test_{}.md", i));
                file.write_str(&format!("---\ntitle: Test {}\n---\n\n# Test {}\n\nContent {}", i, i, i))
                    .unwrap();
                file
            })
            .collect();

        // Initial stats should be zero
        let initial_stats = cache.stats();
        assert_eq!(initial_stats.entries, 0);
        assert_eq!(initial_stats.hits, 0);
        assert_eq!(initial_stats.misses, 0);
        assert_eq!(initial_stats.hit_rate, 0.0);

        // First access - should all be misses
        for file in &files {
            let _doc = cache.get_or_parse(file.path()).unwrap();
        }

        let stats_after_first = cache.stats();
        assert_eq!(stats_after_first.entries, 10, "Should have 10 cached entries");
        assert_eq!(stats_after_first.hits, 0, "First access should be all misses");
        assert_eq!(stats_after_first.misses, 10, "Should have 10 misses");
        assert_eq!(stats_after_first.hit_rate, 0.0, "Hit rate should be 0%");

        // Second access - should all be hits (warm cache)
        for file in &files {
            let _doc = cache.get_or_parse(file.path()).unwrap();
        }

        let stats_after_second = cache.stats();
        assert_eq!(stats_after_second.entries, 10);
        assert_eq!(stats_after_second.hits, 10, "Second access should all hit cache");
        assert_eq!(stats_after_second.misses, 10, "Misses should still be 10");
        assert_eq!(stats_after_second.hit_rate, 0.5, "Hit rate should be 50% (10/20)");

        // Third access - should be 20 hits total
        for file in &files {
            let _doc = cache.get_or_parse(file.path()).unwrap();
        }

        let stats_after_third = cache.stats();
        assert_eq!(stats_after_third.hits, 20, "Third access should add 10 more hits");
        assert_eq!(stats_after_third.misses, 10);
        assert_eq!(
            (stats_after_third.hit_rate * 100.0).round() / 100.0,
            0.67,
            "Hit rate should be ~66.67% (20/30)"
        );

        // Modify one file - should cause 1 miss
        std::thread::sleep(std::time::Duration::from_millis(10));
        files[0].write_str("# Modified\n\nNew content").unwrap();
        let _doc = cache.get_or_parse(files[0].path()).unwrap();

        let stats_after_modify = cache.stats();
        assert_eq!(stats_after_modify.hits, 20);
        assert_eq!(stats_after_modify.misses, 11, "Modified file should cause a miss");
        assert_eq!(
            (stats_after_modify.hit_rate * 100.0).round() / 100.0,
            0.65,
            "Hit rate should be ~64.5% (20/31)"
        );

        // Phase 7 Exit Criteria: Verify hit rate > 90% on warm cache scenario
        // Access all files 9 more times (all should hit)
        for _ in 0..9 {
            for file in &files {
                let _doc = cache.get_or_parse(file.path()).unwrap();
            }
        }

        let final_stats = cache.stats();
        assert_eq!(final_stats.hits, 110); // 20 + (9 * 10) = 110
        assert_eq!(final_stats.misses, 11); // Original 10 + 1 modified
        
        let final_hit_rate = final_stats.hit_rate;
        assert!(
            final_hit_rate > 0.90,
            "Phase 7 Exit Criteria: Hit rate should be > 90%, got {:.2}%",
            final_hit_rate * 100.0
        );

        println!(
            "✅ Cache statistics test passed: {}/{} hits ({:.2}% hit rate)",
            final_stats.hits,
            final_stats.hits + final_stats.misses,
            final_hit_rate * 100.0
        );
    }
}
