# Contributing to Fusabi

Thank you for your interest in contributing to Fusabi! This guide will help you understand the architecture and development workflow.

## Table of Contents

- [Architecture Overview](#architecture-overview)
- [Development Setup](#development-setup)
- [Understanding the 3-Layer Architecture](#understanding-the-3-layer-architecture)
- [Adding New Instructions](#adding-new-instructions)
- [Testing Guidelines](#testing-guidelines)
- [Pull Request Process](#pull-request-process)
- [Code Style](#code-style)

## Architecture Overview

Fusabi is a high-performance embedded scripting engine that compiles F# code to bytecode and executes it on a stack-based virtual machine. The architecture follows a clear 3-layer pipeline:

```
Source Code (.fsx) → AST → Bytecode (.fzb) → VM Execution
     ↓                ↓         ↓               ↓
  [Parser]      [Compiler]  [Serializer]      [VM]
```

### Key Components

- **fusabi-frontend**: Lexer, Parser, Type Checker, and Bytecode Compiler
- **fusabi-vm**: Stack-based VM, Value types, Instructions, and Standard Library
- **fusabi**: High-level API and CLI for running scripts

## Development Setup

### Prerequisites

- **Rust**: 1.70 or later
- **Just**: Command runner for build automation
- **Nu**: Optional, for advanced test scripts

### Getting Started

```bash
# Clone the repository
git clone https://github.com/fusabi-lang/fusabi.git
cd fusabi

# Bootstrap the environment (installs hooks, tools, etc.)
just bootstrap

# Build the project
just build

# Run tests
just test

# Run a script
fus run examples/hello.fsx
```

### Project Structure

```
fusabi/
├── rust/
│   ├── crates/
│   │   ├── fusabi/           # High-level API and CLI
│   │   ├── fusabi-frontend/  # Parser, compiler, type checker
│   │   └── fusabi-vm/        # Virtual machine and runtime
│   └── examples/             # Example host integrations
├── examples/                 # F# example scripts
└── docs/                     # Documentation
```

## Understanding the 3-Layer Architecture

### Layer 1: Source → AST (Parser)

**Location**: `rust/crates/fusabi-frontend/src/`

The frontend converts F# source code into an Abstract Syntax Tree (AST):

1. **Lexer** (`lexer.rs`): Tokenizes source code
   - Produces tokens: `Int(42)`, `Ident("x")`, `Plus`, etc.
   - Tracks source positions for error reporting

2. **Parser** (`parser.rs`): Builds AST from tokens
   - Recursive descent parser
   - Handles expressions, patterns, types, modules
   - Produces `Program` containing module definitions and imports

3. **Type Inference** (`inference.rs`, optional):
   - Hindley-Milner type inference
   - Optional type checking (backward compatible)

**AST Types** (`ast.rs`):

```rust
pub enum Expr {
    Lit(Literal),               // 42, "hello", true
    Var(String),                // x
    Let { name, value, body },  // let x = 42 in x + 1
    Lambda { params, body },    // fun x -> x + 1
    App { func, arg },          // f x
    BinOp { op, left, right },  // 2 + 3
    If { cond, then, else_ },   // if cond then e1 else e2
    Tuple(Vec<Expr>),           // (1, 2, 3)
    List(Vec<Expr>),            // [1; 2; 3]
    Array(Vec<Expr>),           // [|1; 2; 3|]
    Record { fields },          // { x = 1; y = 2 }
    Match { scrutinee, arms },  // match v with | Some x -> x | None -> 0
    // ... more variants
}
```

### Layer 2: AST → Bytecode (Compiler)

**Location**: `rust/crates/fusabi-frontend/src/compiler.rs`

The compiler transforms the AST into bytecode chunks:

**Key Responsibilities**:

- **Constant Pooling**: Deduplicates literals
- **Local Variable Tracking**: Maps variables to stack slots
- **Control Flow**: Emits jumps for if/match/loops
- **Closure Compilation**: Captures upvalues
- **Pattern Matching**: Compiles to decision trees

**Output** (`fusabi_vm::Chunk`):

```rust
pub struct Chunk {
    pub instructions: Vec<Instruction>,  // Bytecode
    pub constants: Vec<Value>,           // Literal pool
    pub name: Option<String>,            // Function name
}
```

**Example Compilation**:

```fsharp
let x = 42 in x + 1
```

Compiles to:

```
LOAD_CONST 0    (42)
STORE_LOCAL 0
LOAD_LOCAL 0
LOAD_CONST 1    (1)
ADD
RETURN
```

### Layer 3: Bytecode → Execution (VM)

**Location**: `rust/crates/fusabi-vm/src/vm.rs`

The VM executes bytecode using a stack-based model:

**VM Components**:

- **Value Stack**: Operands and intermediate results
- **Call Frames**: Function activation records
- **Globals**: Module-level bindings
- **Host Registry**: Native function registry

**Execution Loop** (simplified):

```rust
loop {
    let instruction = frame.fetch_instruction()?;
    match instruction {
        Instruction::LoadConst(idx) => {
            let value = frame.get_constant(idx)?;
            self.push(value);
        }
        Instruction::Add => {
            let b = self.pop_int()?;
            let a = self.pop_int()?;
            self.push(Value::Int(a + b));
        }
        Instruction::Call(argc) => {
            let func = self.pop()?;
            self.call_function(func, argc)?;
        }
        Instruction::Return => {
            let result = self.pop()?;
            self.pop_frame()?;
            if self.frames.is_empty() {
                return Ok(result);
            }
            self.push(result);
        }
        // ... handle other instructions
    }
}
```

**Value Representation** (`value.rs`):

```rust
pub enum Value {
    Int(i64),
    Bool(bool),
    Str(String),
    Unit,
    Tuple(Vec<Value>),
    Cons { head: Box<Value>, tail: Box<Value> },
    Nil,
    Array(Rc<RefCell<Vec<Value>>>),
    Record(Rc<RefCell<HashMap<String, Value>>>),
    Variant { type_name, variant_name, fields },
    Closure(Rc<Closure>),
    NativeFn { name, arity, args },
    HostData(HostData),  // For Rust object interop
}
```

## Adding New Instructions

To add a new bytecode instruction, follow these steps:

### Step 1: Define the Instruction

**File**: `rust/crates/fusabi-vm/src/instruction.rs`

```rust
pub enum Instruction {
    // ... existing instructions

    /// Your new instruction with documentation
    /// Example: Modulo operation (a % b)
    Mod,
}
```

Update the `Display` implementation:

```rust
impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // ... existing cases
            Instruction::Mod => write!(f, "MOD"),
        }
    }
}
```

### Step 2: Update the Compiler

**File**: `rust/crates/fusabi-frontend/src/compiler.rs`

Add compiler logic to emit the instruction:

```rust
impl Compiler {
    fn compile_expr(&mut self, expr: &Expr) -> CompileResult<()> {
        match expr {
            // ... existing cases

            Expr::BinOp { op: BinOp::Mod, left, right } => {
                self.compile_expr(left)?;
                self.compile_expr(right)?;
                self.chunk.emit(Instruction::Mod);
                Ok(())
            }
            // ...
        }
    }
}
```

### Step 3: Implement VM Execution

**File**: `rust/crates/fusabi-vm/src/vm.rs`

Add the execution logic in the VM's run loop:

```rust
impl Vm {
    pub fn run(&mut self) -> Result<Value, VmError> {
        loop {
            let instruction = /* fetch instruction */;
            match instruction {
                // ... existing cases

                Instruction::Mod => {
                    let b = self.pop_int()?;
                    let a = self.pop_int()?;
                    if b == 0 {
                        return Err(VmError::DivisionByZero);
                    }
                    self.push(Value::Int(a % b));
                }
                // ...
            }
        }
    }
}
```

### Step 4: Add Tests

**Create tests at all layers**:

1. **VM Test** (`rust/crates/fusabi-vm/tests/test_instructions.rs`):
```rust
#[test]
fn test_mod_instruction() {
    let mut chunk = Chunk::new();
    chunk.add_constant(Value::Int(10));
    chunk.add_constant(Value::Int(3));
    chunk.emit(Instruction::LoadConst(0));
    chunk.emit(Instruction::LoadConst(1));
    chunk.emit(Instruction::Mod);
    chunk.emit(Instruction::Return);

    let mut vm = Vm::new();
    let result = vm.execute(chunk).unwrap();
    assert_eq!(result.as_int(), Some(1));
}
```

2. **Compiler Test** (`rust/crates/fusabi-frontend/tests/integration_test.rs`):
```rust
#[test]
fn test_modulo_compilation() {
    let source = "10 % 3";
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.as_int(), Some(1));
}
```

3. **Integration Test** (`examples/mod_test.fsx`):
```fsharp
let result = 10 % 3
printfn "10 mod 3 = %d" result
```

### Step 5: Documentation

Update the relevant documentation:

- Add to instruction reference in `docs/ABI.md`
- Update `docs/03-vm-design.md` if it affects architecture
- Add examples to the language spec `docs/02-language-spec.md`

## Testing Guidelines

### Test Organization

- **Unit Tests**: In the same file as the code (`#[cfg(test)]`)
- **Integration Tests**: In `tests/` directory of each crate
- **End-to-End Tests**: F# scripts in `examples/`

### Running Tests

```bash
# All tests
just test

# Specific crate
just test-crate fusabi-vm

# With coverage
just test-coverage

# Single test
cd rust && cargo test test_name
```

### Writing Good Tests

- **Test one thing**: Each test should verify a single behavior
- **Use descriptive names**: `test_list_map_with_closure`
- **Include edge cases**: Empty lists, zero, negative numbers, etc.
- **Test error cases**: Division by zero, type mismatches, etc.

Example:

```rust
#[test]
fn test_list_head_empty_list_error() {
    let mut vm = Vm::new();
    register_stdlib(&mut vm);

    // Test that head of empty list fails gracefully
    let chunk = compile_expr("List.head []").unwrap();
    let result = vm.execute(chunk);

    assert!(result.is_err());
    assert!(matches!(result, Err(VmError::EmptyList)));
}
```

## Pull Request Process

### Before Submitting

1. **Run tests**: `just test`
2. **Format code**: `just fmt`
3. **Lint**: `just lint`
4. **Update docs**: If you changed APIs or architecture

### PR Guidelines

- **Title**: Clear and descriptive (e.g., "feat: Add modulo operator support")
- **Description**: Explain what and why
  - What problem does this solve?
  - How does it work?
  - Any breaking changes?
- **Link issues**: Reference related issues (#123)
- **Small PRs**: Easier to review, faster to merge

### Commit Messages

Follow conventional commits:

```
type(scope): description

feat(vm): add modulo instruction
fix(parser): handle negative numbers in patterns
docs(contributing): add instruction guide
test(compiler): add modulo compilation tests
```

Types: `feat`, `fix`, `docs`, `test`, `refactor`, `perf`, `chore`

## Code Style

### Rust Style

- Follow `rustfmt` defaults (run `just fmt`)
- Use `clippy` recommendations (run `just lint`)
- Document public APIs with doc comments (`///`)
- Add examples to doc comments when helpful

### Documentation Style

```rust
/// Compiles an expression to bytecode
///
/// # Arguments
/// * `expr` - The AST expression to compile
///
/// # Returns
/// A compiled `Chunk` ready for VM execution
///
/// # Errors
/// Returns `CompileError` if compilation fails (e.g., undefined variables)
///
/// # Example
/// ```rust
/// let expr = Expr::Lit(Literal::Int(42));
/// let chunk = Compiler::compile(&expr)?;
/// ```
pub fn compile(expr: &Expr) -> CompileResult<Chunk> {
    // ...
}
```

### Naming Conventions

- **Types**: `PascalCase` (e.g., `Instruction`, `Value`)
- **Functions**: `snake_case` (e.g., `compile_expr`, `pop_int`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `MAX_LOCALS`, `FZB_MAGIC`)
- **Module functions**: Prefix with module name in stdlib (e.g., `list_map`, `string_length`)

## Additional Resources

- [Architecture Documentation](docs/03-vm-design.md)
- [ABI Specification](docs/ABI.md)
- [Language Specification](docs/02-language-spec.md)
- [Security Model](docs/SECURITY.md)
- [Host Interop Guide](docs/host-interop.md)

## Questions?

- **Issues**: Open an issue on GitHub
- **Discussions**: Use GitHub Discussions for questions
- **Discord**: Join our community server (link in README)

Thank you for contributing to Fusabi!
