//! Example: Phage Context Engine Integration
//!
//! This example demonstrates how to use Fusabi as a scripting layer
//! for an AI context composition engine, similar to what the Phage project
//! needs. It shows how to:
//! 1. Initialize the Fusabi engine
//! 2. Load and evaluate .fsb configuration files
//! 3. Extract data from evaluated results
//! 4. Map Fusabi values to application-specific structs

use fusabi::{Engine, Value};
use std::fs;

/// Example context configuration struct
#[derive(Debug)]
struct Context {
    name: String,
    max_tokens: i64,
    temperature: f64,
    system_prompt: String,
}

impl Context {
    /// Convert a Fusabi record value to a Context struct
    fn from_fusabi_value(value: &Value) -> Result<Self, String> {
        // For this example, we expect a record with specific fields
        // In a real implementation, you'd use proper record field access

        // This is a simplified example showing the concept
        Ok(Context {
            name: "example_context".to_string(),
            max_tokens: value.as_int().unwrap_or(4096),
            temperature: 0.7,
            system_prompt: "You are a helpful assistant.".to_string(),
        })
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Phage Integration Example ===\n");

    // 1. Initialize the Fusabi engine
    let mut engine = Engine::new();
    println!("✓ Fusabi engine initialized");

    // 2. Define a sample configuration in Fusabi syntax
    // (Normally this would be loaded from a .fsb file)
    let config_script = r#"
        let maxTokens = 4096 in
        let model = "claude-sonnet-4" in
        maxTokens
    "#;

    // 3. Evaluate the configuration
    let result = engine.eval(config_script)?;
    println!("✓ Configuration evaluated: {:?}", result);

    // 4. Extract the value
    match result.as_int() {
        Some(tokens) => println!("  Max tokens: {}", tokens),
        None => println!("  Result was not an integer"),
    }

    // 5. Example: Loading from a file path
    // (This would be used in production)
    println!("\n=== File Loading Example ===");

    // Create a temporary config file
    let temp_config = "let x = 42 in let y = 58 in x + y";
    fs::write("/tmp/test_config.fsx", temp_config)?;

    // Load and evaluate from file
    let file_result = fusabi::run_file("/tmp/test_config.fsx")?;
    println!("✓ Config loaded from file: {:?}", file_result);

    // Clean up
    fs::remove_file("/tmp/test_config.fsx").ok();

    // 6. Advanced: Using built-in host functions
    println!("\n=== Using Standard Library Functions ===");

    let string_test = r#"
        let text = "Hello, Phage!" in
        String.length text
    "#;
    let str_len = engine.eval(string_test)?;
    println!("✓ String length: {:?}", str_len);

    let list_test = r#"
        let items = [1; 2; 3; 4; 5] in
        List.length items
    "#;
    let list_len = engine.eval(list_test)?;
    println!("✓ List length: {:?}", list_len);

    // 7. Type-checked evaluation
    println!("\n=== Type-Checked Evaluation ===");

    let typed_script = "let x = 42 in x * 2";
    let typed_result = engine.eval_checked(typed_script)?;
    println!("✓ Type-checked result: {:?}", typed_result);

    println!("\n=== Integration Complete ===");
    println!("\nTo use Fusabi in your Phage project:");
    println!("1. Add to Cargo.toml: fusabi = {{ git = \"https://github.com/fusabi-lang/fusabi\" }}");
    println!("2. Import: use fusabi::{{Engine, Value}};");
    println!("3. Create engine: let mut engine = Engine::new();");
    println!("4. Evaluate scripts: let result = engine.eval(script_content)?;");
    println!("5. Extract values: result.as_int(), result.as_str(), etc.");

    Ok(())
}
