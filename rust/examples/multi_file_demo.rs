// Multi-file program demonstration
// Shows how to work with FSRS modules across multiple files

use fsrs_demo::{run_source_with_options, FsrsError, RunOptions};
use std::fs;

fn main() -> Result<(), FsrsError> {
    println!("=== FSRS Multi-File Program Demo ===\n");

    // Read all module files
    let base_path = "examples/multi_file_program";

    println!("Loading modules from {}...", base_path);

    let math = fs::read_to_string(format!("{}/math.fsrs", base_path))?;
    let string_utils = fs::read_to_string(format!("{}/string_utils.fsrs", base_path))?;
    let config = fs::read_to_string(format!("{}/config.fsrs", base_path))?;
    let main = fs::read_to_string(format!("{}/main.fsrs", base_path))?;

    // In a real multi-file system, these would be parsed separately
    // For this demo, we concatenate them
    let program = format!(
        "{}\n\n{}\n\n{}\n\n{}",
        math, string_utils, config, main
    );

    println!("Modules loaded successfully!");
    println!("\nProgram structure:");
    println!("  - Math module: arithmetic utilities");
    println!("  - StringUtils module: string helpers");
    println!("  - Config module: application settings");
    println!("  - Main module: program entry point\n");

    println!("Executing program...\n");

    let options = RunOptions {
        enable_type_checking: false,
        verbose: true,
        strict_mode: false,
    };

    let result = run_source_with_options(&program, options)?;

    println!("\n=== Execution Result ===");
    println!("Result: {:?}", result);
    println!("\n=== Demo Complete ===");

    Ok(())
}
