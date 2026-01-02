use criterion::{black_box, criterion_group, criterion_main, Criterion};
use cue_core::{parse_file, context::{search_workspace_with_mode, SearchMode}};
use std::path::PathBuf;
use std::fs;

// Generate test data if needed
fn setup_test_workspace() -> PathBuf {
    let test_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("benches")
        .join("test_data");
    
    if !test_dir.exists() {
        fs::create_dir_all(&test_dir).unwrap();
        fs::create_dir_all(&test_dir.join(".cuedeck")).unwrap();
        fs::create_dir_all(&test_dir.join(".cuedeck/docs")).unwrap();
        
        // Generate 100 sample markdown files
        for i in 0..100 {
            let content = format!(
                "---\ntags: [test, sample{}]\npriority: {}\n---\n\n# Document {}\n\nThis is sample document number {}.\n\n## Section 1\n\nLorem ipsum dolor sit amet, consectetur adipiscing elit.\n\n## Section 2\n\nSed do eiusmod tempor incididunt ut labore et dolore magna aliqua.\n\n## Keywords\n\nauthentication authorization database api testing concurrent programming\n",
                i % 10,
                if i % 3 == 0 { "high" } else if i % 2 == 0 { "medium" } else { "low" },
                i,
                i
            );
            
            fs::write(
                test_dir.join(".cuedeck/docs").join(format!("doc{:03}.md", i)),
                content
            ).unwrap();
        }
    }
    
    test_dir
}

fn bench_parse_files(c: &mut Criterion) {
    let test_dir = setup_test_workspace();
    let docs_dir = test_dir.join(".cuedeck/docs");
    
    let mut group = c.benchmark_group("parsing");
    
    // Benchmark parsing 10 files
    group.bench_function("parse_10_files", |b| {
        b.iter(|| {
            for i in 0..10 {
                let path = docs_dir.join(format!("doc{:03}.md", i));
                let _ = parse_file(black_box(&path));
            }
        });
    });
    
    // Benchmark parsing 100 files
    group.bench_function("parse_100_files", |b| {
        b.iter(|| {
            for i in 0..100 {
                let path = docs_dir.join(format!("doc{:03}.md", i));
                let _ = parse_file(black_box(&path));
            }
        });
    });
    
    group.finish();
}

fn bench_search(c: &mut Criterion) {
    let test_dir = setup_test_workspace();
    
    let mut group = c.benchmark_group("search");
    
    // Keyword search benchmark
    group.bench_function("keyword_search", |b| {
        b.iter(|| {
            let _ = search_workspace_with_mode(
                black_box(&test_dir),
                black_box("authentication"),
                black_box(SearchMode::Keyword),
                black_box(None),
            );
        });
    });
    
    // Semantic search benchmark (warm cache)
    // Note: First run will be slow due to model download
    // Subsequent runs will use cached embeddings
    #[cfg(feature = "embeddings")]
    group.bench_function("semantic_search_warm", |b| {
        // Pre-warm the cache
        let _ = search_workspace_with_mode(&test_dir, "test", SearchMode::Semantic, None);
        
        b.iter(|| {
            let _ = search_workspace_with_mode(
                black_box(&test_dir),
                black_box("authentication"),
                black_box(SearchMode::Semantic),
                black_box(None),
            );
        });
    });
    
    // Hybrid search benchmark (warm cache)
    #[cfg(feature = "embeddings")]
    group.bench_function("hybrid_search", |b| {
        // Pre-warm the cache
        let _ = search_workspace_with_mode(&test_dir, "test", SearchMode::Hybrid, None);
        
        b.iter(|| {
            let _ = search_workspace_with_mode(
                black_box(&test_dir),
                black_box("authentication"),
                black_box(SearchMode::Hybrid),
                black_box(None),
            );
        });
    });
    
    group.finish();
}

criterion_group!(benches, bench_parse_files, bench_search);
criterion_main!(benches);
