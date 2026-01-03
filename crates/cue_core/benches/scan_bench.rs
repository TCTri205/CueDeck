use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use cue_core::engine::CueEngine;
use std::path::PathBuf;
use std::fs;

// Helper function to create test workspace with N files
fn create_test_workspace(base_name: &str, file_count: usize) -> PathBuf {
    let test_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("benches")
        .join("test_data")
        .join(base_name);
    
    // Clean up if exists
    if test_dir.exists() {
        let _ = fs::remove_dir_all(&test_dir);
    }
    
    fs::create_dir_all(&test_dir).unwrap();
    fs::create_dir_all(test_dir.join(".cuedeck/cards")).unwrap();
    
    // Generate files
    for i in 0..file_count {
        let content = format!(
            "---\ntitle: Test Card {}\ntype: feature\npriority: {}\ntags: [test, sample{}]\n---\n\n# Card {}\n\nThis is test card number {}.\n\n## Implementation\n\n- Step 1: Initialize\n- Step 2: Process\n- Step 3: Complete\n\n## Dependencies\n\n[[card-{}]]\n\n## Notes\n\nLorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.\n",
            i,
            if i % 3 == 0 { "high" } else if i % 2 == 0 { "medium" } else { "low" },
            i % 10,
            i,
            i,
            if i > 0 { i - 1 } else { 0 }
        );
        
        fs::write(
            test_dir.join(".cuedeck/cards").join(format!("card{:04}.md", i)),
            content
        ).unwrap();
    }
    
    test_dir
}

fn bench_scan_100(c: &mut Criterion) {
    let mut group = c.benchmark_group("scan");
    
    // Benchmark: Initial scan of 100 files (cold start)
    group.bench_function(BenchmarkId::new("initial", 100), |b| {
        b.iter_batched(
            || create_test_workspace("scan_100_cold", 100),
            |workspace| {
                let engine = CueEngine::new(black_box(&workspace)).unwrap();
                black_box(engine);
            },
            criterion::BatchSize::LargeInput
        );
    });
    
    // Benchmark: Re-scan with no changes (warm cache, should be very fast)
    group.bench_function(BenchmarkId::new("rescan_no_changes", 100), |b| {
        let workspace = create_test_workspace("scan_100_warm", 100);
        let mut engine = CueEngine::new(&workspace).unwrap();
        
        b.iter(|| {
            engine.scan_all().unwrap();
        });
    });
    
    // Benchmark: Re-scan with 10% changes
    group.bench_function(BenchmarkId::new("rescan_10pct_changes", 100), |b| {
        b.iter_batched(
            || {
                let workspace = create_test_workspace("scan_100_10pct", 100);
                let engine = CueEngine::new(&workspace).unwrap();
                
                // Modify 10 files (10%)
                for i in 0..10 {
                    let file = workspace.join(".cuedeck/cards").join(format!("card{:04}.md", i));
                    fs::write(&file, format!("# Modified Card {}\n\nUpdated content", i)).unwrap();
                }
                
                (workspace, engine)
            },
            |(_, mut engine)| {
                engine.scan_all().unwrap();
                black_box(engine);
            },
            criterion::BatchSize::LargeInput
        );
    });
    
    group.finish();
}

fn bench_scan_1000(c: &mut Criterion) {
    let mut group = c.benchmark_group("scan_large");
    group.sample_size(10); // Fewer samples for large benchmarks
    
    // Benchmark: Initial scan of 1000 files
    group.bench_function(BenchmarkId::new("initial", 1000), |b| {
        b.iter_batched(
            || create_test_workspace("scan_1000_cold", 1000),
            |workspace| {
                let engine = CueEngine::new(black_box(&workspace)).unwrap();
                black_box(engine);
            },
            criterion::BatchSize::LargeInput
        );
    });
    
    // Benchmark: Re-scan with no changes
    group.bench_function(BenchmarkId::new("rescan_no_changes", 1000), |b| {
        let workspace = create_test_workspace("scan_1000_warm", 1000);
        let mut engine = CueEngine::new(&workspace).unwrap();
        
        b.iter(|| {
            engine.scan_all().unwrap();
        });
    });
    
    group.finish();
}

criterion_group!(scan_benches, bench_scan_100, bench_scan_1000);
criterion_main!(scan_benches);
