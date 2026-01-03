use crate::cache::DocumentCache;
use crate::db::{DbManager, migration};
use crate::graph::DependencyGraph;
use cue_common::Result;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use cue_config::Config;

/// Stateful engine for managing workspace state, caching, and graph resolution.
///
/// Designed for efficient incremental updates (watch mode).
pub struct CueEngine {
    workspace_root: PathBuf,
    cache: DocumentCache,
    graph: DependencyGraph,
    // Keep track of active keys to handle deletions effectively during full scan
    known_files: HashSet<PathBuf>,
    config: Config,
    db: Option<DbManager>,  // Optional SQLite backend (hybrid architecture)
}

impl CueEngine {
    /// Initialize a new engine
    pub fn new(workspace_root: &Path) -> Result<Self> {
        let mut cache =
            DocumentCache::new(workspace_root).map_err(|e| std::io::Error::other(e.to_string()))?;
        cache
            .load()
            .map_err(|e| std::io::Error::other(e.to_string()))?;

        // Phase 7.3: Check and perform migration if needed
        let db = if let Ok(true) = migration::needs_migration(workspace_root) {
            tracing::info!("Migration needed. Starting migration from JSON to SQLite...");
            match migration::migrate_json_to_sqlite(workspace_root, &cache) {
                Ok(count) => {
                    tracing::info!("Migration completed successfully ({} files).", count);
                    // Open DB after successful migration
                    let db_path = workspace_root.join(".cue/metadata.db");
                    match DbManager::open(&db_path) {
                        Ok(db) => Some(db),
                        Err(e) => {
                            tracing::error!("Failed to open database after migration: {}", e);
                            None
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Migration failed: {}. Falling back to JSON-only mode.", e);
                    None
                }
            }
        } else {
            // Try to open existing DB if available
            let db_path = workspace_root.join(".cue/metadata.db");
            if db_path.exists() {
                 match DbManager::open(&db_path) {
                    Ok(db) => Some(db),
                    Err(e) => {
                        tracing::error!("Failed to open existing database: {}. Falling back to JSON-only.", e);
                        None
                    }
                }
            } else {
                // First run or JSON-only mode
                // If we want to start using DB from scratch for new projects, we can init it here.
                // For Phase 7.3, we enable it by default for everyone.
                let dot_cue = workspace_root.join(".cue");
                if !dot_cue.exists() {
                    std::fs::create_dir_all(&dot_cue)?;
                }
                match DbManager::open(&db_path) {
                    Ok(db) => Some(db),
                    Err(e) => {
                         tracing::warn!("Failed to initialize new database: {}", e);
                         None
                    }
                }
            }
        };

        // Load config
        let config = Config::load(workspace_root)?;

        // Initialize empty graph
        let graph = DependencyGraph::build(&[])?;

        let mut engine = Self {
            workspace_root: workspace_root.to_path_buf(),
            cache,
            graph,
            known_files: HashSet::new(),
            config,
            db,
        };

        // Initial full scan
        engine.scan_all()?;

        Ok(engine)
    }

    /// Scan all files in workspace and build initial graph
    pub fn scan_all(&mut self) -> Result<()> {
        tracing::info!("Scanning workspace: {:?}", self.workspace_root);

        let cards_dir = self.workspace_root.join(".cuedeck/cards");
        let docs_dir = self.workspace_root.join(".cuedeck/docs");

        let mut paths = Vec::new();

        if cards_dir.exists() {
            for entry in walkdir::WalkDir::new(&cards_dir)
                .follow_links(true)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.file_type().is_file()
                    && entry.path().extension().is_some_and(|e| e == "md")
                {
                    paths.push(entry.path().to_path_buf());
                }
            }
        }

        if docs_dir.exists() {
            for entry in walkdir::WalkDir::new(&docs_dir)
                .follow_links(true)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.file_type().is_file()
                    && entry.path().extension().is_some_and(|e| e == "md")
                {
                    paths.push(entry.path().to_path_buf());
                }
            }
        }

        // Identify deleted files
        let current_files: HashSet<_> = paths.iter().cloned().collect();
        let deleted_files: Vec<_> = self
            .known_files
            .difference(&current_files)
            .cloned()
            .collect();

        for path in deleted_files {
            self.remove_file(&path);
        }

        self.known_files = current_files;

        // Update/Add files
        for path in paths {
            if let Err(e) = self.update_file(&path) {
                tracing::warn!("Failed to update file {:?}: {}", path, e);
            }
        }

        // Save cache
        self.cache
            .save()
            .map_err(|e| std::io::Error::other(e.to_string()))?;

        Ok(())
    }

    /// Handle file system event (update/add)
    pub fn update_file(&mut self, path: &Path) -> Result<()> {
        match self.cache.get_or_parse(path) {
            Ok(doc) => {
                // Phase 7.3: Sync to SQLite if available
                if let Some(db) = &self.db {
                    let size_bytes = std::fs::metadata(path)
                        .map(|m| m.len())
                        .unwrap_or(0);
                    
                    if let Err(e) = db.upsert_file(path, &doc.hash, size_bytes, doc.tokens) {
                        tracing::warn!("Failed to sync file to DB {:?}: {}", path, e);
                    }
                }

                self.graph.add_or_update_document(&doc);
                self.known_files.insert(path.to_path_buf());
                Ok(())
            }
            Err(e) => {
                // If parsing fails (e.g. file is temporarily locked or invalid),
                // we should probably just warn and not crash
                // But we return error here to let caller decide
                Err(std::io::Error::other(format!("Failed to parse {:?}: {}", path, e)).into())
            }
        }
    }

    /// Handle file deletion
    pub fn remove_file(&mut self, path: &Path) {
        // Phase 7.3: Sync deletion to SQLite if available
        if let Some(db) = &self.db {
            if let Err(e) = db.delete_file(path) {
                tracing::warn!("Failed to delete file from DB {:?}: {}", path, e);
            }
        }

        self.cache.invalidate(path);
        self.graph.remove_document(&path.to_path_buf());
        self.known_files.remove(path);
    }

    /// Render the scene
    pub fn render(&self) -> Result<String> {
        // Topological sort
        let sorted_paths = self.graph.sort_topological()?;

        let mut scene = String::from("# Scene Context\n\n");
        scene.push_str(&format!("Generated: {}\n", chrono::Utc::now().to_rfc3339()));

        // Count documents
        let total_docs = sorted_paths
            .iter()
            .filter(|p| self.cache.get(p).is_some())
            .count();
        scene.push_str(&format!("Documents: {}\n\n", total_docs));

        let mut total_tokens = 0;

        for path in sorted_paths {
            if let Some(doc) = self.cache.get(&path) {
                if total_tokens + doc.tokens > self.config.budgets.feature {
                    tracing::warn!("Token limit reached, truncating scene");
                    break;
                }

                scene.push_str(&format!("## {}\n\n", path.display()));
                scene.push_str(&format!(
                    "Tokens: {} | Hash: {}\n\n",
                    doc.tokens,
                    &doc.hash[..8]
                ));

                // Add anchors
                if !doc.anchors.is_empty() {
                    scene.push_str("### Anchors\n\n");
                    for anchor in &doc.anchors {
                        scene.push_str(&format!(
                            "- {} (L{}): {}\n",
                            "#".repeat(anchor.level as usize),
                            anchor.start_line,
                            anchor.header
                        ));
                    }
                    scene.push('\n');
                }

                // Read content
                if let Ok(content) = std::fs::read_to_string(path) {
                    scene.push_str(&content);
                    scene.push_str("\n\n");
                }

                total_tokens += doc.tokens;
            }
        }

        scene.push_str(&format!("\n---\nTotal Tokens: {}\n", total_tokens));

        Ok(scene)
    }

    /// Get reference to internal cache
    pub fn cache(&self) -> &DocumentCache {
        &self.cache
    }

    /// Get reference to internal graph
    pub fn graph(&self) -> &DependencyGraph {
        &self.graph
    }
}
