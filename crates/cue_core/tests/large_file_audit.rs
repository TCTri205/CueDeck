//! Large file parsing memory audit
//!
//! Phase 7.6: Measure memory usage when parsing large markdown files
//! to identify bottlenecks and determine if chunked reading is needed.

use cue_core::parse_file;
use memory_stats::memory_stats;
use std::fs;
use std::path::PathBuf;

/// Convert bytes to MB
fn bytes_to_mb(bytes: usize) -> f64 {
    bytes as f64 / (1024.0 * 1024.0)
}

/// Create a large markdown file with specified size
fn create_large_markdown(size_mb: usize) -> PathBuf {
    let temp_dir = std::env::temp_dir().join("cuedeck_large_file_test");
    fs::create_dir_all(&temp_dir).unwrap();

    let file_path = temp_dir.join(format!("large_{}mb.md", size_mb));

    // Generate content to reach target size
    let target_bytes = size_mb * 1024 * 1024;
    let mut content = String::with_capacity(target_bytes);

    // Add frontmatter
    content.push_str(
        "---\n\
         title: Large File Test\n\
         type: test\n\
         priority: low\n\
         tags: [test, large, performance]\n\
         ---\n\n\
         # Large File Memory Test\n\n",
    );

    // Add repeated content until we reach target size
    let chunk = "This is a test paragraph for memory profiling. It contains typical markdown content \
                 with **bold text**, *italic text*, and `code snippets`. This helps simulate real-world \
                 document parsing memory usage patterns. We repeat this content many times to create \
                 large files for testing.\n\n## Section Header\n\nSome more content here with links \
                 [[related-doc]] and references.\n\n";

    while content.len() < target_bytes {
        content.push_str(chunk);
    }

    // Trim to exact size
    content.truncate(target_bytes);

    fs::write(&file_path, content).unwrap();
    file_path
}

#[test]
fn test_parse_1mb_file() {
    let file_path = create_large_markdown(1);

    let baseline = memory_stats().unwrap().physical_mem;

    let _doc = parse_file(&file_path).expect("Failed to parse 1MB file");

    let after_parse = memory_stats().unwrap().physical_mem;
    let memory_used = after_parse.saturating_sub(baseline);
    let memory_mb = bytes_to_mb(memory_used);

    println!("1MB file - Memory used: {:.2} MB", memory_mb);

    // Should use reasonable memory (< 10MB for 1MB file)
    assert!(
        memory_mb < 10.0,
        "Parsing 1MB file should use < 10MB, got {:.2} MB",
        memory_mb
    );

    // Cleanup
    let _ = fs::remove_file(&file_path);
}

#[test]
fn test_parse_5mb_file() {
    let file_path = create_large_markdown(5);

    let baseline = memory_stats().unwrap().physical_mem;

    let _doc = parse_file(&file_path).expect("Failed to parse 5MB file");

    let after_parse = memory_stats().unwrap().physical_mem;
    let memory_used = after_parse.saturating_sub(baseline);
    let memory_mb = bytes_to_mb(memory_used);

    println!("5MB file - Memory used: {:.2} MB", memory_mb);

    // Should use reasonable memory (< 50MB for 5MB file)
    assert!(
        memory_mb < 50.0,
        "Parsing 5MB file should use < 50MB, got {:.2} MB",
        memory_mb
    );

    // Cleanup
    let _ = fs::remove_file(&file_path);
}

#[test]
fn test_parse_10mb_file() {
    let file_path = create_large_markdown(10);

    let baseline = memory_stats().unwrap().physical_mem;

    let _doc = parse_file(&file_path).expect("Failed to parse 10MB file");

    let after_parse = memory_stats().unwrap().physical_mem;
    let memory_used = after_parse.saturating_sub(baseline);
    let memory_mb = bytes_to_mb(memory_used);

    println!("10MB file - Memory used: {:.2} MB", memory_mb);

    // Should use reasonable memory (< 100MB for 10MB file)
    assert!(
        memory_mb < 100.0,
        "Parsing 10MB file should use < 100MB, got {:.2} MB",
        memory_mb
    );

    // Cleanup
    let _ = fs::remove_file(&file_path);
}

#[test]
#[ignore] // Only run manually for extreme cases
fn test_parse_50mb_file() {
    let file_path = create_large_markdown(50);

    let baseline = memory_stats().unwrap().physical_mem;

    let result = parse_file(&file_path);

    let after_parse = memory_stats().unwrap().physical_mem;
    let memory_used = after_parse.saturating_sub(baseline);
    let memory_mb = bytes_to_mb(memory_used);

    println!("50MB file - Memory used: {:.2} MB", memory_mb);

    if let Ok(_doc) = result {
        println!("✅ Successfully parsed 50MB file");

        // Should not exceed 500MB
        assert!(
            memory_mb < 500.0,
            "Parsing 50MB file should use < 500MB, got {:.2} MB",
            memory_mb
        );
    } else {
        println!("⚠️ Failed to parse 50MB file: {:?}", result);
    }

    // Cleanup
    let _ = fs::remove_file(&file_path);
}

#[test]
fn test_parse_efficiency_ratio() {
    // Test memory efficiency: ratio of memory used to file size
    let sizes = vec![1, 5, 10];

    for size_mb in sizes {
        let file_path = create_large_markdown(size_mb);

        let baseline = memory_stats().unwrap().physical_mem;
        let _doc = parse_file(&file_path).expect("Failed to parse file");
        let after_parse = memory_stats().unwrap().physical_mem;

        let memory_used = after_parse.saturating_sub(baseline);
        let memory_mb = bytes_to_mb(memory_used);
        let ratio = memory_mb / (size_mb as f64);

        println!(
            "{}MB file - Memory: {:.2} MB, Ratio: {:.2}x",
            size_mb, memory_mb, ratio
        );

        // Memory usage should be < 10x file size (reasonable overhead)
        assert!(
            ratio < 10.0,
            "Memory ratio too high: {:.2}x for {}MB file",
            ratio,
            size_mb
        );

        // Cleanup
        let _ = fs::remove_file(&file_path);
    }
}
