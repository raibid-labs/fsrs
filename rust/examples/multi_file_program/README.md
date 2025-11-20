# Multi-File Program Example

This example demonstrates FSRS's multi-file module system with:

- Multiple module files
- Module imports (qualified and open)
- Cross-module function calls
- Standard library integration
- Host function interop

## Structure

```
multi_file_program/
├── math.fsrs         - Math utility module
├── string_utils.fsrs - String utility module
├── config.fsrs       - Configuration module
└── main.fsrs         - Main program entry point
```

## Running

```bash
# From the project root
cargo run --example multi_file_demo
```

## What it demonstrates

1. **Module definitions**: Separate modules in different files
2. **Qualified imports**: Using `Math.add` syntax
3. **Open imports**: Using `open Math` to import all bindings
4. **Nested modules**: Modules within modules
5. **Cross-module composition**: Functions from one module calling functions from another
6. **Standard library usage**: Using List, String, and Option modules
7. **Host interop**: Calling Rust functions from FSRS scripts
