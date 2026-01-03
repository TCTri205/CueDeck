//! Memory usage profiling tests
//!
//! Phase 7.4: Verify memory usage < 100MB for 1000 files

use cue_core::engine::CueEngine;
use memory_stats::memory_stats;
use std::fs;
use std::path::PathBuf;

/// Helper function to create test workspace with N files
fn create_test_workspace(name: &str, file_count: usize) -> PathBuf {
    let test_dir = std::env::temp_dir()
        .join("cuedeck_memory_test")
        .join(name);

    // Clean up if exists
    if test_dir.exists() {
        let _ = fs::remove_dir_all(&test_dir);
    }

    fs::create_dir_all(&test_dir).unwrap();
    fs::create_dir_all(test_dir.join(".cuedeck/cards")).unwrap();

    // Generate files with realistic content
    for i in 0..file_count {
        let content = format!(
            "---\n\
             title: Task Card {}\n\
             type: {}\n\
             priority: {}\n\
             assignee: user{}\n\
             tags: [performance, test, sample{}]\n\
             created: 2026-01-03\n\
             ---\n\n\
             # Card {}: {}\n\n\
             ## Description\n\n\
             This is a test card for memory profiling. It contains realistic content\n\
             to simulate actual workspace usage patterns.\n\n\
             ## Implementation Tasks\n\n\
             - [x] Step 1: Initialize component #{}\n\
             - [ ] Step 2: Implement core logic\n\
             - [ ] Step 3: Add error handling\n\
             - [ ] Step 4: Write unit tests\n\
             - [ ] Step 5: Integration testing\n\n\
             ## Dependencies\n\n\
             This card depends on:\n\
             - [[card-{}]]\n\
             - [[card-{}]]\n\n\
             ## Notes\n\n\
             Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod\n\
             tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam,\n\
             quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.\n\n\
             ### Technical Details\n\n\
             - Language: Rust\n\
             - Module: core::module{}\n\
             - Estimated effort: {} hours\n\n\
             ## References\n\n\
             - [Documentation](https://docs.example.com)\n\
             - [API Reference](https://api.example.com)\n",
            i,
            if i % 3 == 0 { "feature" } else if i % 2 == 0 { "bug" } else { "enhancement" },
            if i % 3 == 0 { "high" } else if i % 2 == 0 { "medium" } else { "low" },
            i % 5,
            i % 10,
            i,
            if i % 4 == 0 { "Feature Implementation" } else { "Bug Fix" },
            i,
            if i > 0 { i - 1 } else { 0 },
            if i > 1 { i - 2 } else { 0 },
            i % 20,
            (i % 10) + 1
        );

        fs::write(
            test_dir.join(".cuedeck/cards").join(format!("card{:04}.md", i)),
            content,
        )
        .unwrap();
    }

    test_dir
}

/// Convert bytes to MB
fn bytes_to_mb(bytes: usize) -> f64 {
    bytes as f64 / (1024.0 * 1024.0)
}

#[test]
fn test_memory_usage_1000_files() {
    // Create workspace with 1000 files
    let workspace = create_test_workspace("memory_1000", 1000);

    // Get baseline memory before initialization
    let baseline_mem = memory_stats()
        .expect("Failed to get memory stats")
        .physical_mem;

    println!("Baseline memory: {:.2} MB", bytes_to_mb(baseline_mem));

    // Initialize engine (this will scan all files)
    let _engine = CueEngine::new(&workspace).expect("Failed to initialize engine");

    // Get peak memory after initialization
    let peak_mem = memory_stats()
        .expect("Failed to get memory stats")
        .physical_mem;

    let memory_used = peak_mem.saturating_sub(baseline_mem);

    println!("Peak memory: {:.2} MB", bytes_to_mb(peak_mem));
    println!("Memory used: {:.2} MB", bytes_to_mb(memory_used));

    // Phase 7 Exit Criteria: Memory usage < 100MB for 1000 files
    const MAX_MEMORY_MB: f64 = 100.0;
    let memory_used_mb = bytes_to_mb(memory_used);

    assert!(
        memory_used_mb < MAX_MEMORY_MB,
        "Memory usage {:.2} MB exceeds limit of {:.2} MB",
        memory_used_mb,
        MAX_MEMORY_MB
    );

    println!("✅ Memory test passed: {:.2} MB < {} MB", memory_used_mb, MAX_MEMORY_MB);

    // Cleanup
    let _ = fs::remove_dir_all(&workspace);
}

#[test]
fn test_memory_usage_baseline_100_files() {
    // Additional baseline test with 100 files
    let workspace = create_test_workspace("memory_100", 100);

    let baseline_mem = memory_stats()
        .expect("Failed to get memory stats")
        .physical_mem;

    let _engine = CueEngine::new(&workspace).expect("Failed to initialize engine");

    let peak_mem = memory_stats()
        .expect("Failed to get memory stats")
        .physical_mem;

    let memory_used = peak_mem.saturating_sub(baseline_mem);
    let memory_used_mb = bytes_to_mb(memory_used);

    println!("100 files - Memory used: {:.2} MB", memory_used_mb);

    // Should be well under 100MB
    assert!(
        memory_used_mb < 50.0,
        "Memory usage for 100 files should be < 50MB, got {:.2} MB",
        memory_used_mb
    );

    println!("✅ Baseline test passed: {:.2} MB < 50 MB", memory_used_mb);

    // Cleanup
    let _ = fs::remove_dir_all(&workspace);
}

#[test]
fn test_memory_with_warm_cache() {
    // Test memory usage with warm cache (second scan)
    let workspace = create_test_workspace("memory_warm", 500);

    let mut engine = CueEngine::new(&workspace).expect("Failed to initialize engine");

    // First scan (cold cache)
    engine.scan_all().expect("Failed to scan");

    // Get memory before second scan
    let before_rescan = memory_stats()
        .expect("Failed to get memory stats")
        .physical_mem;

    // Second scan (warm cache - should be much faster and use less memory)
    engine.scan_all().expect("Failed to rescan");

    let after_rescan = memory_stats()
        .expect("Failed to get memory stats")
        .physical_mem;

    let rescan_memory = after_rescan.saturating_sub(before_rescan);
    let rescan_memory_mb = bytes_to_mb(rescan_memory);

    println!("Warm cache rescan memory: {:.2} MB", rescan_memory_mb);

    // Warm cache should use minimal additional memory
    assert!(
        rescan_memory_mb < 20.0,
        "Warm cache rescan should use < 20MB, got {:.2} MB",
        rescan_memory_mb
    );

    println!("✅ Warm cache test passed: {:.2} MB < 20 MB", rescan_memory_mb);

    // Cleanup
    let _ = fs::remove_dir_all(&workspace);
}
