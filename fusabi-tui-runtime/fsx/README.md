# Fusabi TUI Bindings

Fusabi language bindings for the TUI (Text User Interface) runtime.

## Overview

This directory contains `.fsx` (Fusabi script) bindings for the TUI types defined in the Rust `fusabi-tui-runtime` crate. These bindings allow TUI applications to be written in Fusabi and interact with the underlying Rust implementation.

## Current Limitations

**IMPORTANT**: These bindings use a simplified implementation due to current limitations in the Fusabi parser:

1. **No Record Types**: Fusabi's parser doesn't yet support record type definitions (`type T = { field: type }`). All types use discriminated unions (DUs) with tuples instead.

2. **No List Types in DUs**: The type `T list` syntax in DU definitions causes runtime errors. Types that would normally contain lists (like `Style` with modifiers) are simplified.

3. **No Module System**: The `module` keyword isn't fully implemented, so all definitions are at the top level.

## Structure

```
fsx/
├── tui/
│   ├── color.fsx       - Color type definitions (Black, Red, Green, Rgb, etc.)
│   ├── style.fsx       - Text styling (foreground/background colors)
│   ├── rect.fsx        - Rectangular areas (x, y, width, height)
│   ├── cell.fsx        - Single terminal cell (symbol + style)
│   ├── buffer.fsx      - 2D grid of cells (simplified)
│   ├── layout.fsx      - Layout constraints (simplified)
│   └── symbols.fsx     - Box drawing and block characters
├── tui.fsx             - Main entry point (loads all modules)
├── examples/
│   └── hello_tui.fsx   - Example TUI application
└── README.md           - This file
```

## Type Representations

Due to parser limitations, types are represented differently than idiomatic F#:

### Color (Fully Supported)
```fsharp
type Color =
    | Black | White | Red | Green | Blue | Yellow | Cyan | Magenta
    | Rgb of int * int * int
    | Indexed of int
```

### Style (Simplified)
```fsharp
type OptionColor = NoneColor | SomeColor of Color

type Style =
    | Style of OptionColor * OptionColor  // fg * bg (modifiers omitted)
```

**Note**: Modifier functions (`withBold`, `withItalic`, etc.) exist but are currently no-ops.

### Rect (Fully Supported)
```fsharp
type Rect =
    | Rect of int * int * int * int  // x * y * width * height
```

### Cell (Fully Supported)
```fsharp
type Cell =
    | Cell of string * Style  // symbol * style
```

### Buffer (Simplified)
```fsharp
type Buffer =
    | Buffer of Rect  // area only (content omitted)
```

**Note**: Buffer operations (`setCell`, `getCell`, `setBufferString`) are stubs.

### Layout (Simplified)
```fsharp
type Layout =
    | Layout of Direction * int  // direction * margin (constraints omitted)
```

## Usage

### Loading the Library

```fsharp
#load "tui.fsx"
```

### Creating a Colored Rectangle

```fsharp
let area = createRect 0 0 40 10
let buffer = createBuffer area

let greenStyle = emptyStyle |> withFg Green
let buffer2 = setBufferString 5 5 "Hello!" greenStyle buffer
```

### Box Drawing

```fsharp
let topLeft = lineTopLeft      // "┌"
let horizontal = lineHorizontal // "─"
let topRight = lineTopRight    // "┐"
```

## Verification

All `.fsx` files have been verified to parse correctly with the Fusabi parser:

```bash
cargo run -p fusabi --bin fus -- run fsx/tui/color.fsx   # OK
cargo run -p fusabi --bin fus -- run fsx/tui/style.fsx   # OK
cargo run -p fusabi --bin fus -- run fsx/tui/rect.fsx    # OK
cargo run -p fusabi --bin fus -- run fsx/tui/cell.fsx    # OK
cargo run -p fusabi --bin fus -- run fsx/tui/buffer.fsx  # OK
cargo run -p fusabi --bin fus -- run fsx/tui/layout.fsx  # OK
cargo run -p fusabi --bin fus -- run fsx/tui/symbols.fsx # OK
cargo run -p fusabi --bin fus -- run fsx/tui.fsx         # OK
```

The example compiles to bytecode:
```bash
cargo run -p fusabi --bin fus -- grind fsx/examples/hello_tui.fsx
# Compiled successfully -> hello_tui.fzb
```

## Future Improvements

When Fusabi's parser is enhanced to support:

1. **Record Types**: Convert DU-based types to proper records
   ```fsharp
   type Style = { fg: Color option; bg: Color option; modifiers: Modifier list }
   ```

2. **List Types in DUs**: Add back modifier lists and buffer content
   ```fsharp
   type Style = Style of OptionColor * OptionColor * Modifier list
   type Buffer = Buffer of Rect * Cell list
   ```

3. **Module System**: Organize code into proper modules
   ```fsharp
   module Color =
       type t = Black | White | Red | ...
   ```

## Integration with Rust

These bindings are designed to work with the Rust `fusabi-tui-runtime` crate through FFI:

- Type constructors map to Rust enum variants
- Functions map to Rust methods via the Fusabi VM
- The runtime will provide full implementations of simplified stubs

## License

Same as the parent project.
