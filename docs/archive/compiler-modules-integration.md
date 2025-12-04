# Compiler Integration for Module System

This document describes the compiler integration for the Fusabi module system, enabling module-aware bytecode generation.

## Overview

The module-aware compiler extends the base Fusabi compiler to support programs with module definitions, imports, and qualified names. This allows for better code organization and namespace management.

## Features

### 1. Module Registration

The compiler registers all modules and their bindings before compilation:

```rust
pub fn compile_program(program: &Program) -> CompileResult<Chunk> {
    let mut compiler = Compiler::new();
    let mut registry = ModuleRegistry::new();

    // Phase 1: Register all modules
    for module in &program.modules {
        compiler.register_module(&mut registry, module)?;
    }

    // ... continue with compilation
}
```

### 2. Import Resolution

The `open` statements bring module bindings into the current scope:

```fsharp
module Math =
    let add x y = x + y

open Math
add 5 10  // Unqualified access
```

### 3. Qualified Name Support

Access module bindings using qualified names:

```fsharp
module Math =
    let value = 100

Math.value  // Qualified access
```

## Implementation Details

### Compiler State

The compiler maintains module context:

```rust
pub struct Compiler {
    chunk: Chunk,
    locals: Vec<Local>,
    scope_depth: usize,
    options: CompileOptions,
    type_env: Option<TypeEnv>,

    // Module support
    module_registry: Option<ModuleRegistry>,
    imported_bindings: HashMap<String, Expr>,
}
```

### Three-Phase Compilation

1. **Phase 1: Module Registration**
   - Process all module definitions
   - Build the module registry
   - Register bindings and types

2. **Phase 2: Import Application**
   - Process all `open` statements
   - Bring imported bindings into scope
   - Validate module existence

3. **Phase 3: Main Expression Compilation**
   - Compile the main expression (if present)
   - Resolve qualified names
   - Access imported bindings

### Variable Resolution Order

When compiling a variable reference, the compiler checks in order:

1. **Qualified names** (e.g., `Math.add`)
2. **Local variables** (let bindings in current scope)
3. **Imported bindings** (from open statements)
4. **Error** if not found

## API Usage

### Direct Compilation

```rust
use fusabi_frontend::compiler::Compiler;
use fusabi_frontend::ast::Program;

let program = Program {
    modules: vec![/* ... */],
    imports: vec![/* ... */],
    main_expr: Some(/* ... */),
};

let chunk = Compiler::compile_program(&program)?;
```

### High-Level API

```rust
use fusabi_frontend::compile_program_from_source;

let source = "42";
let chunk = compile_program_from_source(source)?;
```

## Supported Features

| Feature | Status | Example |
|---------|--------|---------|
| Module definitions | ✅ | `module Math = ...` |
| Simple bindings | ✅ | `let x = 42` |
| Expression bindings | ✅ | `let sum = 3 + 4` |
| Recursive bindings | ✅ | `let rec factorial = ...` |
| Import statements | ✅ | `open Math` |
| Qualified names | ✅ | `Math.add` |
| Nested modules | ✅ | `module Outer = module Inner = ...` |
| Multiple imports | ✅ | `open Math; open Utils` |
| Type definitions | ✅ | Registered in module registry |

## Phase 1 Limitations

Due to Phase 1 constraints, the following have limited support:

1. **Lambda Compilation**: Lambdas are not fully compiled in Phase 1
   - Module bindings containing lambdas are registered but emit no instructions
   - This will be addressed in Phase 2 with proper closure support

2. **Type Checking Integration**: Module-level type checking is not yet integrated
   - Type definitions are registered but not enforced
   - Will be added in future phases

3. **Nested Module Access**: Only single-level qualification is supported
   - `Math.add` works
   - `Outer.Inner.func` is not yet supported

## Examples

### Example 1: Simple Constants

```fsharp
module Constants =
    let pi = 3
    let e = 2

Constants.pi + Constants.e  // Result: 5
```

### Example 2: Multiple Modules

```fsharp
module Math =
    let value = 10

module Utils =
    let doubled = 21 + 21

Math.value + Utils.doubled  // Result: 52
```

### Example 3: Imports

```fsharp
module Constants =
    let answer = 42

open Constants
answer  // Result: 42
```

### Example 4: Expressions as Bindings

```fsharp
module Math =
    let sum = 3 + 4

Math.sum  // Result: 7
```

## Testing

The module compiler integration is tested through comprehensive integration tests in `tests/compiler_modules.rs`:

- 18 test cases covering all supported features
- Error handling for undefined modules and bindings
- Support for various data types (int, bool, string)
- Nested module compilation
- Multiple imports and qualified names

Run tests with:
```bash
cargo test --package fusabi-frontend --test compiler_modules
```

## Error Handling

### Module Not Found

```rust
CompileError::ModuleNotFound(String)
```

Occurs when referencing a module that doesn't exist.

### Undefined Variable

```rust
CompileError::UndefinedVariable(String)
```

Occurs when referencing a binding that doesn't exist in a module.

### No Module Context

```rust
CompileError::NoModuleContext
```

Occurs when trying to resolve qualified names without module registry.

## Future Enhancements

1. **Phase 2**: Full lambda and closure support
2. **Type checking**: Integrate module-level type checking
3. **Nested modules**: Support for multi-level qualified names
4. **Module signatures**: Add module interface definitions
5. **Module functors**: Parameterized modules

## Performance Considerations

- Module registration is O(n) where n is the number of bindings
- Qualified name lookup is O(1) via HashMap
- Import resolution is O(m) where m is the number of imported bindings
- No runtime overhead for qualified vs unqualified names

## Integration with Existing Code

The module compiler is fully backward compatible:

- `Compiler::compile()` still works for single expressions
- `Compiler::compile_program()` is the new entry point for programs with modules
- All existing tests continue to pass
- No breaking changes to the API

## Conclusion

The compiler integration for the module system provides a solid foundation for organizing Fusabi code. While Phase 1 has some limitations (primarily around lambda compilation), the core module infrastructure is in place and ready for future enhancements.
