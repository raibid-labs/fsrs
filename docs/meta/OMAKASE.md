# The Omakase Cookbook

> Welcome to the Omakase. Hand-rolled, chef-selected examples of Fusabi in action.

In the spirit of omakase (chef's choice), we've curated a progression of examples that showcase Fusabi's capabilities. Like a fine dining experience, we start with simple appetizers, move through substantial main courses, and finish with fusion dishes that blend Fusabi with Rust.

## Table of Contents

- [Appetizers](#appetizers-simple-bites) - Simple, focused examples to get started
- [Main Courses](#main-courses-full-configurations) - Complete, real-world configurations
- [Fusion](#fusion-rust-interop) - Blending Fusabi scripts with Rust applications

---

## Appetizers: Simple Bites

These examples are your amuse-bouche - small, focused demonstrations of individual features. Perfect for getting a taste of what Fusabi can do.

### Hello, Fusabi

The classic first bite. Simple, elegant, essential.

**File**: `examples/hello.fsx`

```fsharp
// Hello World Example
// This demonstrates the simplest FSRS script

"Hello, FSRS!"
```

**Run it**:
```bash
fus run examples/hello.fsx
# Or compile first for faster execution:
fus grind examples/hello.fsx && fus run examples/hello.fzb
```

---

### Arithmetic Foundations

Basic mathematical expressions - the building blocks of computation.

**File**: `examples/arithmetic.fsx`

Demonstrates:
- Integer arithmetic (`+`, `-`, `*`, `/`)
- Operator precedence
- Let bindings for computed values

**Try it**:
```bash
fus run examples/arithmetic.fsx
```

---

### Boolean Logic

True or false? Simple conditionals and logical operations.

**File**: `examples/boolean_logic.fsx`

Demonstrates:
- Boolean literals (`true`, `false`)
- Logical operators (`&&`, `||`, `not`)
- Comparison operators (`=`, `<>`, `<`, `>`, `<=`, `>=`)

---

### Pattern Matching Basics

The gateway to functional elegance - matching values to actions.

**File**: `examples/pattern_matching_basic.fsx`

```fsharp
let describe n =
  match n with
  | 0 -> "zero"
  | 1 -> "one"
  | 2 -> "two"
  | _ -> "many"

print (describe 0)   // => "zero"
print (describe 5)   // => "many"
```

**What you'll learn**:
- Literal patterns
- Wildcard patterns (`_`)
- Pattern matching on numbers, booleans, and strings

---

### Lists: The Basics

Fusabi's fundamental data structure - the linked list.

**File**: `examples/lists_basic.fsx`

```fsharp
// List literal syntax
let numbers = [1; 2; 3; 4; 5]

// Cons operator (::)
let extended = 0 :: numbers

// Empty list
let empty = []
```

**Demonstrates**:
- List literals with semicolon separators
- Cons operator (`::`) for prepending
- Empty list notation (`[]`)

---

### Tuples: Structured Data

Package multiple values together in a lightweight structure.

**File**: `examples/tuples_basic.fsx`

```fsharp
// Simple pair
let point = (3, 4)

// Triple
let person = ("Alice", 30, "NYC")

// Nested tuples
let complex = ((1, 2), (3, 4))
```

**Demonstrates**:
- Tuple literals
- Accessing tuple elements
- Nested tuples
- Tuple equality

---

### Records: Named Fields

When you need more structure than tuples, records provide named fields.

**File**: `examples/records_basics.fsx`

```fsharp
let person = { name = "Alice"; age = 30; city = "NYC" }

let personName = person.name
let personAge = person.age

// Nested records
let config = {
    app = {
        name = "MyApp";
        version = 1
    };
    server = {
        host = "localhost";
        port = 8080
    }
}

let appName = config.app.name
let serverPort = config.server.port
```

**What you'll learn**:
- Record literals
- Field access with dot notation
- Nested records
- Records in lists

---

### Functions and Currying

Fusabi functions are automatically curried - partial application comes free.

**File**: `examples/currying_simple.fsx`

```fsharp
let add x y = x + y

// Partial application creates specialized functions
let add10 = add 10
let add20 = add 20

let result1 = add10 5   // Returns 15
let result2 = add20 5   // Returns 25
```

**Demonstrates**:
- Multi-parameter functions
- Partial application
- Creating specialized functions from general ones

---

### Pipeline Operator

Chain operations together for readable, top-to-bottom data transformations.

**File**: `examples/pipeline_demo.fsx`

```fsharp
// String transformation pipeline
let rawText = "  HELLO FUSABI WORLD  "

let cleaned = rawText
    |> String.trim
    |> String.toLower
    |> String.split " "
    |> List.length

print cleaned  // => 3

// Function composition
let double x = x * 2
let addTen x = x + 10

let result = 5
    |> double
    |> addTen
    |> double

print result  // => 40
```

**What you'll learn**:
- Pipeline operator (`|>`) for function composition
- Chaining list operations
- Building readable data transformations
- Avoiding nested function calls

---

### Error Handling with Options

Safe error handling using the Option type - no exceptions needed.

**File**: `examples/error_handling.fsx`

```fsharp
// Safe division
let safeDivide x y =
    match y with
    | 0 -> None
    | _ -> Some(x / y)

let result1 = safeDivide 10 2   // Some(5)
let result2 = safeDivide 10 0   // None

// Default values
let value1 = result1 |> Option.defaultValue 0   // => 5
let value2 = result2 |> Option.defaultValue 0   // => 0

// Configuration lookup
let getConfig key =
    match key with
    | "host" -> Some("localhost")
    | "port" -> Some("8080")
    | _ -> None

let host = getConfig "host" |> Option.defaultValue "0.0.0.0"
let timeout = getConfig "timeout" |> Option.defaultValue "30"
```

**Demonstrates**:
- Option type for safe operations
- `Some` and `None` constructors
- `Option.defaultValue` for fallbacks
- Pattern matching on options
- Real-world error handling patterns

---

### Modules: Code Organization

Organize your code into namespaces with modules.

**File**: `examples/modules_basic.fsx`

```fsharp
module Math =
    let add x y = x + y
    let multiply x y = x * y
    let square x = multiply x x

open Math

let result1 = square (add 3 4)
let result2 = Math.add 10 20
```

**What you'll learn**:
- Module definition
- `open` imports
- Qualified vs unqualified access

---

## Main Courses: Full Configurations

These are complete, production-ready examples. Like a well-composed main course, they bring together multiple ingredients into cohesive applications.

### Quickstart: Complete Tour

A comprehensive introduction that brings together all the key features.

**File**: `examples/quickstart.fsx`

This example walks through:
- Basic values and expressions
- Let bindings and functions
- Lists and pattern matching
- Records and structured data
- Higher-order functions
- Pipeline operator
- Recursive functions
- Real-world configuration modeling

Perfect for newcomers who want to see everything in one place.

```fsharp
// Functions
let add x y = x + y
let square x = x * x

// Lists
let numbers = [1; 2; 3; 4; 5]
let count = List.length numbers

// Pattern matching
let classify n =
    match n with
    | 0 -> "zero"
    | 1 -> "one"
    | _ -> "many"

// Pipeline operator
let text = "  hello world  "
let processed = text
    |> String.trim
    |> String.toUpper

// Records
let person = {
    name = "Alice";
    age = 30;
    active = true
}

// Real-world config
let config = {
    server = { host = "localhost"; port = 8080 };
    database = { name = "myapp"; poolSize = 10 };
    features = { debug = true; logging = true }
}
```

**Use case**: Learning Fusabi, reference implementation, teaching

---

### MiniFS Configuration DSL

A complete domain-specific language for terminal multiplexer configuration.

**File**: `examples/minifs_config.fsx`

This example showcases:
- Multiple modules working together
- Discriminated unions for command types
- Record types for structured data
- Nested data structures
- Real-world configuration modeling

```fsharp
module Layouts =
  type Layout =
    | Pane of cmd: string * width: int option
    | Row of Layout list
    | Column of Layout list

  let default =
    layout {
      row {
        pane { cmd "htop"; width 30 }
        column {
          pane { cmd "cargo watch -x test" }
          pane { cmd "cargo watch -x run" }
        }
      }
    }

module Keys =
  type Direction = Left | Right | Up | Down

  type Action =
    | Split of Direction
    | MoveFocus of Direction
    | SendKeys of string
    | RenameTab of string

  type KeyBinding =
    { Key: string
      Action: Action }

  let bindings =
    keys {
      bind "Ctrl-Shift-H" (MoveFocus Left)
      bind "Ctrl-Shift-L" (MoveFocus Right)
      bind "Ctrl-Shift-Enter" (Split Down)
    }

module Config =
  open Layouts
  open Keys

  type Config =
    { Layout : Layout
      KeyBindings : KeyBinding list }

  let config : Config =
    { Layout = Layouts.default
      KeyBindings = Keys.bindings }
```

**Use case**: Configuration files, DSLs, declarative specifications

**Run it**:
```bash
fus run examples/minifs_config.fsx
```

---

### Data Transformation Patterns

Real-world data processing scenarios using Fusabi's functional features.

**File**: `examples/data_transformation.fsx`

This comprehensive example covers:
- CSV-like data parsing
- Log parsing and filtering
- URL path parsing
- Configuration file processing
- JSON-like data structures
- Data aggregation
- Report generation

```fsharp
// CSV parsing
let csvData = "Alice,30,Engineer|Bob,25,Designer|Carol,35,Manager"
let records = String.split "|" csvData
let firstRecord = List.head records
let fields = String.split "," firstRecord

// Building structured data
let parseRecord record =
    let parts = String.split "," record
    let name = List.head parts
    let rest = List.tail parts
    let age = List.head rest
    let remaining = List.tail rest
    let role = List.head remaining
    { name = name; age = age; role = role }

// Log filtering
let logEntries = [
    "INFO: Application started";
    "ERROR: Connection failed";
    "INFO: Processing request"
]

let isError entry = String.startsWith "ERROR:" entry

// URL parsing
let url = "/api/users/123/posts/456"
let segments = String.split "/" url
let cleaned = List.tail segments  // Remove empty first element

// Nested data structures
let user = {
    id = 123;
    profile = {
        name = "Alice";
        email = "alice@example.com"
    };
    settings = {
        theme = "dark";
        notifications = true
    }
}
```

**Use case**: ETL pipelines, log analysis, API transformation, data validation

---

### Configuration Validation

Practical validation patterns using Option types for safe configuration handling.

**File**: `examples/config_validator.fsx`

Demonstrates:
- Validation functions returning Options
- Range checking and constraints
- Combining multiple validations
- Providing default values for invalid configs
- Building validated configuration objects

```fsharp
let config = {
    host = "localhost";
    port = 8080;
    maxConnections = 100;
    timeout = 30;
    enableLogging = true
}

let validatePort port =
    match port with
    | p when p > 0 && p < 65536 -> Some(port)
    | _ -> None

let validateTimeout timeout =
    match timeout with
    | t when t > 0 && t < 3600 -> Some(timeout)
    | _ -> None

let validateConfig cfg =
    let vPort = validatePort cfg.port
    let vTimeout = validateTimeout cfg.timeout

    match (vPort, vTimeout) with
    | (Some(p), Some(t)) ->
        Some({
            host = cfg.host;
            port = p;
            timeout = t;
            enableLogging = cfg.enableLogging
        })
    | _ -> None

let validatedConfig = validateConfig config
```

**Use case**: API request validation, configuration file validation, user input sanitization

---

### Standard Library Tour

A comprehensive demonstration of Fusabi's built-in standard library.

**File**: `examples/stdlib_demo.fsx`

This example demonstrates:
- **List module**: `length`, `head`, `tail`, `reverse`, `append`, `concat`, `isEmpty`
- **String module**: `length`, `trim`, `toUpper`, `toLower`, `split`, `concat`, `contains`, `startsWith`, `endsWith`
- **Option module**: `isSome`, `isNone`, `defaultValue`
- **Pipeline operator** (`|>`): Composing operations functionally

```fsharp
// String cleaning pipeline
let rawInput = "  HELLO WORLD  "
let cleaned = rawInput
    |> String.trim
    |> String.toLower    // "hello world"

// List operations
let numbers = [1; 2; 3; 4; 5]
let count = List.length numbers      // 5
let first = List.head numbers        // 1
let rest = List.tail numbers         // [2; 3; 4; 5]
let backwards = List.reverse numbers // [5; 4; 3; 2; 1]

// Combining list and string operations
let paths = ["/home"; "/user"; "/docs"]
let fullPath = String.concat (List.append paths ["/file.txt"])
// Result: "/home/user/docs/file.txt"
```

**Use case**: Data transformation, text processing, functional pipelines

---

### Fibonacci with Pattern Matching

Computing Fibonacci numbers using recursive pattern matching.

**File**: `examples/lists_recursive.fsx`

Demonstrates:
- Recursive functions
- Pattern matching on lists
- Cons patterns (`::`)
- Building lists recursively

**Use case**: Recursive algorithms, list processing

---

### Advanced Pattern Matching

Pattern matching at its finest - nested patterns, tuples, and records.

**File**: `examples/pattern_matching_nested.fsx`

This example covers:
- Nested pattern matching
- Tuple patterns
- List patterns with cons (`::`)
- Combining multiple pattern types
- Guard patterns (when supported)

**Use case**: Complex data parsing, protocol handling, state machines

---

### Record Pattern Matching

Destructuring records with pattern matching.

**File**: `examples/records_patterns.fsx`

```fsharp
type Person = { name: string; age: int; city: string }

let describe person =
  match person with
  | { age = 0 } -> "newborn"
  | { age = age } when age < 13 -> "child"
  | { age = age } when age < 20 -> "teenager"
  | { age = age } when age < 65 -> "adult"
  | _ -> "senior"
```

**What you'll learn**:
- Record patterns
- Field extraction in patterns
- Combining patterns with guards

---

## Fusion: Rust Interop

These examples demonstrate Fusabi's unique dual-runtime capability - seamlessly blending F# scripts with Rust host applications.

### Bytecode Compilation Demo

Experience the dual-runtime in action - same code, two execution modes.

**File**: `examples/bytecode_demo.fsx`

This example showcases Fusabi's unique capability to run scripts in two modes:

**Interpreted Mode (.fsx)**:
```bash
fus run examples/bytecode_demo.fsx
# Startup: ~10-50ms (includes parsing + compilation + execution)
# Ideal for: Development, rapid iteration, debugging
```

**Compiled Mode (.fzb)**:
```bash
fus grind examples/bytecode_demo.fsx  # Produces bytecode_demo.fzb
fus run examples/bytecode_demo.fzb
# Startup: ~1-5ms (bytecode loading + execution only)
# Ideal for: Production, CLI tools, embedded scripts
```

The script includes:
- Fibonacci computation (CPU-intensive work)
- String processing pipelines
- List operations
- Pattern matching
- Nested records
- Performance benchmarking guidance

```fsharp
// Same code, two execution modes
let rec fibonacci n =
    match n with
    | 0 -> 0
    | 1 -> 1
    | _ -> fibonacci (n - 1) + fibonacci (n - 2)

let fib10 = fibonacci 10
print fib10  // => 55

// String processing
let message = "  Fusabi is FAST  "
let processed = message
    |> String.trim
    |> String.toLower
    |> String.toUpper

// Try timing both modes:
// $ time fus run examples/bytecode_demo.fsx
// $ time fus run examples/bytecode_demo.fzb
```

**When to use each mode**:
- `.fsx` (interpreted): Development, debugging, learning, one-off scripts
- `.fzb` (compiled): Production, distribution, faster startup, source protection

**Performance tip**: The compiled version typically shows 5-10x faster startup with identical execution performance.

---

### Host Interop Demo

The comprehensive guide to embedding Fusabi in Rust applications.

**File**: `examples/host_interop_demo.rs`

This is a complete Rust program that demonstrates:

1. **Registering host functions** - Expose Rust functions to Fusabi scripts
2. **Type conversions** - Bridge between Rust and Fusabi type systems
3. **Value operations** - Working with integers, strings, booleans, lists
4. **Global bindings** - Share state between Rust and scripts
5. **Function composition** - Combining host functions from Rust

**Key examples from the demo**:

```rust
// Example 1: Simple arithmetic
engine.register_fn1("double", |v| {
    let n = v.as_int()
        .ok_or_else(|| fusabi_vm::VmError::Runtime("Expected int".into()))?;
    Ok(Value::Int(n * 2))
});

// Example 2: String manipulation
engine.register_fn1("greet", |v| {
    let name = v.as_str()
        .ok_or_else(|| fusabi_vm::VmError::Runtime("Expected string".into()))?;
    Ok(Value::Str(format!("Hello, {}!", name)))
});

// Example 3: Binary functions
engine.register_fn2("max", |a, b| {
    let x = a.as_int()?;
    let y = b.as_int()?;
    Ok(Value::Int(x.max(y)))
});

// Example 4: List processing
engine.register_fn1("sum", |v| {
    let list = v.list_to_vec()
        .ok_or_else(|| fusabi_vm::VmError::Runtime("Expected list".into()))?;
    let sum: i64 = list.iter().filter_map(|v| v.as_int()).sum();
    Ok(Value::Int(sum))
});

// Example 5: Global bindings
engine.set_global("version", Value::Str("0.1.0".to_string()));
engine.set_global("max_size", Value::Int(1000));
```

**Building and running**:
```bash
cd rust
cargo run --example host_interop_demo
```

**Output**:
```
=== Fusabi Host Interop Demo ===

Example 1: Simple Arithmetic
  double(21) = 42

Example 2: String Manipulation
  greet('World') = Hello, World!

Example 3: Binary Functions
  max(10, 20) = 20

Example 4: List Processing
  sum([1; 2; 3]) = 6

Example 5: List Generation
  range(5) = [1; 2; 3; 4; 5]

...
```

**Use cases**:
- Plugin systems
- Configuration engines
- Rule engines
- Scripting for games
- Data transformation pipelines
- Testing frameworks

---

### Plugin System Example

A complete plugin implementation showing how Fusabi scripts can extend Rust applications.

**File**: `examples/plugin_example.fsx`

This example demonstrates a full-featured plugin system:

**Plugin Features**:
- Plugin metadata and versioning
- Command handlers
- Event processing
- Configuration validation
- State management
- Utility functions

```fsharp
// Plugin metadata
let pluginInfo = {
    name = "example-plugin";
    version = 1;
    author = "Fusabi Team";
    description = "Example plugin demonstrating the plugin API"
}

// Command handlers
let handleGreeting name =
    String.concat ["Hello, "; name; " from the plugin!"]

let handleCalculation x y operation =
    match operation with
    | "add" -> x + y
    | "subtract" -> x - y
    | "multiply" -> x * y
    | "divide" ->
        match y with
        | 0 -> 0  // Safe division
        | _ -> x / y
    | _ -> 0

// Event handlers
let onStartup config =
    let message = String.concat ["Plugin started with config: "; config.environment]
    { success = true; message = message }

let onRequest request =
    match request.method with
    | "GET" -> { status = 200; body = "GET response from plugin" }
    | "POST" -> { status = 201; body = "POST response from plugin" }
    | _ -> { status = 405; body = "Method not allowed" }

// Configuration validation
let validatePluginConfig cfg =
    let hasName = String.length cfg.name > 0
    let hasValidPort = cfg.port > 0 && cfg.port < 65536
    let hasValidTimeout = cfg.timeout > 0
    hasName && hasValidPort && hasValidTimeout
```

**Rust host integration**:
```rust
let mut engine = FusabiEngine::new();

// Register host functions for plugins
engine.register_fn1("log", |msg| {
    println!("[LOG] {}", msg);
    Ok(Value::Unit)
});

// Load the plugin
engine.load_plugin("plugin_example.fsx")?;

// Call plugin functions
let greeting = engine.call("handleGreeting", &[
    Value::Str("Alice".to_string())
])?;

let result = engine.call("handleCalculation", &[
    Value::Int(10),
    Value::Int(5),
    Value::Str("add".to_string())
])?;
```

**Use cases**:
1. **Application extensibility** - Users add custom features without recompiling
2. **Game scripting** - AI behavior, quest logic, item effects
3. **Data processing** - Custom transformations, validation rules
4. **Configuration management** - Dynamic routing, feature flags, policy enforcement
5. **Testing frameworks** - Test definitions, mock data, assertions

**Benefits of Fusabi plugins**:
- Hot-reload without restarting the host application
- Sandboxed execution for safety
- Type-safe communication via the Value API
- Fast startup with .fzb compiled plugins
- F# syntax for plugin authors

---

### Embedding Patterns

Here are common patterns for embedding Fusabi in your Rust application:

#### Pattern 1: Simple Script Executor

```rust
use fusabi_vm::{Vm, Value};

fn execute_script(script: &str) -> Result<Value, fusabi_vm::VmError> {
    let mut vm = Vm::new();
    vm.eval(script)
}

// Usage
let result = execute_script("40 + 2")?;
assert_eq!(result.as_int(), Some(42));
```

#### Pattern 2: Reusable Engine with Host Functions

```rust
use fusabi_demo::FusabiEngine;
use fusabi_vm::Value;

struct MyApp {
    engine: FusabiEngine,
}

impl MyApp {
    fn new() -> Self {
        let mut engine = FusabiEngine::new();

        // Register app-specific functions
        engine.register_fn1("log", |v| {
            println!("[APP] {}", v);
            Ok(Value::Unit)
        });

        engine.register_fn2("add", |a, b| {
            let x = a.as_int()?;
            let y = b.as_int()?;
            Ok(Value::Int(x + y))
        });

        Self { engine }
    }

    fn run_user_script(&mut self, script: &str) -> Result<Value, fusabi_vm::VmError> {
        self.engine.eval(script)
    }
}
```

#### Pattern 3: Configuration Loading

```rust
use fusabi_vm::{Vm, Value};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct AppConfig {
    host: String,
    port: i64,
    debug: bool,
}

fn load_config(script: &str) -> Result<AppConfig, Box<dyn std::error::Error>> {
    let mut vm = Vm::new();
    let result = vm.eval(script)?;

    // Convert Fusabi record to Rust struct
    // (Implementation depends on your needs)
    let config = convert_to_config(result)?;
    Ok(config)
}

// Script example: examples/app_config.fsx
// let config = {
//     host = "localhost";
//     port = 8080;
//     debug = true
// }
```

#### Pattern 4: Plugin System

```rust
use fusabi_vm::Value;
use std::collections::HashMap;

struct PluginManager {
    engine: FusabiEngine,
    plugins: HashMap<String, String>,
}

impl PluginManager {
    fn new() -> Self {
        let mut engine = FusabiEngine::new();

        // Register plugin API
        engine.register_fn1("register_command", |v| {
            // Plugin registration logic
            Ok(Value::Unit)
        });

        Self {
            engine,
            plugins: HashMap::new(),
        }
    }

    fn load_plugin(&mut self, name: &str, script: &str) -> Result<(), fusabi_vm::VmError> {
        self.plugins.insert(name.to_string(), script.to_string());
        self.engine.eval(script)?;
        Ok(())
    }
}
```

---

## Dual-Runtime Examples

These examples specifically demonstrate Fusabi's unique capability to run both as interpreted scripts and compiled bytecode.

### Hello World - Both Modes

```bash
# Mode 1: Direct interpretation (slower startup, faster iteration)
fus run examples/hello.fsx

# Mode 2: Compile to bytecode (faster startup, optimized)
fus grind examples/hello.fsx    # Produces hello.fzb
fus run examples/hello.fzb      # Runs the bytecode

# The output is identical, but startup time differs:
# - .fsx: ~10-50ms (includes parsing + compilation + execution)
# - .fzb: ~1-5ms (bytecode loading + execution only)
```

### When to Use Each Mode

**Use `.fsx` (interpreted) when**:
- Developing and iterating rapidly
- Running one-off scripts
- Script changes frequently
- Startup time is not critical
- You want to read/modify the source easily

**Use `.fzb` (compiled) when**:
- Deploying to production
- Running the same script repeatedly
- Startup time is critical (CLI tools, hot paths)
- You want to protect source code
- Distribution/packaging (smaller files, faster loading)

### Fibonacci - Performance Comparison

```bash
# Interpreted mode
time fus run examples/fibonacci.fsx

# Compiled mode
fus grind examples/fibonacci.fsx
time fus run examples/fibonacci.fzb

# The compiled version typically shows ~5-10x faster startup
# with identical execution performance
```

---

## Recipe Index

Quick reference of all examples by category:

### Getting Started
- `hello.fsx` - Hello world
- `quickstart.fsx` - Comprehensive tour of all features
- `arithmetic.fsx` - Basic math
- `boolean_logic.fsx` - Booleans and logic
- `conditionals.fsx` - If-then-else

### Data Structures
- `lists_basic.fsx` - List fundamentals
- `lists_operations.fsx` - List manipulation
- `lists_nested.fsx` - Nested lists
- `lists_recursive.fsx` - Recursive list processing
- `tuples_basic.fsx` - Tuple basics
- `tuples_nested.fsx` - Nested tuples
- `tuples_equality.fsx` - Tuple comparison
- `records_basics.fsx` - Record fundamentals
- `records_basic.fsx` - Record operations
- `records_advanced.fsx` - Advanced record features
- `arrays_basic.fsx` - Array basics
- `arrays_operations.fsx` - Array manipulation
- `arrays_nested.fsx` - Nested arrays
- `arrays_updates.fsx` - Array updates

### Pattern Matching
- `pattern_matching_basic.fsx` - Pattern basics
- `pattern_matching_tuples.fsx` - Tuple patterns
- `pattern_matching_nested.fsx` - Nested patterns
- `pattern_matching_functions.fsx` - Function patterns
- `records_patterns.fsx` - Record patterns

### Functions
- `currying_simple.fsx` - Basic currying
- `currying_higher_order.fsx` - Higher-order functions
- `fibonacci.fsx` - Recursive functions
- `pipeline_demo.fsx` - Pipeline operator and composition

### Error Handling
- `error_handling.fsx` - Option types and safe operations

### Modules
- `modules_basic.fsx` - Module basics
- `modules_math.fsx` - Math module example
- `modules_nested.fsx` - Nested modules
- `modules_compiled.fsx` - Compiled modules

### Standard Library
- `stdlib_demo.fsx` - Comprehensive stdlib tour

### Real-World Applications
- `minifs_config.fsx` - Configuration DSL
- `data_transformation.fsx` - Data processing patterns
- `config_validator.fsx` - Configuration validation

### Dual-Runtime / Performance
- `bytecode_demo.fsx` - .fsx vs .fzb comparison

### Rust Interop
- `host_interop_demo.rs` - Comprehensive host interop
- `plugin_example.fsx` - Plugin system implementation

---

## Tips and Best Practices

### Writing Idiomatic Fusabi

1. **Embrace immutability** - Let bindings are immutable by default
2. **Use pattern matching** - More elegant than if-then-else chains
3. **Leverage the pipeline operator** - `|>` for readable data transformations
4. **Keep functions small** - Compose larger operations from smaller functions
5. **Use modules for organization** - Group related functions together
6. **Type annotations are optional** - But can improve clarity for complex functions

### Performance Tips

1. **Compile for production** - Use `.fzb` files in production for faster startup
2. **Batch operations** - Process lists in bulk rather than element-by-element
3. **Use tail recursion** - When possible, for better stack usage
4. **Minimize string allocations** - Reuse strings where possible
5. **Profile before optimizing** - Measure first, optimize second

### Development Workflow

```bash
# 1. Write your script
vim my_script.fsx

# 2. Run directly during development
fus run my_script.fsx

# 3. Test with different inputs
fus run my_script.fsx --input test_data.json

# 4. Compile for deployment
fus grind my_script.fsx

# 5. Deploy the bytecode
cp my_script.fzb /opt/my_app/scripts/

# 6. Run in production
fus run /opt/my_app/scripts/my_script.fzb
```

---

## Next Steps

- Read the [Language Specification](02-language-spec.md) for complete syntax reference
- Explore [VM Design](03-vm-design.md) to understand bytecode execution
- Check out [Host Interop Documentation](../rust/docs/HOST_OBJECTS_AND_MODULES.md) for embedding details
- Review [Bytecode Format](bytecode-format.md) for `.fzb` file structure

---

**Itadakimasu!** Enjoy your Fusabi journey.
