use cue_test_helpers::prelude::*;
use assert_fs::TempDir;
use predicates::prelude::*;

fn create_task(temp: &TempDir, title: &str, args: &[&str]) -> String {
    let output = cue_command()
        .current_dir(temp.path())
        .arg("card")
        .arg("create")
        .arg(title)
        .args(args)
        .output()
        .unwrap();
    
    // Extract ID from stderr "Created task: <ID> at ..."
    let stderr = String::from_utf8_lossy(&output.stderr);
    stderr
        .lines()
        .find(|l| l.contains("Created task:"))
        .and_then(|l| l.split("Created task:").nth(1))
        .and_then(|s| s.split_whitespace().next())
        .expect("Failed to extract ID")
        .to_string()
}

#[test]
fn test_cli_filter_priority() {
    let temp = init_workspace();

    let id_high = create_task(&temp, "High Priority Task", &["--priority", "high"]);
    let id_low = create_task(&temp, "Low Priority Task", &["--priority", "low"]);

    // Filter by high
    cue_command()
        .current_dir(temp.path())
        .arg("list")
        .arg("--status")
        .arg("all")
        .arg("--priority")
        .arg("high")
        .assert()
        .success()
        .stderr(predicate::str::contains(&id_high))
        .stderr(predicate::str::contains(&id_low).not());
}

#[test]
fn test_cli_filter_tags() {
    let temp = init_workspace();

    let id_backend = create_task(&temp, "Backend Task", &["--tags", "backend"]);
    let id_frontend = create_task(&temp, "Frontend Task", &["--tags", "frontend"]);

    // Filter by backend
    cue_command()
        .current_dir(temp.path())
        .arg("list")
        .arg("--status")
        .arg("all")
        .arg("--tags")
        .arg("backend")
        .assert()
        .success()
        .stderr(predicate::str::contains(&id_backend))
        .stderr(predicate::str::contains(&id_frontend).not());
}

#[test]
fn test_cli_filter_date_created() {
    // Note: Since we can't easily fudge file creation time in these tests without external crates or sleeps,
    // we primarily test that the flag is accepted and filters correctly based on *current* time (e.g., created today).
    // For rigorous date testing, we'd need to mock the clock or FS, but verifying the CLI accepts the flag 
    // and passes it to core (which is unit tested) is sufficient for integration correctness.

    let temp = init_workspace();
    let id_task = create_task(&temp, "Today Task", &[]);

    // Filter created > 1 day ago (should include today's task)
    cue_command()
        .current_dir(temp.path())
        .arg("list")
        .arg("--status")
        .arg("all")
        .arg("--created")
        .arg(">1d") 
        .assert()
        .success()
        .stderr(predicate::str::contains(&id_task)); // Should match

    // Filter created < 1 day ago (Before (Now - 1d) -> Older than 1d)
    cue_command()
        .current_dir(temp.path())
        .arg("list")
        .arg("--status")
        .arg("all")
        .arg("--created")
        .arg("<1d") 
        .assert()
        .success()
        .stderr(predicate::str::contains(&id_task).not()); // Should NOT match (task is new)
}

#[test]
fn test_cli_combined_filters() {
    let temp = init_workspace();

    let id_target = create_task(&temp, "Target Task", &["--priority", "critical", "--tags", "bug"]);
    let id_noise1 = create_task(&temp, "Noise 1", &["--priority", "low", "--tags", "bug"]); // Wrong priority
    let id_noise2 = create_task(&temp, "Noise 2", &["--priority", "critical", "--tags", "feature"]); // Wrong tag

    cue_command()
        .current_dir(temp.path())
        .arg("list")
        .arg("--status")
        .arg("all")
        .arg("--priority")
        .arg("critical")
        .arg("--tags")
        .arg("bug")
        .assert()
        .success()
        .stderr(predicate::str::contains(&id_target))
        .stderr(predicate::str::contains(&id_noise1).not())
        .stderr(predicate::str::contains(&id_noise2).not());
}
