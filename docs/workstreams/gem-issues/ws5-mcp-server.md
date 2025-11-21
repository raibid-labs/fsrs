# Workstream 5: Ecosystem - MCP Server

## Status
🟡 Ready to Start (fully independent)

## Overview
Create a Model Context Protocol (MCP) server to allow AI agents like Claude to interact with a running Fusabi instance. This enables powerful AI-driven scripting and debugging workflows.

## Objectives
- [ ] Create new crate `fusabi-mcp` with MCP server
- [ ] Implement stdio-based MCP transport
- [ ] Expose tool: `eval_fusabi(script: string) -> string`
- [ ] Expose tool: `get_context() -> json` (dumps globals)
- [ ] Implement script execution with timeout
- [ ] Capture stdout from scripts
- [ ] Add error handling and safety measures

## Agent Assignment
**Suggested Agent Type**: `ai-engineer`, `backend-developer`, `rust-pro`
**Skill Requirements**: MCP protocol, Rust async, AI tooling, systems programming

## Dependencies
- None (new crate, fully independent)

## Tasks

### Task 5.1: Create New Crate fusabi-mcp
**Description**: Set up new Rust crate for MCP server.

**Deliverables**:
- New crate `rust/crates/fusabi-mcp`
- Basic project structure
- Dependencies: `serde`, `serde_json`, `tokio`, `fusabi`

**Files to Create/Modify**:
- `rust/crates/fusabi-mcp/Cargo.toml`
- `rust/crates/fusabi-mcp/src/main.rs`
- `rust/crates/fusabi-mcp/src/lib.rs`

**Implementation**:
```toml
# rust/crates/fusabi-mcp/Cargo.toml

[package]
name = "fusabi-mcp"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "fusabi-mcp"
path = "src/main.rs"

[dependencies]
fusabi = { path = "../fusabi" }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.35", features = ["full"] }
anyhow = "1.0"
```

```rust
// rust/crates/fusabi-mcp/src/lib.rs

pub mod server;
pub mod tools;
```

**Validation**:
```bash
cd rust/crates/fusabi-mcp
cargo build
# Should compile successfully
```

---

### Task 5.2: Implement MCP Protocol Types
**Description**: Define MCP protocol message types (JSON-RPC 2.0).

**Deliverables**:
- `Request`, `Response`, `Notification` types
- Tool definition structures
- JSON serialization/deserialization

**Files to Create/Modify**:
- `rust/crates/fusabi-mcp/src/protocol.rs`

**Implementation**:
```rust
// rust/crates/fusabi-mcp/src/protocol.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(tag = "method", content = "params")]
pub enum Request {
    #[serde(rename = "tools/list")]
    ListTools,

    #[serde(rename = "tools/call")]
    CallTool { name: String, arguments: serde_json::Value },
}

#[derive(Debug, Serialize)]
pub struct Response {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    pub result: Option<serde_json::Value>,
    pub error: Option<Error>,
}

#[derive(Debug, Serialize)]
pub struct Error {
    pub code: i32,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}
```

**Validation**:
```rust
#[test]
fn test_deserialize_request() {
    let json = r#"{"method": "tools/list", "params": {}}"#;
    let req: Request = serde_json::from_str(json).unwrap();
    assert!(matches!(req, Request::ListTools));
}
```

---

### Task 5.3: Implement Stdio Transport
**Description**: Implement stdio-based communication for MCP protocol.

**Deliverables**:
- Read JSON-RPC messages from stdin
- Write responses to stdout
- Handle line-delimited JSON
- Async I/O with tokio

**Files to Create/Modify**:
- `rust/crates/fusabi-mcp/src/transport.rs`

**Implementation**:
```rust
// rust/crates/fusabi-mcp/src/transport.rs

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::io::{stdin, stdout};

pub struct StdioTransport {
    reader: BufReader<tokio::io::Stdin>,
    writer: tokio::io::Stdout,
}

impl StdioTransport {
    pub fn new() -> Self {
        Self {
            reader: BufReader::new(stdin()),
            writer: stdout(),
        }
    }

    pub async fn read_message(&mut self) -> anyhow::Result<String> {
        let mut line = String::new();
        self.reader.read_line(&mut line).await?;
        Ok(line)
    }

    pub async fn write_message(&mut self, message: &str) -> anyhow::Result<()> {
        self.writer.write_all(message.as_bytes()).await?;
        self.writer.write_all(b"\n").await?;
        self.writer.flush().await?;
        Ok(())
    }
}
```

**Validation**:
```rust
#[tokio::test]
async fn test_stdio_transport() {
    // Create test message
    let request = r#"{"method": "tools/list", "params": {}}"#;

    // Would need to mock stdin/stdout for proper testing
    // For now, integration test by running the server
}
```

---

### Task 5.4: Implement eval_fusabi Tool
**Description**: Implement tool to evaluate Fusabi scripts.

**Deliverables**:
- `eval_fusabi(script: string) -> Result<string>`
- Execute script in isolated VM instance
- Capture stdout output
- Timeout mechanism (default 5 seconds)
- Error handling

**Files to Create/Modify**:
- `rust/crates/fusabi-mcp/src/tools/eval.rs`

**Implementation**:
```rust
// rust/crates/fusabi-mcp/src/tools/eval.rs

use fusabi::*;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::timeout;

pub struct EvalTool;

impl EvalTool {
    pub async fn execute(script: String) -> anyhow::Result<String> {
        // Capture stdout
        let output = Arc::new(Mutex::new(Vec::new()));
        let output_clone = output.clone();

        // Run with timeout
        let result = timeout(Duration::from_secs(5), async move {
            // Parse and compile script
            let ast = Parser::parse(&script)?;
            let mut compiler = Compiler::new();
            let chunk = compiler.compile_program(&ast)?;

            // Execute in VM
            let mut vm = Vm::new();

            // Redirect stdout (simplified - actual impl would use custom writer)
            let result = vm.execute(&chunk)?;

            Ok::<_, anyhow::Error>(format!("{:?}", result))
        }).await;

        match result {
            Ok(Ok(result)) => {
                let captured = String::from_utf8_lossy(&output.lock().unwrap());
                Ok(format!("{}\nResult: {}", captured, result))
            }
            Ok(Err(e)) => Err(anyhow::anyhow!("Execution error: {}", e)),
            Err(_) => Err(anyhow::anyhow!("Timeout: script took too long")),
        }
    }
}
```

**Validation**:
```rust
#[tokio::test]
async fn test_eval_simple_script() {
    let script = r#"printfn "Hello from Fusabi!""#;
    let result = EvalTool::execute(script.into()).await.unwrap();
    assert!(result.contains("Hello from Fusabi!"));
}

#[tokio::test]
async fn test_eval_with_error() {
    let script = "invalid syntax!!!";
    let result = EvalTool::execute(script.into()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_eval_timeout() {
    let script = "let rec infinite() = infinite() in infinite()";
    let result = EvalTool::execute(script.into()).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Timeout"));
}
```

---

### Task 5.5: Implement get_context Tool
**Description**: Implement tool to inspect VM global state.

**Deliverables**:
- `get_context() -> json` dumps all globals
- Return variable names and values
- Format as JSON for AI consumption

**Files to Create/Modify**:
- `rust/crates/fusabi-mcp/src/tools/context.rs`

**Implementation**:
```rust
// rust/crates/fusabi-mcp/src/tools/context.rs

use fusabi::*;
use serde_json::{json, Value};

pub struct ContextTool;

impl ContextTool {
    pub fn execute(vm: &Vm) -> anyhow::Result<Value> {
        let mut globals = serde_json::Map::new();

        for (name, value) in vm.globals.iter() {
            globals.insert(
                name.clone(),
                value_to_json(value),
            );
        }

        Ok(json!({
            "globals": globals,
            "stack_depth": vm.stack.len(),
        }))
    }
}

fn value_to_json(value: &fusabi::Value) -> Value {
    match value {
        fusabi::Value::Int(n) => json!(n),
        fusabi::Value::Float(f) => json!(f),
        fusabi::Value::Bool(b) => json!(b),
        fusabi::Value::String(s) => json!(s),
        fusabi::Value::Unit => json!(null),
        fusabi::Value::List(l) => {
            let items: Vec<_> = l.borrow().iter().map(value_to_json).collect();
            json!(items)
        }
        _ => json!("<complex value>"),
    }
}
```

**Validation**:
```rust
#[test]
fn test_get_context() {
    let mut vm = Vm::new();
    vm.globals.insert("x".into(), fusabi::Value::Int(42));
    vm.globals.insert("name".into(), fusabi::Value::String("Fusabi".into()));

    let context = ContextTool::execute(&vm).unwrap();
    assert_eq!(context["globals"]["x"], 42);
    assert_eq!(context["globals"]["name"], "Fusabi");
}
```

---

### Task 5.6: Implement MCP Server Loop
**Description**: Main server loop handling MCP requests.

**Deliverables**:
- Main server loop reading from stdin
- Dispatch requests to appropriate tools
- Send responses to stdout
- Error handling and logging

**Files to Create/Modify**:
- `rust/crates/fusabi-mcp/src/server.rs`

**Implementation**:
```rust
// rust/crates/fusabi-mcp/src/server.rs

use crate::protocol::*;
use crate::transport::StdioTransport;
use crate::tools::{EvalTool, ContextTool};
use serde_json::json;

pub struct McpServer {
    transport: StdioTransport,
    vm: fusabi::Vm,
}

impl McpServer {
    pub fn new() -> Self {
        Self {
            transport: StdioTransport::new(),
            vm: fusabi::Vm::new(),
        }
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        loop {
            let message = self.transport.read_message().await?;
            let request: Request = serde_json::from_str(&message)?;

            let response = self.handle_request(request).await;
            let response_json = serde_json::to_string(&response)?;

            self.transport.write_message(&response_json).await?;
        }
    }

    async fn handle_request(&mut self, request: Request) -> Response {
        match request {
            Request::ListTools => self.handle_list_tools(),
            Request::CallTool { name, arguments } => {
                self.handle_call_tool(&name, arguments).await
            }
        }
    }

    fn handle_list_tools(&self) -> Response {
        let tools = vec![
            Tool {
                name: "eval_fusabi".into(),
                description: "Execute Fusabi script and return result".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "script": {
                            "type": "string",
                            "description": "Fusabi script to execute"
                        }
                    },
                    "required": ["script"]
                }),
            },
            Tool {
                name: "get_context".into(),
                description: "Get current VM global variables".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {}
                }),
            },
        ];

        Response {
            jsonrpc: "2.0".into(),
            id: None,
            result: Some(json!({ "tools": tools })),
            error: None,
        }
    }

    async fn handle_call_tool(&mut self, name: &str, arguments: serde_json::Value) -> Response {
        let result = match name {
            "eval_fusabi" => {
                let script = arguments["script"].as_str().unwrap_or("");
                EvalTool::execute(script.into()).await
                    .map(|output| json!({ "output": output }))
            }
            "get_context" => {
                ContextTool::execute(&self.vm)
            }
            _ => Err(anyhow::anyhow!("Unknown tool: {}", name)),
        };

        match result {
            Ok(result) => Response {
                jsonrpc: "2.0".into(),
                id: None,
                result: Some(result),
                error: None,
            },
            Err(e) => Response {
                jsonrpc: "2.0".into(),
                id: None,
                result: None,
                error: Some(Error {
                    code: -32000,
                    message: e.to_string(),
                }),
            },
        }
    }
}
```

**Validation**:
Integration test by running the server and sending MCP requests manually.

---

### Task 5.7: Create Main Binary
**Description**: Create main binary entry point for MCP server.

**Deliverables**:
- `main.rs` that starts MCP server
- CLI arguments (optional: port, log level)
- Error handling and logging

**Files to Create/Modify**:
- `rust/crates/fusabi-mcp/src/main.rs`

**Implementation**:
```rust
// rust/crates/fusabi-mcp/src/main.rs

use fusabi_mcp::server::McpServer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    eprintln!("Fusabi MCP Server starting...");

    let mut server = McpServer::new();
    server.run().await?;

    Ok(())
}
```

**Validation**:
```bash
cd rust/crates/fusabi-mcp
cargo run
# Server starts and waits for stdin

# In another terminal, test with:
echo '{"method": "tools/list", "params": {}}' | cargo run
# Should return list of tools

echo '{"method": "tools/call", "params": {"name": "eval_fusabi", "arguments": {"script": "printfn \"Hello!\""}}}' | cargo run
# Should execute script and return output
```

---

### Task 5.8: Add Documentation and Examples
**Description**: Document MCP server usage and provide examples.

**Deliverables**:
- README for fusabi-mcp crate
- Usage examples
- Claude Desktop integration instructions

**Files to Create/Modify**:
- `rust/crates/fusabi-mcp/README.md`
- `docs/mcp-integration.md`

**Content**:
```markdown
# Fusabi MCP Server

Model Context Protocol server for Fusabi.

## Usage

Start the server:
```bash
fusabi-mcp
```

## Tools

- `eval_fusabi(script: string)`: Execute Fusabi script
- `get_context()`: Get VM global variables

## Claude Desktop Integration

Add to `claude_desktop_config.json`:
```json
{
  "mcpServers": {
    "fusabi": {
      "command": "fusabi-mcp"
    }
  }
}
```

## Examples

```json
{"method": "tools/call", "params": {"name": "eval_fusabi", "arguments": {"script": "let x = 42 in x * 2"}}}
```
```

**Validation**:
```bash
# Read README
cat rust/crates/fusabi-mcp/README.md
# Should be clear and comprehensive
```

---

## Definition of Done
- [ ] New crate `fusabi-mcp` created
- [ ] MCP protocol types implemented
- [ ] Stdio transport working
- [ ] `eval_fusabi` tool implemented with timeout
- [ ] `get_context` tool implemented
- [ ] MCP server loop handling requests
- [ ] Main binary running successfully
- [ ] Comprehensive tests passing
- [ ] Documentation complete (README, integration guide)
- [ ] Tested with Claude Desktop
- [ ] PR ready for review

## Agent Coordination Hooks
```bash
# BEFORE Work:
npx claude-flow@alpha hooks pre-task --description "ws5-mcp-server"
npx claude-flow@alpha hooks session-restore --session-id "swarm-fusabi-gem-ws5"

# DURING Work:
npx claude-flow@alpha hooks post-edit --file "rust/crates/fusabi-mcp/src/server.rs" --memory-key "swarm/fusabi-gem/ws5/mcp-server"
npx claude-flow@alpha hooks post-edit --file "rust/crates/fusabi-mcp/src/tools/eval.rs" --memory-key "swarm/fusabi-gem/ws5/eval-tool"
npx claude-flow@alpha hooks notify --message "MCP server complete"

# AFTER Work:
npx claude-flow@alpha hooks post-task --task-id "ws5-complete"
npx claude-flow@alpha hooks session-end --export-metrics true
```

## Estimated Effort
**Duration**: 2-3 days
**Complexity**: Medium

## References
- [Model Context Protocol Spec](https://modelcontextprotocol.io/)
- [JSON-RPC 2.0 Spec](https://www.jsonrpc.org/specification)
- [Tokio Async I/O](https://tokio.rs/)

## Notes
- **Security**: Scripts run with full VM access (no sandboxing yet). Document this clearly.
- **Timeout**: Default 5 seconds prevents infinite loops
- **Future Work**:
  - Resource limits (memory, CPU)
  - Persistent VM state across requests
  - Breakpoint/debugger support
  - Watch expressions

## File Conflicts
- **No conflicts** with other workstreams (new crate)
- Safe to run in parallel with all other workstreams
