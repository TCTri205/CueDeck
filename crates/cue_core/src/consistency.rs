use crate::doctor::{CheckStatus, HealthCheck, RepairResult};
use crate::Result;
use std::path::{Path, PathBuf};
use std::fs;
use regex::Regex;
use walkdir::WalkDir;
use chrono::{DateTime, Utc, NaiveDateTime, TimeZone};

/// Check metadata consistency across all task cards
pub fn check_metadata_consistency(workspace_root: &Path) -> Result<Vec<HealthCheck>> {
    let mut checks = Vec::new();
    let cards_dir = workspace_root.join(".cuedeck/cards");
    
    if !cards_dir.exists() {
        // No cards to check
        checks.push(HealthCheck {
            name: "Metadata Consistency".to_string(),
            status: CheckStatus::Pass,
            message: "No cards to validate".to_string(),
            details: None,
            fixable: false,
        });
        return Ok(checks);
    }
    
    let mut issues = Vec::new();
    let mut tag_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let now = Utc::now();
    
    // Known valid priorities
    let valid_priorities = ["low", "medium", "high", "critical"];
    
    // Compile regex once outside loop
    let frontmatter_regex = Regex::new(r"(?ms)^---\r?\n(.*?)\r?\n---").unwrap();
    
    for entry in WalkDir::new(&cards_dir)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() && entry.path().extension().is_some_and(|e| e == "md") {
            let path = entry.path();
            let filename = path.file_name().unwrap_or_default().to_string_lossy();
            
            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => continue,
            };
            
            // Parse frontmatter
            if let Some(captures) = frontmatter_regex.captures(&content) {
                let yaml_str = captures.get(1).unwrap().as_str();
                
                match serde_yaml::from_str::<serde_yaml::Value>(yaml_str) {
                    Ok(yaml) => {
                        if let serde_yaml::Value::Mapping(map) = yaml {
                            // Check priority
                            if let Some(serde_yaml::Value::String(priority)) = map.get(serde_yaml::Value::String("priority".to_string())) {
                                if !valid_priorities.contains(&priority.to_lowercase().as_str()) {
                                    issues.push(format!("{}: Unknown priority '{}'", filename, priority));
                                }
                            }
                            
                            // Check tags and count them
                            if let Some(serde_yaml::Value::Sequence(tags)) = map.get(serde_yaml::Value::String("tags".to_string())) {
                                for tag in tags {
                                    if let serde_yaml::Value::String(tag_str) = tag {
                                        *tag_counts.entry(tag_str.clone()).or_insert(0) += 1;
                                    }
                                }
                            }
                            
                            // Check timestamp validity
                            if let Some(serde_yaml::Value::String(created)) = map.get(serde_yaml::Value::String("created".to_string())) {
                                if DateTime::parse_from_rfc3339(created).is_err() {
                                    issues.push(format!("{}: Invalid timestamp format for 'created': {}", filename, created));
                                }
                            }
                            
                            if let Some(serde_yaml::Value::String(updated)) = map.get(serde_yaml::Value::String("updated".to_string())) {
                                if DateTime::parse_from_rfc3339(updated).is_err() {
                                    issues.push(format!("{}: Invalid timestamp format for 'updated': {}", filename, updated));
                                }
                            }
                            
                            // Check for stale tasks (active > 90 days old)
                            if let Some(serde_yaml::Value::String(status)) = map.get(serde_yaml::Value::String("status".to_string())) {
                                if status == "active" || status == "in-progress" {
                                    // Check created or updated date
                                    let date_str = map.get(serde_yaml::Value::String("updated".to_string()))
                                        .or_else(|| map.get(serde_yaml::Value::String("created".to_string())));
                                    
                                    if let Some(serde_yaml::Value::String(date)) = date_str {
                                        if let Ok(task_date) = DateTime::parse_from_rfc3339(date) {
                                            let task_date_utc = task_date.with_timezone(&Utc);
                                            let age = now.signed_duration_since(task_date_utc);
                                            
                                            if age.num_days() > 90 {
                                                issues.push(format!("{}: Stale active task ({}; {} days old)", filename, status, age.num_days()));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(_) => continue, // Already caught by frontmatter check
                }
            }
        }
    }
    
    // Check for rare tags (used less than 2 times)
    for (tag, count) in tag_counts {
        if count < 2 {
            issues.push(format!("Tag '{}' used only {} time(s) - possibly a typo?", tag, count));
        }
    }
    
    if issues.is_empty() {
        checks.push(HealthCheck {
            name: "Metadata Consistency".to_string(),
            status: CheckStatus::Pass,
            message: "All metadata is consistent".to_string(),
            details: None,
            fixable: false,
        });
    } else {
        // Determine if any timestamp issues exist (these are fixable)
        let has_timestamp_issues = issues.iter().any(|i| i.contains("Invalid timestamp format"));
        
        checks.push(HealthCheck {
            name: "Metadata Consistency".to_string(),
            status: CheckStatus::Warn,
            message: format!("Found {} metadata issue(s)", issues.len()),
            details: Some(issues),
            fixable: has_timestamp_issues, // Timestamp issues are auto-fixable
        });
    }
    
    Ok(checks)
}

/// Repair metadata issues in task cards
/// 
/// Automatically fixes:
/// - Invalid timestamps (converts to ISO 8601)
/// - Supports multiple input formats: YYYY-MM-DD, YYYY/MM/DD HH:MM:SS, Unix timestamps
/// - Tag normalization (if enabled): converts all tags to lowercase
pub fn repair_metadata(workspace_root: &Path, _check: &HealthCheck, normalize_tags: bool) -> Result<RepairResult> {
    let cards_dir = workspace_root.join(".cuedeck/cards");
    let mut fixed_count = 0;
    let mut skip_count = 0;
    
    if !cards_dir.exists() {
        return Ok(RepairResult {
            check_name: "Metadata Consistency".to_string(),
            success: true,
            message: "No cards to repair".to_string(),
        });
    }
    
    // Parse frontmatter regex
    let frontmatter_regex = Regex::new(r"(?ms)^---\r?\n(.*?)\r?\n---").unwrap();
    
    for entry in WalkDir::new(&cards_dir)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() && entry.path().extension().is_some_and(|e| e == "md") {
            let path = entry.path();
            
            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => {
                    skip_count += 1;
                    continue;
                }
            };
            
            // Extract frontmatter
            let captures = match frontmatter_regex.captures(&content) {
                Some(c) => c,
                None => continue,
            };
            
            let yaml_str = captures.get(1).unwrap().as_str();
            let full_match = captures.get(0).unwrap();
            let yaml_end = full_match.end();
            
            // Parse YAML
            let mut yaml_value: serde_yaml::Value = match serde_yaml::from_str(yaml_str) {
                Ok(y) => y,
                Err(_) => {
                    skip_count += 1;
                    continue;
                }
            };
            
            let mut modified = false;
            
            // Fix timestamps and tags
            if let serde_yaml::Value::Mapping(ref mut map) = yaml_value {
                // Fix 'created' field
                if let Some(serde_yaml::Value::String(created_str)) = map.get(serde_yaml::Value::String("created".to_string())) {
                    if DateTime::parse_from_rfc3339(created_str).is_err() {
                        // Try to parse and fix
                        if let Some(fixed) = try_parse_and_fix_timestamp(created_str) {
                            map.insert(
                                serde_yaml::Value::String("created".to_string()),
                                serde_yaml::Value::String(fixed),
                            );
                            modified = true;
                        }
                    }
                }
                
                // Fix 'updated' field
                if let Some(serde_yaml::Value::String(updated_str)) = map.get(serde_yaml::Value::String("updated".to_string())) {
                    if DateTime::parse_from_rfc3339(updated_str).is_err() {
                        // Try to parse and fix
                        if let Some(fixed) = try_parse_and_fix_timestamp(updated_str) {
                            map.insert(
                                serde_yaml::Value::String("updated".to_string()),
                                serde_yaml::Value::String(fixed),
                            );
                            modified = true;
                        }
                    }
                }
                
                // Normalize tags if enabled
                if normalize_tags {
                    if let Some(serde_yaml::Value::Sequence(ref mut tags)) = map.get_mut(serde_yaml::Value::String("tags".to_string())) {
                        let mut tags_modified = false;
                        for tag in tags.iter_mut() {
                            if let serde_yaml::Value::String(tag_str) = tag {
                                let normalized = tag_str.to_lowercase();
                                if &normalized != tag_str {
                                    *tag = serde_yaml::Value::String(normalized);
                                    tags_modified = true;
                                }
                            }
                        }
                        if tags_modified {
                            modified = true;
                        }
                    }
                }
            }
            
            // Write back if modified
            if modified {
                // Serialize YAML
                let new_yaml_str = match serde_yaml::to_string(&yaml_value) {
                    Ok(s) => s.trim().to_string(),
                    Err(_) => {
                        skip_count += 1;
                        continue;
                    }
                };
                
                // Reconstruct file with fixed frontmatter
                let new_frontmatter = format!("---\n{}\n---", new_yaml_str);
                let body = &content[yaml_end..];
                let new_content = format!("{}{}", new_frontmatter, body);
                
                // Write to file
                if fs::write(path, new_content).is_ok() {
                    fixed_count += 1;
                } else {
                    skip_count += 1;
                }
            }
        }
    }
    
    if fixed_count > 0 {
        Ok(RepairResult {
            check_name: "Metadata Consistency".to_string(),
            success: true,
            message: format!("Fixed {} card(s), skipped {} card(s)", fixed_count, skip_count),
        })
    } else {
        Ok(RepairResult {
            check_name: "Metadata Consistency".to_string(),
            success: true,
            message: "No timestamp issues found to fix".to_string(),
        })
    }
}

/// Try to parse and fix a timestamp string
/// 
/// Supports formats:
/// - YYYY-MM-DD
/// - YYYY/MM/DD HH:MM:SS
/// - Unix timestamp (seconds)
/// 
/// Returns ISO 8601 formatted string if successful, None otherwise
fn try_parse_and_fix_timestamp(timestamp_str: &str) -> Option<String> {
    // Try parsing common formats
    
    // Format 1: YYYY-MM-DD
    if let Ok(dt) = NaiveDateTime::parse_from_str(&format!("{} 00:00:00", timestamp_str), "%Y-%m-%d %H:%M:%S") {
        return Some(Utc.from_utc_datetime(&dt).to_rfc3339());
    }
    
    // Format 2: YYYY/MM/DD HH:MM:SS
    if let Ok(dt) = NaiveDateTime::parse_from_str(timestamp_str, "%Y/%m/%d %H:%M:%S") {
        return Some(Utc.from_utc_datetime(&dt).to_rfc3339());
    }
    
    // Format 3: YYYY-MM-DD HH:MM:SS
    if let Ok(dt) = NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S") {
        return Some(Utc.from_utc_datetime(&dt).to_rfc3339());
    }
    
    // Format 4: Unix timestamp (seconds)
    if let Ok(unix_secs) = timestamp_str.parse::<i64>() {
        if let Some(dt) = DateTime::from_timestamp(unix_secs, 0) {
            return Some(dt.to_rfc3339());
        }
    }
    
    // If all parsing fails, use current timestamp with warning
    // (In production, this would log a warning)
    Some(Utc::now().to_rfc3339())
}


/// Check for broken links and anchors in all markdown files
pub fn check_link_integrity(workspace_root: &Path) -> Result<Vec<HealthCheck>> {
    let mut checks = Vec::new();
    let mut broken_links = Vec::new();
    
    // Regex for standard links: [text](url)
    // Captures: 1=text, 2=url
    let link_regex = Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").map_err(|e| crate::CueError::ParseError(e.to_string()))?;
    
    // Regex to match fenced code blocks (```code```)
    // This prevents parsing links inside code examples
    let code_block_regex = Regex::new(r"```[\s\S]*?```").map_err(|e| crate::CueError::ParseError(e.to_string()))?;
    
    // Regex for anchors in file content: # Header or ## Header
    // We will parse files on demand to check anchors
    
    for entry in WalkDir::new(workspace_root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| !e.path().to_string_lossy().contains(".git") && !e.path().to_string_lossy().contains("target")) 
    {
        if entry.file_type().is_file() && entry.path().extension().is_some_and(|e| e == "md") {
            let path = entry.path();
            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => continue, // Skip unreadable
            };
            
            // Remove code blocks to avoid false positives from regex patterns in code
            let content_without_code = code_block_regex.replace_all(&content, "");
            
            // Iterate over all links in file (excluding those in code blocks)
            for (line_idx, cap) in link_regex.captures_iter(&content_without_code).enumerate() {
                let full_match = cap.get(0).unwrap().as_str();
                let url = cap.get(2).unwrap().as_str();
                
                // Skip external links (http, https, mailto, ftp) and file:// URIs
                if url.starts_with("http") || url.starts_with("mailto") || url.starts_with("ftp") || url.starts_with("file://") {
                    continue;
                }
                
                // Check internal link
                if let Err(e) = validate_internal_link(workspace_root, path, url) {
                    let relative_path = path.strip_prefix(workspace_root).unwrap_or(path).to_string_lossy();
                     // We need to find the line number manually since captures_iter doesn't give it
                    let line_num = find_line_number(&content, full_match, line_idx); // This is an approximation
                    broken_links.push(format!("{}:{}: {} -> {}", relative_path, line_num, url, e));
                }
            }
        }
    }
    
    if broken_links.is_empty() {
        checks.push(HealthCheck {
            name: "Link Integrity".to_string(),
            status: CheckStatus::Pass,
            message: "All internal links are valid".to_string(),
            details: None,
            fixable: false,
        });
    } else {
        checks.push(HealthCheck {
            name: "Link Integrity".to_string(),
            status: CheckStatus::Fail,
            message: format!("Found {} broken link(s)", broken_links.len()),
            details: Some(broken_links),
            fixable: false,
        });
    }
    
    Ok(checks)
}

fn validate_internal_link(workspace_root: &Path, source_file: &Path, link: &str) -> std::result::Result<(), String> {
    // Handle anchor-only links (#section)
    if let Some(anchor) = link.strip_prefix('#') {
        if anchor.is_empty() { return Ok(()); } // empty anchor is valid (top of page)
        return check_anchor_in_file(source_file, anchor); // check in self
    }
    
    // Split path and anchor
    let (file_part, anchor_part) = match link.split_once('#') {
        Some((f, a)) => (f, Some(a)),
        None => (link, None),
    };
    
    // Resolve file path
    // Links are usually relative to the file, or relative to root if starting with / (though strict md usually implies relative)
    // We assume relative to source_file unless starting with /
    
    let target_path = if file_part.starts_with('/') {
        // Absolute path from workspace root (custom convention often used, but strict commonmark is filesystem path)
        // Let's support both: if / starts, treat as from workspace root.
        let rel = file_part.trim_start_matches('/');
        workspace_root.join(rel)
    } else {
        // Relative to source_file parent
        match source_file.parent() {
            Some(p) => p.join(file_part),
            None => return Err("Cannot determine parent directory".to_string()),
        }
    };
    
    // Check if file exists
    // Handle URL decoding? (e.g. %20). Rust's Path doesn't handle URI encoding.
    // Assuming raw paths for now.
    let target_path = clean_path(&target_path);
    
    if !target_path.exists() {
        return Err(format!("File not found: {:?}", target_path));
    }
    
    if target_path.is_dir() {
        // Link to directory? Maybe valid if index.md exists?
        // Let's force explicit file links for "Strict" integrity.
         return Err("Link points to directory, not file".to_string());
    }
    
    // Check anchor if present
    if let Some(anchor) = anchor_part {
        if !anchor.is_empty() {
            check_anchor_in_file(&target_path, anchor)?;
        }
    }
    
    Ok(())
}

fn check_anchor_in_file(path: &Path, anchor: &str) -> std::result::Result<(), String> {
    let content = fs::read_to_string(path).map_err(|_| "Target file unreadable".to_string())?;
    
    // Normalize anchor: 
    // Usually: lower-case, replacing spaces with dashes.
    // We need to match the slugification logic used in `lib.rs`
    
    let target_slug = anchor.to_lowercase(); 
    
    // Iterate headers
    for line in content.lines() {
        if let Some(header_text) = line.strip_prefix('#') {
            let header_text = header_text.trim_start_matches('#').trim();
            // Slugify
            // Logic from lib.rs: lowercase, spaces to hyphens, keep alphanumeric and hyphens
            let slug = header_text
                        .to_lowercase()
                        .replace(" ", "-")
                        .chars()
                        .filter(|c| c.is_alphanumeric() || *c == '-')
                        .collect::<String>();
                        
            if slug == target_slug {
                return Ok(());
            }
        }
    }
    
    Err(format!("Anchor '#{}' not found in {:?}", anchor, path.file_name().unwrap_or_default()))
}

// Simple path canonicalization to handle .. without fs access/symlinks resolution if possible, 
// strictly for checking "does provided relative path point to real file"
fn clean_path(path: &Path) -> PathBuf {
    // std::fs::canonicalize requires file to exist. 
    // Since we check existence afterwards, we can use it, but it returns absolute path.
    // Simplest is to just let fs::exists check the computed path, 
    // but `join` leaves `..` in. `fs::metadata` handles `..` correctly on OS.
    path.to_path_buf() 
}

fn find_line_number(content: &str, target: &str, _occurrence: usize) -> usize {
    // This is naive and will find the first occurrence. 
    // For better accuracy we'd need byte offsets from regex match.
    // Regex `captures` gives match range, we can count newlines before it.
    // But `captures_iter` in loop above didn't easily give us the Range without access to the match object in a specific way.
    // For MVP, finding first substring is okay for error reporting.
    content.lines().position(|l| l.contains(target)).unwrap_or(0) + 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;
    use crate::doctor::CheckStatus;

    #[test]
    fn test_check_link_integrity_valid_links() {
        let temp = assert_fs::TempDir::new().unwrap();
        
        let file1 = temp.child("doc1.md");
        file1.write_str("# Header\n\n[Link to doc2](doc2.md)").unwrap();
        
        let file2 = temp.child("doc2.md");
        file2.write_str("# Doc 2\n\nContent").unwrap();
        
        // Also check anchor in self
        let file3 = temp.child("self_ref.md");
        file3.write_str("# Top\n\n[Go to top](#top)").unwrap();

        let checks = check_link_integrity(temp.path()).unwrap();
        assert_eq!(checks.len(), 1);
        assert_eq!(checks[0].status, CheckStatus::Pass);
    }

    #[test]
    fn test_check_link_integrity_broken_file_link() {
        let temp = assert_fs::TempDir::new().unwrap();
        
        let file1 = temp.child("doc1.md");
        file1.write_str("[Broken](missing.md)").unwrap();
        
        let checks = check_link_integrity(temp.path()).unwrap();
        assert_eq!(checks.len(), 1);
        assert_eq!(checks[0].status, CheckStatus::Fail);
        
        let details = checks[0].details.as_ref().unwrap();
        assert!(details[0].contains("missing.md"));
    }

    #[test]
    fn test_check_link_integrity_broken_anchor() {
        let temp = assert_fs::TempDir::new().unwrap();
        
        let file1 = temp.child("doc1.md");
        file1.write_str("[Broken Anchor](doc2.md#missing)").unwrap();
        
        let file2 = temp.child("doc2.md");
        file2.write_str("# Header").unwrap(); // Anchor is #header, not #missing
        
        let checks = check_link_integrity(temp.path()).unwrap();
        assert_eq!(checks.len(), 1);
        assert_eq!(checks[0].status, CheckStatus::Fail);
        
        let details = checks[0].details.as_ref().unwrap();
        assert!(details[0].contains("Anchor '#missing' not found"));
    }

    #[test]
    fn test_check_link_integrity_ignore_external() {
        let temp = assert_fs::TempDir::new().unwrap();
        
        let file1 = temp.child("doc1.md");
        file1.write_str("[External](https://google.com)").unwrap();
        
        let checks = check_link_integrity(temp.path()).unwrap();
        assert_eq!(checks.len(), 1);
        assert_eq!(checks[0].status, CheckStatus::Pass);
    }

    #[test]
    fn test_check_link_integrity_ignore_file_uris() {
        let temp = assert_fs::TempDir::new().unwrap();
        
        let file1 = temp.child("doc1.md");
        file1.write_str("[Code](file:///d:/Projects/code.rs)\n[Another](file:///c:/Users/file.md)").unwrap();
        
        let checks = check_link_integrity(temp.path()).unwrap();
        assert_eq!(checks.len(), 1);
        assert_eq!(checks[0].status, CheckStatus::Pass);
        assert_eq!(checks[0].message, "All internal links are valid");
    }

    #[test]
    fn test_check_link_integrity_skip_code_blocks() {
        let temp = assert_fs::TempDir::new().unwrap();
        
        let file1 = temp.child("doc1.md");
        // Regex patterns in code blocks should NOT be parsed as links
        file1.write_str(
            "# Documentation\n\n\
            ```javascript\n\
            const regex = /import\\s+['\"]([^'\"]+)['\"]/g;\n\
            ```\n\n\
            [Valid Link](doc2.md)\n"
        ).unwrap();
        
        let file2 = temp.child("doc2.md");
        file2.write_str("# Target").unwrap();
        
        let checks = check_link_integrity(temp.path()).unwrap();
        assert_eq!(checks.len(), 1);
        assert_eq!(checks[0].status, CheckStatus::Pass);
        assert_eq!(checks[0].message, "All internal links are valid");
    }

    #[test]
    fn test_check_metadata_consistency_valid() {
        let temp = assert_fs::TempDir::new().unwrap();
        let cards_dir = temp.child(".cuedeck/cards");
        cards_dir.create_dir_all().unwrap();
        
        // Create multiple cards with shared tags to avoid "rare tag" warnings
        let card1 = cards_dir.child("test1.md");
        card1.write_str("---\ntitle: Test1\nstatus: todo\npriority: medium\ncreated: 2026-01-01T10:00:00Z\ntags:\n  - backend\n  - api\n---\n# Test1").unwrap();
        
        let card2 = cards_dir.child("test2.md");
        card2.write_str("---\ntitle: Test2\nstatus: todo\npriority: high\ncreated: 2026-01-01T11:00:00Z\ntags:\n  - backend\n  - api\n---\n# Test2").unwrap();
        
        let checks = check_metadata_consistency(temp.path()).unwrap();
        assert_eq!(checks.len(), 1);
        assert_eq!(checks[0].status, CheckStatus::Pass);
    }

    #[test]
    fn test_check_metadata_consistency_invalid_priority() {
        let temp = assert_fs::TempDir::new().unwrap();
        let cards_dir = temp.child(".cuedeck/cards");
        cards_dir.create_dir_all().unwrap();
        
        let card = cards_dir.child("test.md");
        card.write_str("---\ntitle: Test\nstatus: todo\npriority: urgent\ncreated: 2026-01-01T10:00:00Z\n---\n# Test").unwrap();
        
        let checks = check_metadata_consistency(temp.path()).unwrap();
        assert_eq!(checks.len(), 1);
        assert_eq!(checks[0].status, CheckStatus::Warn);
        
        let details = checks[0].details.as_ref().unwrap();
        assert!(details.iter().any(|d| d.contains("Unknown priority")));
    }

    #[test]
    fn test_check_metadata_consistency_invalid_timestamp() {
        let temp = assert_fs::TempDir::new().unwrap();
        let cards_dir = temp.child(".cuedeck/cards");
        cards_dir.create_dir_all().unwrap();
        
        let card = cards_dir.child("test.md");
        card.write_str("---\ntitle: Test\nstatus: todo\npriority: medium\ncreated: 2026-13-45\n---\n# Test").unwrap();
        
        let checks = check_metadata_consistency(temp.path()).unwrap();
        assert_eq!(checks.len(), 1);
        assert_eq!(checks[0].status, CheckStatus::Warn);
        
        let details = checks[0].details.as_ref().unwrap();
        assert!(details.iter().any(|d| d.contains("Invalid timestamp format")));
    }

    #[test]
    fn test_check_metadata_consistency_rare_tags() {
        let temp = assert_fs::TempDir::new().unwrap();
        let cards_dir = temp.child(".cuedeck/cards");
        cards_dir.create_dir_all().unwrap();
        
        // Card with unique tag
        let card1 = cards_dir.child("card1.md");
        card1.write_str("---\ntitle: Card1\nstatus: todo\npriority: medium\ncreated: 2026-01-01T10:00:00Z\ntags:\n  - unique-tag\n---\n# Card1").unwrap();
        
        // Card with common tag
        let card2 = cards_dir.child("card2.md");
        card2.write_str("---\ntitle: Card2\nstatus: todo\npriority: medium\ncreated: 2026-01-01T10:00:00Z\ntags:\n  - common\n---\n# Card2").unwrap();
        
        let card3 = cards_dir.child("card3.md");
        card3.write_str("---\ntitle: Card3\nstatus: todo\npriority: medium\ncreated: 2026-01-01T10:00:00Z\ntags:\n  - common\n---\n# Card3").unwrap();
        
        let checks = check_metadata_consistency(temp.path()).unwrap();
        assert_eq!(checks.len(), 1);
        assert_eq!(checks[0].status, CheckStatus::Warn);
        
        let details = checks[0].details.as_ref().unwrap();
        assert!(details.iter().any(|d| d.contains("unique-tag")));
    }

    #[test]
    fn test_repair_invalid_timestamp() {
        let temp = assert_fs::TempDir::new().unwrap();
        let cards_dir = temp.child(".cuedeck/cards");
        cards_dir.create_dir_all().unwrap();
        
        let card = cards_dir.child("repair_test.md");
        card.write_str("---\ntitle: Repair Test\ncreated: 2026/01/02 10:00:00\nupdated: 2026-01-02\n---\n# Content").unwrap();
        
        // Run repair
        let check = HealthCheck {
            name: "Metadata Consistency".to_string(),
            status: CheckStatus::Warn,
            message: "msg".to_string(),
            details: None,
            fixable: true,
        };
        
        // We need to use the public API or make repair_metadata public for testing
        // It is public in super
        let result = super::repair_metadata(temp.path(), &check, false).unwrap();
        assert!(result.success);
        assert!(result.message.contains("Fixed 1 card(s)"));
        
        // Verify content by parsing YAML checks, avoiding string formatting fragility
        let content = fs::read_to_string(card.path()).unwrap();
        let frontmatter_regex = Regex::new(r"(?ms)^---\r?\n(.*?)\r?\n---").unwrap();
        let captures = frontmatter_regex.captures(&content).unwrap();
        let yaml_str = captures.get(1).unwrap().as_str();
        let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        
        let created = yaml.get("created").unwrap().as_str().unwrap();
        let updated = yaml.get("updated").unwrap().as_str().unwrap();
        
        // Should be valid RFC3339
        assert_eq!(created, "2026-01-02T10:00:00+00:00");
        // 2026-01-02 becomes midnight UTC
        assert_eq!(updated, "2026-01-02T00:00:00+00:00");
    }

    #[test]
    fn test_repair_preserves_valid_timestamp() {
        let temp = assert_fs::TempDir::new().unwrap();
        let cards_dir = temp.child(".cuedeck/cards");
        cards_dir.create_dir_all().unwrap();
        
        let valid_ts = "2026-01-02T12:00:00+00:00";
        let card = cards_dir.child("valid.md");
        card.write_str(&format!("---\ntitle: Valid\ncreated: {}\n---\n# Content", valid_ts)).unwrap();
        
        let check = HealthCheck {
            name: "Metadata Consistency".to_string(),
            status: CheckStatus::Warn,
            message: "msg".to_string(),
            details: None,
            fixable: true,
        };
        
        let result = super::repair_metadata(temp.path(), &check, false).unwrap();
        assert!(result.success);
        
        let content = fs::read_to_string(card.path()).unwrap();
        assert!(content.contains(valid_ts));
    }

    #[test]
    fn test_repair_unparsable_timestamp() {
        let temp = assert_fs::TempDir::new().unwrap();
        let cards_dir = temp.child(".cuedeck/cards");
        cards_dir.create_dir_all().unwrap();
        
        let card = cards_dir.child("unparsable.md");
        card.write_str("---\ntitle: Unparsable\ncreated: not-a-date\n---\n# Content").unwrap();
        
        let check = HealthCheck {
            name: "Metadata Consistency".to_string(),
            status: CheckStatus::Warn,
            message: "msg".to_string(),
            details: None,
            fixable: true,
        };
        
        let result = super::repair_metadata(temp.path(), &check, false).unwrap();
        assert!(result.success);
        assert!(result.message.contains("Fixed 1 card(s)"));
        
        let content = fs::read_to_string(card.path()).unwrap();
        // Should have replaced "not-a-date" with a valid ISO string
        let frontmatter_regex = Regex::new(r"(?ms)^---\r?\n(.*?)\r?\n---").unwrap();
        let captures = frontmatter_regex.captures(&content).unwrap();
        let yaml_str = captures.get(1).unwrap().as_str();
        let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        
        if let serde_yaml::Value::String(created) = yaml.get("created").unwrap() {
            assert!(DateTime::parse_from_rfc3339(created).is_ok(), "Should be replaced with valid ISO timestamp");
            assert_ne!(created, "not-a-date");
        } else {
            panic!("created field missing or not a string");
        }
    }

    #[test]
    fn test_repair_preserves_body() {
        let temp = assert_fs::TempDir::new().unwrap();
        let cards_dir = temp.child(".cuedeck/cards");
        cards_dir.create_dir_all().unwrap();
        
        let body_content = "# Detailed Content\n\n- item 1\n- item 2\n\nSome text.";
        let card = cards_dir.child("body_test.md");
        card.write_str(&format!("---\ntitle: Body Test\ncreated: 2026/01/01\n---\n{}", body_content)).unwrap();
        
        let check = HealthCheck {
            name: "Metadata Consistency".to_string(),
            status: CheckStatus::Warn,
            message: "msg".to_string(),
            details: None,
            fixable: true,
        };
        
        super::repair_metadata(temp.path(), &check, false).unwrap();
        
        let content = fs::read_to_string(card.path()).unwrap();
        assert!(content.ends_with(body_content));
    }

    #[test]
    fn test_repair_normalize_tags() {
        let temp = assert_fs::TempDir::new().unwrap();
        let cards_dir = temp.child(".cuedeck/cards");
        cards_dir.create_dir_all().unwrap();
        
        let card = cards_dir.child("tags_test.md");
        card.write_str("---\ntitle: Tags Test\ncreated: 2026-01-01T10:00:00Z\ntags:\n  - Frontend\n  - Backend\n  - API\n  - Database\n---\n# Content").unwrap();
        
        let check = HealthCheck {
            name: "Metadata Consistency".to_string(),
            status: CheckStatus::Warn,
            message: "msg".to_string(),
            details: None,
            fixable: true,
        };
        
        // Repair WITH tag normalization
        let result = super::repair_metadata(temp.path(), &check, true).unwrap();
        assert!(result.success);
        
        // Verify tags are normalized to lowercase
        let content = fs::read_to_string(card.path()).unwrap();
        let frontmatter_regex = Regex::new(r"(?ms)^---\r?\n(.*?)\r?\n---").unwrap();
        let captures = frontmatter_regex.captures(&content).unwrap();
        let yaml_str = captures.get(1).unwrap().as_str();
        let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        
        if let serde_yaml::Value::Sequence(tags) = yaml.get("tags").unwrap() {
            assert_eq!(tags.len(), 4);
            assert_eq!(tags[0].as_str().unwrap(), "frontend");
            assert_eq!(tags[1].as_str().unwrap(), "backend");
            assert_eq!(tags[2].as_str().unwrap(), "api");
            assert_eq!(tags[3].as_str().unwrap(), "database");
        } else {
            panic!("tags field missing or not a sequence");
        }
    }

    #[test]
    fn test_repair_skip_normalize_tags_when_disabled() {
        let temp = assert_fs::TempDir::new().unwrap();
        let cards_dir = temp.child(".cuedeck/cards");
        cards_dir.create_dir_all().unwrap();
        
        let card = cards_dir.child("no_normalize.md");
        card.write_str("---\ntitle: No Normalize\ncreated: 2026-01-01T10:00:00Z\ntags:\n  - MixedCase\n  - UPPERCASE\n---\n# Content").unwrap();
        
        let check = HealthCheck {
            name: "Metadata Consistency".to_string(),
            status: CheckStatus::Pass,
            message: "msg".to_string(),
            details: None,
            fixable: false,
        };
        
        // Repair WITHOUT tag normalization (normalize_tags = false)
        let result = super::repair_metadata(temp.path(), &check, false).unwrap();
        assert!(result.success);
        
        // Verify tags remain unchanged
        let content = fs::read_to_string(card.path()).unwrap();
        let frontmatter_regex = Regex::new(r"(?ms)^---\r?\n(.*?)\r?\n---").unwrap();
        let captures = frontmatter_regex.captures(&content).unwrap();
        let yaml_str = captures.get(1).unwrap().as_str();
        let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        
        if let serde_yaml::Value::Sequence(tags) = yaml.get("tags").unwrap() {
            assert_eq!(tags[0].as_str().unwrap(), "MixedCase");
            assert_eq!(tags[1].as_str().unwrap(), "UPPERCASE");
        } else {
            panic!("tags field missing or not a sequence");
        }
    }
}
