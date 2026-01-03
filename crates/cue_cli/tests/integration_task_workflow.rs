use cue_test_helpers::prelude::*;
use assert_fs::TempDir;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;

/// Helper to create a test workspace
fn setup_workspace() -> (TempDir, PathBuf) {
    let temp = init_workspace();
    let workspace = temp.path().to_path_buf();
    (temp, workspace)
}

/// Helper to extract task ID from creation output
fn extract_task_id(output: &[u8]) -> String {
    // CLI writes to stderr: "✓ Created task: abc123 at ..."
    let stderr_text = String::from_utf8_lossy(output);
    stderr_text
        .lines()
        .find(|line| line.contains("Created task:"))
        .and_then(|line| {
            // Parse: "✓ Created task: abc123 at ..."
            line.split("Created task:")
                .nth(1)
                .and_then(|s| s.split_whitespace().next())
        })
        .expect("Could not extract task ID")
        .to_string()
}

#[test]
fn test_end_to_end_task_workflow() {
    let (_temp, workspace) = setup_workspace();

    // Step 1: Create root task (Task A - Database)
    let output_a = cue_command()
        .current_dir(&workspace)
        .args(["card", "create", "Setup Database", "--priority", "high"])
        .assert()
        .success()
        .get_output()
        .stderr
        .clone();
    
    let task_a = extract_task_id(&output_a);

    // Step 2: Create Task B depending on A (Auth Table)
    let output_b = cue_command()
        .current_dir(&workspace)
        .args([
            "card", "create", "Create Auth Table",
            "--depends-on", &task_a,
            "--tags", "backend,auth",
        ])
        .assert()
        .success()
        .get_output()
        .stderr
        .clone();
    
    let task_b = extract_task_id(&output_b);

    // Step 3: Create Task C depending on A (User Table)
    let output_c = cue_command()
        .current_dir(&workspace)
        .args([
            "card", "create", "Create User Table",
            "--depends-on", &task_a,
            "--tags", "backend",
        ])
        .assert()
        .success()
        .get_output()
        .stderr
        .clone();
    
    let task_c = extract_task_id(&output_c);

    // Step 4: Create Task D depending on both B and C (Login)
    let depends_on = format!("{},{}", task_b, task_c);
    let output_d = cue_command()
        .current_dir(&workspace)
        .args([
            "card", "create", "Implement Login",
            "--depends-on", &depends_on,
            "--tags", "auth,frontend",
            "--priority", "critical",
        ])
        .assert()
        .success()
        .get_output()
        .stderr
        .clone();
    
    let task_d = extract_task_id(&output_d);

    // Step 5: Verify dependency chain via deps command
    cue_command()
        .current_dir(&workspace)
        .args(["card", "deps", &task_d])
        .assert()
        .success()
        .stderr(predicate::str::contains(&task_b))
        .stderr(predicate::str::contains(&task_c));

    // Step 6: Verify reverse dependencies
    cue_command()
        .current_dir(&workspace)
        .args(["card", "deps", &task_a, "--reverse"])
        .assert()
        .success()
        .stderr(predicate::str::contains(&task_b))
        .stderr(predicate::str::contains(&task_c));

    // Step 7: Validate graph (should pass)
    cue_command()
        .current_dir(&workspace)
        .args(["card", "validate"])
        .assert()
        .success()
        .stderr(predicate::str::contains("All task dependencies are valid"));

    // Step 8: Create E -> D, verify it works
    let output_e = cue_command()
        .current_dir(&workspace)
        .args([
            "card", "create", "Task E",
            "--depends-on", &task_d,
        ])
        .assert()
        .success()
        .get_output()
        .stderr
        .clone();
    
    let task_e = extract_task_id(&output_e);

    // Step 9: Verify task file contents for task E
    let task_e_path = workspace.join(format!(".cuedeck/cards/{}.md", task_e));
    let content = fs::read_to_string(&task_e_path).expect("Could not read task file");
    
    assert!(content.contains("title: Task E"));
    assert!(content.contains(&task_d));
}

#[test]
fn test_dependency_validation_prevents_nonexistent() {
    let (_temp, workspace) = setup_workspace();

    // Try to create task with nonexistent dependency
    cue_command()
        .current_dir(&workspace)
        .args([
            "card", "create", "Test Task",
            "--depends-on", "xyz999",  // This task doesn't exist
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Dependency not found: xyz999"));
}

#[test]
fn test_complex_dependency_graph() {
    let (_temp, workspace) = setup_workspace();

    // Create a more complex graph:
    //   A
    //  / \
    // B   C
    // |   |
    // D   E
    //  \ /
    //   F

    let output = cue_command()
        .current_dir(&workspace)
        .args(["card", "create", "Task A"])
        .output()
        .unwrap();
    let task_a = extract_task_id(&output.stderr);

    let output = cue_command()
        .current_dir(&workspace)
        .args(["card", "create", "Task B", "--depends-on", &task_a])
        .output()
        .unwrap();
    let task_b = extract_task_id(&output.stderr);

    let output = cue_command()
        .current_dir(&workspace)
        .args(["card", "create", "Task C", "--depends-on", &task_a])
        .output()
        .unwrap();
    let task_c = extract_task_id(&output.stderr);

    let output = cue_command()
        .current_dir(&workspace)
        .args(["card", "create", "Task D", "--depends-on", &task_b])
        .output()
        .unwrap();
    let task_d = extract_task_id(&output.stderr);

    let output = cue_command()
        .current_dir(&workspace)
        .args(["card", "create", "Task E", "--depends-on", &task_c])
        .output()
        .unwrap();
    let task_e = extract_task_id(&output.stderr);

    let depends = format!("{},{}", task_d, task_e);
    let output = cue_command()
        .current_dir(&workspace)
        .args(["card", "create", "Task F", "--depends-on", &depends])
        .output()
        .unwrap();
    let task_f = extract_task_id(&output.stderr);

    // Validate entire graph
    cue_command()
        .current_dir(&workspace)
        .args(["card", "validate"])
        .assert()
        .success();

    // Verify F's dependencies
    cue_command()
        .current_dir(&workspace)
        .args(["card", "deps", &task_f])
        .assert()
        .success()
        .stderr(predicate::str::contains(&task_d))
        .stderr(predicate::str::contains(&task_e));

    // Verify A's dependents (should show B and C)
    cue_command()
        .current_dir(&workspace)
        .args(["card", "deps", &task_a, "--reverse"])
        .assert()
        .success()
        .stderr(predicate::str::contains(&task_b))
        .stderr(predicate::str::contains(&task_c));
}

#[test]
fn test_metadata_preservation() {
    let (_temp, workspace) = setup_workspace();

    let output = cue_command()
        .current_dir(&workspace)
        .args([
            "card", "create", "Test Metadata",
            "--tags", "a,b,c",
            "--priority", "high",
            "--assignee", "@developer",
        ])
        .output()
        .unwrap();
    
    let task_id = extract_task_id(&output.stderr);

    // Verify frontmatter
    let task_path = workspace.join(format!(".cuedeck/cards/{}.md", task_id));
    let content = fs::read_to_string(&task_path).expect("Could not read task");

    assert!(content.contains("title: Test Metadata"));
    assert!(content.contains("priority: high"));
    assert!(content.contains("assignee: @developer"));
    assert!(content.contains("- a"));
    assert!(content.contains("- b"));
    assert!(content.contains("- c"));
}

#[test]
fn test_validate_specific_task() {
    let (_temp, workspace) = setup_workspace();

    let output = cue_command()
        .current_dir(&workspace)
        .args(["card", "create", "Task A"])
        .output()
        .unwrap();
    let task_a = extract_task_id(&output.stderr);

    let output = cue_command()
        .current_dir(&workspace)
        .args(["card", "create", "Task B", "--depends-on", &task_a])
        .output()
        .unwrap();
    let task_b = extract_task_id(&output.stderr);

    // Validate specific task
    cue_command()
        .current_dir(&workspace)
        .args(["card", "validate", &task_b])
        .assert()
        .success()
        .stderr(predicate::str::contains("dependencies are valid"));
}
