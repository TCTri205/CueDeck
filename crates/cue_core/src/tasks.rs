use crate::parse_file;
use cue_common::{CueError, Document, Result};
use std::path::{Path, PathBuf};
use std::fs;

/// List all task cards in the workspace, optionally filtering
#[tracing::instrument(skip(workspace_root))]
pub fn list_tasks(workspace_root: &Path, status_filter: Option<&str>, assignee_filter: Option<&str>) -> Result<Vec<Document>> {
    let cards_dir = workspace_root.join(".cuedeck/cards");
    let mut tasks = Vec::new();

    if !cards_dir.exists() {
        return Ok(tasks);
    }

    for entry in walkdir::WalkDir::new(&cards_dir)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok()) 
    {
        if entry.file_type().is_file() && entry.path().extension().is_some_and(|e| e == "md") {
            // Parse the file
            match parse_file(entry.path()) {
                Ok(doc) => {
                    // Filter logic
                    if let Some(meta) = &doc.frontmatter {
                        if let Some(status) = status_filter {
                            if meta.status != status {
                                continue;
                            }
                        }
                        if let Some(assignee) = assignee_filter {
                            if meta.assignee.as_deref() != Some(assignee) {
                                continue;
                            }
                        }
                        tasks.push(doc);
                    }
                },
                Err(e) => tracing::warn!("Failed to parse card {:?}: {}", entry.path(), e),
            }
        }
    }

    // Sort by created date (newest first)? Or Priority?
    // Let's sort by priority then created.
    // For now simple sort.
    tasks.sort_by(|a, b| {
        let p_a = a.frontmatter.as_ref().map(|m| priority_score(&m.priority)).unwrap_or(0);
        let p_b = b.frontmatter.as_ref().map(|m| priority_score(&m.priority)).unwrap_or(0);
        p_b.cmp(&p_a) // Higher priority first
    });

    Ok(tasks)
}

fn priority_score(p: &str) -> i32 {
    match p.to_lowercase().as_str() {
        "critical" => 4,
        "high" => 3,
        "medium" => 2,
        "low" => 1,
        _ => 0,
    }
}

/// Create a new task card
pub fn create_task(workspace_root: &Path, title: &str) -> Result<PathBuf> {
    use rand::Rng;
    
    let id: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(6)
        .map(char::from)
        .collect::<String>()
        .to_lowercase();
        
    let filename = workspace_root.join(".cuedeck/cards").join(format!("{}.md", id));
    
    // Ensure dir exists
    if let Some(parent) = filename.parent() {
        fs::create_dir_all(parent)?;
    }
    
    let template = format!(r#"---
title: "{}"
status: todo
assignee: ""
priority: medium
created: {}
---

# {}

## Description

[Add description]
"#, title, chrono::Utc::now().to_rfc3339(), title);

    fs::write(&filename, template)?;
    Ok(filename)
}

/// Update a task's metadata
pub fn update_task(workspace_root: &Path, id: &str, updates: serde_json::Map<String, serde_json::Value>) -> Result<Document> {
    let path = workspace_root.join(".cuedeck/cards").join(format!("{}.md", id));
    
    if !path.exists() {
        return Err(CueError::FileNotFound { path: path.to_string_lossy().to_string() });
    }
    
    // Read and Parse
    // usage of parse_file is good for reading, but writing back requires manipulating raw content
    // to preserve body while updating frontmatter.
    
    let content = fs::read_to_string(&path)?;
    let frontmatter_regex = regex::Regex::new(r"(?ms)^---\r?\n(.*?)\r?\n---").unwrap();
    
    if let Some(captures) = frontmatter_regex.captures(&content) {
        let yaml_str = captures.get(1).unwrap().as_str();
        let mut meta: serde_yaml::Value = serde_yaml::from_str(yaml_str)
            .map_err(|e| CueError::ParseError(e.to_string()))?;
            
        // Apply updates
        if let serde_yaml::Value::Mapping(ref mut map) = meta {
            for (k, v) in updates {
                // Convert JSON value to YAML value (basic types)
                let yaml_v = match v {
                    serde_json::Value::String(s) => serde_yaml::Value::String(s),
                    serde_json::Value::Bool(b) => serde_yaml::Value::Bool(b),
                    serde_json::Value::Number(n) => {
                        if let Some(i) = n.as_i64() { serde_yaml::Value::Number(i.into()) }
                        else if let Some(f) = n.as_f64() { serde_yaml::Value::Number(f.into()) }
                        else { continue }
                    },
                    serde_json::Value::Null => serde_yaml::Value::Null,
                    _ => continue, // Skip arrays/objects for now
                };
                map.insert(serde_yaml::Value::String(k), yaml_v);
            }
        }
        
        let new_yaml = serde_yaml::to_string(&meta)
            .map_err(|e| CueError::ParseError(e.to_string()))?;
            
        // Reconstruct content
        let body_start = captures.get(0).unwrap().end();
        let new_content = format!("---\n{}---{}", new_yaml.trim(), &content[body_start..]);
        
        fs::write(&path, new_content)?;
        
        // Return updated doc
        parse_file(&path)
    } else {
        Err(CueError::ParseError("No frontmatter found in card".to_string()))
    }
}
