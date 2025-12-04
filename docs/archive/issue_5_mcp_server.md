# Issue 5: [Ecosystem] Create Fusabi-MCP Server

**Labels:** `feature`, `area:ecosystem`, `good-first-issue`

## Context
Implement the Model Context Protocol (MCP) to allow AI agents (like Claude) to interact with a running Fusabi instance.

## Implementation Plan
**Objective:** Create a new crate `fusabi-mcp`.

1.  **New Crate**: `cargo new rust/crates/fusabi-mcp`.
2.  **Dependencies**: Add `serde`, `serde_json`, `tokio`, `fusabi`.
3.  **Server Implementation**:
    * Implement a stdio-based MCP server.
    * Expose tool: `eval_fusabi(script: string) -> string`.
    * Expose tool: `get_context() -> json` (dumps global variables).
4.  **Integration**:
    * In `eval_fusabi`, spin up a `fusabi::Engine`.
    * Execute the script with a timeout (safety).
    * Capture `stdout` from the script and return it as the tool result.
