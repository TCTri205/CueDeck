use std::path::{Path, PathBuf};
use std::collections::HashSet;
use cue_common::Result;
use crate::cache::DocumentCache;
use crate::graph::DependencyGraph;

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
}

impl CueEngine {
    /// Initialize a new engine
    pub fn new(workspace_root: &Path) -> Result<Self> {
        let mut cache = DocumentCache::new(workspace_root)
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        cache.load()
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        
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
                if entry.file_type().is_file() && entry.path().extension().is_some_and(|e| e == "md") {
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
                if entry.file_type().is_file() && entry.path().extension().is_some_and(|e| e == "md") {
                    paths.push(entry.path().to_path_buf());
                }
            }
        }
        
        // Identify deleted files
        let current_files: HashSet<_> = paths.iter().cloned().collect();
        let deleted_files: Vec<_> = self.known_files.difference(&current_files).cloned().collect();
        
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
        self.cache.save().map_err(|e| std::io::Error::other(e.to_string()))?;
        
        Ok(())
    }
    
    /// Handle file system event (update/add)
    pub fn update_file(&mut self, path: &Path) -> Result<()> {
        match self.cache.get_or_parse(path) {
            Ok(doc) => {
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
        let total_docs = sorted_paths.iter().filter(|p| self.cache.get(p).is_some()).count();
        scene.push_str(&format!("Documents: {}\n\n", total_docs));
        
        let mut total_tokens = 0;
        
        for path in sorted_paths {
            if let Some(doc) = self.cache.get(&path) {
                if total_tokens + doc.tokens > self.config.budgets.feature {
                    tracing::warn!("Token limit reached, truncating scene");
                    break;
                }
                
                scene.push_str(&format!("## {}\n\n", path.display()));
                scene.push_str(&format!("Tokens: {} | Hash: {}\n\n", doc.tokens, &doc.hash[..8]));
                
                // Add anchors
                 if !doc.anchors.is_empty() {
                    scene.push_str("### Anchors\n\n");
                    for anchor in &doc.anchors {
                        scene.push_str(&format!("- {} (L{}): {}\n", 
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
