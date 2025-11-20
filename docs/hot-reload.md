# Hot-Reload System

The FSRS hot-reload system enables automatic script reloading without application restart, targeting **<100ms reload time**.

## Features

- **File Watching**: Automatic detection of file changes
- **Debouncing**: Handles rapid file modifications efficiently
- **Error Recovery**: Keeps old version on compilation failure
- **Performance Tracking**: Detailed metrics for each reload
- **Callbacks**: Customizable reload event handlers
- **Multi-extension**: Supports `.fsrs` and `.fs` files

## Quick Start

```rust
use fsrs_vm::{HotReloadEngine, Chunk};
use fsrs_frontend::compile_program_from_source;

// Create engine with FSRS compiler
let mut engine = HotReloadEngine::new_with_compiler("script.fsrs", |source| {
    compile_program_from_source(source).map_err(|e| e.to_string())
})?;

// Initial load
let stats = engine.reload()?;
println!("Loaded in {}ms", stats.reload_time_ms);

// Start watching
engine.start()?;

// Wait for changes
while let Some(event) = engine.wait_for_change() {
    let stats = engine.reload()?;
    if stats.success {
        if let Some(chunk) = engine.current_chunk() {
            // Execute new version
            vm.execute(chunk.clone())?;
        }
    }
}
```

## Architecture

### Components

1. **FileWatcher**: Low-level file system monitoring
   - Uses `notify` crate for cross-platform support
   - Configurable debounce duration (default: 100ms)
   - Event filtering (modify, create, delete)

2. **HotReloadEngine**: High-level reload coordination
   - Manages compilation pipeline
   - Tracks reload statistics
   - Maintains current chunk state
   - Callback system for notifications

### Workflow

```
File Change → FileWatcher → Debounce → Read Source → Compile → Update Chunk
                                                          ↓
                                                      Callbacks
```

## API Reference

### HotReloadEngine

#### Creation

```rust
// With custom compiler
let engine = HotReloadEngine::new_with_compiler(path, |source| {
    // Your compiler function
    Ok(Chunk::new())
})?;

// With default FSRS compiler (requires fsrs-frontend integration)
let engine = HotReloadEngine::new(path)?;
```

#### Control Methods

```rust
engine.start()?;              // Start watching
engine.stop()?;               // Stop watching
engine.is_watching();         // Check status
```

#### Reload Methods

```rust
// Manual reload
let stats = engine.reload()?;

// Reload and get chunk if successful
let chunk = engine.reload_and_get_chunk()?;

// Auto-reload loop (blocking)
engine.watch_and_reload()?;
```

#### Event Methods

```rust
// Wait for next change
let event = engine.wait_for_change();

// Wait with timeout
let event = engine.wait_for_change_timeout(Duration::from_secs(5));

// Drain pending events
let events = engine.drain_events();
```

#### Query Methods

```rust
engine.current_chunk();           // Get current compiled chunk
engine.script_path();             // Get watched path
engine.reload_count();            // Number of successful reloads
engine.time_since_last_reload();  // Duration since last reload
```

#### Callbacks

```rust
engine.on_reload(|stats: &ReloadStats| {
    println!("Reload took {}ms", stats.reload_time_ms);
    if !stats.success {
        eprintln!("Error: {}", stats.error_message.unwrap());
    }
});
```

### ReloadStats

Statistics about a reload operation:

```rust
pub struct ReloadStats {
    pub reload_time_ms: u64,        // Total time (target: <100ms)
    pub compile_time_ms: u64,       // Compilation time
    pub success: bool,              // Success flag
    pub error_message: Option<String>,  // Error if failed
    pub source_size_bytes: usize,   // Source file size
}
```

Methods:
- `stats.meets_target()`: Returns `true` if reload < 100ms

### FileEvent

```rust
pub enum FileEvent {
    Modified(PathBuf),
    Created(PathBuf),
    Deleted(PathBuf),
}
```

## Performance

### Targets

- **Total reload time**: < 100ms
- **File detection**: < 100ms
- **Compilation**: < 50ms (simple scripts)

### Optimization Strategies

1. **Debouncing**: Prevents excessive recompilation
   - Default: 100ms window
   - Configurable via `FileWatcher::with_debounce()`

2. **Async Watching**: File watching doesn't block compilation
   - Events queued in channel
   - Non-blocking event check with `try_next_event()`

3. **Error Recovery**: Failed compilations don't require full reload
   - Old chunk preserved
   - Compilation errors logged
   - Next successful compile replaces old chunk

## Examples

### Basic Hot-Reload Loop

```rust
let mut engine = HotReloadEngine::new_with_compiler("app.fsrs", compile_fn)?;

engine.start()?;

loop {
    if let Some(_event) = engine.wait_for_change() {
        // Debounce rapid changes
        thread::sleep(Duration::from_millis(100));
        engine.drain_events();

        // Reload
        match engine.reload() {
            Ok(stats) if stats.success => {
                println!("✅ Reloaded in {}ms", stats.reload_time_ms);
                if let Some(chunk) = engine.current_chunk() {
                    vm.execute(chunk.clone())?;
                }
            }
            Ok(stats) => {
                eprintln!("❌ Compilation failed: {}", stats.error_message.unwrap());
            }
            Err(e) => {
                eprintln!("❌ Reload error: {}", e);
            }
        }
    }
}
```

### With Metrics Tracking

```rust
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

let total_time = Arc::new(AtomicU64::new(0));
let success_count = Arc::new(AtomicUsize::new(0));
let failure_count = Arc::new(AtomicUsize::new(0));

let total_clone = total_time.clone();
let success_clone = success_count.clone();
let failure_clone = failure_count.clone();

engine.on_reload(move |stats| {
    total_clone.fetch_add(stats.reload_time_ms, Ordering::SeqCst);
    if stats.success {
        success_clone.fetch_add(1, Ordering::SeqCst);
    } else {
        failure_clone.fetch_add(1, Ordering::SeqCst);
    }
});

// ... run hot-reload loop ...

// Print statistics
let total_reloads = success_count.load(Ordering::SeqCst) + failure_count.load(Ordering::SeqCst);
let avg_time = total_time.load(Ordering::SeqCst) / total_reloads as u64;

println!("Total reloads: {}", total_reloads);
println!("Success: {}", success_count.load(Ordering::SeqCst));
println!("Failures: {}", failure_count.load(Ordering::SeqCst));
println!("Average reload time: {}ms", avg_time);
```

### Terminal Configuration Example

```rust
// Terminal emulator with hot-reload config
struct TerminalConfig {
    engine: HotReloadEngine,
    vm: Vm,
    host_registry: HostRegistry,
}

impl TerminalConfig {
    fn new(config_path: &Path) -> Result<Self, Box<dyn Error>> {
        let host_registry = HostRegistry::new();

        let mut engine = HotReloadEngine::new_with_compiler(config_path, |source| {
            compile_program_from_source(source).map_err(|e| e.to_string())
        })?;

        // Load initial config
        engine.reload()?;

        Ok(TerminalConfig {
            engine,
            vm: Vm::new(),
            host_registry,
        })
    }

    fn watch_for_changes(&mut self) -> Result<(), Box<dyn Error>> {
        self.engine.start()?;

        while let Some(_) = self.engine.wait_for_change() {
            thread::sleep(Duration::from_millis(100));
            self.engine.drain_events();

            if let Ok(stats) = self.engine.reload() {
                if stats.success {
                    println!("Config reloaded in {}ms", stats.reload_time_ms);
                    self.apply_config()?;
                } else {
                    eprintln!("Config error: {}", stats.error_message.unwrap());
                }
            }
        }

        Ok(())
    }

    fn apply_config(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(chunk) = self.engine.current_chunk() {
            self.vm.execute(chunk.clone())?;
        }
        Ok(())
    }
}
```

## Testing

### Unit Tests

33+ unit tests covering:
- File watching
- Event debouncing
- Reload mechanics
- Error handling
- Callback system
- State management

### Integration Tests

7+ integration tests covering:
- Full workflow (watch → modify → reload)
- Performance targets
- Error recovery
- Callback system
- File extension validation
- Rapid modifications (debouncing)
- Large file handling

Run tests:
```bash
# Unit tests
cargo test -p fsrs-vm hot_reload --lib

# Integration tests
cargo test -p fsrs-vm --test test_hot_reload

# All tests
cargo test -p fsrs-vm
```

## Troubleshooting

### Slow Reload Times

1. **Check compilation time**: `stats.compile_time_ms`
2. **Optimize compiler**: Profile and optimize bottlenecks
3. **Reduce source size**: Split large scripts into modules

### Missed File Changes

1. **Check file system events**: Some editors use atomic writes
2. **Increase debounce**: `FileWatcher::with_debounce(Duration::from_millis(200))`
3. **Watch directory**: Watch parent directory instead of file

### Memory Leaks

1. **Old chunks**: Ensure old chunks are dropped when replaced
2. **Callbacks**: Avoid capturing large data in callbacks
3. **VM state**: Create new VM for each execution if needed

## Future Enhancements

- **Multi-file watching**: Watch multiple script files
- **Module hot-reload**: Reload individual modules
- **Incremental compilation**: Only recompile changed modules
- **WebAssembly support**: Hot-reload in browser environments
- **Remote reloading**: Reload scripts over network

## References

- [notify crate](https://docs.rs/notify/) - File system notification library
- [crossbeam-channel](https://docs.rs/crossbeam-channel/) - Concurrent channel implementation
- FSRS VM Documentation
- FSRS Frontend Compiler Documentation
