# FSRS Embedding Guide

**Complete guide to embedding FSRS in your Rust applications**

## Table of Contents

1. [Introduction](#introduction)
2. [Quick Start](#quick-start)
3. [Architecture Overview](#architecture-overview)
4. [Host Function Registration](#host-function-registration)
5. [Value Marshalling](#value-marshalling)
6. [Configuration Patterns](#configuration-patterns)
7. [Error Handling](#error-handling)
8. [Best Practices](#best-practices)
9. [Examples](#examples)
10. [Advanced Topics](#advanced-topics)

---

## Introduction

FSRS (F# Script Runtime System) is designed to be embedded in Rust applications as a configuration and scripting layer. This guide shows you how to integrate FSRS into your application to provide users with a powerful, type-safe scripting API.

### Why FSRS for Embedding?

- **Type Safety**: Hindley-Milner type inference catches errors at compile time
- **Functional Programming**: Clean, expressive syntax for configuration
- **Performance**: Bytecode VM with efficient execution
- **Rust Integration**: Seamless interop with Rust types and functions
- **Hot-Reload Ready**: Designed for live configuration updates

### Common Use Cases

- **Terminal Emulators**: WezTerm-style configuration (see example)
- **Game Scripting**: Level logic, AI behaviors, game rules
- **Plugin Systems**: Extensible applications with user scripts
- **Configuration Files**: Complex configuration with logic and computation
- **Build Tools**: Custom build scripts and automation
- **Testing Frameworks**: Test case definitions and assertions

---

## Quick Start

### 1. Add Dependencies

```toml
[dependencies]
fsrs-frontend = { path = "../fsrs-frontend" }
fsrs-vm = { path = "../fsrs-vm" }
fsrs-demo = { path = "../fsrs-demo" }
```

### 2. Create Engine

```rust
use fsrs_demo::FsrsEngine;

let mut engine = FsrsEngine::new();
```

### 3. Register Host Functions

```rust
use fsrs_vm::{Value, VmError};

engine.register_fn1("greet", |v| {
    let name = v.as_str()
        .ok_or_else(|| VmError::Runtime("Expected string".into()))?;
    println!("Hello, {}!", name);
    Ok(Value::Unit)
});
```

### 4. Execute Script

```rust
use fsrs_frontend::{Lexer, Parser, Compiler};

let source = r#"greet "World""#;

let mut lexer = Lexer::new(source);
let tokens = lexer.tokenize()?;
let mut parser = Parser::new(tokens);
let ast = parser.parse()?;
let chunk = Compiler::compile(&ast)?;

// Execute (host integration pending VM update)
// vm.execute_with_host(chunk, &engine.host_registry)?;
```

### 5. Complete Example

See `examples/terminal_config/simple_demo.rs` for a working example.

---

## Architecture Overview

### Component Stack

```
┌─────────────────────────────────────┐
│     Your Rust Application          │
│  (Terminal, Game, Plugin System)   │
└─────────────────────────────────────┘
                 ↓
┌─────────────────────────────────────┐
│        FsrsEngine                   │
│  • Host Function Registry           │
│  • Global Bindings                  │
│  • Type Marshalling                 │
└─────────────────────────────────────┘
                 ↓
┌─────────────────────────────────────┐
│        FSRS Frontend                │
│  • Lexer → Tokens                   │
│  • Parser → AST                     │
│  • Type Inference (optional)        │
│  • Compiler → Bytecode              │
└─────────────────────────────────────┘
                 ↓
┌─────────────────────────────────────┐
│         FSRS VM                     │
│  • Bytecode Interpreter             │
│  • Value Stack                      │
│  • Call Frames                      │
│  • Host Function Calls              │
└─────────────────────────────────────┘
```

### Data Flow

```
User Script (config.fsrs)
    ↓ [Lexer]
Tokens
    ↓ [Parser]
AST (Abstract Syntax Tree)
    ↓ [Type Checker - optional]
Typed AST
    ↓ [Compiler]
Bytecode Chunk
    ↓ [VM]
Execution
    ↓ [Host Functions]
Your Rust Application
```

---

## Host Function Registration

### Basic Registration

#### Nullary Function (0 arguments)

```rust
engine.register_fn0("getCurrentTime", || {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    Ok(Value::Int(timestamp))
});
```

#### Unary Function (1 argument)

```rust
engine.register_fn1("log", |v| {
    println!("{:?}", v);
    Ok(Value::Unit)
});

engine.register_fn1("uppercase", |v| {
    let s = v.as_str()
        .ok_or_else(|| VmError::Runtime("Expected string".into()))?;
    Ok(Value::Str(s.to_uppercase()))
});
```

#### Binary Function (2 arguments)

```rust
engine.register_fn2("add", |a, b| {
    let x = a.as_int()
        .ok_or_else(|| VmError::Runtime("First arg must be int".into()))?;
    let y = b.as_int()
        .ok_or_else(|| VmError::Runtime("Second arg must be int".into()))?;
    Ok(Value::Int(x + y))
});
```

#### Ternary Function (3 arguments)

```rust
engine.register_fn3("clamp", |val, min, max| {
    let v = val.as_int().ok_or_else(|| VmError::Runtime("Expected int".into()))?;
    let mn = min.as_int().ok_or_else(|| VmError::Runtime("Expected int".into()))?;
    let mx = max.as_int().ok_or_else(|| VmError::Runtime("Expected int".into()))?;
    Ok(Value::Int(v.clamp(mn, mx)))
});
```

#### Variadic Function (variable arguments)

```rust
engine.register("sum", |args| {
    let total: i64 = args.iter()
        .filter_map(|v| v.as_int())
        .sum();
    Ok(Value::Int(total))
});

engine.register("concat", |args| {
    let result: String = args.iter()
        .map(|v| v.as_str().unwrap_or(""))
        .collect();
    Ok(Value::Str(result))
});
```

### Stateful Functions

Use closures to capture state:

```rust
use std::sync::{Arc, Mutex};

let counter = Arc::new(Mutex::new(0i64));
let counter_clone = Arc::clone(&counter);

engine.register_fn0("nextId", move || {
    let mut count = counter_clone.lock().unwrap();
    *count += 1;
    Ok(Value::Int(*count))
});
```

### Shared Application State

```rust
use std::rc::Rc;
use std::cell::RefCell;

struct AppState {
    tabs: Vec<Tab>,
    active_index: usize,
}

let state = Rc::new(RefCell::new(AppState::new()));

// Clone Rc for each closure
{
    let state = Rc::clone(&state);
    engine.register_fn1("createTab", move |v| {
        let title = v.as_str().unwrap();
        let id = state.borrow_mut().create_tab(title);
        Ok(Value::Int(id))
    });
}

{
    let state = Rc::clone(&state);
    engine.register_fn1("closeTab", move |v| {
        let id = v.as_int().unwrap();
        state.borrow_mut().close_tab(id);
        Ok(Value::Unit)
    });
}
```

---

## Value Marshalling

### FSRS Value Types

```rust
pub enum Value {
    Int(i64),           // 64-bit signed integer
    Bool(bool),         // Boolean
    Str(String),        // String
    Unit,               // Unit/void type
    Tuple(Vec<Value>),  // Tuple of values
    Cons { .. },        // List cons cell
    Nil,                // Empty list
    Array(..),          // Mutable array
    Record(..),         // Record/struct
    Variant { .. },     // Discriminated union
}
```

### Extracting Values from FSRS

```rust
// Integer
let n = value.as_int()
    .ok_or_else(|| VmError::Runtime("Expected int".into()))?;

// Boolean
let b = value.as_bool()
    .ok_or_else(|| VmError::Runtime("Expected bool".into()))?;

// String
let s = value.as_str()
    .ok_or_else(|| VmError::Runtime("Expected string".into()))?;

// Tuple
let tuple = value.as_tuple()
    .ok_or_else(|| VmError::Runtime("Expected tuple".into()))?;

// List
let list = value.list_to_vec()
    .ok_or_else(|| VmError::Runtime("Expected list".into()))?;
```

### Creating Values for FSRS

```rust
// Primitives
let int_val = Value::Int(42);
let bool_val = Value::Bool(true);
let str_val = Value::Str("hello".to_string());
let unit_val = Value::Unit;

// Tuple
let tuple_val = Value::Tuple(vec![
    Value::Int(1),
    Value::Str("two".to_string()),
    Value::Bool(true),
]);

// List
let list_val = Value::vec_to_cons(vec![
    Value::Int(1),
    Value::Int(2),
    Value::Int(3),
]);

// Record
use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

let mut fields = HashMap::new();
fields.insert("name".to_string(), Value::Str("Alice".to_string()));
fields.insert("age".to_string(), Value::Int(30));
let record_val = Value::Record(Rc::new(RefCell::new(fields)));
```

### Type Conversion Patterns

```rust
// Safe unwrapping with default
let n = value.as_int().unwrap_or(0);
let s = value.as_str().unwrap_or("default");
let b = value.as_bool().unwrap_or(false);

// Pattern matching
match value {
    Value::Int(n) => println!("Number: {}", n),
    Value::Str(s) => println!("String: {}", s),
    Value::Bool(b) => println!("Boolean: {}", b),
    Value::Tuple(elements) => {
        println!("Tuple with {} elements", elements.len());
    }
    _ => println!("Other type"),
}

// Validation helper
fn expect_int(value: Value) -> Result<i64, VmError> {
    value.as_int()
        .ok_or_else(|| VmError::Runtime(format!(
            "Expected int, got {}",
            value.type_name()
        )))
}
```

---

## Configuration Patterns

### Pattern 1: Simple Configuration

**config.fsrs:**
```fsharp
// Simple key-value configuration
let fontSize = 14
let fontFamily = "Fira Code"
let theme = "dark"
let enableLigatures = true

// Export config
{ fontSize; fontFamily; theme; enableLigatures }
```

**Rust:**
```rust
let config = engine.get_global("fontSize")?;
let font_size = config.as_int()?;
```

### Pattern 2: Computed Configuration

**config.fsrs:**
```fsharp
// Configuration with computation
let environment = getEnv "APP_ENV"

let databaseUrl =
    if environment = "production" then
        "postgres://prod-db:5432/app"
    else
        "postgres://localhost:5432/app_dev"

let maxConnections =
    if environment = "production" then 100 else 10

{ databaseUrl; maxConnections }
```

### Pattern 3: Modular Configuration

**config.fsrs:**
```fsharp
module Database =
    let host = "localhost"
    let port = 5432
    let name = "myapp"

    let connectionString =
        concat "postgres://" host ":" (intToString port) "/" name

module Logging =
    let level = "info"
    let format = "json"
    let destination = "stdout"

// Export modules
{ database = Database.connectionString
  logging = { level = Logging.level
              format = Logging.format } }
```

### Pattern 4: Conditional Features

**config.fsrs:**
```fsharp
let isDebugMode = getEnv "DEBUG" = "1"

let features = {
    enableMetrics = isDebugMode;
    verboseLogging = isDebugMode;
    enableCache = not isDebugMode;
    cacheTimeout = if isDebugMode then 60 else 3600
}

features
```

### Pattern 5: Dynamic Tab Configuration

**config.fsrs:**
```fsharp
module TabManager =
    let createWorkspaceTabs projectName =
        let srcTab = createTab (concat projectName "/src")
        let testTab = createTab (concat projectName "/tests")
        let docsTab = createTab (concat projectName "/docs")
        [srcTab; testTab; docsTab]

let projectTabs = TabManager.createWorkspaceTabs "myproject"
projectTabs
```

---

## Error Handling

### Host Function Errors

```rust
// Return VmError for script-visible errors
engine.register_fn1("divide", |v| {
    let n = v.as_int()
        .ok_or_else(|| VmError::Runtime("Expected int".into()))?;

    if n == 0 {
        return Err(VmError::DivisionByZero);
    }

    Ok(Value::Int(100 / n))
});
```

### Rust Error Propagation

```rust
use std::io;

// Convert Rust errors to VmError
engine.register_fn1("readFile", |v| {
    let path = v.as_str()
        .ok_or_else(|| VmError::Runtime("Expected string".into()))?;

    let content = std::fs::read_to_string(path)
        .map_err(|e| VmError::Runtime(format!("IO error: {}", e)))?;

    Ok(Value::Str(content))
});
```

### Error Recovery

```rust
// Graceful degradation
let result = engine.call_host("riskyOperation", &[]);
let value = match result {
    Ok(v) => v,
    Err(e) => {
        eprintln!("Operation failed: {}", e);
        Value::Unit // Default value
    }
};
```

### Validation Helpers

```rust
fn validate_args(args: &[Value], expected: usize) -> Result<(), VmError> {
    if args.len() != expected {
        return Err(VmError::Runtime(format!(
            "Expected {} arguments, got {}",
            expected,
            args.len()
        )));
    }
    Ok(())
}

engine.register("myFunc", |args| {
    validate_args(args, 2)?;
    let a = args[0].as_int().ok_or_else(|| VmError::Runtime("Arg 0 must be int".into()))?;
    let b = args[1].as_str().ok_or_else(|| VmError::Runtime("Arg 1 must be string".into()))?;
    // ... function body
    Ok(Value::Unit)
});
```

---

## Best Practices

### 1. Namespace Host Functions

Use prefixes to organize functions:

```rust
// Good
engine.register_fn1("Tab.create", ...);
engine.register_fn1("Tab.close", ...);
engine.register_fn1("Window.split", ...);
engine.register_fn1("Window.resize", ...);

// Bad
engine.register_fn1("create", ...);  // Ambiguous
engine.register_fn1("close", ...);   // Ambiguous
```

### 2. Provide Helper Functions

Make common operations easy:

```rust
// String manipulation
engine.register("concat", ...);
engine.register_fn1("uppercase", ...);
engine.register_fn1("lowercase", ...);

// List operations
engine.register_fn1("length", ...);
engine.register_fn2("nth", ...);

// Type conversions
engine.register_fn1("intToString", ...);
engine.register_fn1("stringToInt", ...);
```

### 3. Document Host API

```rust
/// Creates a new tab with the specified title.
///
/// # Arguments
/// - title: String - The tab title
///
/// # Returns
/// Int - The new tab ID
///
/// # Example
/// ```fsharp
/// let tab = createTab "Terminal"
/// ```
engine.register_fn1("createTab", ...);
```

### 4. Use Type-Safe Wrappers

```rust
struct TabId(i64);

impl TabId {
    fn from_value(v: Value) -> Result<Self, VmError> {
        v.as_int()
            .map(TabId)
            .ok_or_else(|| VmError::Runtime("Expected TabId (int)".into()))
    }

    fn to_value(&self) -> Value {
        Value::Int(self.0)
    }
}
```

### 5. Validate Configuration

```rust
// Validate loaded configuration
fn validate_config(engine: &FsrsEngine) -> Result<(), String> {
    let font_size = engine.get_global("fontSize")
        .and_then(|v| v.as_int())
        .ok_or("Missing or invalid fontSize")?;

    if font_size < 8 || font_size > 72 {
        return Err(format!("fontSize {} out of range (8-72)", font_size));
    }

    Ok(())
}
```

### 6. Provide Good Error Messages

```rust
// Bad
Err(VmError::Runtime("Error".into()))

// Good
Err(VmError::Runtime(format!(
    "Failed to create tab '{}': maximum of {} tabs reached",
    title, MAX_TABS
)))
```

---

## Examples

### Complete Terminal Configuration Example

See `examples/terminal_config/` for a production-ready example showing:

- Host function registration
- Shared mutable state
- Type marshalling
- Modular FSRS configuration
- Error handling
- Unit tests

### Minimal Embedding Example

```rust
use fsrs_demo::FsrsEngine;
use fsrs_vm::{Value, VmError};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut engine = FsrsEngine::new();

    // Register a simple function
    engine.register_fn1("greet", |v| {
        let name = v.as_str().unwrap_or("World");
        println!("Hello, {}!", name);
        Ok(Value::Unit)
    });

    // Call it
    engine.call_host("greet", &[Value::Str("FSRS".to_string())])?;

    Ok(())
}
```

---

## Advanced Topics

### Hot-Reload Support

*(Coming in Phase 3)*

```rust
use notify::{Watcher, RecursiveMode};

let mut engine = FsrsEngine::new();
// ... register functions ...

// Watch config file
let (tx, rx) = channel();
let mut watcher = notify::watcher(tx, Duration::from_millis(100))?;
watcher.watch("config.fsrs", RecursiveMode::NonRecursive)?;

// Reload on change
loop {
    match rx.recv() {
        Ok(event) => {
            println!("Reloading configuration...");
            engine.reload_config("config.fsrs")?;
        }
        Err(e) => eprintln!("Watch error: {}", e),
    }
}
```

### Precompiled Scripts

```rust
// Compile once
let bytecode = compile_script("config.fsrs")?;
std::fs::write("config.fsrs.bin", bytecode)?;

// Load precompiled
let bytecode = std::fs::read("config.fsrs.bin")?;
engine.load_compiled(&bytecode)?;
```

### Sandboxing

```rust
// Restrict available host functions
let mut sandbox_engine = FsrsEngine::new();

// Only allow safe operations
sandbox_engine.register_fn1("log", ...);
sandbox_engine.register("concat", ...);

// Don't register file I/O, network, etc.
```

### Performance Optimization

```rust
// Cache frequently accessed globals
let font_size = engine.get_global("fontSize")?;
// Cache it instead of repeated lookups

// Batch operations
let tab_ids: Vec<i64> = (0..10)
    .map(|i| {
        engine.call_host("createTab", &[
            Value::Str(format!("Tab {}", i))
        ])
        .and_then(|v| v.as_int().ok_or_else(|| "...".into()))
    })
    .collect::<Result<Vec<_>, _>>()?;
```

---

## Troubleshooting

### Common Issues

**Problem: "Host function not found"**
```rust
// Solution: Ensure function is registered before use
engine.register_fn1("myFunc", ...);
```

**Problem: "Type mismatch" errors**
```rust
// Solution: Use proper type checking
let n = value.as_int()
    .ok_or_else(|| VmError::Runtime("Expected int".into()))?;
```

**Problem: "Borrow checker errors with Rc<RefCell>"**
```rust
// Solution: Keep borrows short
{
    let mut state = shared_state.borrow_mut();
    state.modify();
} // Borrow dropped here
```

### Debugging

```rust
// Enable verbose logging
engine.register_fn1("debug", |v| {
    println!("[DEBUG] {:?}", v);
    Ok(Value::Unit)
});

// Inspect engine state
println!("Host functions: {:?}", engine.host_function_names());
println!("Globals: {:?}", engine.get_all_globals());
```

---

## Next Steps

1. **Try the examples**: Run `cargo run --example terminal_config`
2. **Read the docs**: Check `/docs/` for detailed specifications
3. **Explore patterns**: See `/examples/` for more use cases
4. **Build your integration**: Start with simple host functions
5. **Add complexity gradually**: Modules, state, hot-reload

---

## Resources

- **Project Repository**: https://github.com/raibid-labs/fsrs
- **Language Specification**: `/docs/02-language-spec.md`
- **Host Interop Design**: `/docs/HOST_INTEROP.md`
- **VM Architecture**: `/docs/03-vm-design.md`
- **Example Code**: `/examples/terminal_config/`

---

**Happy Embedding!** 🚀
