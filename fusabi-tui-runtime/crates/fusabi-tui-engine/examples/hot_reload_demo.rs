//! Hot reload demo for the Fusabi TUI engine.
//!
//! This example demonstrates:
//! - Creating a dashboard engine
//! - Enabling hot reload
//! - Loading a dashboard file
//! - Watching for file changes
//! - Automatic reloading on file modification
//!
//! # Usage
//!
//! 1. Create a test dashboard file:
//!    ```
//!    echo "let x = 42" > test_dashboard.fsx
//!    ```
//!
//! 2. Run this example:
//!    ```
//!    cargo run --example hot_reload_demo
//!    ```
//!
//! 3. Modify the test_dashboard.fsx file while the example is running:
//!    ```
//!    echo "let x = 100" > test_dashboard.fsx
//!    ```
//!
//! 4. Watch the console output to see the hot reload in action.
//!
//! Press Ctrl+C to exit.

use fusabi_tui_engine::prelude::*;
use fusabi_tui_render::test::TestRenderer;
use std::fs::File;
use std::io::Write as IoWrite;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Fusabi TUI Hot Reload Demo");
    println!("==========================\n");

    // Create a temporary test file
    let test_file_path = PathBuf::from("test_dashboard.fsx");
    create_test_file(&test_file_path)?;

    println!("Created test file: {}", test_file_path.display());
    println!("Modify this file to see hot reload in action.\n");

    // Create a test renderer (80x24 terminal)
    let renderer = TestRenderer::new(80, 24);

    // Create the dashboard engine
    let mut engine = DashboardEngine::new(renderer, PathBuf::from("."));

    // Enable hot reload with 300ms debounce
    engine.enable_hot_reload_with_debounce(300)?;
    println!("Hot reload enabled (300ms debounce)");

    // Load the test dashboard file
    engine.load(&test_file_path)?;
    println!("Loaded dashboard from: {}", test_file_path.display());

    // Display the loaded content
    display_file_content(&test_file_path)?;

    // Initial render
    engine.render()?;
    println!("\nInitial render complete.\n");

    println!("Watching for changes... (Press Ctrl+C to exit)\n");

    // Main event loop
    let mut iteration = 0;
    let mut last_reload_time = std::time::Instant::now();

    loop {
        iteration += 1;

        // Simulate event handling
        // In a real application, you would read actual keyboard/mouse events here
        if iteration % 100 == 0 {
            // Simulate a tick event every 100 iterations
            let action = engine.handle_event(Event::Tick)?;
            if action.is_quit() {
                break;
            }
        }

        // Poll for file changes
        if let Some(changes) = engine.poll_changes() {
            if !changes.is_empty() {
                let elapsed = last_reload_time.elapsed();
                println!("\n[{}] File changes detected:", format_duration(elapsed));
                for path in &changes {
                    println!("  - {}", path.display());
                }

                // Reload the dashboard
                print!("Reloading dashboard... ");
                engine.reload()?;
                println!("done!");

                // Display the new content
                display_file_content(&test_file_path)?;

                // Render the updated dashboard
                print!("Re-rendering... ");
                engine.render()?;
                println!("done!\n");

                last_reload_time = std::time::Instant::now();
            }
        }

        // Render if the state is dirty
        if engine.state().dirty {
            engine.render()?;
        }

        // Sleep for ~60 FPS
        thread::sleep(Duration::from_millis(16));

        // Exit after 1 minute for demo purposes
        if iteration > 3750 {
            // ~60 seconds at 60 FPS
            println!("\nDemo timeout reached. Exiting...");
            break;
        }
    }

    // Cleanup
    println!("\nCleaning up...");
    std::fs::remove_file(&test_file_path).ok();
    println!("Removed test file: {}", test_file_path.display());

    println!("\nDemo complete!");

    Ok(())
}

/// Create a test dashboard file with some initial content.
fn create_test_file(path: &Path) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    writeln!(file, "// Test Dashboard")?;
    writeln!(file, "// Edit this file to see hot reload in action!")?;
    writeln!(file)?;
    writeln!(file, "let x = 42")?;
    writeln!(file, "let y = x * 2")?;
    writeln!(file)?;
    writeln!(file, "printfn \"Result: %d\" y")?;
    file.flush()?;
    Ok(())
}

/// Display the content of a file.
fn display_file_content(path: &Path) -> std::io::Result<()> {
    let content = std::fs::read_to_string(path)?;
    println!("\nCurrent file content:");
    println!("---");
    for line in content.lines() {
        println!("{}", line);
    }
    println!("---");
    Ok(())
}

/// Format a duration for display.
fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    let millis = duration.subsec_millis();

    if secs > 0 {
        format!("{}.{:03}s", secs, millis)
    } else {
        format!("{}ms", millis)
    }
}
