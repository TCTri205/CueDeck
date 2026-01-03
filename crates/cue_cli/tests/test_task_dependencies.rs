//! Additional integration tests for task management with dependencies

use cue_test_helpers::prelude::*;
use predicates::prelude::*;
use std::fs;

#[test]
fn test_task_create_with_metadata() {
    let temp = init_workspace();

    // Create task with metadata
    cue_command()
        .current_dir(temp.path())
        .args([
            "card",
            "create",
            "Test Task with Metadata",
            "--tags",
            "auth,backend",
            "--priority",
            "high",
            "--assignee",
            "developer",
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains("Created task"))
        .stderr(predicate::str::contains("Tags: auth, backend"))
        .stderr(predicate::str::contains("Priority: high"))
        .stderr(predicate::str::contains("Assignee: developer"));

    // Verify card was created in .cuedeck/cards/
    let cards_dir = temp.path().join(".cuedeck/cards");
    assert!(cards_dir.exists());

    // Find the created card
    let entries: Vec<_> = fs::read_dir(&cards_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(entries.len(), 1, "Should have created exactly one card");

    // Verify frontmatter
    let card_path = entries[0].path();
    let content = fs::read_to_string(&card_path).unwrap();
    assert!(content.contains("title: Test Task with Metadata"));
    assert!(content.contains("priority: high"));
    assert!(content.contains("assignee: \"developer\""));
    assert!(content.contains("tags:"));
    assert!(content.contains("- auth"));
    assert!(content.contains("- backend"));
}

#[test]
fn test_task_create_with_dependencies() {
    let temp = init_workspace();

    // Create first task
    cue_command()
        .current_dir(temp.path())
        .args(["card", "new", "Base Task"])
        .assert()
        .success();

    // Get the ID of the first task
    let cards_dir = temp.path().join(".cuedeck/cards");
    let first_task_id = fs::read_dir(&cards_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .next()
        .unwrap()
        .path()
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .to_string();

    // Create second task depending on first
    cue_command()
        .current_dir(temp.path())
        .args([
            "card",
            "create",
            "Dependent Task",
            "--depends-on",
            &first_task_id,
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains("Created task"))
        .stderr(predicate::str::contains(format!("Depends on: {}", first_task_id)));

    // Verify dependency in frontmatter
    let entries: Vec<_> = fs::read_dir(&cards_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(entries.len(), 2, "Should have two tasks");

    // Find the dependent task (not the first one)
    let dependent_task = entries
        .iter()
        .find(|e| e.path().file_stem().unwrap() != first_task_id.as_str())
        .unwrap();

    let content = fs::read_to_string(dependent_task.path()).unwrap();
    assert!(content.contains("depends_on:"));
    assert!(content.contains(&format!("- {}", first_task_id)));
}

#[test]
fn test_task_deps_command() {
    let temp = init_workspace();

    // Create base task
    cue_command()
        .current_dir(temp.path())
        .args(["card", "new", "Base Task"])
        .assert()
        .success();

    let cards_dir = temp.path().join(".cuedeck/cards");
    let base_id = fs::read_dir(&cards_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .next()
        .unwrap()
        .path()
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .to_string();

    // Create dependent task
    cue_command()
        .current_dir(temp.path())
        .args(["card", "create", "Dependent", "--depends-on", &base_id])
        .assert()
        .success();

    let dependent_id = fs::read_dir(&cards_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .find(|e| e.path().file_stem().unwrap() != base_id.as_str())
        .unwrap()
        .path()
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .to_string();

    // Test deps command (show dependencies)
    cue_command()
        .current_dir(temp.path())
        .args(["card", "deps", &dependent_id])
        .assert()
        .success()
        .stderr(predicate::str::contains("Dependencies for"))
        .stderr(predicate::str::contains(&base_id));

    // Test deps --reverse (show dependents)
    cue_command()
        .current_dir(temp.path())
        .args(["card", "deps", &base_id, "--reverse"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Tasks depending on"))
        .stderr(predicate::str::contains(&dependent_id));
}

#[test]
fn test_task_validate_success() {
    let temp = init_workspace();

    // Create tasks with valid dependencies (A -> B)
    cue_command()
        .current_dir(temp.path())
        .args(["card", "new", "Task A"])
        .assert()
        .success();

    let cards_dir = temp.path().join(".cuedeck/cards");
    let task_a_id = fs::read_dir(&cards_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .next()
        .unwrap()
        .path()
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .to_string();

    cue_command()
        .current_dir(temp.path())
        .args(["card", "create", "Task B", "--depends-on", &task_a_id])
        .assert()
        .success();

    // Validate entire graph
    cue_command()
        .current_dir(temp.path())
        .args(["card", "validate"])
        .assert()
        .success()
        .stderr(predicate::str::contains("All task dependencies are valid"));
}

#[test]
fn test_task_validate_circular_dependency_prevention() {
    let temp = init_workspace();

    // Create task A
    cue_command()
        .current_dir(temp.path())
        .args(["card", "new", "Task A"])
        .assert()
        .success();

    let cards_dir = temp.path().join(".cuedeck/cards");
    let task_a_id = fs::read_dir(&cards_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .next()
        .unwrap()
        .path()
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .to_string();

    // Try to create task B depending on A, then manually edit A to depend on B
    // This should be caught by validation
    cue_command()
        .current_dir(temp.path())
        .args(["card", "create", "Task B", "--depends-on", &task_a_id])
        .assert()
        .success();

    let task_b_id = fs::read_dir(&cards_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .find(|e| e.path().file_stem().unwrap() != task_a_id.as_str())
        .unwrap()
        .path()
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .to_string();

    // Manually create circular dependency by editing task A's frontmatter
    let task_a_path = cards_dir.join(format!("{}.md", task_a_id));
    let content = fs::read_to_string(&task_a_path).unwrap();
    let modified = content.replace(
        "created:",
        &format!("depends_on:\n  - {}\ncreated:", task_b_id),
    );
    fs::write(&task_a_path, modified).unwrap();

    // Validation should now fail
    cue_command()
        .current_dir(temp.path())
        .args(["card", "validate"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Validation failed"));
}

#[test]
fn test_task_create_invalid_priority() {
    let temp = init_workspace();

    // Try to create task with invalid priority
    cue_command()
        .current_dir(temp.path())
        .args(["card", "create", "Test", "--priority", "invalid"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid priority"));
}

#[test]
fn test_task_create_nonexistent_dependency() {
    let temp = init_workspace();

    // Try to create task depending on non-existent task
    cue_command()
        .current_dir(temp.path())
        .args(["card", "create", "Test", "--depends-on", "nonexist"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Dependency not found"));
}
