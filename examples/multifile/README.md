# Multi-file Module System Example

This directory demonstrates the `#load` directive for multi-file module support in Fusabi.

## Files

- `utils.fsx` - Basic utility functions
- `math.fsx` - Math functions that depend on utils
- `main.fsx` - Main entry point that uses both modules

## Usage

The files demonstrate:
1. Loading dependencies with `#load "path.fsx"`
2. Circular dependency detection (utils is loaded once, not twice)
3. Module scoping across files

## How it works

```fsharp
// main.fsx loads both math.fsx and utils.fsx
#load "math.fsx"  // This also loads utils.fsx transitively
#load "utils.fsx" // Already loaded, uses cached version

// All modules are now available
let result = Math.pythagorean 3 4  // Uses Math.pythagorean from math.fsx
let simple = Utils.add 10 20       // Uses Utils.add from utils.fsx
```

The loader:
- Resolves paths relative to the current file
- Detects and prevents circular dependencies
- Caches loaded files to avoid recompilation
