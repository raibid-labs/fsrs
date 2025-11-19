# Compiler Integration for Type Checking

This document describes the compiler integration points for optional type checking in FSRS.

## Overview

The FSRS compiler now supports optional type checking before code generation. This is implemented through a layered architecture that maintains backward compatibility while enabling gradual adoption of type checking features.

## Architecture

### 1. Compilation Options

The `CompileOptions` struct controls compilation behavior:

```rust
pub struct CompileOptions {
    /// Enable type checking before compilation
    pub enable_type_checking: bool,
    /// Strict mode - treat warnings as errors
    pub strict_mode: bool,
    /// Allow type warnings (only relevant if enable_type_checking is true)
    pub allow_warnings: bool,
}
```

**Default behavior**: Type checking is disabled for backward compatibility.

### 2. Compiler API

The compiler provides three main entry points:

#### Basic Compilation (Backward Compatible)

```rust
let chunk = Compiler::compile(&ast)?;
```

This uses the default options with type checking disabled.

#### Type-Checked Compilation

```rust
let chunk = Compiler::compile_checked(&ast)?;
```

This enables type checking with default settings.

#### Custom Options

```rust
let options = CompileOptions {
    enable_type_checking: true,
    strict_mode: false,
    allow_warnings: true,
};
let chunk = Compiler::compile_with_options(&ast, options)?;
```

This provides full control over compilation behavior.

### 3. Type Checking Integration

When type checking is enabled, the compiler performs the following steps:

1. **Parse** the source code into an AST
2. **Type Check** the AST using Hindley-Milner inference (when available)
3. **Compile** the AST to bytecode
4. **Execute** the bytecode in the VM

The type checking phase is currently a placeholder that will be replaced with the full type inference implementation once available.

## Usage

### From Rust Code

```rust
use fsrs_frontend::{Compiler, CompileOptions};
use fsrs_frontend::ast::Expr;

// Without type checking
let chunk = Compiler::compile(&expr)?;

// With type checking
let chunk = Compiler::compile_checked(&expr)?;

// With custom options
let options = CompileOptions {
    enable_type_checking: true,
    strict_mode: true,
    allow_warnings: false,
};
let chunk = Compiler::compile_with_options(&expr, options)?;
```

### From fsrs-demo Library

```rust
use fsrs_demo::{run_source, run_source_checked, run_source_with_options, RunOptions};

// Without type checking (backward compatible)
let result = run_source("let x = 42 in x + 1")?;

// With type checking
let result = run_source_checked("let x = 42 in x + 1")?;

// With custom options
let options = RunOptions {
    enable_type_checking: true,
    verbose: false,
    strict_mode: false,
};
let result = run_source_with_options("let x = 42 in x + 1", options)?;
```

## Error Handling

The compiler now includes type errors in its error enum:

```rust
pub enum CompileError {
    UndefinedVariable(String),
    TooManyConstants,
    TooManyLocals,
    InvalidJumpOffset,
    UnsupportedFloat,
    TupleTooLarge,
    TypeError(String),        // New: Type checking errors
    CodeGenError(String),     // New: Code generation errors
}
```

Type errors are propagated through the compilation pipeline and can be caught at any level.

## Typed AST (Optional)

The `typed_ast` module provides optional type-annotated AST nodes:

```rust
use fsrs_frontend::typed_ast::{TypedExpr, TypedPattern, Span};

// Create a typed expression
let typed = TypedExpr::new(expr, ty);

// With span information
let span = Span::new(0, 10, 1, 1);
let typed = TypedExpr::with_span(expr, ty, span);

// Convert back to untyped
let untyped = typed.into_expr();
```

These are useful for:
- Type-directed code generation
- Type-aware optimizations
- Debugging and diagnostics

## Migration Guide

### For Existing Code

No changes required! The default behavior is to compile without type checking:

```rust
// This still works exactly as before
let result = run_source("let x = 42 in x + 1")?;
```

### To Enable Type Checking

Simply switch to the checked variants:

```rust
// Old
let result = run_source("let x = 42 in x + 1")?;

// New (with type checking)
let result = run_source_checked("let x = 42 in x + 1")?;
```

### For Advanced Use Cases

Use `RunOptions` for fine-grained control:

```rust
let options = RunOptions {
    enable_type_checking: true,
    verbose: true,  // Print compilation stages
    strict_mode: true,  // Treat warnings as errors
};
let result = run_source_with_options(source, options)?;
```

## Type Annotation Support (Future)

The parser will support optional type annotations:

```fsharp
let x: int = 5 in x + 10
```

Type annotations will be:
- Optional (type inference fills in the gaps)
- Validated against inferred types
- Used to guide type inference

## Integration with Type Inference

Once the type inference module is complete, the compiler's `type_check` method will:

1. Create a type inference context
2. Run Hindley-Milner type inference on the AST
3. Solve type constraints
4. Apply substitutions to get final types
5. Store the type environment for code generation

The placeholder implementation:

```rust
fn type_check(&mut self, _expr: &Expr) -> CompileResult<Type> {
    // TODO: Replace with actual type inference when available
    self.type_env = Some(TypeEnv::new());
    Ok(Type::Unit)
}
```

Will be replaced with:

```rust
fn type_check(&mut self, expr: &Expr) -> CompileResult<Type> {
    let mut inference = TypeInference::new();
    let env = TypeEnv::new();

    // Run type inference
    let ty = inference.infer(expr, &env)
        .map_err(|e| CompileError::TypeError(format!("{}", e)))?;

    // Solve constraints
    let subst = inference.solve_constraints()
        .map_err(|e| CompileError::TypeError(format!("{}", e)))?;

    // Apply substitution to get final type
    let final_ty = subst.apply_type(&ty);

    // Store type environment for compilation
    self.type_env = Some(env);

    Ok(final_ty)
}
```

## Testing

Comprehensive integration tests are provided in `tests/test_type_checking.rs`:

- Backward compatibility tests
- Type checking enabled tests
- Error handling tests
- Consistency tests (checking vs non-checking produce same results)
- Edge case tests

Run tests:

```bash
cargo test --package fsrs-demo test_type_checking
```

## Performance Considerations

- Type checking adds minimal overhead when disabled (single boolean check)
- Type checking phase is separate from code generation
- No impact on bytecode execution performance
- Type environment is optional and only allocated when needed

## Future Enhancements

1. **Type Annotations**: Parser support for explicit type annotations
2. **Type Inference**: Full Hindley-Milner inference implementation
3. **Type Errors**: Beautiful error messages with suggestions
4. **Type-Directed Optimization**: Use type information for code generation
5. **Incremental Type Checking**: Cache type information across compilations

## References

- [Hindley-Milner Type Inference](../types/hindley-milner.md)
- [Compiler Architecture](./architecture.md)
- [Error Handling](./error-handling.md)
- [Testing Strategy](./testing.md)
