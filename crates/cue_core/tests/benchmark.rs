use assert_fs::prelude::*;
use cue_core::engine::CueEngine;
use std::time::Instant;

#[test]
#[ignore]
fn benchmark_performance() {
    let temp = assert_fs::TempDir::new().unwrap();
    let root = temp.path();

    // Create structure
    temp.child(".cuedeck/cards").create_dir_all().unwrap();
    // Empty config is valid? Assuming yes or defaults.
    temp.child("cue.toml").write_str("").unwrap();

    let count = 200; // Start small for quick check, can increase
    println!("Generating {} files...", count);

    for i in 0..count {
        let content = format!(
            "---\ntitle: Card {}\n---\n\n# Card {}\n\nLink to [[Card {}]]\nOther link [[Card {}]]\n\nSome content to making file larger...", 
            i, i, (i + 1) % count, (i + 2) % count
        );
        temp.child(format!(".cuedeck/cards/card_{}.md", i))
            .write_str(&content)
            .unwrap();
    }

    // Cold Start
    println!("Starting cold init...");
    let start = Instant::now();
    // Engine new() initializes config, cache, graph, and scans all files
    let engine = CueEngine::new(root).expect("Failed to init engine");
    let cold_time = start.elapsed();
    println!("Cold init ({} files): {:.2?}", count, cold_time);

    // Render
    let start = Instant::now();
    let _output = engine.render().expect("Failed to render");
    let render_time = start.elapsed();
    println!("Render: {:.2?}", render_time);

    // Hot Start (load from cache)
    // Cache was saved in scan_all, so next new() should pick it up
    println!("Starting hot init...");
    let start = Instant::now();
    let _engine2 = CueEngine::new(root).expect("Failed to init engine 2");
    let hot_time = start.elapsed();
    println!("Hot init: {:.2?}", hot_time);

    // Assertions
    // Note: Cold init includes heavy I/O and parsing.
    // Hot init should be I/O (cache load) + deserialization, skipping parsing.

    if hot_time >= cold_time {
        println!(
            "WARNING: Hot init ({:?}) not faster than cold init ({:?})",
            hot_time, cold_time
        );
    } else {
        println!(
            "SUCCESS: Hot init is {:.1}x faster",
            cold_time.as_secs_f32() / hot_time.as_secs_f32()
        );
    }
}

#[test]
#[ignore]
fn benchmark_sqlite_batch_ops() {
    use cue_core::db::DbManager;
    use std::path::PathBuf;

    let temp = assert_fs::TempDir::new().unwrap();
    let db_path = temp.child("bench.db");
    
    // Test parameters
    let file_count = 1000;
    
    // 1. Single Insert Benchmark
    {
        let db = DbManager::open(db_path.path()).unwrap();
        // Clear existing
        let _ = db_path.path().to_owned(); // Keep temp alive
        
        println!("\nStarting Single Insert Benchmark ({} files)...", file_count);
        let start = Instant::now();
        
        for i in 0..file_count {
            db.upsert_file(
                &PathBuf::from(format!("file_{}.md", i)),
                &format!("hash_{}", i),
                1000,
                0, // tokens (dummy)
            ).unwrap();
        }
        
        let duration = start.elapsed();
        println!("Single Insert: {:?} ({:.2} ms/op)", 
            duration, 
            duration.as_secs_f64() * 1000.0 / file_count as f64
        );
    }

    // 2. Batch Insert Benchmark
    let temp_batch = assert_fs::TempDir::new().unwrap();
    let db_batch_path = temp_batch.child("bench_batch.db");
    {
        let mut db = DbManager::open(db_batch_path.path()).unwrap();
        
        let files: Vec<_> = (0..file_count)
            .map(|i| (
                PathBuf::from(format!("file_{}.md", i)),
                format!("hash_{}", i),
                0, // modified_at (dummy)
                1000u64,
                0 // tokens (dummy)
            ))
            .collect();

        println!("\nStarting Batch Insert Benchmark ({} files)...", file_count);
        let start = Instant::now();
        
        db.upsert_files_batch(&files).unwrap();
        
        let duration = start.elapsed();
        println!("Batch Insert:  {:?} ({:.4} ms/op)", 
            duration, 
            duration.as_secs_f64() * 1000.0 / file_count as f64
        );
    }
}
