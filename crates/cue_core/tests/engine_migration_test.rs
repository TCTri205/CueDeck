use cue_core::engine::CueEngine;
use cue_core::db::DbManager;
use assert_fs::prelude::*;
// use std::path::Path; // Unused
// use predicates::prelude::*; // Unused

#[test]
fn test_engine_migration_flow() {
    let temp = assert_fs::TempDir::new().unwrap();
    let workspace_root = temp.path();

    // 1. Setup initial state: valid JSON cache, no DB
    let dot_cue = temp.child(".cue");
    let cards_dir = temp.child(".cuedeck/cards");
    let cache_dir = temp.child(".cuedeck/cache");
    
    std::fs::create_dir_all(dot_cue.path()).unwrap();
    std::fs::create_dir_all(cards_dir.path()).unwrap();
    std::fs::create_dir_all(cache_dir.path()).unwrap();

    // Create a real file
    let test_file = cards_dir.child("test.md");
    test_file.write_str("# Test File\nContent").unwrap();

    // Manually create a JSON cache entry mimicking pre-migration state
    // We can use DocumentCache to help us create the file, but we need to ensure DB is absent
    {
        // Use a temporary engine or cache to creating the bin file
        // Or just manually mock it if access is hard. 
        // Better: Instantiate an engine *without* DB logic (impossible now as logic is in new)
        // Actually, we can just let engine create it, then delete DB.
        let mut engine = CueEngine::new(workspace_root).unwrap();
        // This creates cache and empty DB (fresh start)
        
        // Force sync to disk
        engine.scan_all().unwrap();
    }

    // Now manually DELETE the database to simulate "Old Version" state
    let db_path = dot_cue.child("metadata.db");
    if db_path.exists() {
        std::fs::remove_file(db_path.path()).unwrap();
    }
    
    // Ensure JSON cache exists
    let cache_file = cache_dir.child("documents.bin");
    assert!(cache_file.exists());
    
    // 2. Initialize Engine (should trigger migration)
    let _engine = CueEngine::new(workspace_root).unwrap();
    
    // 3. Verify DB created and populated
    assert!(db_path.exists());
    
    // Check if data is in DB
    let db = DbManager::open(db_path.path()).unwrap();
    let stats = db.get_stats().unwrap();
    assert_eq!(stats.file_count, 1);
    
    let file = db.get_file(test_file.path()).unwrap().unwrap();
    assert!(file.tokens > 0);
}

#[test]
fn test_engine_sync_operations() {
    let temp = assert_fs::TempDir::new().unwrap();
    let workspace_root = temp.path();
    let cards_dir = temp.child(".cuedeck/cards");
    std::fs::create_dir_all(cards_dir.path()).unwrap();
    
    let mut engine = CueEngine::new(workspace_root).unwrap();
    let db_path = workspace_root.join(".cue/metadata.db");
    let db = DbManager::open(&db_path).unwrap();

    // 1. Add file
    let file1 = cards_dir.child("file1.md");
    file1.write_str("# File 1").unwrap();
    
    engine.update_file(file1.path()).unwrap();
    
    let stats = db.get_stats().unwrap();
    assert_eq!(stats.file_count, 1);
    
    // 2. Update file
    file1.write_str("# File 1 Updated").unwrap();
    engine.update_file(file1.path()).unwrap();
    // (Hash check would be good here if exposed, but stats count should be same)
     let stats2 = db.get_stats().unwrap();
    assert_eq!(stats2.file_count, 1);
    
    let _metadata = db.get_file(file1.path()).unwrap().unwrap();
    // simple check that it exists
    
    // 3. Delete file
    engine.remove_file(file1.path());
    let stats3 = db.get_stats().unwrap();
    assert_eq!(stats3.file_count, 0);
}
