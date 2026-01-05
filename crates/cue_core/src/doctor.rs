//! Workspace health diagnostics
//!
//! This module provides comprehensive workspace validation and health checks.

use crate::task_graph::TaskGraph;
use crate::{CueError, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Complete health check report for a workspace
#[derive(Debug, Serialize, Deserialize)]
pub struct DoctorReport {
    pub healthy: bool,
    pub checks: Vec<HealthCheck>,
    pub stats: Option<WorkspaceStats>,
}

/// Individual health check result
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthCheck {
    pub name: String,
    pub status: CheckStatus,
    pub message: String,
    pub details: Option<Vec<String>>,
    pub fixable: bool,
}

/// Health check status
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CheckStatus {
    Pass,
    Warn,
    Fail,
}

/// Workspace statistics summary
#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceStats {
    pub total_tasks: usize,
    pub total_deps: usize,
    pub orphaned_tasks: usize,
    pub max_depth: usize,
}

/// Report of repair attempts
#[derive(Debug, Serialize, Deserialize)]
pub struct RepairReport {
    pub total_attempted: usize,
    pub successful: usize,
    pub failed: usize,
    pub details: Vec<RepairResult>,
}

/// Individual repair result
#[derive(Debug, Serialize, Deserialize)]
pub struct RepairResult {
    pub check_name: String,
    pub success: bool,
    pub message: String,
}

/// Run all workspace diagnostics
pub fn run_diagnostics(workspace_root: &Path) -> Result<DoctorReport> {
    let mut checks = Vec::new();

    // Check 1: Config validation
    checks.push(check_config_syntax(workspace_root));

    // Check 2: Workspace structure
    checks.push(check_workspace_structure(workspace_root));

    // Check 3: Card frontmatter
    checks.push(check_card_frontmatter(workspace_root));

    // Check 4: Link Integrity (Consistency)
    checks.extend(crate::consistency::check_link_integrity(workspace_root)?);

    // Check 5: Metadata Consistency
    checks.extend(crate::consistency::check_metadata_consistency(workspace_root)?);

    // Check 6: Task graph validation
    checks.extend(check_task_graph(workspace_root)?);

    let healthy = checks.iter().all(|c| c.status == CheckStatus::Pass);

    // Gather stats
    let stats = if workspace_root.join(".cuedeck/cards").exists() {
        Some(gather_workspace_stats(workspace_root)?)
    } else {
        None
    };

    Ok(DoctorReport {
        healthy,
        checks,
        stats,
    })
}

/// Check config TOML syntax
fn check_config_syntax(workspace_root: &Path) -> HealthCheck {
    use std::fs;

    let config_path = workspace_root.join(".cuedeck/config.toml");

    if !config_path.exists() {
        return HealthCheck {
            name: "Config File".to_string(),
            status: CheckStatus::Warn,
            message: "No config.toml found".to_string(),
            details: None,
            fixable: true,  // Can auto-create default config
        };
    }

    match fs::read_to_string(&config_path) {
        Ok(_content) => {
            // Basic validation: file is readable
            // Full TOML validation could be added later with toml crate
            HealthCheck {
                name: "Config File".to_string(),
                status: CheckStatus::Pass,
                message: "Config file exists and is readable".to_string(),
                details: None,
                fixable: false,
            }
        }
        Err(e) => HealthCheck {
            name: "Config File".to_string(),
            status: CheckStatus::Fail,
            message: "Cannot read config file".to_string(),
            details: Some(vec![format!("{}", e)]),
            fixable: false,
        },
    }
}

/// Check workspace directory structure
fn check_workspace_structure(workspace_root: &Path) -> HealthCheck {
    let cuedeck_dir = workspace_root.join(".cuedeck");
    let cards_dir = workspace_root.join(".cuedeck/cards");

    if !cuedeck_dir.exists() {
        return HealthCheck {
            name: "Workspace Structure".to_string(),
            status: CheckStatus::Fail,
            message: ".cuedeck directory missing".to_string(),
            details: None,
            fixable: true,  // Can auto-create directory
        };
    }

    if !cards_dir.exists() {
        return HealthCheck {
            name: "Workspace Structure".to_string(),
            status: CheckStatus::Warn,
            message: "cards directory missing".to_string(),
            details: None,
            fixable: true,  // Can auto-create directory
        };
    }

    HealthCheck {
        name: "Workspace Structure".to_string(),
        status: CheckStatus::Pass,
        message: "All required directories exist".to_string(),
        details: None,
        fixable: false,
    }
}

/// Check card frontmatter validity
fn check_card_frontmatter(workspace_root: &Path) -> HealthCheck {
    use std::fs;
    use walkdir::WalkDir;

    let cards_dir = workspace_root.join(".cuedeck/cards");

    if !cards_dir.exists() {
        return HealthCheck {
            name: "Card Frontmatter".to_string(),
            status: CheckStatus::Pass,
            message: "No cards to validate".to_string(),
            details: None,
            fixable: false,
        };
    }

    let mut invalid_cards = Vec::new();

    for entry in WalkDir::new(&cards_dir)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() && entry.path().extension().is_some_and(|e| e == "md") {
            if let Ok(content) = fs::read_to_string(entry.path()) {
                if let Err(e) = validate_frontmatter(&content) {
                    let card_name = entry
                        .path()
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .to_string();
                    invalid_cards.push(format!("{}: {}", card_name, e));
                }
            }
        }
    }

    if invalid_cards.is_empty() {
        HealthCheck {
            name: "Card Frontmatter".to_string(),
            status: CheckStatus::Pass,
            message: "All card frontmatter valid".to_string(),
            details: None,
            fixable: false,
        }
    } else {
        HealthCheck {
            name: "Card Frontmatter".to_string(),
            status: CheckStatus::Fail,
            message: format!("Found {} card(s) with invalid frontmatter", invalid_cards.len()),
            details: Some(invalid_cards),
            fixable: false,
        }
    }
}

/// Validate frontmatter YAML syntax
fn validate_frontmatter(content: &str) -> Result<()> {
    use regex::Regex;

    let frontmatter_regex = Regex::new(r"(?ms)^---\r?\n(.*?)\r?\n---")
        .map_err(|_| CueError::ParseError("Regex compilation failed".to_string()))?;

    let captures = frontmatter_regex
        .captures(content)
        .ok_or_else(|| CueError::ParseError("No frontmatter found".to_string()))?;

    let yaml_str = captures
        .get(1)
        .ok_or_else(|| CueError::ParseError("Empty frontmatter".to_string()))?
        .as_str();

    // Try parsing YAML
    let _: serde_yaml::Value = serde_yaml::from_str(yaml_str)
        .map_err(|e| CueError::ParseError(format!("Invalid YAML: {}", e)))?;

    Ok(())
}

/// Check task graph for issues
fn check_task_graph(workspace_root: &Path) -> Result<Vec<HealthCheck>> {
    let mut checks = Vec::new();

    let cards_dir = workspace_root.join(".cuedeck/cards");
    if !cards_dir.exists() {
        // No cards, no graph issues
        return Ok(checks);
    }

    let graph = TaskGraph::from_workspace(workspace_root)?;

    // Check: Circular dependencies
    if let Err(e) = graph.validate_dependencies() {
        checks.push(HealthCheck {
            name: "Task Dependencies".to_string(),
            status: CheckStatus::Fail,
            message: "Circular dependency detected".to_string(),
            details: Some(vec![e.to_string()]),
            fixable: false,
        });
    } else {
        checks.push(HealthCheck {
            name: "Task Dependencies".to_string(),
            status: CheckStatus::Pass,
            message: "No circular dependencies".to_string(),
            details: None,
            fixable: false,
        });
    }

    // Check: Orphaned tasks
    let orphaned = graph.find_orphaned_tasks();
    if !orphaned.is_empty() {
        checks.push(HealthCheck {
            name: "Orphaned Tasks".to_string(),
            status: CheckStatus::Warn,
            message: format!("Found {} orphaned task(s)", orphaned.len()),
            details: Some(orphaned),
            fixable: false,
        });
    } else {
        checks.push(HealthCheck {
            name: "Orphaned Tasks".to_string(),
            status: CheckStatus::Pass,
            message: "No orphaned tasks".to_string(),
            details: None,
            fixable: false,
        });
    }

    // Check: Missing dependencies
    let missing = graph.check_missing_dependencies(workspace_root);
    if !missing.is_empty() {
        let details: Vec<String> = missing
            .iter()
            .map(|(from, to)| format!("{} -> {} (not found)", from, to))
            .collect();
        checks.push(HealthCheck {
            name: "Missing Dependencies".to_string(),
            status: CheckStatus::Fail,
            message: format!("Found {} missing dependency reference(s)", missing.len()),
            details: Some(details),
            fixable: false,
        });
    } else {
        checks.push(HealthCheck {
            name: "Missing Dependencies".to_string(),
            status: CheckStatus::Pass,
            message: "All dependencies exist".to_string(),
            details: None,
            fixable: false,
        });
    }

    Ok(checks)
}

/// Gather workspace statistics
pub fn gather_workspace_stats(workspace_root: &Path) -> Result<WorkspaceStats> {
    let graph = TaskGraph::from_workspace(workspace_root)?;
    let stats = graph.get_graph_stats();

    Ok(WorkspaceStats {
        total_tasks: stats.total_tasks,
        total_deps: stats.total_dependencies,
        orphaned_tasks: stats.orphaned_tasks,
        max_depth: stats.max_dependency_depth,
    })
}

/// Run automatic repairs for fixable issues
pub fn run_repairs(workspace_root: &Path, report: &DoctorReport, normalize_tags: bool) -> Result<RepairReport> {
    let mut results = Vec::new();

    for check in &report.checks {
        // Only attempt repairs for fixable checks that failed or warned
        if !check.fixable || check.status == CheckStatus::Pass {
            continue;
        }

        // Dispatch to appropriate repair function based on check name
        let result = match check.name.as_str() {
            "Workspace Structure" => repair_workspace_structure(workspace_root),
            "Config File" => repair_config_file(workspace_root),
            "Metadata Consistency" => crate::consistency::repair_metadata(workspace_root, check, normalize_tags),
            _ => continue, // Skip non-repairable checks
        };

        // Collect results, continuing even if one fails
        match result {
            Ok(r) => results.push(r),
            Err(e) => results.push(RepairResult {
                check_name: check.name.clone(),
                success: false,
                message: format!("Repair failed: {}", e),
            }),
        }
    }

    let successful = results.iter().filter(|r| r.success).count();
    let failed = results.len() - successful;

    Ok(RepairReport {
        total_attempted: results.len(),
        successful,
        failed,
        details: results,
    })
}

/// Repair workspace directory structure
fn repair_workspace_structure(workspace_root: &Path) -> Result<RepairResult> {
    use std::fs;

    let directories = [
        ".cuedeck",
        ".cuedeck/cards",
        ".cuedeck/context",
        ".cuedeck/cache",
    ];

    let mut created = Vec::new();

    for dir in &directories {
        let dir_path = workspace_root.join(dir);
        if !dir_path.exists() {
            fs::create_dir_all(&dir_path)?;
            created.push(dir.to_string());
        }
    }

    let message = if created.is_empty() {
        "All directories already exist".to_string()
    } else {
        format!("Created directories: {}", created.join(", "))
    };

    Ok(RepairResult {
        check_name: "Workspace Structure".to_string(),
        success: true,
        message,
    })
}

/// Repair missing config file
fn repair_config_file(workspace_root: &Path) -> Result<RepairResult> {
    use std::fs;

    let config_path = workspace_root.join(".cuedeck/config.toml");

    if config_path.exists() {
        return Ok(RepairResult {
            check_name: "Config File".to_string(),
            success: true,
            message: "Config file already exists".to_string(),
        });
    }

    // Ensure .cuedeck directory exists
    let cuedeck_dir = workspace_root.join(".cuedeck");
    if !cuedeck_dir.exists() {
        fs::create_dir_all(&cuedeck_dir)?;
    }

    // Create default config content
    let default_config = r#"# CueDeck Configuration
# For full documentation, see: https://github.com/TCTri205/CueDeck

[general]
# Default assignee for new tasks
default_assignee = ""

# Default priority for new tasks
default_priority = "medium"

[cache]
# Enable caching for improved performance
enabled = true

# Cache TTL in seconds
ttl = 3600

[logging]
# Log level: trace, debug, info, warn, error
level = "info"
"#;

    fs::write(&config_path, default_config)?;

    Ok(RepairResult {
        check_name: "Config File".to_string(),
        success: true,
        message: "Created default config.toml".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repair_workspace_structure_creates_dirs() {
        let temp = assert_fs::TempDir::new().unwrap();
        let root = temp.path();

        // Initial state: no .cuedeck dir
        assert!(!root.join(".cuedeck").exists());

        // Run repair
        let result = repair_workspace_structure(root).unwrap();

        // Verify result
        assert!(result.success);
        assert!(root.join(".cuedeck").exists());
        assert!(root.join(".cuedeck/cards").exists());
        assert!(root.join(".cuedeck/context").exists());
        assert!(root.join(".cuedeck/cache").exists());
    }

    #[test]
    fn test_repair_workspace_structure_idempotent() {
        let temp = assert_fs::TempDir::new().unwrap();
        let root = temp.path();

        // Run repair twice
        repair_workspace_structure(root).unwrap();
        let result = repair_workspace_structure(root).unwrap();

        // Verify second run
        assert!(result.success);
        assert_eq!(result.message, "All directories already exist");
    }

    #[test]
    fn test_repair_config_file_creates_default() {
        let temp = assert_fs::TempDir::new().unwrap();
        let root = temp.path();

        // Ensure .cuedeck exists
        std::fs::create_dir(root.join(".cuedeck")).unwrap();

        // Run repair
        let result = repair_config_file(root).unwrap();

        // Verify result
        assert!(result.success);
        assert!(root.join(".cuedeck/config.toml").exists());
        
        // Verify content
        let content = std::fs::read_to_string(root.join(".cuedeck/config.toml")).unwrap();
        assert!(content.contains("[general]"));
        assert!(content.contains("default_priority = \"medium\""));
    }

    #[test]
    fn test_repair_config_file_preserves_existing() {
        let temp = assert_fs::TempDir::new().unwrap();
        let root = temp.path();

        // Setup existing config
        std::fs::create_dir(root.join(".cuedeck")).unwrap();
        let config_path = root.join(".cuedeck/config.toml");
        std::fs::write(&config_path, "existing_value = 1").unwrap();

        // Run repair
        let result = repair_config_file(root).unwrap();

        // Verify result
        assert!(result.success);
        assert_eq!(result.message, "Config file already exists");

        // Verify content preserved
        let content = std::fs::read_to_string(&config_path).unwrap();
        assert_eq!(content, "existing_value = 1");
    }

    #[test]
    fn test_run_repairs_only_fixable_issues() {
        let temp = assert_fs::TempDir::new().unwrap();
        let root = temp.path();

        // Create a report with fixable and non-fixable issues
        let report = DoctorReport {
            healthy: false,
            checks: vec![
                HealthCheck {
                    name: "Workspace Structure".to_string(),
                    status: CheckStatus::Fail,
                    message: "Missing dirs".to_string(),
                    details: None,
                    fixable: true,
                },
                HealthCheck {
                    name: "Task Dependencies".to_string(),
                    status: CheckStatus::Fail,
                    message: "Circular dependency".to_string(),
                    details: None,
                    fixable: false,
                },
            ],
            stats: None,
        };

        // Run repairs
        let result = run_repairs(root, &report, false).unwrap();

        // Verify only fixable issue was attempted
        assert_eq!(result.total_attempted, 1);
        assert_eq!(result.successful, 1);
        assert_eq!(result.details[0].check_name, "Workspace Structure");
        
        // Verify repair actually happened
        assert!(root.join(".cuedeck").exists());
    }
}
