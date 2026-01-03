//! Parser unit tests

use cue_core::parse_file;
use cue_test_helpers::workspace::create_temp_md;


#[test]
fn test_parse_empty_file() {
    let (_temp, path) = create_temp_md("");
    let doc = parse_file(&path).unwrap();

    assert_eq!(doc.anchors.len(), 0, "Empty file should have no anchors");
    // assert!(doc.tokens > 0); // Even empty has minimal tokens - verifying specific behavior
    // TempDir auto-cleans on drop
}

#[test]
fn test_parse_with_anchors() {
    let content = "# Header 1\nSome text\n## Header 2";
    let (_temp, path) = create_temp_md(content);
    let doc = parse_file(&path).unwrap();

    assert_eq!(doc.anchors.len(), 2, "Should parse 2 headers");
    assert_eq!(doc.anchors[0].level, 1, "First header should be level 1");
    assert_eq!(
        doc.anchors[0].header, "Header 1",
        "First header text should match"
    );
    assert_eq!(doc.anchors[1].level, 2, "Second header should be level 2");
    assert_eq!(
        doc.anchors[1].header, "Header 2",
        "Second header text should match"
    );
}

#[test]
fn test_slug_generation() {
    let content = "# Login Flow (API)";
    let (_temp, path) = create_temp_md(content);
    let doc = parse_file(&path).unwrap();

    assert_eq!(doc.anchors.len(), 1, "Should have one anchor");
    assert_eq!(
        doc.anchors[0].slug, "login-flow-api",
        "Slug should be lowercase with hyphens"
    );
}

#[test]
fn test_multiple_heading_levels() {
    let content = "# H1\n## H2\n### H3\n#### H4\n##### H5\n###### H6";
    let (_temp, path) = create_temp_md(content);
    let doc = parse_file(&path).unwrap();

    assert_eq!(doc.anchors.len(), 6, "Should parse all 6 heading levels");
    for (i, anchor) in doc.anchors.iter().enumerate() {
        assert_eq!(
            anchor.level,
            (i + 1) as u8,
            "Level should match heading depth"
        );
    }
}

#[test]
fn test_hash_generation() {
    let content = "# Test Content";
    let (_temp, path) = create_temp_md(content);
    let doc = parse_file(&path).unwrap();

    assert!(!doc.hash.is_empty(), "Hash should not be empty");
    assert_eq!(
        doc.hash.len(),
        64,
        "SHA256 hash should be 64 hex characters"
    );
}

#[test]
fn test_file_not_found() {
    use std::path::PathBuf;
    let path = PathBuf::from("nonexistent_file_xyz123.md");
    let result = parse_file(&path);

    assert!(
        result.is_err(),
        "Parsing nonexistent file should return error"
    );
}
