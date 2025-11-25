//! Fusabi MCP Server Binary
//!
//! This executable implements a Model Context Protocol (MCP) server for Fusabi,
//! allowing AI assistants like Claude to execute F# scripts via standard I/O.
//!
//! # Usage
//!
//! ```bash
//! fusabi-mcp
//! ```
//!
//! The server reads JSON-RPC requests from stdin and writes responses to stdout.
//! All logging and diagnostics are sent to stderr.

use fusabi_mcp::McpServer;
use std::process;

#[tokio::main]
async fn main() {
    let mut server = McpServer::new();

    if let Err(e) = server.run().await {
        eprintln!("Server error: {}", e);
        process::exit(1);
    }
}
