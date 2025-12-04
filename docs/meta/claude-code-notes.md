# Notes and Task Breakdown for Claude Code

This document is written directly for a code assistant like Claude Code to help implement the project step‑by‑step.

The project root is `fsrs/`. The Rust workspace lives in `fsrs/rust/`.

## Overall goal

Implement:

1. A **Mini‑F# front‑end** (`fusabi-frontend` crate)
2. A **bytecode VM** (`fusabi-vm` crate)
3. A **demo host** using both (`fusabi-demo` crate)

Follow the phases in order. Avoid trying to implement everything at once.

---

## Phase 1: Core AST, tokenizer, parser, and trivial VM

### 1.1 Create core AST in `fusabi-frontend`

File: `rust/crates/fusabi-frontend/src/ast.rs` (create if missing)

Tasks:

- Define enums/structs for:
  - `Literal` (int, float, bool, string)
  - `BinOp` (Add, Sub, Mul, Div, Eq, Neq, Lt, Lte, Gt, Gte, And, Or)
  - `Expr`:
    - `Var`
    - `Lit`
    - `Lambda`
    - `App`
    - `Let`
    - `If`
    - `Match` (for now, keep arm patterns simple: variable or wildcard)
- Keep this minimal; more constructs can be added later.

### 1.2 Implement tokenizer

File: `rust/crates/fusabi-frontend/src/lexer.rs`

Tasks:

- Implement a simple lexer that:
  - Converts source text to a sequence of tokens with positions.
  - Supports identifiers, literals, punctuation, and keywords.
- Define a `Token` enum (ident, literal, keyword, symbol).
- Include basic error reporting for unknown characters.

### 1.3 Implement a minimal parser

File: `rust/crates/fusabi-frontend/src/parser.rs`

Tasks:

- Implement a recursive‑descent parser that can parse:
  - `let` bindings (non‑recursive),
  - Simple function definitions (`let f x = expr`),
  - Applications (`f x y`),
  - `if/then/else`,
  - Parenthesized expressions,
  - Integer literals and variables.

For now:

- Ignore modules, records, DUs, pattern matching, and CEs.
- Parse a file into a list of top‑level `let` bindings.

### 1.4 Export a simple `compile_to_bytecode` stub

File: `rust/crates/fusabi-frontend/src/lib.rs`

Tasks:

- Public API:

  ```rust
  pub fn compile_to_bytecode(source: &str) -> Result<fusabi_vm::Chunk, FrontendError> { ... }
  ```

- For Phase 1, this can:
  - Parse the file,
  - Only handle a trivial subset (e.g. a single expression or `let main = ...`),
  - Emit a hard‑coded or dummy `Chunk` that returns an integer.

---

## Phase 1 VM

### 1.5 Define VM structs and Value

File: `rust/crates/fusabi-vm/src/lib.rs`

Tasks:

- Define:

  ```rust
  pub enum Value {
      Int(i64),
      Bool(bool),
      Str(String),
      Unit,
  }

  pub enum Instruction {
      LoadConst(u16),
      Add,
      Sub,
      Mul,
      Div,
      Return,
  }

  pub struct Chunk {
      pub instructions: Vec<Instruction>,
      pub constants: Vec<Value>,
  }

  pub struct Vm {
      pub stack: Vec<Value>,
      pub chunk: Chunk,
      pub ip: usize,
  }
  ```

- Implement:

  ```rust
  impl Vm {
      pub fn new(chunk: Chunk) -> Self { ... }
      pub fn run(&mut self) -> Result<Value, VmError> { ... }
  }
  ```

- The initial interpreter only needs to support:
  - `LoadConst`,
  - Arithmetic (`Add`, `Sub`, `Mul`, `Div`),
  - `Return`.

### 1.6 Wire the demo

File: `rust/crates/fusabi-demo/src/main.rs`

Tasks:

- Read `../examples/fusabi_config.fsx`.
- Call `compile_to_bytecode(...)`.
- Instantiate the VM and run it.
- Print the resulting `Value`.

For now, you can ignore the content of `fusabi_config.fsx` and just feed a hard‑coded string.

---

## Phase 2: Extend language and VM

After Phase 1 works, move on:

### 2.1 Extend AST with let‑rec, tuples, lists

- Add variants to `Expr` for:
  - `LetRec`,
  - `Tuple`,
  - `List`.

### 2.2 Introduce a simple type system (optional at first)

- Create `types.rs` with:
  - `Type` enum: `Int`, `Bool`, `String`, `Arrow(Box<Type>, Box<Type>)`, etc.
- Implement a basic type inference pass or, initially, a type checker with explicit annotations.

### 2.3 Add function calls to VM

- Extend `Value` with `Closure`.
- Define `Frame` and call stack in `Vm`.
- Add `Call` and `Return` instructions.

### 2.4 Basic pattern matching

- Extend `Expr::Match`.
- Implement pattern compilation (`match` over booleans and ints first).
- Add simple jump instructions (`Jump`, `JumpIfFalse`).

---

## Phase 3: Records, DUs, and embedding

Follow `03-vm-design.md` and:

- Implement `Record` and `Variant` in `fusabi-vm`.
- Extend the front‑end to support:
  - Record type declarations and construction,
  - DU type declarations and construction,
  - Pattern matching over records and DUs.
- Add a small host API in `fusabi-demo` to simulate a terminal:
  - Register built‑in functions for logging and pseudo actions.
  - Create a fake `TabInfo` record and call into a script function to format its title.

---

## General guidance for Claude Code

When asking Claude Code for help, you can use prompts like:

> “You are editing `rust/crates/fusabi-frontend/src/parser.rs`. Implement a minimal parser for the Mini‑F# subset described in `docs/02-language-spec.md`, phase 1. Use the existing AST definitions in `ast.rs`. The lexer is already available in `lexer.rs`. Focus only on let‑bindings, integer literals, identifiers, function definitions (`let f x = expr`), and infix `+`/`-`/`*`/`/`.”

Or:

> “You are editing `rust/crates/fusabi-vm/src/lib.rs`. Implement the interpreter loop for the `Vm::run` method to handle the instructions defined in the `Instruction` enum. You only need to support `LoadConst`, `Add`, `Sub`, `Mul`, `Div`, and `Return` for now.”

Keep prompts:

- File‑scoped (tell it which file).
- Phase‑scoped (tell it which subset to implement).
- Anchored in the docs (`02-language-spec.md`, `03-vm-design.md`).

This should make the assistant’s contributions coherent and aligned with the overall design.
