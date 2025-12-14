# Multi-file Module System

Fusabi supports organizing code across multiple files using the `#load` directive.

## Overview

The `#load` directive allows you to include other `.fsx` files in your Fusabi program, enabling:

- **Code Organization**: Split large applications into manageable files
- **Reusability**: Share utilities across multiple scripts
- **Team Collaboration**: Multiple developers can work on different files
- **Hot Reload**: Only reload changed files (future TUI support)

## Syntax

```fsharp
#load "path/to/file.fsx"
```

### Path Resolution

1. **Relative paths**: Resolved from the current file's directory
2. **Absolute paths**: Used as-is

```fsharp
// Relative to current file
#load "utils.fsx"
#load "./components/button.fsx"
#load "../shared/http.fsx"

// Absolute path
#load "/home/user/libs/mylib.fsx"
```

## Example

### File Structure

```
project/
├── utils.fsx
├── math.fsx
└── main.fsx
```

### utils.fsx
```fsharp
module Utils =
    let add x y = x + y
    let multiply x y = x * y
    let square x = x * x
```

### math.fsx
```fsharp
#load "utils.fsx"

module Math =
    let pythagorean a b =
        let a2 = Utils.square a
        let b2 = Utils.square b
        Utils.add a2 b2
```

### main.fsx
```fsharp
#load "math.fsx"
#load "utils.fsx"  // Already loaded, uses cached version

let result = Math.pythagorean 3 4  // 25
let simple = Utils.add 10 20       // 30
```

## Behavior

### Caching

Files are loaded only once. Subsequent `#load` directives for the same file use the cached version.

### Circular Dependency Detection

The loader automatically detects and reports circular dependencies:

```fsharp
// a.fsx
#load "b.fsx"  // ERROR if b.fsx loads a.fsx

// b.fsx
#load "a.fsx"  // Circular dependency detected: a.fsx -> b.fsx -> a.fsx
```

### Evaluation Order

1. Directives are processed top-to-bottom
2. Each `#load` blocks until the file is fully evaluated
3. Loaded modules/bindings become available immediately after

```fsharp
#load "a.fsx"      // A's bindings now available
let x = A.func ()  // OK

#load "b.fsx"      // B's bindings now available
let y = B.func ()  // OK
```

## API Usage (Rust)

For programmatic file loading:

```rust
use fusabi_frontend::{FileLoader, LoadError};
use std::path::PathBuf;

let mut loader = FileLoader::new(PathBuf::from("."));
let loaded = loader.load("main.fsx", &PathBuf::from("entry.fsx"))?;

// Access the parsed program
println!("Loaded {} items", loaded.program.items.len());
```

## Error Handling

The loader provides clear error messages for common issues:

- **File Not Found**: `File not found: path/to/file.fsx`
- **Circular Dependency**: `Circular dependency detected: a.fsx -> b.fsx -> a.fsx`
- **Parse Error**: `Parse error in file.fsx:5:10: Unexpected token '}'`
- **Lex Error**: `Lex error in file.fsx: Unterminated string literal`

## See Also

- [RFC-003: Multi-file Module System](../design/RFC-003-MULTIFILE-MODULES.md)
- [Examples](../../examples/multifile/)
