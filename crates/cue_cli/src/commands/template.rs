//! Task template management

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::fs;

/// List available templates
pub fn list_templates(workspace_root: &Path) -> Result<Vec<String>> {
    let templates_dir = workspace_root.join(".cuedeck/templates");
    
    if !templates_dir.exists() {
        return Ok(Vec::new());
    }
    
    let mut templates = Vec::new();
    
    for entry in fs::read_dir(&templates_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) == Some("md") {
            if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                templates.push(name.to_string());
            }
        }
    }
    
    templates.sort();
    Ok(templates)
}

/// Create task from template with variable substitution
pub fn create_from_template(
    workspace_root: &Path,
    template_name: &str,
    title: &str,
) -> Result<PathBuf> {
    let template_path = workspace_root
        .join(".cuedeck/templates")
        .join(format!("{}.md", template_name));
    
    if !template_path.exists() {
        anyhow::bail!("Template '{}' not found in .cuedeck/templates/", template_name);
    }
    
    // Read template content
    let mut content = fs::read_to_string(&template_path)
        .context("Failed to read template file")?;
    
    // Variable substitution
    let now = chrono::Local::now();
    let date_str = now.format("%Y-%m-%d").to_string();
    
    let user = std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_else(|_| "user".to_string());
    
    // Generate 6-character random ID
    let id: String = rand::random::<u32>()
        .to_string()
        .chars()
        .take(6)
        .collect();
    
    content = content.replace("{{date}}", &date_str);
    content = content.replace("{{user}}", &user);
    content = content.replace("{{title}}", title);
    content = content.replace("{{id}}", &id);
    
    // Create task using existing API
    let task_path = cue_core::tasks::create_task(workspace_root, title)?;
    
    // Write template content (overwrites the basic task created above)
    fs::write(&task_path, content)
        .context("Failed to write template content")?;
    
    Ok(task_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_workspace() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let templates_dir = temp_dir.path().join(".cuedeck/templates");
        fs::create_dir_all(&templates_dir).unwrap();
        
        // Create test template
        let template_content = r#"---
id: {{id}}
title: {{title}}
created: {{date}}
---

# {{title}}

Created by {{user}} on {{date}}
"#;
        fs::write(
            templates_dir.join("test.md"),
            template_content,
        ).unwrap();
        
        // Create cards directory
        fs::create_dir_all(temp_dir.path().join(".cuedeck/cards")).unwrap();
        
        temp_dir
    }

    #[test]
    fn test_list_templates() {
        let temp_dir = create_test_workspace();
        let templates = list_templates(temp_dir.path()).unwrap();
        
        assert_eq!(templates, vec!["test"]);
    }

    #[test]
    fn test_list_templates_empty() {
        let temp_dir = TempDir::new().unwrap();
        let templates = list_templates(temp_dir.path()).unwrap();
        
        assert!(templates.is_empty());
    }

    #[test]
    fn test_create_from_template() {
        let temp_dir = create_test_workspace();
        
        let path = create_from_template(temp_dir.path(), "test", "My Test Task").unwrap();
        let content = fs::read_to_string(&path).unwrap();
        
        // Verify title replaced
        assert!(content.contains("My Test Task"));
        assert!(!content.contains("{{title}}"));
        
        // Verify date replaced
        assert!(!content.contains("{{date}}"));
        
        // Verify user replaced
        assert!(!content.contains("{{user}}"));
        
        // Verify ID replaced
        assert!(!content.contains("{{id}}"));
    }

    #[test]
    fn test_create_from_nonexistent_template() {
        let temp_dir = create_test_workspace();
        
        let result = create_from_template(temp_dir.path(), "nonexistent", "Test");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }
}
