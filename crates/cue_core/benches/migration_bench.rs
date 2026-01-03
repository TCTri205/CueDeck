use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use cue_core::db::{DbManager, migration};
use cue_core::cache::DocumentCache;

use std::fs;
use assert_fs::prelude::*;

fn create_dummy_cache(dir: &std::path::Path, count: usize) {
    let dot_cue = dir.join(".cue");
    fs::create_dir_all(&dot_cue).unwrap();
    let cards_dir = dir.join(".cuedeck/cards");
    fs::create_dir_all(&cards_dir).unwrap();

    // Create real files so cache can parse them
    for i in 0..count {
        let path = cards_dir.join(format!("file_{}.md", i));
        fs::write(&path, "some content with tokens").unwrap();
    }

    // Initialize cache and load these files
    // This is slow, but we only do it once per setup
    let mut cache = DocumentCache::new(dir).unwrap();
    // We can't easily populate cache without public methods or scanning
    // So we'll cheat or just benchmark the migration part assuming cache exists.
    // Actually, let's use a simpler approach: create the documents.bin file manually if definitions are public?
    // No, definitions are not public enough.
    // Let's rely on the engine or cache to parse.
    // Since DocumentCache::load is what we need, and populate via scan.
    
    // For the benchmark, we want to measure `migrate_json_to_sqlite`.
    // That function requires a populated `DocumentCache`.
    // Since we made `entries` pub(crate), we can't access it here.
    // But we can populate it via `get_or_parse`.
    
    for i in 0..count {
        let path = cards_dir.join(format!("file_{}.md", i));
        let _ = cache.get_or_parse(&path);
    }
    
    cache.save().unwrap();
}

fn bench_migration(c: &mut Criterion) {
    let mut group = c.benchmark_group("migration");
    
    for count in [10, 100].iter() {
        group.bench_with_input(BenchmarkId::new("migrate_json_to_sqlite", count), count, |b, &count| {
            b.iter_with_setup(
                || {
                    let temp = assert_fs::TempDir::new().unwrap();
                    create_dummy_cache(temp.path(), count);
                    // Load cache back
                    let mut cache = DocumentCache::new(temp.path()).unwrap();
                    cache.load().unwrap();
                    (temp, cache)
                },
                |(temp, cache)| {
                    let _ = migration::migrate_json_to_sqlite(temp.path(), &cache).unwrap();
                },
            );
        });
    }
    group.finish();
}

fn bench_token_sum(c: &mut Criterion) {
    let mut group = c.benchmark_group("tokens");
    
    // Setup a DB with files
    let temp = assert_fs::TempDir::new().unwrap();
    let db_path = temp.child("metadata.db");
    let mut db = DbManager::open(db_path.path()).unwrap();
    
    // Insert 1000 files
    let tx = db.begin_transaction().unwrap();
    for i in 0..1000 {
        tx.execute(
            "INSERT INTO files (path, hash, modified_at, size_bytes, tokens) VALUES (?1, ?2, ?3, ?4, ?5)",
            &[&format!("file_{}.md", i), &"hash", &100i64, &100i64, &100i64],
        ).unwrap();
    }
    tx.commit().unwrap();
    
    group.bench_function("db_get_total_tokens_1000", |b| {
        b.iter(|| {
            db.get_total_tokens().unwrap()
        })
    });
    
    group.finish();
}

criterion_group!(benches, bench_migration, bench_token_sum);
criterion_main!(benches);
