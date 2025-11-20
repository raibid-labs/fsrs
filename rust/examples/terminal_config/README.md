# Terminal Emulator Configuration Demo

**Production-ready FSRS embedding example showcasing host interop in a real-world terminal emulator configuration system.**

## Quick Start

### Running the Demo

```bash
# From the rust/ directory
cargo run --package fsrs-demo --example simple_terminal_demo

# Or run tests
cargo test --package fsrs-demo --example simple_terminal_demo
```

## Overview

This example demonstrates how to embed FSRS in a Rust application to provide a powerful, type-safe scripting API for user configuration - similar to WezTerm's Lua configuration.

### Working Demo

The `simple_terminal_demo.rs` example (located in `/crates/fsrs-demo/examples/`) provides a fully functional demonstration of:

- **Host Function Registration**: Exposing Rust functions to FSRS scripts
- **Shared Mutable State**: Using `Arc<Mutex<T>>` for thread-safe state management
- **Type-Safe Marshalling**: Automatic value conversion between Rust and FSRS
- **Error Handling**: Production-quality error propagation
- **Production Patterns**: Real-world configuration scenarios

## Demo Output

```
╔═══════════════════════════════════════════════════════════════╗
║   FSRS Terminal Configuration Demo                           ║
║   Host Interop Example                                       ║
╚═══════════════════════════════════════════════════════════════╝

═══ Demo 1: Basic Configuration ===

Executing configuration via host functions...

  [Host] Created tab 'Terminal' with ID 1
  [Host] Created tab 'Editor' with ID 2
  [Host] Created tab 'Logs' with ID 3

=== Current Tabs ===
  0: Terminal (ID: 1) [ACTIVE]
  1: Editor (ID: 2)
  2: Logs (ID: 3)

...
```

## Features Demonstrated

### 1. Host Function Registration

```rust
fn register_terminal_api(engine: &mut FsrsEngine, state: Arc<Mutex<TerminalState>>) {
    // Create tab function
    let state = Arc::clone(&state);
    engine.register_fn1("createTab", move |v| {
        let title = v.as_str()
            .ok_or_else(|| VmError::Runtime("createTab expects string".into()))?;
        let tab_id = state.lock().unwrap().create_tab(title);
        Ok(Value::Int(tab_id))
    });

    // Additional functions...
}
```

### 2. Shared State Management

```rust
// Create shared state
let state = Arc::new(Mutex::new(TerminalState::new()));

// Clone for each host function
let state_clone = Arc::clone(&state);

// Access from Rust
state.lock().unwrap().list_tabs();
```

### 3. Type-Safe Value Marshalling

```rust
// Extract values
let title = value.as_str()
    .ok_or_else(|| VmError::Runtime("Expected string".into()))?;

// Return values
Ok(Value::Int(tab_id))
Ok(Value::Bool(success))
Ok(Value::Str(message))
```

### 4. Comprehensive Tests

```rust
#[test]
fn test_create_tab_via_host() {
    let state = Arc::new(Mutex::new(TerminalState::new()));
    let mut engine = FsrsEngine::new();
    register_terminal_api(&mut engine, Arc::clone(&state));

    let result = engine
        .call_host("createTab", &[Value::Str("Test".to_string())])
        .unwrap();

    assert_eq!(result, Value::Int(1));
    assert_eq!(state.lock().unwrap().tabs.len(), 1);
}
```

## Project Structure

```
examples/terminal_config/
├── config.fsrs                              # Example FSRS configuration
├── README.md                                # This file
└── /crates/fsrs-demo/examples/
    ├── simple_terminal_demo.rs              # Working demo
    └── terminal_config.rs                   # Full VM integration (future)
```

## Host API

### Tab Management

```rust
createTab(title: string) -> int
```
Creates a new tab. Returns the tab ID.

```rust
closeTab(tabId: int) -> bool
```
Closes the specified tab. Returns success status.

```rust
log(value: any) -> unit
```
Logs a value to the console.

```rust
concat(parts: string...) -> string
```
Concatenates multiple strings.

## Configuration Examples

### Basic Usage

```fsharp
// Create tabs
let terminal = createTab "Terminal"
let editor = createTab "Editor"
let logs = createTab "Logs"

// Manage tabs
closeTab logs

// Return configuration
{ terminal; editor }
```

### Advanced Usage

```fsharp
// Modular configuration
module TabManager =
    let createWorkspace name =
        let src = createTab (concat name "/src")
        let tests = createTab (concat name "/tests")
        [src; tests]

let tabs = TabManager.createWorkspace "myproject"
tabs
```

## Testing

Run all tests:

```bash
cargo test --package fsrs-demo --example simple_terminal_demo

# Run with output
cargo test --package fsrs-demo --example simple_terminal_demo -- --nocapture
```

### Test Coverage

- ✓ Terminal state operations
- ✓ Host function registration
- ✓ Value marshalling
- ✓ Error handling
- ✓ Bulk operations

## Next Steps

### Immediate Extensions

1. **VM Integration**: Integrate host registry with VM execution
2. **FSRS Scripts**: Load and execute actual FSRS configuration files
3. **Hot-Reload**: Add file watching for live configuration updates
4. **Tab Formatters**: Call FSRS functions from Rust to format tab titles

### Production Features

1. **Configuration Validation**: Validate loaded configuration
2. **Error Recovery**: Graceful fallback to default configuration
3. **Keybinding Integration**: Wire up actual keyboard events
4. **Color Scheme Application**: Apply colors to terminal UI
5. **Persistent State**: Save configuration between sessions

## Documentation

- **Embedding Guide**: `/docs/EMBEDDING_GUIDE.md`
- **Terminal Demo Walkthrough**: `/docs/terminal_demo_walkthrough.md`
- **Host Interop Design**: `/docs/HOST_INTEROP.md`
- **Language Specification**: `/docs/02-language-spec.md`

## Implementation Notes

### Thread Safety

The demo uses `Arc<Mutex<T>>` for shared state because host functions require `Send + Sync` bounds. This ensures thread-safe access to the terminal state.

### Error Handling

All host functions return `Result<Value, VmError>`, enabling proper error propagation from Rust to FSRS scripts.

### Value Conversion

The `Value` enum provides type-safe conversion methods:
- `as_int()` → `Option<i64>`
- `as_str()` → `Option<&str>`
- `as_bool()` → `Option<bool>`

## Performance Considerations

- **Locking Strategy**: Keep lock holds short to avoid contention
- **Value Cloning**: Minimize unnecessary clones
- **Batch Operations**: Group related operations together

## Common Patterns

### Pattern 1: Closure Capture

```rust
{
    let state = Arc::clone(&state);
    engine.register_fn1("funcName", move |v| {
        let mut state = state.lock().unwrap();
        // Use state...
        Ok(Value::Unit)
    });
}
```

### Pattern 2: Type Extraction

```rust
let value = arg.as_type()
    .ok_or_else(|| VmError::Runtime(
        format!("Expected {}, got {}", expected, arg.type_name())
    ))?;
```

### Pattern 3: Short Lock Scopes

```rust
// Good
let result = {
    let state = state.lock().unwrap();
    state.get_value()
};

// Bad - holds lock too long
let state = state.lock().unwrap();
do_lots_of_work();
state.set_value(x);
```

## Troubleshooting

### Compilation Errors

**Error: "cannot be sent between threads safely"**
- Solution: Use `Arc` instead of `Rc`

**Error: "cannot be shared between threads safely"**
- Solution: Use `Mutex` instead of `RefCell`

### Runtime Errors

**Error: "Host function not found"**
- Solution: Ensure function is registered before calling

**Error: "Expected X, got Y"**
- Solution: Add proper type checking in host functions

## Resources

- **Working Example**: `/crates/fsrs-demo/examples/simple_terminal_demo.rs`
- **FSRS Config**: `/examples/terminal_config/config.fsrs`
- **Documentation**: `/docs/EMBEDDING_GUIDE.md`

## Contributing

This demo is part of the FSRS project Phase 3. Contributions and feedback welcome!

---

**Built with FSRS** - F# Script Runtime System
