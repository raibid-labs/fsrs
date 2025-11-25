//! Basic usage example for the Fusabi MCP Server
//!
//! This example demonstrates how to use the MCP server programmatically,
//! sending JSON-RPC requests and receiving responses.
//!
//! To run: cargo run --example basic_usage

use serde_json::json;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Fusabi MCP Server - Basic Usage Example\n");

    // Start the MCP server process
    let mut child = Command::new("cargo")
        .args(&["run", "--bin", "fusabi-mcp"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let mut stdin = child.stdin.take().expect("Failed to open stdin");
    let stdout = child.stdout.take().expect("Failed to open stdout");
    let mut reader = BufReader::new(stdout);

    // Helper function to send request and receive response
    let mut send_request = |request: serde_json::Value| -> Result<String, Box<dyn std::error::Error>> {
        let request_str = serde_json::to_string(&request)?;
        eprintln!("→ Sending: {}", request_str);

        writeln!(stdin, "{}", request_str)?;
        stdin.flush()?;

        let mut response = String::new();
        reader.read_line(&mut response)?;
        eprintln!("← Received: {}\n", response.trim());

        Ok(response)
    };

    // 1. Initialize the server
    println!("1. Initializing server...");
    let init_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {}
    });
    send_request(init_request)?;

    // 2. List available tools
    println!("2. Listing available tools...");
    let list_request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    });
    let tools_response = send_request(list_request)?;
    let tools: serde_json::Value = serde_json::from_str(&tools_response)?;
    println!("   Available tools: {}\n",
        tools["result"]["tools"]
            .as_array()
            .map(|arr| arr.len())
            .unwrap_or(0)
    );

    // 3. Execute a simple script
    println!("3. Executing simple arithmetic: 42 + 58");
    let eval_request = json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "eval_fusabi",
            "arguments": {
                "script": "42 + 58"
            }
        }
    });
    send_request(eval_request)?;

    // 4. Execute a function definition
    println!("4. Executing function: let add x y = x + y in add 20 22");
    let func_request = json!({
        "jsonrpc": "2.0",
        "id": 4,
        "method": "tools/call",
        "params": {
            "name": "eval_fusabi",
            "arguments": {
                "script": "let add x y = x + y in add 20 22"
            }
        }
    });
    send_request(func_request)?;

    // 5. Execute a list operation
    println!("5. Executing list operation: [1; 2; 3; 4; 5]");
    let list_request = json!({
        "jsonrpc": "2.0",
        "id": 5,
        "method": "tools/call",
        "params": {
            "name": "eval_fusabi",
            "arguments": {
                "script": "[1; 2; 3; 4; 5]"
            }
        }
    });
    send_request(list_request)?;

    // 6. Get runtime context
    println!("6. Getting runtime context...");
    let context_request = json!({
        "jsonrpc": "2.0",
        "id": 6,
        "method": "tools/call",
        "params": {
            "name": "get_context",
            "arguments": {}
        }
    });
    send_request(context_request)?;

    // Cleanup: terminate the server
    drop(stdin);
    child.wait()?;

    println!("\n✓ All examples completed successfully!");
    Ok(())
}
