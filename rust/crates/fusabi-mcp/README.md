# Fusabi MCP Server

A [Model Context Protocol (MCP)](https://modelcontextprotocol.io) server for the Fusabi scripting language, enabling AI assistants like Claude to execute F# scripts and interact with the Fusabi runtime.

## Features

- **Execute Fusabi Scripts**: Run F#-like code via the `eval_fusabi` tool
- **Query Runtime Context**: Get information about available host functions via `get_context`
- **Timeout Protection**: Automatic 5-second timeout for long-running scripts
- **JSON-RPC Protocol**: Standard MCP communication over stdin/stdout

## Installation

### From Source

```bash
cargo build --release -p fusabi-mcp
```

The binary will be available at `target/release/fusabi-mcp`.

### Add to PATH

```bash
# Copy to your local bin directory
cp target/release/fusabi-mcp ~/.local/bin/

# Or create a symlink
ln -s $(pwd)/target/release/fusabi-mcp ~/.local/bin/fusabi-mcp
```

## Configuration

### Claude Desktop

Add the Fusabi MCP server to your Claude Desktop configuration:

**macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
**Windows**: `%APPDATA%\Claude\claude_desktop_config.json`
**Linux**: `~/.config/Claude/claude_desktop_config.json`

```json
{
  "mcpServers": {
    "fusabi": {
      "command": "/path/to/fusabi-mcp"
    }
  }
}
```

Replace `/path/to/fusabi-mcp` with the actual path to the binary.

### Example Configuration

```json
{
  "mcpServers": {
    "fusabi": {
      "command": "/home/user/.local/bin/fusabi-mcp"
    }
  }
}
```

## Available Tools

### 1. eval_fusabi

Execute a Fusabi (F#-like) script and return the result.

**Parameters:**
- `script` (string): The Fusabi script to execute

**Examples:**

```fsharp
(* Simple arithmetic *)
42 + 58

(* Let bindings *)
let x = 42 in x * 2

(* Functions *)
let add x y = x + y
let result = add 20 22
result

(* Lists *)
let numbers = [1; 2; 3; 4; 5]
List.map (fun x -> x * 2) numbers

(* Pattern matching *)
let rec factorial n =
    match n with
    | 0 | 1 -> 1
    | _ -> n * factorial (n - 1)
factorial 5

(* Records *)
let person = { name = "Alice"; age = 30 }
person.name
```

### 2. get_context

Get the current Fusabi runtime context, including all registered host functions.

**Parameters:** None

**Returns:**
- `host_functions`: Array of available host function names
- `timeout_seconds`: Script execution timeout in seconds
- `fusabi_version`: MCP server version

## Usage Examples

### Using with Claude

After configuring the MCP server, you can ask Claude to execute Fusabi scripts:

> "Use Fusabi to calculate the factorial of 10"

> "Create a Fusabi function that filters even numbers from a list"

> "Show me what host functions are available in Fusabi"

### Direct Usage (Testing)

You can test the server directly by sending JSON-RPC requests:

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | fusabi-mcp

echo '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' | fusabi-mcp

echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"eval_fusabi","arguments":{"script":"42 + 58"}}}' | fusabi-mcp
```

## Development

### Building

```bash
cargo build -p fusabi-mcp
```

### Running Tests

```bash
cargo test -p fusabi-mcp
```

### Running the Server

```bash
cargo run -p fusabi-mcp
```

## Architecture

The server implements the MCP protocol by:

1. **Reading** JSON-RPC requests from stdin
2. **Processing** requests through the Fusabi engine
3. **Writing** JSON-RPC responses to stdout
4. **Logging** diagnostics to stderr

The server maintains a Fusabi engine instance and executes each script in a separate tokio blocking task with timeout protection.

## Timeout Protection

Scripts have a default timeout of 5 seconds. If a script takes longer than this to execute, it will be automatically terminated:

```rust
let mut server = McpServer::with_timeout(Duration::from_secs(10)); // Custom timeout
```

## Error Handling

The server follows JSON-RPC 2.0 error codes:

- `-32700`: Parse error (invalid JSON)
- `-32600`: Invalid request
- `-32601`: Method not found
- `-32602`: Invalid params
- `-32603`: Internal error (execution failure)

## Security Considerations

- **Timeouts**: All script executions are protected by timeouts to prevent infinite loops
- **Sandboxing**: Scripts run in the Fusabi VM without direct system access
- **Isolation**: Each request creates a fresh engine instance

## License

MIT

## Related Projects

- [Fusabi](https://github.com/fusabi-lang/fusabi) - The Fusabi scripting language
- [MCP](https://modelcontextprotocol.io) - Model Context Protocol specification

## Contributing

Contributions are welcome! Please ensure:

- All tests pass: `cargo test -p fusabi-mcp`
- Code is formatted: `cargo fmt`
- No clippy warnings: `cargo clippy -p fusabi-mcp`

## Support

For issues or questions:
- Open an issue on [GitHub](https://github.com/fusabi-lang/fusabi/issues)
- Tag it with `mcp-server` label
