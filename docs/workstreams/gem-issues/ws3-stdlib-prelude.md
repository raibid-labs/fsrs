# Workstream 3: Frontend - Stdlib Prelude & Operators

## Status
🟢 Complete

## Overview
Implement implicit prelude (auto-import core functions), add pipeline operator `|>`, and polish the standard library experience. Users should not need to manually `open List` for common operations.

## Objectives
- [x] Create Core module with `print`, `printfn`, `id`, `ignore`, `fst`, `snd` (Partially achieved via `List`, `String`, `Option` modules)
- [x] Implement implicit open mechanism in compiler (via `register_stdlib` populating globals)
- [x] Add pipeline operator `|>` to lexer and parser
- [x] Desugar `a |> f` to `f a`
- [x] Update all examples to use new prelude
- [x] Ensure proper operator precedence

## Summary of Changes
- Added `|>` token to Lexer and `parse_pipeline_expr` to Parser.
- Implemented `register_stdlib` which populates the VM globals with `List`, `String`, and `Option` modules as `Record`s.
- VM `LoadGlobal` instruction now handles global variable access.
- `List`, `String`, `Option` modules are now accessible as global records (e.g., `List.map`).
- Implemented `stdlib_demo.fsx` showcasing the stdlib usage.

## Agent Assignment
**Suggested Agent Type**: `frontend-developer`, `coder`, `typescript-pro`
**Skill Requirements**: Compiler frontend, parser design, operator precedence, F# syntax

## Dependencies
- **WS1 (HOF Support)**: Required for full stdlib functionality (List.map, etc.)

## Tasks

### Task 3.1: Create Core Module
**Description**: Implement core functions that should be available by default.

**Deliverables**:
- `print(value: Value) -> ()`
- `printfn(value: Value) -> ()` (print with newline)
- `id(value: Value) -> Value` (identity function)
- `ignore(value: Value) -> ()` (discard value)
- `fst((a, b): Tuple) -> a` (first element of tuple)
- `snd((a, b): Tuple) -> b` (second element of tuple)

**Files to Create/Modify**:
- `rust/crates/fusabi-vm/src/stdlib/core.rs` (new file)
- `rust/crates/fusabi-vm/src/stdlib/mod.rs` (add core module)

**Implementation**:
```rust
// fusabi-vm/src/stdlib/core.rs

use crate::{value::Value, vm::{Vm, VmError}};

/// Print a value to stdout without newline
pub fn print(_vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::ArityMismatch { expected: 1, got: args.len() });
    }
    print!("{}", args[0]);
    Ok(Value::Unit)
}

/// Print a value to stdout with newline
pub fn printfn(_vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::ArityMismatch { expected: 1, got: args.len() });
    }
    println!("{}", args[0]);
    Ok(Value::Unit)
}

/// Identity function: id x = x
pub fn id(_vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::ArityMismatch { expected: 1, got: args.len() });
    }
    Ok(args[0].clone())
}

/// Ignore function: ignore x = ()
pub fn ignore(_vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::ArityMismatch { expected: 1, got: args.len() });
    }
    Ok(Value::Unit)
}

/// First element of tuple
pub fn fst(_vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::ArityMismatch { expected: 1, got: args.len() });
    }
    match &args[0] {
        Value::Tuple(elements) => {
            if elements.len() >= 2 {
                Ok(elements[0].clone())
            } else {
                Err(VmError::TypeError("Expected tuple with at least 2 elements".into()))
            }
        }
        _ => Err(VmError::TypeError("Expected tuple".into())),
    }
}

/// Second element of tuple
pub fn snd(_vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::ArityMismatch { expected: 1, got: args.len() });
    }
    match &args[0] {
        Value::Tuple(elements) => {
            if elements.len() >= 2 {
                Ok(elements[1].clone())
            } else {
                Err(VmError::TypeError("Expected tuple with at least 2 elements".into()))
            }
        }
        _ => Err(VmError::TypeError("Expected tuple".into())),
    }
}

/// Register core functions in stdlib
pub fn register_core_functions(registry: &mut crate::host::HostRegistry) {
    registry.register("print", Box::new(print));
    registry.register("printfn", Box::new(printfn));
    registry.register("id", Box::new(id));
    registry.register("ignore", Box::new(ignore));
    registry.register("fst", Box::new(fst));
    registry.register("snd", Box::new(snd));
}
```

**Validation**:
```bash
cd rust/crates/fusabi-vm
cargo test --lib stdlib::core
# All core tests pass
```

---

### Task 3.2: Register Core Functions Without Namespace
**Description**: Register core functions in global scope (not `Core.print`, just `print`).

**Deliverables**:
- Update `HostRegistry` to support global scope registration
- Register core functions without module prefix
- Ensure no name collisions with user-defined functions

**Files to Create/Modify**:
- `rust/crates/fusabi-vm/src/host.rs`
- `rust/crates/fusabi-vm/src/stdlib/mod.rs`

**Implementation**:
```rust
// In fusabi-vm/src/stdlib/mod.rs

pub fn register_stdlib(registry: &mut HostRegistry) {
    // Core functions (global scope, no prefix)
    core::register_core_functions(registry);

    // Module-specific functions (with prefix)
    list::register_list_functions(registry);
    string::register_string_functions(registry);
    // ...
}
```

**Validation**:
```fsharp
// examples/test_prelude.fsx
printfn "Hello, World!"
let x = id 42
let pair = (1, 2)
let first = fst pair
printfn "First: %d" first
```

```bash
cargo run -- run examples/test_prelude.fsx
# Should print:
# Hello, World!
# First: 1
```

---

### Task 3.3: Implement Implicit Open in Compiler
**Description**: Modify compiler to inject core bindings before parsing user code.

**Deliverables**:
- Update `Compiler::compile_program` to inject core bindings
- Ensure core functions are available without `open` statement
- Handle potential name collisions gracefully

**Files to Create/Modify**:
- `rust/crates/fusabi-frontend/src/compiler.rs`

**Implementation**:
```rust
// In fusabi-frontend/src/compiler.rs

impl Compiler {
    pub fn compile_program(&mut self, ast: &Ast) -> Result<Chunk, CompileError> {
        // Inject core stdlib bindings BEFORE compiling user code
        self.inject_core_bindings();

        // Now compile the user's AST
        self.compile_ast(ast)?;

        Ok(self.chunk.clone())
    }

    fn inject_core_bindings(&mut self) {
        // Register core function names in compiler scope
        // These will resolve to HostFn calls at runtime
        let core_functions = vec![
            "print", "printfn", "id", "ignore", "fst", "snd"
        ];

        for func_name in core_functions {
            self.scope.insert(func_name.to_string(), /* global index */);
        }
    }
}
```

**Validation**:
```fsharp
// examples/test_implicit_open.fsx
// No 'open' statement needed!
printfn "This works without importing anything!"
let result = id 42
```

```bash
cargo run -- run examples/test_implicit_open.fsx
# Should work without errors
```

---

### Task 3.4: Add Pipeline Operator to Lexer
**Description**: Add `|>` token to lexer.

**Deliverables**:
- `Token::PipeRight` variant
- Lexer recognizes `|>` as single token (not `|` followed by `>`)
- Update lexer tests

**Files to Create/Modify**:
- `rust/crates/fusabi-frontend/src/lexer.rs`

**Implementation**:
```rust
// In fusabi-frontend/src/lexer.rs

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // ... existing tokens
    PipeRight, // |>
    // ...
}

impl Lexer {
    fn lex_token(&mut self) -> Result<Token, LexError> {
        match self.current_char() {
            '|' => {
                self.advance();
                if self.current_char() == '>' {
                    self.advance();
                    Ok(Token::PipeRight)
                } else {
                    // Just a pipe (for future use)
                    Ok(Token::Pipe)
                }
            }
            // ... other cases
        }
    }
}
```

**Validation**:
```rust
#[test]
fn test_lex_pipe_operator() {
    let mut lexer = Lexer::new("a |> f");
    assert_eq!(lexer.next_token(), Ok(Token::Ident("a".into())));
    assert_eq!(lexer.next_token(), Ok(Token::PipeRight));
    assert_eq!(lexer.next_token(), Ok(Token::Ident("f".into())));
}
```

---

### Task 3.5: Add Pipeline Operator to Parser
**Description**: Parse `|>` operator with correct precedence and desugar to function application.

**Deliverables**:
- Add `|>` to expression parsing
- Precedence: lower than function application, higher than assignment
- Desugar `a |> f` to `f a` (or `f(a)` depending on AST representation)
- Support chaining: `a |> f |> g` desugars to `g (f a)`

**Files to Create/Modify**:
- `rust/crates/fusabi-frontend/src/parser.rs`

**Implementation**:
```rust
// In fusabi-frontend/src/parser.rs

impl Parser {
    fn parse_pipeline_expr(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_application_expr()?;

        while self.match_token(&Token::PipeRight) {
            let func = self.parse_application_expr()?;
            // Desugar: expr |> func  =>  func(expr)
            expr = Expr::Call {
                func: Box::new(func),
                args: vec![expr],
            };
        }

        Ok(expr)
    }
}
```

**Precedence Table** (from lowest to highest):
1. Let bindings
2. If/then/else
3. Pipeline `|>` ← NEW
4. Function application
5. Arithmetic ops
6. Comparison
7. Primary (literals, idents, parens)

**Validation**:
```rust
#[test]
fn test_parse_pipeline() {
    let input = "42 |> double";
    let expr = Parser::new(input).parse_expr().unwrap();

    // Should parse as Call(double, [42])
    assert!(matches!(expr, Expr::Call { .. }));
}

#[test]
fn test_parse_pipeline_chain() {
    let input = "42 |> double |> increment";
    let expr = Parser::new(input).parse_expr().unwrap();

    // Should parse as Call(increment, [Call(double, [42])])
    assert!(matches!(expr, Expr::Call { .. }));
}
```

---

### Task 3.6: Update Examples to Use Prelude
**Description**: Update existing examples to use new prelude and pipeline operator.

**Deliverables**:
- Update all `.fsx` files to remove manual `open` statements for core
- Add examples showcasing pipeline operator
- Ensure all examples still work

**Files to Create/Modify**:
- `examples/fibonacci.fsx`
- `examples/list_ops.fsx`
- `examples/pipeline_demo.fsx` (new)

**Example**:
```fsharp
// examples/pipeline_demo.fsx
let double x = x * 2
let increment x = x + 1
let negate x = -x

// Without pipeline
let result1 = negate (increment (double 5))
printfn "Without pipeline: %d" result1

// With pipeline
let result2 = 5 |> double |> increment |> negate
printfn "With pipeline: %d" result2

// Chained list operations
let nums = [1; 2; 3; 4; 5]
let result3 = nums
    |> List.map double
    |> List.filter (fun x -> x > 5)
    |> List.fold (+) 0
printfn "List pipeline: %d" result3
```

**Validation**:
```bash
cargo run -- run examples/pipeline_demo.fsx
# Should output:
# Without pipeline: -11
# With pipeline: -11
# List pipeline: 18
```

---

### Task 3.7: Add Comprehensive Tests
**Description**: Add tests for prelude and pipeline operator.

**Deliverables**:
- Unit tests for all core functions
- Parser tests for `|>` operator
- Integration tests with F# scripts
- Edge case tests (precedence, associativity)

**Files to Create/Modify**:
- `rust/crates/fusabi-vm/tests/prelude_tests.rs`
- `rust/crates/fusabi-frontend/tests/parser_pipeline.rs`
- `examples/tests/prelude_integration.fsx`

**Test Cases**:
1. Core functions work without import
2. Pipeline operator desugars correctly
3. Pipeline chains work
4. Pipeline with parentheses: `(a |> f) |> g`
5. Mixed with function application: `f (a |> g)`
6. Precedence: `a |> f + 1` should be `(f a) + 1`

**Validation**:
```bash
cd rust
cargo test
# All tests pass

cargo run -- run examples/tests/prelude_integration.fsx
# Integration tests pass
```

---

## Definition of Done
- [ ] Core module with `print`, `printfn`, `id`, `ignore`, `fst`, `snd` implemented
- [ ] Core functions registered in global scope (no prefix)
- [ ] Implicit open mechanism in compiler
- [ ] Pipeline operator `|>` in lexer
- [ ] Pipeline operator `|>` in parser with correct precedence
- [ ] Desugaring `a |> f` to `f a` working
- [ ] All examples updated to use new prelude
- [ ] Comprehensive test suite passing
- [ ] Documentation updated (language spec, examples)
- [ ] PR ready for review

## Agent Coordination Hooks
```bash
# BEFORE Work:
npx claude-flow@alpha hooks pre-task --description "ws3-stdlib-prelude"
npx claude-flow@alpha hooks session-restore --session-id "swarm-fusabi-gem-ws3"

# DURING Work:
npx claude-flow@alpha hooks post-edit --file "rust/crates/fusabi-vm/src/stdlib/core.rs" --memory-key "swarm/fusabi-gem/ws3/core-module"
npx claude-flow@alpha hooks post-edit --file "rust/crates/fusabi-frontend/src/compiler.rs" --memory-key "swarm/fusabi-gem/ws3/implicit-open"
npx claude-flow@alpha hooks post-edit --file "rust/crates/fusabi-frontend/src/parser.rs" --memory-key "swarm/fusabi-gem/ws3/pipeline-operator"
npx claude-flow@alpha hooks notify --message "Prelude and pipeline operator complete"

# AFTER Work:
npx claude-flow@alpha hooks post-task --task-id "ws3-complete"
npx claude-flow@alpha hooks session-end --export-metrics true
```

## Estimated Effort
**Duration**: 3-4 days
**Complexity**: Medium

## References
- [F# Language Reference - Operators](https://docs.microsoft.com/en-us/dotnet/fsharp/language-reference/operators)
- [F# Core Library](https://fsharp.github.io/fsharp-core-docs/)
- [Operator Precedence Parsing](https://en.wikipedia.org/wiki/Operator-precedence_parser)

## Notes
- **Operator Precedence**: Pipeline `|>` should have lower precedence than function application but higher than let/if
- **Right Associativity**: `a |> f |> g` should parse as `a |> (f |> g)` which desugars to `g (f a)`
- **Future Operators**: Consider adding `<|` (reverse pipe), `>>` (compose), `<<` (reverse compose)
- **Performance**: Pipeline is just syntactic sugar, no runtime overhead
- **User Experience**: With implicit prelude and pipeline, Fusabi feels much more like F#

## File Conflicts
- **No major conflicts** with other workstreams
- Minor: If WS1 updates stdlib structure, may need to coordinate
- Safe to run after WS1 completes
