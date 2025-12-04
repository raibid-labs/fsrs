# Fusabi Module System

## Overview

The Fusabi module system enables code organization and reusability through named modules, imports, and qualified names. This implementation follows F# module semantics.

## Architecture

### Core Components

1. **AST Extensions** (`crates/fusabi-frontend/src/ast.rs`)
   - `ModuleDef`: Named module with items
   - `ModuleItem`: Items that can appear in modules (let bindings, types, nested modules)
   - `Import`: Open/import statements
   - `Program`: Top-level structure with modules and main expression

2. **Module Registry** (`crates/fusabi-frontend/src/modules.rs`)
   - `ModuleRegistry`: Registry for name resolution
   - `Module`: Compiled module with bindings and types
   - `ModulePath`: Path for nested modules
   - `TypeDefinition`: Type exports from modules

3. **Lexer Updates** (`crates/fusabi-frontend/src/lexer.rs`)
   - Added `Open` token for import statements
   - Added `Module` token for module definitions

## Features

### 1. Module Definitions

Define modules with named bindings:

```fsharp
module Math =
    let add x y = x + y
    let multiply x y = x * y
    let square x = multiply x x
```

### 2. Open Imports

Bring module bindings into scope:

```fsharp
open Math

let result = add 5 10  // Uses Math.add
```

### 3. Qualified Names

Access module members explicitly:

```fsharp
let result = Math.add 5 10  // Explicit module qualification
```

### 4. Nested Modules

Support for module nesting:

```fsharp
module Geometry =
    module Point =
        let make x y = (x, y)
        let distance p1 p2 = ...

    let origin = Point.make 0 0

let p = Geometry.Point.make 3 4
```

## Implementation Details

### AST Structure

```rust
// Module definition
pub struct ModuleDef {
    pub name: String,
    pub items: Vec<ModuleItem>,
}

// Module items (bindings, types, nested modules)
pub enum ModuleItem {
    Let(String, Expr),
    LetRec(Vec<(String, Expr)>),
    TypeDef(TypeDefinition),
    Module(Box<ModuleDef>),
}

// Import statement
pub struct Import {
    pub module_path: Vec<String>,  // e.g., ["Math", "Geometry"]
    pub is_qualified: bool,         // false for "open"
}

// Complete program
pub struct Program {
    pub modules: Vec<ModuleDef>,
    pub imports: Vec<Import>,
    pub main_expr: Option<Expr>,
}
```

### Module Registry

```rust
pub struct ModuleRegistry {
    modules: HashMap<String, Module>,
}

impl ModuleRegistry {
    // Register a module
    pub fn register_module(
        &mut self,
        name: String,
        bindings: HashMap<String, Expr>,
        types: HashMap<String, TypeDefinition>,
    );

    // Resolve qualified name (e.g., "Math.add")
    pub fn resolve_qualified(&self, module_name: &str, binding_name: &str) -> Option<&Expr>;

    // Get all bindings for "open" imports
    pub fn get_module_bindings(&self, module_name: &str) -> Option<&HashMap<String, Expr>>;
}
```

### Variable Resolution Order

When resolving a variable name:

1. Check **local scope** (current bindings)
2. Check **opened modules** (imports via "open")
3. Check **qualified names** (Module.function syntax)
4. Error if not found

## Usage Examples

### Example 1: Basic Module

```fsharp
module Math =
    let add x y = x + y
    let multiply x y = x * y

open Math

let result = multiply (add 3 4) 2  // Result: 14
```

### Example 2: Nested Modules

```fsharp
module Geometry =
    module Point =
        let make x y = (x, y)
        let x p = let (x, _) = p in x
        let y p = let (_, y) = p in y

let p = Geometry.Point.make 10 20
let px = Geometry.Point.x p  // Result: 10
```

### Example 3: Multiple Modules

```fsharp
module String =
    let length s = ...
    let upper s = ...

module List =
    let length xs = ...
    let map f xs = ...

// Use qualified names to avoid conflicts
let s_len = String.length "hello"
let l_len = List.length [1; 2; 3]
```

### Example 4: Library Pattern

```fsharp
module MathLib =
    let abs x = if x < 0 then -x else x
    let max x y = if x > y then x else y

    let rec factorial n =
        if n <= 1 then 1
        else n * factorial (n - 1)

open MathLib

let a = abs (-42)      // 42
let b = max 10 20      // 20
let c = factorial 5    // 120
```

## Files Modified/Created

### Modified Files

1. `/crates/fusabi-frontend/src/ast.rs`
   - Added `ModuleDef`, `ModuleItem`, `Import`, `Program` types
   - Added Display implementations

2. `/crates/fusabi-frontend/src/lexer.rs`
   - Added `Open` and `Module` tokens
   - Added keyword matching for "open" and "module"

3. `/crates/fusabi-frontend/src/lib.rs`
   - Added `modules` module
   - Re-exported module-related types

### Created Files

1. `/crates/fusabi-frontend/src/modules.rs`
   - Module registry implementation
   - Name resolution system
   - Type definitions for module system

2. `/examples/modules_basic.fsx`
   - Basic module usage example

3. `/examples/modules_nested.fsx`
   - Nested modules example

4. `/examples/modules_math.fsx`
   - Math library module example

5. `/docs/module_system.md`
   - This documentation file

## Testing

The module system includes unit tests in `modules.rs`:

```bash
cargo test --package fusabi-frontend modules
```

Key test coverage:
- Module registration and lookup
- Qualified name resolution
- Module bindings retrieval
- Nested module paths
- Type definitions in modules

## Next Steps

To fully integrate the module system:

1. **Parser Extensions** (Not yet implemented)
   - Add `parse_program()` method
   - Add `parse_module()` method
   - Add `parse_import()` method
   - Add `parse_module_items()` method

2. **Compiler Integration** (Not yet implemented)
   - Update compiler to use ModuleRegistry
   - Handle qualified variable lookup
   - Handle open imports during compilation
   - Proper scope management

3. **Type Checker Integration** (Future)
   - Track module types in TypeEnv
   - Type check across module boundaries
   - Handle type exports and imports

4. **Additional Features** (Future)
   - Module signatures/interfaces
   - Private vs public bindings
   - Module aliases
   - Selective imports (open Module(specific, bindings))

## Design Decisions

### Why This Architecture?

1. **Separation of Concerns**: Module registry is separate from AST parsing and compilation
2. **Flexibility**: Easy to extend with additional features
3. **Type Safety**: Strong typing with Rust enums and structs
4. **F# Compatibility**: Follows F# module semantics where possible
5. **Incremental Implementation**: Can be implemented and tested in phases

### Limitations (Current Phase)

- Parser support not yet implemented
- Compiler integration not yet complete
- No module signatures/interfaces
- No selective imports
- All bindings are public
- No module aliases

### Performance Considerations

- HashMap-based lookups: O(1) average case
- Linear search for qualified names: acceptable for typical module sizes
- No caching yet - can be added if needed

## Related Documentation

- [F# Module Documentation](https://docs.microsoft.com/en-us/dotnet/fsharp/language-reference/modules)
- [Fusabi AST Documentation](../crates/fusabi-frontend/src/ast.rs)
- [Fusabi Parser Documentation](../crates/fusabi-frontend/src/parser.rs)

## Conclusion

The module system provides a solid foundation for code organization in Fusabi. The current implementation includes the core data structures and registry system. The next phase will add parser support and compiler integration to make the system fully functional.
