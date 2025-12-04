# Fusabi Host Interop API Architecture

## Overview

This document specifies the Rust host interop API for Fusabi (F#-to-Rust Script Engine). The design prioritizes:
- **Ergonomic API**: Natural Rust usage patterns
- **Type Safety**: Compile-time guarantees where possible
- **Performance**: Zero-cost abstractions, minimal allocations
- **Hot-Reload**: Seamless script updates without host restart
- **Flexibility**: Support diverse embedding scenarios

---

## 1. Host API Design

### 1.1 Core API Surface

```rust
use fusabi::{Engine, Value, ScriptError, Result};

// Basic engine lifecycle
let mut engine = Engine::new();

// Configuration builder pattern
let mut engine = Engine::builder()
    .with_memory_limit(1024 * 1024 * 100) // 100MB
    .with_max_recursion_depth(256)
    .enable_hot_reload(true)
    .build()?;

// Load scripts
engine.load_script("config.fsx")?;
engine.load_script_from_string("let x = 42", "inline")?;

// Register host functions
engine.register_function("log", host_log)?;
engine.register_function("http_fetch", http_fetch)?;

// Call script functions
let result: Value = engine.call("get_tab_title", &[tab_info.into()])?;

// Extract typed results
let title: String = result.try_into()?;

// Evaluate expressions
let value: i64 = engine.eval::<i64>("40 + 2")?;

// Access script variables
let config: Value = engine.get_global("app_config")?;

// Check function existence
if engine.has_function("on_startup") {
    engine.call("on_startup", &[])?;
}

// Cleanup
engine.unload_script("config.fsx")?;
```

### 1.2 Advanced Features

```rust
// Module system
engine.load_module("plugins/theme.fsx", "theme")?;
let color: String = engine.call_qualified("theme.get_background", &[])?;

// Async support
async fn async_example(engine: &mut Engine) -> Result<()> {
    let future = engine.call_async("fetch_data", &[url.into()]).await?;
    let data: Vec<u8> = future.try_into()?;
    Ok(())
}

// Error handling with context
match engine.call("risky_operation", &[]) {
    Ok(value) => println!("Success: {:?}", value),
    Err(ScriptError::Runtime { message, stack_trace, .. }) => {
        eprintln!("Runtime error: {}", message);
        eprintln!("Stack trace:\n{}", stack_trace);
    }
    Err(ScriptError::TypeError { expected, actual, .. }) => {
        eprintln!("Type error: expected {}, got {}", expected, actual);
    }
    Err(e) => eprintln!("Error: {:?}", e),
}

// Hot-reload with callbacks
engine.on_reload(|script_path, result| {
    match result {
        Ok(_) => println!("Reloaded: {}", script_path),
        Err(e) => eprintln!("Reload failed: {}", e),
    }
});

// Watch for changes
engine.watch_script("config.fsx")?;
```

---

## 2. Value Marshalling

### 2.1 Value Type System

```rust
/// Core value type that bridges Rust and F# script world
#[derive(Debug, Clone)]
pub enum Value {
    // Primitives
    Unit,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(Rc<str>),

    // Collections
    List(Rc<Vec<Value>>),
    Array(Rc<Vec<Value>>),
    Map(Rc<HashMap<String, Value>>),

    // Functions
    Function(FunctionRef),
    HostFunction(HostFunctionRef),

    // User types
    Record(Rc<Record>),
    Tuple(Rc<Vec<Value>>),
    Union { tag: String, data: Box<Value> },

    // Special
    Reference(Rc<RefCell<Value>>),
}

#[derive(Debug, Clone)]
pub struct Record {
    pub type_name: String,
    pub fields: HashMap<String, Value>,
}
```

### 2.2 Rust to Fusabi Conversion

```rust
// Trait for converting Rust types to FSRS values
pub trait IntoValue {
    fn into_value(self) -> Value;
}

// Automatic implementations
impl IntoValue for i64 {
    fn into_value(self) -> Value {
        Value::Int(self)
    }
}

impl IntoValue for String {
    fn into_value(self) -> Value {
        Value::String(self.into())
    }
}

impl<T: IntoValue> IntoValue for Vec<T> {
    fn into_value(self) -> Value {
        let values: Vec<Value> = self.into_iter()
            .map(|v| v.into_value())
            .collect();
        Value::List(Rc::new(values))
    }
}

// Derive macro for custom types
use fusabi::IntoValue;

#[derive(IntoValue)]
struct TabInfo {
    title: String,
    pane_id: i64,
    is_active: bool,
}

// Expands to:
impl IntoValue for TabInfo {
    fn into_value(self) -> Value {
        Value::Record(Rc::new(Record {
            type_name: "TabInfo".to_string(),
            fields: HashMap::from([
                ("title".to_string(), self.title.into_value()),
                ("pane_id".to_string(), self.pane_id.into_value()),
                ("is_active".to_string(), self.is_active.into_value()),
            ]),
        }))
    }
}

// Manual conversion
let tab_info = TabInfo {
    title: "Terminal".to_string(),
    pane_id: 1,
    is_active: true,
};

let value: Value = tab_info.into(); // Uses From<T> trait
```

### 2.3 Fusabi to Rust Conversion

```rust
// Trait for extracting Rust types from Fusabi values
pub trait TryFromValue: Sized {
    type Error;
    fn try_from_value(value: Value) -> Result<Self, Self::Error>;
}

// Automatic implementations
impl TryFromValue for i64 {
    type Error = ScriptError;

    fn try_from_value(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Int(i) => Ok(i),
            _ => Err(ScriptError::type_error("i64", value.type_name())),
        }
    }
}

impl TryFromValue for String {
    type Error = ScriptError;

    fn try_from_value(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::String(s) => Ok(s.to_string()),
            _ => Err(ScriptError::type_error("String", value.type_name())),
        }
    }
}

// Derive macro for custom types
#[derive(TryFromValue)]
struct Config {
    theme: String,
    font_size: i64,
    enabled: bool,
}

// Usage
let config_value: Value = engine.get_global("user_config")?;
let config: Config = config_value.try_into()?;

// Alternative: pattern matching
match config_value {
    Value::Record(record) => {
        let theme = record.fields.get("theme")
            .and_then(|v| v.as_string())
            .unwrap_or("dark");
        // ...
    }
    _ => return Err(ScriptError::type_error("Config", config_value.type_name())),
}
```

### 2.4 Safe Borrowing with Lifetimes

```rust
// Value references for zero-copy access
impl Value {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s.as_ref()),
            _ => None,
        }
    }

    pub fn as_slice(&self) -> Option<&[Value]> {
        match self {
            Value::List(list) | Value::Array(list) => Some(list.as_ref()),
            _ => None,
        }
    }

    pub fn as_record(&self) -> Option<&Record> {
        match self {
            Value::Record(record) => Some(record.as_ref()),
            _ => None,
        }
    }
}

// Usage example
let name_value = engine.get_global("username")?;
if let Some(name) = name_value.as_str() {
    println!("Hello, {}!", name); // No allocation
}

// For mutable access
let list_ref = engine.get_global("items")?;
if let Value::Reference(cell) = list_ref {
    let mut borrowed = cell.borrow_mut();
    // Modify the value
    *borrowed = Value::List(Rc::new(vec![Value::Int(1), Value::Int(2)]));
}
```

---

## 3. Function Registration

### 3.1 Host Function Registration

```rust
// Function signature types
type HostFn = dyn Fn(&[Value]) -> Result<Value> + Send + Sync;
type AsyncHostFn = dyn Fn(&[Value]) -> Pin<Box<dyn Future<Output = Result<Value>>>> + Send + Sync;

// Simple function registration
fn host_log(args: &[Value]) -> Result<Value> {
    for arg in args {
        println!("{:?}", arg);
    }
    Ok(Value::Unit)
}

engine.register_function("log", host_log)?;

// Typed function wrapper
engine.register_typed_function("add", |a: i64, b: i64| -> i64 {
    a + b
})?;

// Expands to:
engine.register_function("add", |args: &[Value]| -> Result<Value> {
    if args.len() != 2 {
        return Err(ScriptError::argument_count_mismatch(2, args.len()));
    }
    let a: i64 = args[0].clone().try_into()?;
    let b: i64 = args[1].clone().try_into()?;
    let result = a + b;
    Ok(result.into_value())
})?;

// Macro for cleaner syntax
register_host_functions!(engine, {
    "log" => host_log,
    "add" => |a: i64, b: i64| a + b,
    "concat" => |a: String, b: String| format!("{}{}", a, b),
});
```

### 3.2 Advanced Function Registration

```rust
// Stateful host functions with closures
let counter = Arc::new(AtomicI64::new(0));
let counter_clone = counter.clone();

engine.register_function("next_id", move |_args: &[Value]| -> Result<Value> {
    let id = counter_clone.fetch_add(1, Ordering::SeqCst);
    Ok(Value::Int(id))
})?;

// Async functions
engine.register_async_function("fetch_url", |args: &[Value]| async move {
    let url: String = args[0].clone().try_into()?;
    let response = reqwest::get(&url).await?;
    let body = response.text().await?;
    Ok(Value::String(body.into()))
})?;

// Variadic functions
engine.register_function("sum", |args: &[Value]| -> Result<Value> {
    let mut total = 0i64;
    for arg in args {
        let n: i64 = arg.clone().try_into()?;
        total += n;
    }
    Ok(Value::Int(total))
})?;

// Functions with context access
engine.register_context_function("get_env", |ctx: &Context, args: &[Value]| {
    let var_name: String = args[0].clone().try_into()?;
    let value = ctx.environment.get(&var_name)
        .ok_or_else(|| ScriptError::runtime(format!("Env var not found: {}", var_name)))?;
    Ok(Value::String(value.clone().into()))
})?;
```

### 3.3 Error Handling Across FFI Boundary

```rust
// Host function error types
#[derive(Debug, thiserror::Error)]
pub enum HostError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Custom error: {0}")]
    Custom(String),
}

// Convert to ScriptError
impl From<HostError> for ScriptError {
    fn from(err: HostError) -> Self {
        ScriptError::HostFunction {
            message: err.to_string(),
            source: Some(Box::new(err)),
        }
    }
}

// Usage in host functions
fn read_file(args: &[Value]) -> Result<Value> {
    let path: String = args[0].clone().try_into()?;

    let content = std::fs::read_to_string(&path)
        .map_err(HostError::from)?;

    Ok(Value::String(content.into()))
}

// Propagation to script
/*
F# script:
try
    let content = read_file "config.txt"
    printfn "%s" content
with
| :? HostError as e -> printfn "Host error: %s" e.Message
*/
```

---

## 4. Script Loading & Hot-Reload

### 4.1 Script Loading Strategies

```rust
// Simple file loading
engine.load_script("config.fsx")?;

// Load with custom module name
engine.load_script_as("config.fsx", "app_config")?;

// Load from string (for testing, inline scripts)
engine.load_script_from_string(r#"
    let greet name = printfn "Hello, %s!" name
"#, "inline_script")?;

// Precompiled scripts (for distribution)
let compiled = engine.compile_script("config.fsx")?;
std::fs::write("config.fsx.bin", &compiled)?;

// Later, load precompiled
let compiled = std::fs::read("config.fsx.bin")?;
engine.load_compiled(&compiled, "config")?;

// Module loading with dependencies
engine.load_module_graph(&[
    "core/utils.fsx",
    "plugins/theme.fsx",
    "config.fsx", // Depends on previous modules
])?;
```

### 4.2 Hot-Reload Architecture

```rust
use fusabi::hotreload::{Watcher, ReloadStrategy};

// Enable hot-reload with file watching
let mut engine = Engine::builder()
    .enable_hot_reload(true)
    .with_reload_strategy(ReloadStrategy::Incremental)
    .build()?;

// Watch specific scripts
engine.watch_script("config.fsx")?;
engine.watch_directory("plugins/")?;

// Reload callbacks
engine.on_before_reload(|script_path| {
    println!("Reloading: {}", script_path);
});

engine.on_reload_success(|script_path, elapsed| {
    println!("Reloaded {} in {:?}", script_path, elapsed);
});

engine.on_reload_error(|script_path, error| {
    eprintln!("Failed to reload {}: {}", script_path, error);
    // Keep old version on error
});

// Manual reload trigger
engine.reload_script("config.fsx")?;
engine.reload_all()?;

// Reload strategies
pub enum ReloadStrategy {
    /// Recompile entire module and replace
    Full,

    /// Only recompile changed functions (faster, more complex)
    Incremental,

    /// Preserve existing state, update code only
    Stateful,
}
```

### 4.3 State Preservation Across Reloads

```rust
// Mark values for preservation
/*
F# script (config.fsx):
[<Persistent>]
let mutable connection_pool = create_pool()

[<Persistent>]
let mutable user_sessions = Map.empty

// This will be reset on reload
let temp_cache = []
*/

// Host API for state management
engine.preserve_global("connection_pool")?;
engine.preserve_global("user_sessions")?;

// Get snapshot before reload
let snapshot = engine.snapshot_state(&["connection_pool", "user_sessions"])?;

// Reload script
engine.reload_script("config.fsx")?;

// Restore state
engine.restore_state(snapshot)?;

// Automatic state preservation
let mut engine = Engine::builder()
    .enable_hot_reload(true)
    .preserve_persistent_values(true)
    .build()?;
```

### 4.4 File Watching Implementation

```rust
use notify::{Watcher, RecursiveMode};

// Internal implementation (exposed via Engine API)
pub struct HotReloadWatcher {
    watcher: notify::RecommendedWatcher,
    watched_scripts: HashMap<PathBuf, ScriptId>,
    engine_handle: Arc<Mutex<Engine>>,
}

impl HotReloadWatcher {
    pub fn watch_script(&mut self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref().canonicalize()?;
        self.watcher.watch(&path, RecursiveMode::NonRecursive)?;
        Ok(())
    }

    fn on_file_change(&mut self, path: &Path) {
        if let Some(script_id) = self.watched_scripts.get(path) {
            let mut engine = self.engine_handle.lock().unwrap();

            match engine.reload_script_by_id(*script_id) {
                Ok(_) => {
                    // Trigger success callback
                }
                Err(e) => {
                    // Trigger error callback, keep old version
                }
            }
        }
    }
}
```

---

## 5. Memory Management

### 5.1 GC Strategy

Fusabi uses a **hybrid reference counting + cycle detection** approach:

```rust
// Reference counting for immediate deallocation
pub struct Value {
    // Rc for cheap cloning and automatic cleanup
    // Most values don't have cycles
}

// Cycle detector runs periodically
pub struct GarbageCollector {
    /// All values that might be part of cycles
    potential_cycles: Vec<Weak<RefCell<Value>>>,

    /// Roots (globals, stack values) that keep values alive
    roots: HashSet<*const Value>,
}

impl GarbageCollector {
    pub fn collect(&mut self) -> CollectionStats {
        // Mark phase: trace from roots
        let mut marked = HashSet::new();
        for root in &self.roots {
            self.mark(*root, &mut marked);
        }

        // Sweep phase: collect unmarked cycles
        let mut freed = 0;
        self.potential_cycles.retain(|weak| {
            if let Some(value) = weak.upgrade() {
                let ptr = Rc::as_ptr(&value) as *const Value;
                if !marked.contains(&ptr) {
                    freed += 1;
                    false // Remove from potential_cycles
                } else {
                    true // Keep
                }
            } else {
                false // Already dropped
            }
        });

        CollectionStats { freed_count: freed }
    }
}

// GC configuration
let mut engine = Engine::builder()
    .gc_threshold(1024 * 1024) // Trigger GC after 1MB allocated
    .gc_interval(Duration::from_secs(30)) // Or every 30 seconds
    .enable_cycle_detection(true)
    .build()?;

// Manual GC control
engine.gc_collect()?;
let stats = engine.gc_stats();
println!("Live objects: {}, freed: {}", stats.live_count, stats.freed_count);
```

### 5.2 Integration with Rust Ownership

```rust
// Values owned by engine
let value = engine.get_global("config")?; // Returns owned Value

// Borrowing from engine for temporary access
engine.with_global("config", |config: &Value| {
    // Read-only access, no clone needed
    if let Some(theme) = config.as_record()?.fields.get("theme") {
        println!("Theme: {:?}", theme);
    }
    Ok(())
})?;

// Mutable borrowing
engine.with_global_mut("counter", |counter: &mut Value| {
    if let Value::Int(n) = counter {
        *n += 1;
    }
    Ok(())
})?;

// Transfer ownership to script
let config = load_config_from_file()?;
engine.set_global("app_config", config.into_value())?;

// Transfer ownership from script to Rust
let result = engine.call("compute_result", &[])?;
let owned_string: String = result.try_into()?; // Moves Value
```

### 5.3 Handling Cycles

```rust
// Example cycle in F# script:
/*
type Node = { value: int; mutable next: Node option }

let rec node1 = { value = 1; next = Some node2 }
and node2 = { value = 2; next = Some node1 }
*/

// Detection via weak references
impl Value {
    pub fn create_cyclic_ref(&self) -> Value {
        // Returns a weak reference that doesn't prevent deallocation
        Value::WeakReference(Rc::downgrade(self.as_rc()))
    }
}

// Automatic cycle detection
let mut gc = engine.gc();
gc.detect_cycles(); // Marks values in cycles
gc.collect_cycles(); // Breaks cycles and frees memory

// Manual cycle breaking
engine.break_cycle(&["node1", "node2"])?;
```

---

## 6. Error Handling

### 6.1 Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum ScriptError {
    #[error("Parse error at {location}: {message}")]
    Parse {
        message: String,
        location: SourceLocation,
    },

    #[error("Type error: expected {expected}, got {actual}")]
    TypeError {
        expected: String,
        actual: String,
        location: Option<SourceLocation>,
    },

    #[error("Runtime error: {message}")]
    Runtime {
        message: String,
        stack_trace: StackTrace,
    },

    #[error("Function '{name}' not found")]
    FunctionNotFound {
        name: String,
    },

    #[error("Argument count mismatch: expected {expected}, got {actual}")]
    ArgumentCountMismatch {
        expected: usize,
        actual: usize,
    },

    #[error("Host function error: {message}")]
    HostFunction {
        message: String,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Compilation error: {0}")]
    Compilation(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub file: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct StackTrace {
    pub frames: Vec<StackFrame>,
}

#[derive(Debug, Clone)]
pub struct StackFrame {
    pub function_name: String,
    pub location: SourceLocation,
}
```

### 6.2 Runtime Error Handling

```rust
// Catching errors in host
match engine.call("risky_function", &[]) {
    Ok(value) => println!("Success: {:?}", value),
    Err(ScriptError::Runtime { message, stack_trace }) => {
        eprintln!("Runtime error: {}", message);
        eprintln!("\nStack trace:");
        for frame in &stack_trace.frames {
            eprintln!("  at {} ({}:{}:{})",
                frame.function_name,
                frame.location.file,
                frame.location.line,
                frame.location.column
            );
        }
    }
    Err(e) => eprintln!("Error: {}", e),
}

// Result type alias
pub type Result<T> = std::result::Result<T, ScriptError>;

// Helper methods
impl ScriptError {
    pub fn runtime(message: impl Into<String>) -> Self {
        ScriptError::Runtime {
            message: message.into(),
            stack_trace: StackTrace::capture(),
        }
    }

    pub fn type_error(expected: impl Into<String>, actual: impl Into<String>) -> Self {
        ScriptError::TypeError {
            expected: expected.into(),
            actual: actual.into(),
            location: None,
        }
    }
}
```

### 6.3 Stack Traces

```rust
// Automatic stack trace capture
impl StackTrace {
    pub fn capture() -> Self {
        // Captures current call stack from VM
        let frames = VM::current()
            .map(|vm| vm.capture_stack())
            .unwrap_or_default();

        StackTrace { frames }
    }

    pub fn display(&self) -> String {
        let mut output = String::new();
        for (i, frame) in self.frames.iter().enumerate() {
            output.push_str(&format!("  {}: {} at {}:{}:{}\n",
                i,
                frame.function_name,
                frame.location.file,
                frame.location.line,
                frame.location.column
            ));
        }
        output
    }
}

// Pretty printing
println!("{}", stack_trace.display());

// Output:
//   0: compute_result at config.fsx:42:5
//   1: process_data at config.fsx:28:12
//   2: main at config.fsx:10:1
```

### 6.4 Error Recovery

```rust
// Fallible operations with recovery
let result = engine.call("maybe_failing_function", &[]);

let value = match result {
    Ok(v) => v,
    Err(ScriptError::Runtime { .. }) => {
        // Recover with default value
        Value::Int(0)
    }
    Err(e) => return Err(e),
};

// Try-catch in scripts
/*
F# script:
try
    let result = risky_operation()
    result
with
| :? RuntimeError as e ->
    printfn "Error: %s" e.Message
    default_value
*/

// Panic handling
engine.set_panic_handler(|panic_info| {
    eprintln!("Script panic: {}", panic_info.message);
    // Log, restart, or handle gracefully
});
```

---

## 7. Example Use Cases

### 7.1 Terminal Emulator Config (WezTerm-style)

```rust
// config.fsx (F# script)
/*
module Config

open Wezterm

let font_size = 12.0
let font_family = "JetBrains Mono"

let color_scheme = {
    foreground = "#d4d4d4"
    background = "#1e1e1e"
    cursor_bg = "#aeafad"
    cursor_border = "#aeafad"
    selection_fg = "#1e1e1e"
    selection_bg = "#d7ba7d"
    ansi = [
        "#1e1e1e"; "#f48771"; "#90a959"; "#f2cc5f"
        "#6699cc"; "#aa759f"; "#6699cc"; "#d4d4d4"
    ]
    brights = [
        "#5a5a5a"; "#f48771"; "#90a959"; "#f2cc5f"
        "#6699cc"; "#aa759f"; "#6699cc"; "#ffffff"
    ]
}

let on_tab_bar_click tab_info =
    if tab_info.button = "Left" then
        switch_to_tab tab_info.tab_id
    elif tab_info.button = "Middle" then
        close_tab tab_info.tab_id
    else
        show_tab_menu tab_info

let format_tab_title tab_info =
    let process = tab_info.active_pane.foreground_process_name
    let cwd = tab_info.active_pane.current_working_dir
    sprintf "[%d: %s - %s]" tab_info.tab_index process cwd
*/

// Rust host implementation
use fusabi::Engine;

struct TerminalConfig {
    engine: Engine,
}

impl TerminalConfig {
    pub fn new() -> Result<Self> {
        let mut engine = Engine::builder()
            .enable_hot_reload(true)
            .build()?;

        // Register host functions
        engine.register_typed_function("switch_to_tab", Self::host_switch_to_tab)?;
        engine.register_typed_function("close_tab", Self::host_close_tab)?;
        engine.register_typed_function("show_tab_menu", Self::host_show_tab_menu)?;

        // Load config
        engine.load_script("config.fsx")?;
        engine.watch_script("config.fsx")?;

        Ok(Self { engine })
    }

    pub fn get_font_size(&self) -> Result<f64> {
        let value = self.engine.get_global("Config.font_size")?;
        value.try_into()
    }

    pub fn get_color_scheme(&self) -> Result<ColorScheme> {
        let value = self.engine.get_global("Config.color_scheme")?;
        value.try_into()
    }

    pub fn format_tab_title(&mut self, tab_info: &TabInfo) -> Result<String> {
        let result = self.engine.call("Config.format_tab_title", &[tab_info.clone().into()])?;
        result.try_into()
    }

    pub fn on_tab_click(&mut self, tab_info: &TabInfo) -> Result<()> {
        self.engine.call("Config.on_tab_bar_click", &[tab_info.clone().into()])?;
        Ok(())
    }

    // Host functions
    fn host_switch_to_tab(tab_id: i64) -> Result<()> {
        // Implementation
        Ok(())
    }

    fn host_close_tab(tab_id: i64) -> Result<()> {
        // Implementation
        Ok(())
    }

    fn host_show_tab_menu(tab_info: TabInfo) -> Result<()> {
        // Implementation
        Ok(())
    }
}

#[derive(IntoValue, TryFromValue)]
struct TabInfo {
    tab_id: i64,
    tab_index: i64,
    button: String,
    active_pane: PaneInfo,
}

#[derive(IntoValue, TryFromValue)]
struct PaneInfo {
    foreground_process_name: String,
    current_working_dir: String,
}

#[derive(TryFromValue)]
struct ColorScheme {
    foreground: String,
    background: String,
    cursor_bg: String,
    cursor_border: String,
    selection_fg: String,
    selection_bg: String,
    ansi: Vec<String>,
    brights: Vec<String>,
}
```

### 7.2 Plugin System

```rust
// plugin.fsx (F# script)
/*
module Plugin

[<PluginMetadata>]
let metadata = {
    name = "code_formatter"
    version = "1.0.0"
    author = "fusabi-lang"
    description = "Code formatting plugin"
}

let on_load () =
    register_command "format" format_code
    register_keybind "Ctrl+Shift+F" "format"
    printfn "Code formatter plugin loaded"

let on_unload () =
    unregister_command "format"
    printfn "Code formatter plugin unloaded"

let format_code context =
    let code = context.selected_text
    let formatted = format_rust_code code
    replace_selection context formatted

let format_rust_code code =
    // Call host formatter
    rust_fmt code
*/

// Rust host
struct PluginManager {
    engine: Engine,
    plugins: HashMap<String, PluginHandle>,
}

impl PluginManager {
    pub fn load_plugin(&mut self, path: &str) -> Result<()> {
        let mut engine = Engine::new();

        // Register plugin API
        engine.register_typed_function("register_command", Self::host_register_command)?;
        engine.register_typed_function("register_keybind", Self::host_register_keybind)?;
        engine.register_typed_function("rust_fmt", Self::host_rust_fmt)?;
        engine.register_typed_function("replace_selection", Self::host_replace_selection)?;

        // Load plugin script
        engine.load_script(path)?;

        // Get metadata
        let metadata: PluginMetadata = engine.get_global("Plugin.metadata")?.try_into()?;

        // Call on_load
        engine.call("Plugin.on_load", &[])?;

        // Store plugin
        self.plugins.insert(metadata.name.clone(), PluginHandle {
            engine,
            metadata,
        });

        Ok(())
    }

    pub fn unload_plugin(&mut self, name: &str) -> Result<()> {
        if let Some(mut handle) = self.plugins.remove(name) {
            handle.engine.call("Plugin.on_unload", &[])?;
        }
        Ok(())
    }

    pub fn execute_command(&mut self, plugin_name: &str, command: &str, context: &CommandContext) -> Result<()> {
        let handle = self.plugins.get_mut(plugin_name)
            .ok_or_else(|| ScriptError::runtime(format!("Plugin not found: {}", plugin_name)))?;

        handle.engine.call("Plugin.format_code", &[context.clone().into()])?;
        Ok(())
    }

    // Host functions
    fn host_register_command(name: String, handler: String) -> Result<()> {
        // Register command in host
        Ok(())
    }

    fn host_rust_fmt(code: String) -> Result<String> {
        // Use rustfmt
        Ok(code) // Placeholder
    }

    fn host_replace_selection(context: CommandContext, text: String) -> Result<()> {
        // Replace selection in editor
        Ok(())
    }
}

#[derive(TryFromValue)]
struct PluginMetadata {
    name: String,
    version: String,
    author: String,
    description: String,
}

#[derive(IntoValue, Clone)]
struct CommandContext {
    selected_text: String,
    file_path: String,
    cursor_position: i64,
}

struct PluginHandle {
    engine: Engine,
    metadata: PluginMetadata,
}
```

### 7.3 Configuration Files with Logic

```rust
// app_config.fsx (F# script)
/*
module AppConfig

open System

let environment =
    match get_env "APP_ENV" with
    | "production" -> "production"
    | "staging" -> "staging"
    | _ -> "development"

let database_url =
    if environment = "production" then
        get_env "DATABASE_URL"
    else
        "postgresql://localhost/myapp_dev"

let cache_ttl =
    if environment = "production" then
        TimeSpan.FromHours(24.0)
    else
        TimeSpan.FromMinutes(5.0)

let feature_flags = {
    enable_beta_features = environment <> "production"
    enable_analytics = environment = "production"
    max_upload_size =
        if environment = "production" then 100 * 1024 * 1024
        else 10 * 1024 * 1024
}

let compute_api_endpoint service_name =
    let region = get_env "AWS_REGION" |> Option.defaultValue "us-east-1"
    let base_url =
        if environment = "production" then
            sprintf "https://%s.%s.api.example.com" service_name region
        else
            sprintf "http://localhost:%d" (hash_service_name service_name)
    base_url

let hash_service_name name =
    // Simple hash to consistent port
    3000 + (name.GetHashCode() % 1000)
*/

// Rust host
struct AppConfig {
    engine: Engine,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let mut engine = Engine::new();

        // Register host functions
        engine.register_function("get_env", |args: &[Value]| {
            let key: String = args[0].clone().try_into()?;
            let value = std::env::var(&key).ok();
            Ok(value.map(|s| Value::String(s.into())).unwrap_or(Value::Unit))
        })?;

        // Load config
        engine.load_script("app_config.fsx")?;

        Ok(Self { engine })
    }

    pub fn database_url(&self) -> Result<String> {
        let value = self.engine.get_global("AppConfig.database_url")?;
        value.try_into()
    }

    pub fn cache_ttl(&self) -> Result<std::time::Duration> {
        let value = self.engine.get_global("AppConfig.cache_ttl")?;
        // Convert TimeSpan to Duration
        let seconds: f64 = value.as_record()
            .and_then(|r| r.fields.get("TotalSeconds"))
            .and_then(|v| v.as_float())
            .ok_or_else(|| ScriptError::type_error("TimeSpan", "unknown"))?;
        Ok(std::time::Duration::from_secs_f64(seconds))
    }

    pub fn get_api_endpoint(&mut self, service: &str) -> Result<String> {
        let result = self.engine.call("AppConfig.compute_api_endpoint", &[Value::String(service.into())])?;
        result.try_into()
    }

    pub fn feature_flags(&self) -> Result<FeatureFlags> {
        let value = self.engine.get_global("AppConfig.feature_flags")?;
        value.try_into()
    }
}

#[derive(TryFromValue)]
struct FeatureFlags {
    enable_beta_features: bool,
    enable_analytics: bool,
    max_upload_size: i64,
}

// Usage
fn main() -> Result<()> {
    let config = AppConfig::load()?;

    println!("Database: {}", config.database_url()?);
    println!("Cache TTL: {:?}", config.cache_ttl()?);
    println!("Auth API: {}", config.get_api_endpoint("auth")?);

    let flags = config.feature_flags()?;
    println!("Beta features: {}", flags.enable_beta_features);

    Ok(())
}
```

---

## 8. Performance Considerations

### 8.1 Hot-Path Optimization

```rust
// Value caching for frequently accessed globals
pub struct CachedGlobal<T> {
    engine: Arc<Mutex<Engine>>,
    name: String,
    cached: Arc<RwLock<Option<T>>>,
}

impl<T: TryFromValue + Clone> CachedGlobal<T> {
    pub fn new(engine: Arc<Mutex<Engine>>, name: impl Into<String>) -> Self {
        Self {
            engine,
            name: name.into(),
            cached: Arc::new(RwLock::new(None)),
        }
    }

    pub fn get(&self) -> Result<T> {
        // Fast path: read from cache
        {
            let cached = self.cached.read().unwrap();
            if let Some(value) = cached.as_ref() {
                return Ok(value.clone());
            }
        }

        // Slow path: fetch from engine and cache
        let mut engine = self.engine.lock().unwrap();
        let value: T = engine.get_global(&self.name)?.try_into()?;

        let mut cached = self.cached.write().unwrap();
        *cached = Some(value.clone());

        Ok(value)
    }

    pub fn invalidate(&self) {
        let mut cached = self.cached.write().unwrap();
        *cached = None;
    }
}

// Usage
let font_size = CachedGlobal::<f64>::new(engine.clone(), "Config.font_size");
let size = font_size.get()?; // First call: fetches from engine
let size = font_size.get()?; // Subsequent calls: instant cache hit

// Invalidate on reload
engine.on_reload(|_, _| {
    font_size.invalidate();
});
```

### 8.2 Minimize Allocations

```rust
// String interning for frequently used strings
pub struct StringInterner {
    strings: HashMap<String, Rc<str>>,
}

impl StringInterner {
    pub fn intern(&mut self, s: impl AsRef<str>) -> Rc<str> {
        let s = s.as_ref();
        self.strings.entry(s.to_string())
            .or_insert_with(|| s.into())
            .clone()
    }
}

// Use in Value construction
impl Value {
    pub fn new_string(s: impl Into<String>, interner: &mut StringInterner) -> Self {
        Value::String(interner.intern(s.into()))
    }
}

// Arena allocation for temporary values
use bumpalo::Bump;

pub struct Arena {
    bump: Bump,
}

impl Arena {
    pub fn new() -> Self {
        Self { bump: Bump::new() }
    }

    pub fn alloc_value(&self, value: Value) -> &Value {
        self.bump.alloc(value)
    }

    pub fn reset(&mut self) {
        self.bump.reset();
    }
}

// Usage in hot loops
let mut arena = Arena::new();

for item in items {
    let temp_value = arena.alloc_value(item.into());
    engine.call("process", &[temp_value.clone()])?;
}

arena.reset(); // Bulk deallocation
```

### 8.3 Efficient Callbacks

```rust
// Zero-copy callbacks with borrowing
pub trait HostFunctionZeroCopy {
    fn call_borrowed(&self, args: &[&Value]) -> Result<Value>;
}

// Register zero-copy function
engine.register_zero_copy_function("log", |args: &[&Value]| {
    for arg in args {
        // No clone needed, just borrow
        print!("{:?} ", arg);
    }
    println!();
    Ok(Value::Unit)
})?;

// Function inlining for simple host functions
#[inline(always)]
fn fast_add(a: i64, b: i64) -> i64 {
    a + b
}

engine.register_typed_function("add", fast_add)?;

// Batch function calls
let results = engine.call_batch(&[
    ("fn1", vec![arg1.clone()]),
    ("fn2", vec![arg2.clone()]),
    ("fn3", vec![arg3.clone()]),
])?;

// Better than:
// let r1 = engine.call("fn1", &[arg1])?;
// let r2 = engine.call("fn2", &[arg2])?;
// let r3 = engine.call("fn3", &[arg3])?;
```

### 8.4 Benchmarking

```rust
// Built-in performance profiling
let mut engine = Engine::builder()
    .enable_profiling(true)
    .build()?;

engine.call("expensive_function", &[])?;

let profile = engine.profile_report();
println!("Function calls: {}", profile.total_calls);
println!("Total time: {:?}", profile.total_duration);
println!("\nTop functions by time:");
for (func, duration) in profile.top_functions(10) {
    println!("  {}: {:?}", func, duration);
}

// Micro-benchmarks
#[bench]
fn bench_call_overhead(b: &mut Bencher) {
    let mut engine = Engine::new();
    engine.load_script_from_string("let noop () = ()", "bench").unwrap();

    b.iter(|| {
        engine.call("noop", &[]).unwrap();
    });
}

#[bench]
fn bench_value_conversion(b: &mut Bencher) {
    b.iter(|| {
        let value = Value::Int(42);
        let n: i64 = value.try_into().unwrap();
        black_box(n);
    });
}
```

---

## Summary

This architecture provides:

1. **Ergonomic API**: Natural Rust patterns with builder, typed functions, and derive macros
2. **Type Safety**: Strong typing with compile-time checks via TryFromValue/IntoValue
3. **Performance**: Zero-copy where possible, caching, inlining, and profiling tools
4. **Hot-Reload**: File watching, incremental recompilation, state preservation
5. **Memory Safety**: Hybrid GC with cycle detection, Rust ownership integration
6. **Error Handling**: Rich error types, stack traces, graceful recovery
7. **Flexibility**: Supports diverse use cases from config files to full plugin systems

The API design prioritizes developer experience while maintaining the performance and safety guarantees expected from a Rust-based system.

---

## Next Steps

1. Implement core `Engine` API in `/src/host/engine.rs`
2. Implement `Value` type system in `/src/runtime/value.rs`
3. Implement GC in `/src/runtime/gc.rs`
4. Implement hot-reload watcher in `/src/runtime/hotreload.rs`
5. Create derive macros for `IntoValue` and `TryFromValue` in `/src/runtime/macros.rs`
6. Write integration tests in `/tests/host_interop_test.rs`
7. Create examples in `/examples/terminal_config.rs`, `/examples/plugin_system.rs`

See `/docs/ARCHITECTURE.md` for overall system design.
