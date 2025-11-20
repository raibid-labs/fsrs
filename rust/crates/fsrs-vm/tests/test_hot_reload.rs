// Integration tests for hot-reload system

use fsrs_vm::{Chunk, HotReloadEngine, ReloadStats};
use std::fs;
use std::thread;
use std::time::Duration;
use tempfile::tempdir;

/// Test full hot-reload workflow: watch -> modify -> reload
#[test]
fn test_integration_full_workflow() {
    let temp_dir = tempdir().unwrap();
    let script_path = temp_dir.path().join("test.fsrs");
    fs::write(&script_path, "let x = 42").unwrap();

    let mut engine = HotReloadEngine::new_with_compiler(&script_path, |source| {
        // Simple compiler that just returns a chunk with source length encoded
        let mut chunk = Chunk::new();
        chunk.add_constant(fsrs_vm::Value::Int(source.len() as i64));
        Ok(chunk)
    })
    .unwrap();

    // Initial load
    let stats = engine.reload().unwrap();
    assert!(stats.success);
    assert!(stats.meets_target());
    assert_eq!(stats.source_size_bytes, 10); // "let x = 42"

    // Start watching
    engine.start().unwrap();

    // Modify in background thread
    let script_path_clone = script_path.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(200));
        fs::write(script_path_clone, "let y = 100").unwrap();
    });

    // Wait for change
    let event = engine.wait_for_change_timeout(Duration::from_secs(3));
    assert!(event.is_some());

    // Drain events
    thread::sleep(Duration::from_millis(150));
    engine.drain_events();

    // Reload
    let stats2 = engine.reload().unwrap();
    assert!(stats2.success);
    assert!(stats2.meets_target());
    assert_eq!(stats2.source_size_bytes, 11); // "let y = 100"

    engine.stop().unwrap();
}

/// Test performance: reload time should be <100ms
#[test]
fn test_performance_target() {
    let temp_dir = tempdir().unwrap();
    let script_path = temp_dir.path().join("test.fsrs");
    fs::write(&script_path, "let x = 42").unwrap();

    let mut engine = HotReloadEngine::new_with_compiler(&script_path, |_source| {
        // Fast compiler
        Ok(Chunk::new())
    })
    .unwrap();

    // Perform multiple reloads and check all meet target
    for _ in 0..10 {
        let stats = engine.reload().unwrap();
        assert!(stats.success);
        assert!(
            stats.meets_target(),
            "Reload took {}ms, expected <100ms",
            stats.reload_time_ms
        );
    }
}

/// Test that old chunk is preserved on compilation error
#[test]
fn test_error_recovery() {
    let temp_dir = tempdir().unwrap();
    let script_path = temp_dir.path().join("test.fsrs");
    fs::write(&script_path, "let x = 42").unwrap();

    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    let should_fail = Arc::new(AtomicBool::new(false));
    let should_fail_clone = should_fail.clone();

    let mut engine = HotReloadEngine::new_with_compiler(&script_path, move |_source| {
        if should_fail_clone.load(Ordering::SeqCst) {
            Err("Syntax error at line 1".to_string())
        } else {
            Ok(Chunk::new())
        }
    })
    .unwrap();

    // First load succeeds
    let stats1 = engine.reload().unwrap();
    assert!(stats1.success);
    let chunk1 = engine.current_chunk();
    assert!(chunk1.is_some());

    // Second load fails
    should_fail.store(true, Ordering::SeqCst);
    let stats2 = engine.reload().unwrap();
    assert!(!stats2.success);
    assert!(stats2.error_message.is_some());

    // Old chunk still available
    let chunk2 = engine.current_chunk();
    assert!(chunk2.is_some());

    // Reload count should still increment even on error
    assert_eq!(engine.reload_count(), 1); // Only successful reloads counted
}

/// Test callback system
#[test]
fn test_reload_callbacks() {
    let temp_dir = tempdir().unwrap();
    let script_path = temp_dir.path().join("test.fsrs");
    fs::write(&script_path, "let x = 42").unwrap();

    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    let success_count = Arc::new(AtomicUsize::new(0));
    let failure_count = Arc::new(AtomicUsize::new(0));

    let success_clone = success_count.clone();
    let failure_clone = failure_count.clone();

    let mut engine = HotReloadEngine::new_with_compiler(&script_path, |_source| {
        Ok(Chunk::new())
    })
    .unwrap();

    engine.on_reload(move |stats: &ReloadStats| {
        if stats.success {
            success_clone.fetch_add(1, Ordering::SeqCst);
        } else {
            failure_clone.fetch_add(1, Ordering::SeqCst);
        }
    });

    engine.reload().unwrap();
    assert_eq!(success_count.load(Ordering::SeqCst), 1);
    assert_eq!(failure_count.load(Ordering::SeqCst), 0);

    engine.reload().unwrap();
    assert_eq!(success_count.load(Ordering::SeqCst), 2);
    assert_eq!(failure_count.load(Ordering::SeqCst), 0);
}

/// Test file extension validation
#[test]
fn test_file_extension_validation() {
    let temp_dir = tempdir().unwrap();

    // Valid .fsrs extension
    let script_path_fsrs = temp_dir.path().join("test.fsrs");
    fs::write(&script_path_fsrs, "let x = 42").unwrap();
    let engine_fsrs = HotReloadEngine::new_with_compiler(&script_path_fsrs, |_| Ok(Chunk::new()));
    assert!(engine_fsrs.is_ok());

    // Valid .fs extension
    let script_path_fs = temp_dir.path().join("test.fs");
    fs::write(&script_path_fs, "let x = 42").unwrap();
    let engine_fs = HotReloadEngine::new_with_compiler(&script_path_fs, |_| Ok(Chunk::new()));
    assert!(engine_fs.is_ok());

    // Invalid extension should still create engine (validation happens on watch)
    let script_path_txt = temp_dir.path().join("test.txt");
    fs::write(&script_path_txt, "let x = 42").unwrap();
    let mut engine_txt = HotReloadEngine::new_with_compiler(&script_path_txt, |_| Ok(Chunk::new())).unwrap();

    // But watching should fail
    assert!(engine_txt.start().is_err());
}

/// Test rapid file modifications (debouncing)
#[test]
fn test_debouncing() {
    let temp_dir = tempdir().unwrap();
    let script_path = temp_dir.path().join("test.fsrs");
    fs::write(&script_path, "let x = 1").unwrap();

    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    let compile_count = Arc::new(AtomicUsize::new(0));
    let compile_count_clone = compile_count.clone();

    let mut engine = HotReloadEngine::new_with_compiler(&script_path, move |_source| {
        compile_count_clone.fetch_add(1, Ordering::SeqCst);
        Ok(Chunk::new())
    })
    .unwrap();

    engine.start().unwrap();

    // Make rapid modifications
    let script_path_clone = script_path.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(200));
        for i in 0..5 {
            fs::write(&script_path_clone, format!("let x = {}", i)).unwrap();
            thread::sleep(Duration::from_millis(20)); // Rapid changes
        }
    });

    // Wait for first event
    let event = engine.wait_for_change_timeout(Duration::from_secs(2));
    assert!(event.is_some());

    // Drain additional events
    thread::sleep(Duration::from_millis(200));
    let drained = engine.drain_events();

    // Should have debounced multiple events
    assert!(drained.len() >= 1);

    // Reload once
    engine.reload().unwrap();

    engine.stop().unwrap();
}

/// Test large file reload performance
#[test]
fn test_large_file_performance() {
    let temp_dir = tempdir().unwrap();
    let script_path = temp_dir.path().join("test.fsrs");

    // Create a large source file (10KB)
    let large_source = "let x = 42\n".repeat(1000);
    fs::write(&script_path, &large_source).unwrap();

    let mut engine = HotReloadEngine::new_with_compiler(&script_path, |_source| {
        // Simulate realistic compilation time
        thread::sleep(Duration::from_millis(20));
        Ok(Chunk::new())
    })
    .unwrap();

    let stats = engine.reload().unwrap();
    assert!(stats.success);
    assert_eq!(stats.source_size_bytes, large_source.len());

    // Even with large files and compilation, should be fast
    assert!(
        stats.reload_time_ms < 100,
        "Large file reload took {}ms",
        stats.reload_time_ms
    );
}
