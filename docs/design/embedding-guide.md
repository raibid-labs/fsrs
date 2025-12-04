# Fusabi Embedding Guide

## Production Deployment with Bytecode Compilation

This guide shows you how to embed Fusabi in production Rust applications using the bytecode compilation API. Perfect for observability agents, plugin systems, configuration engines, and other scenarios where startup performance matters.

## Table of Contents

1. [Quick Start](#quick-start)
2. [Bytecode Compilation API](#bytecode-compilation-api)
3. [Production Patterns](#production-patterns)
4. [Hibana Use Case: Observability Agent](#hibana-use-case)
5. [Performance Optimization](#performance-optimization)
6. [Error Handling](#error-handling)
7. [Security Considerations](#security-considerations)
8. [Best Practices](#best-practices)

---

## Quick Start

### Basic Embedding (Development Mode)

```rust
use fusabi::Engine;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut engine = Engine::new();

    // Register host functions
    engine.register_fn1("log", |msg| {
        println!("LOG: {:?}", msg);
        Ok(fusabi::Value::Unit)
    });

    // Execute script
    let result = engine.eval(r#"
        let process_event event =
            log ("Processing: " + event)
        in
        process_event "user_login"
    "#)?;

    Ok(())
}
```

### Production Mode with Bytecode

```rust
use fusabi::{compile_to_bytecode, execute_bytecode};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Development: Compile .fsx to .fzb
    let source = fs::read_to_string("config.fsx")?;
    let bytecode = compile_to_bytecode(&source)?;
    fs::write("config.fzb", &bytecode)?;

    // Production: Load and execute .fzb
    let bytecode = fs::read("config.fzb")?;
    let result = execute_bytecode(&bytecode)?;

    println!("Result: {:?}", result);
    Ok(())
}
```

---

## Bytecode Compilation API

### Core Functions

#### `compile_to_bytecode(source: &str) -> Result<Vec<u8>, FusabiError>`

Compiles Fusabi source code to bytecode.

```rust
use fusabi::compile_to_bytecode;

let source = r#"
    let factorial n =
        if n <= 1 then 1
        else n * factorial (n - 1)
    in
    factorial 10
"#;

let bytecode = compile_to_bytecode(source)?;
println!("Compiled {} bytes", bytecode.len());
```

**When to use:**
- Build-time compilation
- Dynamic script compilation
- Caching compiled scripts

#### `compile_file_to_bytecode(path: &str) -> Result<Vec<u8>, FusabiError>`

Compiles a .fsx file to bytecode.

```rust
use fusabi::compile_file_to_bytecode;
use std::fs;

// Compile and save
let bytecode = compile_file_to_bytecode("pipeline.fsx")?;
fs::write("pipeline.fzb", &bytecode)?;
```

**When to use:**
- CLI tools (e.g., `myapp compile script.fsx`)
- Build scripts
- Deployment automation

#### `execute_bytecode(bytecode: &[u8]) -> Result<Value, FusabiError>`

Executes compiled bytecode.

```rust
use fusabi::execute_bytecode;
use std::fs;

let bytecode = fs::read("script.fzb")?;
let result = execute_bytecode(&bytecode)?;
```

**When to use:**
- Production execution
- Loading pre-compiled scripts
- Low-latency startup

#### `compile_to_chunk(source: &str) -> Result<Chunk, FusabiError>`

Lower-level API returning internal `Chunk` representation.

```rust
use fusabi::compile_to_chunk;
use fusabi_vm::{Vm, serialize_chunk};

let chunk = compile_to_chunk("let x = 42 in x")?;

// Inspect
println!("Instructions: {}", chunk.instructions.len());
println!("Constants: {}", chunk.constants.len());

// Execute directly
let mut vm = Vm::new();
fusabi_vm::stdlib::register_stdlib(&mut vm);
let result = vm.execute(chunk)?;
```

**When to use:**
- Bytecode inspection/debugging
- Custom serialization
- VM integration

---

## Production Patterns

### 1. AOT Compilation Pattern

**Use case:** Compile scripts during build/deployment, ship only bytecode.

```rust
// build.rs or deployment script
use fusabi::compile_file_to_bytecode;
use std::fs;
use std::path::Path;

fn compile_all_scripts() -> Result<(), Box<dyn std::error::Error>> {
    let scripts = vec![
        "config/pipeline.fsx",
        "config/filters.fsx",
        "config/transforms.fsx",
    ];

    for script in scripts {
        let bytecode = compile_file_to_bytecode(script)?;

        let output = Path::new(script)
            .with_extension("fzb");

        fs::write(&output, &bytecode)?;
        println!("Compiled: {} -> {}", script, output.display());
    }

    Ok(())
}
```

**Deployment:**
```
# Build time
cargo build --release
./target/release/compile_scripts  # Generates .fzb files

# Production
# Ship only .fzb files, no source needed
```

### 2. JIT Cache Pattern

**Use case:** Compile on first run, cache bytecode for subsequent runs.

```rust
use fusabi::{compile_to_bytecode, execute_bytecode};
use std::fs;
use std::path::Path;

fn load_or_compile(
    source_path: &str,
    cache_path: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Check if cache exists and is fresh
    if Path::new(cache_path).exists() {
        let source_time = fs::metadata(source_path)?.modified()?;
        let cache_time = fs::metadata(cache_path)?.modified()?;

        if cache_time > source_time {
            println!("[CACHE HIT] Loading {}", cache_path);
            return Ok(fs::read(cache_path)?);
        }
    }

    // Cache miss or stale - recompile
    println!("[CACHE MISS] Compiling {}", source_path);
    let source = fs::read_to_string(source_path)?;
    let bytecode = compile_to_bytecode(&source)?;

    // Save to cache
    fs::write(cache_path, &bytecode)?;
    println!("[CACHED] Saved to {}", cache_path);

    Ok(bytecode)
}

// Usage
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bytecode = load_or_compile(
        "config/pipeline.fsx",
        "/tmp/fusabi_cache/pipeline.fzb",
    )?;

    let result = execute_bytecode(&bytecode)?;
    println!("Result: {:?}", result);
    Ok(())
}
```

### 3. Memory Cache Pattern

**Use case:** Hot paths requiring microsecond latency.

```rust
use fusabi::{compile_to_bytecode, execute_bytecode};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

struct BytecodeCache {
    cache: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl BytecodeCache {
    fn new() -> Self {
        BytecodeCache {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn get_or_compile(
        &self,
        key: &str,
        source: &str,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // Try read lock first
        if let Some(bytecode) = self.cache.read().ok()?.get(key).cloned() {
            return Ok(bytecode);
        }

        // Cache miss - compile
        let bytecode = compile_to_bytecode(source)?;

        // Store in cache
        if let Ok(mut cache) = self.cache.write() {
            cache.insert(key.to_string(), bytecode.clone());
        }

        Ok(bytecode)
    }

    fn execute(&self, key: &str, source: &str) -> Result<fusabi::Value, Box<dyn std::error::Error>> {
        let bytecode = self.get_or_compile(key, source)?;
        Ok(execute_bytecode(&bytecode)?)
    }
}

// Usage
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cache = BytecodeCache::new();

    // First call compiles
    let result = cache.execute("transform_1", "let x = 42 in x * 2")?;

    // Subsequent calls use cached bytecode (fast!)
    let result = cache.execute("transform_1", "let x = 42 in x * 2")?;

    Ok(())
}
```

### 4. Hybrid Pattern

**Use case:** Development mode uses source, production uses bytecode.

```rust
use fusabi::{compile_to_bytecode, execute_bytecode, run_file};
use std::fs;

fn execute_config(path: &str) -> Result<fusabi::Value, Box<dyn std::error::Error>> {
    // Check if bytecode version exists
    let bytecode_path = path.replace(".fsx", ".fzb");

    if cfg!(debug_assertions) {
        // Development: always use source for fast iteration
        println!("[DEV MODE] Executing source: {}", path);
        Ok(run_file(path)?)
    } else {
        // Production: use bytecode if available
        if fs::metadata(&bytecode_path).is_ok() {
            println!("[PROD MODE] Executing bytecode: {}", bytecode_path);
            let bytecode = fs::read(&bytecode_path)?;
            Ok(execute_bytecode(&bytecode)?)
        } else {
            // Fallback to source if bytecode not found
            eprintln!("Warning: Bytecode not found, falling back to source");
            Ok(run_file(path)?)
        }
    }
}
```

---

## Hibana Use Case

**Hibana** is an observability agent that uses Fusabi for pipeline configuration and data transformations.

### Architecture

```
┌─────────────────────────────────────────┐
│         Hibana Agent                    │
├─────────────────────────────────────────┤
│  ┌─────────────┐   ┌─────────────┐     │
│  │  Pipeline   │   │  Transform  │     │
│  │  Config     │   │  Scripts    │     │
│  │  (.fsx)     │   │  (.fsx)     │     │
│  └──────┬──────┘   └──────┬──────┘     │
│         │                 │             │
│         ▼                 ▼             │
│  ┌──────────────────────────────┐      │
│  │   Fusabi Bytecode Cache      │      │
│  │   (In-Memory + Disk)         │      │
│  └──────┬───────────────────────┘      │
│         │                               │
│         ▼                               │
│  ┌──────────────────────────────┐      │
│  │   Fusabi VM Pool (tokio)     │      │
│  └──────┬───────────────────────┘      │
│         │                               │
│         ▼                               │
│  ┌──────────────────────────────┐      │
│  │   Event Processing           │      │
│  └──────────────────────────────┘      │
└─────────────────────────────────────────┘
```

### Implementation

```rust
use fusabi::{compile_to_bytecode, execute_bytecode, Engine};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Hibana pipeline configuration
struct HibanaPipeline {
    bytecode_cache: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl HibanaPipeline {
    fn new() -> Self {
        HibanaPipeline {
            bytecode_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Load pipeline from source or cache
    async fn load_pipeline(
        &self,
        name: &str,
        source: &str,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // Check cache
        {
            let cache = self.bytecode_cache.read().await;
            if let Some(bytecode) = cache.get(name) {
                return Ok(bytecode.clone());
            }
        }

        // Compile and cache
        let bytecode = compile_to_bytecode(source)?;

        {
            let mut cache = self.bytecode_cache.write().await;
            cache.insert(name.to_string(), bytecode.clone());
        }

        Ok(bytecode)
    }

    /// Execute pipeline on event
    async fn process_event(
        &self,
        pipeline_name: &str,
        pipeline_source: &str,
        event_data: serde_json::Value,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        // Load bytecode (cached)
        let bytecode = self.load_pipeline(pipeline_name, pipeline_source).await?;

        // Execute in blocking task (Fusabi VM is not async)
        let result = tokio::task::spawn_blocking(move || {
            execute_bytecode(&bytecode)
        }).await??;

        // Convert result to JSON
        // (implement your conversion logic here)
        Ok(serde_json::json!({"result": format!("{:?}", result)}))
    }
}

// Example usage
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pipeline = HibanaPipeline::new();

    let filter_pipeline = r#"
        let filter_errors logs =
            logs |> List.filter (fun log ->
                log.severity = "ERROR"
            )
        in
        filter_errors
    "#;

    // Process event
    let event = serde_json::json!({
        "logs": [
            {"severity": "INFO", "msg": "ok"},
            {"severity": "ERROR", "msg": "fail"}
        ]
    });

    let result = pipeline.process_event(
        "filter_errors",
        filter_pipeline,
        event,
    ).await?;

    println!("Filtered: {:?}", result);
    Ok(())
}
```

### Hot Reload Support

```rust
use notify::{Watcher, RecursiveMode, watcher};
use std::sync::mpsc::channel;
use std::time::Duration;

fn watch_and_reload(
    pipeline: Arc<HibanaPipeline>,
    config_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = channel();
    let mut watcher = watcher(tx, Duration::from_secs(2))?;

    watcher.watch(config_dir, RecursiveMode::Recursive)?;

    loop {
        match rx.recv() {
            Ok(event) => {
                println!("File changed: {:?}", event);
                // Recompile and update cache
                // (implementation here)
            }
            Err(e) => println!("Watch error: {:?}", e),
        }
    }
}
```

---

## Performance Optimization

### Compilation Performance

```rust
use fusabi::compile_to_bytecode;
use std::time::Instant;

fn benchmark_compilation(source: &str) {
    let start = Instant::now();
    let bytecode = compile_to_bytecode(source).unwrap();
    let compile_time = start.elapsed();

    println!("Compilation: {:?}", compile_time);
    println!("Bytecode size: {} bytes", bytecode.len());
    println!("Compression ratio: {:.1}%",
             100.0 * bytecode.len() as f64 / source.len() as f64);
}
```

**Typical results:**
- Simple scripts: 1-5ms compilation
- Complex pipelines: 10-50ms compilation
- Bytecode size: 60-80% of source

### Execution Performance

```rust
use fusabi::{compile_to_bytecode, execute_bytecode};
use std::time::Instant;

fn benchmark_execution(bytecode: &[u8], iterations: usize) {
    let start = Instant::now();

    for _ in 0..iterations {
        let _ = execute_bytecode(bytecode).unwrap();
    }

    let total_time = start.elapsed();
    let avg_time = total_time / iterations as u32;

    println!("Average execution: {:?}", avg_time);
    println!("Throughput: {:.0} ops/sec",
             iterations as f64 / total_time.as_secs_f64());
}
```

**Typical results:**
- Simple expressions: 10-100 microseconds
- Function calls: 100-500 microseconds
- Complex pipelines: 1-10 milliseconds

### Memory Optimization

```rust
// Reuse VM instances instead of creating new ones
use fusabi_vm::{Vm, deserialize_chunk};
use std::sync::{Arc, Mutex};

struct VmPool {
    vms: Arc<Mutex<Vec<Vm>>>,
    size: usize,
}

impl VmPool {
    fn new(size: usize) -> Self {
        let mut vms = Vec::new();
        for _ in 0..size {
            let mut vm = Vm::new();
            fusabi_vm::stdlib::register_stdlib(&mut vm);
            vms.push(vm);
        }

        VmPool {
            vms: Arc::new(Mutex::new(vms)),
            size,
        }
    }

    fn execute(&self, bytecode: &[u8]) -> Result<fusabi::Value, Box<dyn std::error::Error>> {
        let chunk = deserialize_chunk(bytecode)?;

        // Get VM from pool
        let mut vm = self.vms.lock().unwrap().pop()
            .unwrap_or_else(|| {
                let mut vm = Vm::new();
                fusabi_vm::stdlib::register_stdlib(&mut vm);
                vm
            });

        // Execute
        let result = vm.execute(chunk)?;

        // Return VM to pool
        self.vms.lock().unwrap().push(vm);

        Ok(result)
    }
}
```

---

## Error Handling

### Comprehensive Error Handling

```rust
use fusabi::{compile_to_bytecode, execute_bytecode, FusabiError};

fn safe_compile_and_execute(source: &str) -> Result<fusabi::Value, String> {
    // Compile
    let bytecode = compile_to_bytecode(source).map_err(|e| {
        match e {
            FusabiError::Lex(err) => format!("Syntax error: {}", err),
            FusabiError::Parse(err) => format!("Parse error: {}", err),
            FusabiError::Compile(err) => format!("Compilation error: {}", err),
            _ => format!("Unexpected error: {}", e),
        }
    })?;

    // Execute
    execute_bytecode(&bytecode).map_err(|e| {
        match e {
            FusabiError::Runtime(err) => format!("Runtime error: {}", err),
            FusabiError::Serde(err) => format!("Bytecode error: {}", err),
            _ => format!("Execution error: {}", e),
        }
    })
}
```

### Validation Before Production

```rust
use fusabi::compile_to_bytecode;
use fusabi_vm::deserialize_chunk;

fn validate_bytecode(bytecode: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    // Check magic bytes
    if !bytecode.starts_with(fusabi_vm::FZB_MAGIC) {
        return Err("Invalid bytecode: bad magic bytes".into());
    }

    // Check version
    if bytecode[4] != fusabi_vm::FZB_VERSION {
        return Err(format!(
            "Bytecode version mismatch: found {}, expected {}",
            bytecode[4],
            fusabi_vm::FZB_VERSION
        ).into());
    }

    // Verify deserialization
    let chunk = deserialize_chunk(bytecode)?;

    // Check chunk is non-empty
    if chunk.instructions.is_empty() {
        return Err("Invalid bytecode: empty chunk".into());
    }

    Ok(())
}
```

---

## Security Considerations

### 1. Input Validation

```rust
use fusabi::compile_to_bytecode;

fn safe_compile(source: &str) -> Result<Vec<u8>, String> {
    // Size limit
    if source.len() > 1_000_000 {
        return Err("Script too large (max 1MB)".into());
    }

    // Complexity heuristics (example)
    let depth = source.matches("let").count();
    if depth > 100 {
        return Err("Script too complex (max 100 bindings)".into());
    }

    compile_to_bytecode(source).map_err(|e| e.to_string())
}
```

### 2. Sandboxing

```rust
use fusabi::Engine;
use std::time::{Duration, Instant};

fn execute_with_timeout(
    source: &str,
    timeout: Duration,
) -> Result<fusabi::Value, String> {
    let start = Instant::now();

    // This is a simplified example
    // In production, use a separate thread with timeout
    let mut engine = Engine::new();
    let result = engine.eval(source).map_err(|e| e.to_string())?;

    if start.elapsed() > timeout {
        return Err("Execution timeout".into());
    }

    Ok(result)
}
```

### 3. Resource Limits

```rust
// Limit memory usage (conceptual)
fn execute_with_limits(bytecode: &[u8]) -> Result<fusabi::Value, String> {
    // Check bytecode size
    if bytecode.len() > 10_000_000 {
        return Err("Bytecode too large".into());
    }

    execute_bytecode(bytecode).map_err(|e| e.to_string())
}
```

---

## Best Practices

### 1. Development Workflow

```bash
# Development
cargo run -- config/pipeline.fsx          # Fast iteration

# Pre-production
cargo run -- compile config/pipeline.fsx  # Generate .fzb

# Production
cargo run -- config/pipeline.fzb          # Fast startup
```

### 2. Version Control

```gitignore
# .gitignore
*.fzb                 # Don't commit bytecode
/cache/              # Don't commit cache
```

Keep source files (`.fsx`) in version control, generate bytecode during deployment.

### 3. CI/CD Integration

```yaml
# .github/workflows/build.yml
- name: Compile Fusabi Scripts
  run: |
    cargo build --release
    ./target/release/myapp compile-all

- name: Validate Bytecode
  run: |
    cargo test --release -- --test-threads=1
```

### 4. Monitoring

```rust
use fusabi::compile_to_bytecode;
use std::time::Instant;

fn compile_with_metrics(name: &str, source: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let start = Instant::now();
    let result = compile_to_bytecode(source);
    let duration = start.elapsed();

    // Log metrics
    println!("METRIC compilation_time{{script=\"{}\"}} {:?}", name, duration);

    match &result {
        Ok(bytecode) => {
            println!("METRIC bytecode_size{{script=\"{}\"}} {}", name, bytecode.len());
        }
        Err(e) => {
            eprintln!("METRIC compilation_error{{script=\"{}\"}} 1", name);
        }
    }

    result
}
```

---

## Summary

**Development Mode:**
- Use `Engine::eval()` for fast iteration
- Hot-reload source files
- No compilation overhead

**Production Mode:**
- Use `compile_to_bytecode()` + `execute_bytecode()`
- AOT or JIT compilation patterns
- Bytecode caching for performance

**Hibana Pattern:**
- Async-safe bytecode caching
- VM pooling for concurrency
- Hot-reload support

**Performance:**
- 3-5x faster startup with bytecode
- 60-80% smaller files
- Microsecond execution times

**See Also:**
- [Bytecode Format Specification](bytecode-format.md)
- [Host Interop Guide](host-interop.md)
- [API Documentation](https://docs.rs/fusabi)
