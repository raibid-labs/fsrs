// Hot-Reload Demo for FSRS
// Demonstrates watching a script file and reloading it on changes

use fsrs_frontend::compile_program_from_source;
use fsrs_vm::{Chunk, HotReloadEngine, ReloadStats, Vm};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::{env, fs};

fn main() {
    println!("🔥 FSRS Hot-Reload Demo\n");

    // Get script path from args or use default
    let script_path = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            // Create a temporary script for demonstration
            let temp_path = PathBuf::from("/tmp/demo.fsrs");
            fs::write(&temp_path, "let x = 42 in x + 8").unwrap();
            println!("Created demo script at: {}", temp_path.display());
            temp_path
        });

    println!("Watching: {}\n", script_path.display());

    // Create hot-reload engine with FSRS compiler
    let mut engine = HotReloadEngine::new_with_compiler(&script_path, |source| {
        println!("  📝 Compiling...");
        compile_program_from_source(source).map_err(|e| e.to_string())
    })
    .expect("Failed to create hot-reload engine");

    // Register reload callback
    engine.on_reload(|stats: &ReloadStats| {
        println!("\n  ✨ Reload complete!");
        println!("     Time: {}ms", stats.reload_time_ms);
        println!("     Compile: {}ms", stats.compile_time_ms);
        println!("     Source size: {} bytes", stats.source_size_bytes);
        println!("     Target met: {}", stats.meets_target());

        if !stats.success {
            println!("     ❌ Error: {}", stats.error_message.as_ref().unwrap());
        }
    });

    // Initial load
    println!("📦 Initial load...");
    match engine.reload() {
        Ok(stats) => {
            if stats.success {
                println!("  ✅ Success! Took {}ms\n", stats.reload_time_ms);

                // Execute the initial version
                if let Some(chunk) = engine.current_chunk() {
                    execute_chunk(chunk);
                }
            } else {
                println!("  ❌ Failed: {}\n", stats.error_message.unwrap());
            }
        }
        Err(e) => {
            eprintln!("  ❌ Error: {}\n", e);
            return;
        }
    }

    // Start watching
    println!("👀 Watching for changes... (Press Ctrl+C to stop)\n");

    engine.start().expect("Failed to start watching");

    // Run watch loop
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    // Handle Ctrl+C
    ctrlc::set_handler(move || {
        running_clone.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl+C handler");

    let mut reload_count = 0;

    while running.load(Ordering::SeqCst) {
        // Wait for change with timeout
        if let Some(event) = engine.wait_for_change_timeout(Duration::from_millis(500)) {
            println!("📢 Change detected: {:?}", event);

            // Debounce - drain rapid changes
            thread::sleep(Duration::from_millis(100));
            let drained = engine.drain_events();
            if !drained.is_empty() {
                println!("  (Debounced {} additional events)", drained.len());
            }

            // Reload
            match engine.reload() {
                Ok(stats) => {
                    reload_count += 1;

                    if stats.success {
                        println!("\n  Reload #{} successful!", reload_count);

                        // Execute the new version
                        if let Some(chunk) = engine.current_chunk() {
                            execute_chunk(chunk);
                        }
                    } else {
                        println!(
                            "\n  Reload #{} failed: {}",
                            reload_count,
                            stats.error_message.unwrap()
                        );
                        println!("  (Keeping previous version)\n");
                    }
                }
                Err(e) => {
                    eprintln!("  Error reloading: {}\n", e);
                }
            }
        }
    }

    // Cleanup
    engine.stop().ok();
    println!("\n👋 Stopped watching. Total reloads: {}", reload_count);
}

/// Execute a chunk and display the result
fn execute_chunk(chunk: &Chunk) {
    let mut vm = Vm::new();
    match vm.execute(chunk.clone()) {
        Ok(result) => {
            println!("  🎯 Result: {}", result);
        }
        Err(e) => {
            println!("  ❌ Execution error: {}", e);
        }
    }
}
