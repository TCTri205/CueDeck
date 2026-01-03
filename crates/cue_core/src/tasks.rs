use crate::parse_file;
use crate::task_filters::{matches_date_filter, matches_date_filter_mtime, matches_tag_filter, TaskFilters};
use cue_common::{CueError, Document, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// List all task cards in the workspace, optionally filtering
#[tracing::instrument(skip(workspace_root))]
pub fn list_tasks(
    workspace_root: &Path,
    status_filter: Option<&str>,
    assignee_filter: Option<&str>,
) -> Result<Vec<Document>> {
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
                }
                Err(e) => tracing::warn!("Failed to parse card {:?}: {}", entry.path(), e),
            }
        }
    }

    // Sort by created date (newest first)? Or Priority?
    // Let's sort by priority then created.
    // For now simple sort.
    tasks.sort_by(|a, b| {
        let p_a = a
            .frontmatter
            .as_ref()
            .map(|m| priority_score(&m.priority))
            .unwrap_or(0);
        let p_b = b
            .frontmatter
            .as_ref()
            .map(|m| priority_score(&m.priority))
            .unwrap_or(0);
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
    create_task_with_metadata(workspace_root, title, None, None, None, None)
}

/// Create a new task card with metadata
pub fn create_task_with_metadata(
    workspace_root: &Path,
    title: &str,
    tags: Option<Vec<String>>,
    priority: Option<&str>,
    assignee: Option<&str>,
    depends_on: Option<Vec<String>>,
) -> Result<PathBuf> {
    use rand::Rng;

    // Validate dependencies exist before creating task
    if let Some(deps) = &depends_on {
        for dep_id in deps {
            let dep_path = workspace_root
                .join(".cuedeck/cards")
                .join(format!("{}.md", dep_id));
            if !dep_path.exists() {
                return Err(CueError::DependencyNotFound(dep_id.clone()));
            }
        }

        // Generate task ID first for validation
        let id: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(6)
            .map(char::from)
            .collect::<String>()
            .to_lowercase();

        // Check for circular dependencies before creating task
        validate_task_dependencies(workspace_root, &id, deps)?;

        // Continue with the generated ID
        return create_task_with_id(workspace_root, &id, title, tags, priority, assignee, depends_on);
    }

    // No dependencies, create task normally
    let id: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(6)
        .map(char::from)
        .collect::<String>()
        .to_lowercase();
    
    create_task_with_id(workspace_root, &id, title, tags, priority, assignee, depends_on)
}

/// Internal helper to create task with pre-generated ID
fn create_task_with_id(
    workspace_root: &Path,
    id: &str,
    title: &str,
    tags: Option<Vec<String>>,
    priority: Option<&str>,
    assignee: Option<&str>,
    depends_on: Option<Vec<String>>,
) -> Result<PathBuf> {

    let filename = workspace_root
        .join(".cuedeck/cards")
        .join(format!("{}.md", id));

    // Ensure dir exists
    if let Some(parent) = filename.parent() {
        fs::create_dir_all(parent)?;
    }

    // Build frontmatter with optional fields
    let priority_str = priority.unwrap_or("medium");
    let created_str = chrono::Utc::now().to_rfc3339();

    let mut frontmatter = format!(
        r#"---
title: {}
status: todo
priority: {}
created: {}"#,
        title, priority_str, created_str
    );

    // Add assignee if provided (don't quote if starts with @)
    if let Some(a) = assignee {
        if !a.is_empty() {
            if a.starts_with('@') {
                frontmatter.push_str(&format!("\nassignee: {}", a));
            } else {
                frontmatter.push_str(&format!("\nassignee: \"{}\"", a));
            }
        }
    }

    // Add tags if provided
    if let Some(tag_list) = tags {
        if !tag_list.is_empty() {
            frontmatter.push_str("\ntags:");
            for tag in tag_list {
                frontmatter.push_str(&format!("\n  - {}", tag));
            }
        }
    }

    // Add depends_on if provided
    if let Some(dep_list) = depends_on {
        if !dep_list.is_empty() {
            frontmatter.push_str("\ndepends_on:");
            for dep in dep_list {
                frontmatter.push_str(&format!("\n  - {}", dep));
            }
        }
    }

    frontmatter.push_str("\n---\n");

    let template = format!(
        r#"{}
# {}

## Description

[Add description]
"#,
        frontmatter, title
    );

    fs::write(&filename, template)?;
    Ok(filename)
}

/// Update a task's metadata
pub fn update_task(
    workspace_root: &Path,
    id: &str,
    updates: serde_json::Map<String, serde_json::Value>,
) -> Result<Document> {
    let path = workspace_root
        .join(".cuedeck/cards")
        .join(format!("{}.md", id));

    if !path.exists() {
        return Err(CueError::FileNotFound {
            path: path.to_string_lossy().to_string(),
        });
    }

    // Read and Parse
    // usage of parse_file is good for reading, but writing back requires manipulating raw content
    // to preserve body while updating frontmatter.

    let content = fs::read_to_string(&path)?;
    let frontmatter_regex = regex::Regex::new(r"(?ms)^---\r?\n(.*?)\r?\n---").unwrap();

    if let Some(captures) = frontmatter_regex.captures(&content) {
        let yaml_str = captures.get(1).unwrap().as_str();
        let mut meta: serde_yaml::Value =
            serde_yaml::from_str(yaml_str).map_err(|e| CueError::ParseError(e.to_string()))?;

        // Apply updates
        if let serde_yaml::Value::Mapping(ref mut map) = meta {
            for (k, v) in updates {
                // Convert JSON value to YAML value (basic types)
                let yaml_v = match v {
                    serde_json::Value::String(s) => serde_yaml::Value::String(s),
                    serde_json::Value::Bool(b) => serde_yaml::Value::Bool(b),
                    serde_json::Value::Number(n) => {
                        if let Some(i) = n.as_i64() {
                            serde_yaml::Value::Number(i.into())
                        } else if let Some(f) = n.as_f64() {
                            serde_yaml::Value::Number(f.into())
                        } else {
                            continue;
                        }
                    }
                    serde_json::Value::Null => serde_yaml::Value::Null,
                    _ => continue, // Skip arrays/objects for now
                };
                map.insert(serde_yaml::Value::String(k), yaml_v);
            }
            
            // Auto-set 'updated' timestamp
            let updated_str = chrono::Utc::now().to_rfc3339();
            map.insert(
                serde_yaml::Value::String("updated".to_string()),
                serde_yaml::Value::String(updated_str),
            );
        }

        let new_yaml =
            serde_yaml::to_string(&meta).map_err(|e| CueError::ParseError(e.to_string()))?;

        // Reconstruct content
        let body_start = captures.get(0).unwrap().end();
        let new_content = format!("---\n{}---{}", new_yaml.trim(), &content[body_start..]);

        fs::write(&path, new_content)?;

        // Return updated doc
        parse_file(&path)
    } else {
        Err(CueError::ParseError(
            "No frontmatter found in card".to_string(),
        ))
    }
}

/// Validate task dependencies don't create cycles
pub fn validate_task_dependencies(
    workspace_root: &Path,
    task_id: &str,
    new_deps: &[String],
) -> Result<()> {
    use crate::task_graph::TaskGraph;

    // Build current task graph
    let mut graph = TaskGraph::from_workspace(workspace_root)?;

    // Simulate adding new dependencies
    for dep_id in new_deps {
        // Check if would create cycle
        if graph.would_create_cycle(task_id, dep_id) {
            return Err(CueError::CircularDependency(format!(
                "{} -> {}",
                task_id, dep_id
            )));
        }

        // Check if dependency exists
        let dep_path = workspace_root
            .join(".cuedeck/cards")
            .join(format!("{}.md", dep_id));
        if !dep_path.exists() {
            return Err(CueError::DependencyNotFound(dep_id.clone()));
        }

        // Add temporarily to check next dependency
        graph.add_dependency(task_id, dep_id)?;
    }

    Ok(())
}

/// Get task dependencies
pub fn get_task_dependencies(
    workspace_root: &Path,
    task_id: &str,
) -> Result<Vec<cue_common::TaskDependency>> {
    use crate::task_graph::TaskGraph;

    let graph = TaskGraph::from_workspace(workspace_root)?;
    let deps = graph.get_dependencies(task_id);

    Ok(deps
        .into_iter()
        .map(|to_id| cue_common::TaskDependency {
            from_id: task_id.to_string(),
            to_id,
        })
        .collect())
}

/// Get tasks that depend on this task (reverse dependencies)
pub fn get_task_dependents(
    workspace_root: &Path,
    task_id: &str,
) -> Result<Vec<cue_common::TaskDependency>> {
    use crate::task_graph::TaskGraph;

    let graph = TaskGraph::from_workspace(workspace_root)?;
    let dependents = graph.get_dependents(task_id);

    Ok(dependents
        .into_iter()
        .map(|from_id| cue_common::TaskDependency {
            from_id,
            to_id: task_id.to_string(),
        })
        .collect())
}

/// List all task cards with advanced filtering
#[tracing::instrument(skip(workspace_root))]
pub fn list_tasks_filtered(
    workspace_root: &Path,
    filters: &TaskFilters,
) -> Result<Vec<Document>> {
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
            match parse_file(entry.path()) {
                Ok(doc) => {
                    if let Some(meta) = &doc.frontmatter {
                        // Status filter
                        if let Some(status) = &filters.status {
                            if &meta.status != status {
                                continue;
                            }
                        }

                        // Assignee filter
                        if let Some(assignee) = &filters.assignee {
                            if meta.assignee.as_deref() != Some(assignee.as_str()) {
                                continue;
                            }
                        }

                        // Priority filter
                        if let Some(priority) = &filters.priority {
                            if meta.priority.to_lowercase() != priority.to_lowercase() {
                                continue;
                            }
                        }

                        // Tags filter (OR logic)
                        if let Some(tag_list) = &filters.tags {
                            if !matches_tag_filter(&meta.tags, tag_list) {
                                continue;
                            }
                        }

                        // Created date filter
                        if let Some(date_filter) = &filters.created {
                            if !matches_date_filter(&meta.created, date_filter)? {
                                continue;
                            }
                        }

                        // Updated date filter
                        if let Some(date_filter) = &filters.updated {
                            // First try 'updated' field, fallback to file mtime
                            let matches = if meta.updated.is_some() {
                                matches_date_filter(&meta.updated, date_filter)?
                            } else {
                                // Fallback to file modification time
                                match entry.metadata() {
                                    Ok(metadata) => match metadata.modified() {
                                        Ok(mtime) => matches_date_filter_mtime(&mtime, date_filter)?,
                                        Err(_) => false,
                                    },
                                    Err(_) => false,
                                }
                            };
                            
                            if !matches {
                                continue;
                            }
                        }

                        tasks.push(doc);
                    }
                }
                Err(e) => tracing::warn!("Failed to parse card {:?}: {}", entry.path(), e),
            }
        }
    }

    // Sort by priority then created (existing logic)
    tasks.sort_by(|a, b| {
        let p_a = a
            .frontmatter
            .as_ref()
            .map(|m| priority_score(&m.priority))
            .unwrap_or(0);
        let p_b = b
            .frontmatter
            .as_ref()
            .map(|m| priority_score(&m.priority))
            .unwrap_or(0);
        p_b.cmp(&p_a) // Higher priority first
    });

    Ok(tasks)
}

