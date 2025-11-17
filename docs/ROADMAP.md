# FSRS Project Roadmap

**Version**: 0.1.0-alpha
**Status**: Bootstrap Phase
**Last Updated**: November 17, 2025

---

## Executive Summary

FSRS (F# Script Runtime System) is an experimental Mini-F# dialect with an embeddable Rust bytecode VM, designed to replace Lua-style scripting in applications like terminal emulators (e.g., WezTerm). The goal is to provide F#-style developer ergonomics (records, DUs, pattern matching, pipelines) in a small, eager, expression-oriented language suitable for embedded scripting and configuration.

### Vision

- **F#-Style Ergonomics**: Records, discriminated unions, pattern matching, pipelines, and simple modules
- **Embedded Scripting**: Lua-class bytecode VM implemented entirely in Rust (no .NET, no LLVM, no WASM)
- **Host Integration**: Designed for Rust host applications with hot-path callbacks
- **Performance**: Target 5-10M operations/second with Lua-comparable startup time

### Key Differentiators

1. **First lightweight embeddable F# VM** - No full .NET runtime required
2. **Native Rust integration** - Zero-cost abstractions for host interop
3. **Hot-reload support** - Development-friendly reloading without restart
4. **Type-safe scripting** - Hindley-Milner type inference for safety

---

## Current State (November 2025)

### What Exists

- **Skeletal Rust workspace**: `fsrs-frontend`, `fsrs-vm`, `fsrs-demo` crates
- **Design documents**: Language spec, VM design, architecture overview
- **Build infrastructure**: Justfile, Nushell scripts, bootstrap tooling
- **Research**: Bytecode VM patterns, embedding strategies, host interop design

### What's Missing

- Parser, lexer, and AST implementation
- Bytecode compiler and runtime
- Type inference engine
- Host interop layer
- Comprehensive test suite

---

## Development Phases

### Phase 1: MVP - Core Language & Interpreter (Weeks 1-3)

**Goal**: Prove the concept with a working Mini-F# interpreter that can execute simple scripts.

#### Milestone 1.1: Frontend Foundation (Week 1)

**Tasks**:
1. **Core AST** (`fsrs-frontend/src/ast.rs`):
   - Define `Literal`, `BinOp`, `Expr` enums
   - Support: variables, literals, lambdas, let-bindings, if/then/else
   - Basic pattern matching (variables and wildcards)

2. **Tokenizer** (`fsrs-frontend/src/lexer.rs`):
   - Implement lexer for identifiers, literals, keywords, operators
   - Token position tracking for error reporting
   - Handle whitespace and comments

3. **Parser** (`fsrs-frontend/src/parser.rs`):
   - Recursive-descent parser for Phase 1 subset
   - Parse: let-bindings, function definitions, applications, arithmetic
   - Simple error recovery

**Success Criteria**:
- Parse: `let add x y = x + y`
- Parse: `let result = if x > 0 then x + 1 else 0`
- Clear error messages with line/column info

#### Milestone 1.2: VM Foundation (Week 2)

**Tasks**:
1. **Value Representation** (`fsrs-vm/src/value.rs`):
   ```rust
   pub enum Value {
       Int(i64),
       Bool(bool),
       Str(String),
       Unit,
   }
   ```

2. **Bytecode** (`fsrs-vm/src/bytecode.rs`):
   ```rust
   pub enum Instruction {
       LoadConst(u16),
       Add, Sub, Mul, Div,
       Eq, Lt, Gt,
       Jump(i16),
       JumpIfFalse(i16),
       Return,
   }

   pub struct Chunk {
       instructions: Vec<Instruction>,
       constants: Vec<Value>,
   }
   ```

3. **VM Interpreter** (`fsrs-vm/src/vm.rs`):
   - Stack-based interpreter loop
   - Frame management for function calls
   - Arithmetic operations

**Success Criteria**:
- Execute: `LoadConst(0), LoadConst(1), Add, Return` → returns `1`
- Stack overflow protection
- Clear runtime errors

#### Milestone 1.3: End-to-End Integration (Week 3)

**Tasks**:
1. **Bytecode Compilation** (`fsrs-frontend/src/compiler.rs`):
   - Compile AST to bytecode chunks
   - Constant pool management
   - Jump offset calculation

2. **Demo Host** (`fsrs-demo/src/main.rs`):
   - Load `.fsrs` files
   - Compile to bytecode
   - Execute in VM
   - Display results

3. **Testing**:
   - Unit tests for lexer, parser, compiler, VM
   - Integration tests for example scripts
   - Error handling tests

**Success Criteria**:
- Execute: `examples/arithmetic.fsrs` with additions, multiplications
- Execute: `examples/conditional.fsrs` with if/then/else
- Execution time < 10ms for trivial scripts

**Phase 1 Deliverables**:
- Working interpreter for integer arithmetic
- If/then/else conditionals
- Simple functions (no closures yet)
- Basic error reporting
- 50+ unit tests

---

### Phase 2: Language Features (Weeks 4-7)

**Goal**: Extend the language to support essential functional programming features.

#### Milestone 2.1: Functions & Closures (Week 4)

**Tasks**:
1. **Closure Support**:
   - Extend `Value` with `Closure(Gc<Closure>)`
   - Implement upvalue capture (open/closed system)
   - Frame-based call stack

2. **Let-Rec**:
   - Recursive function bindings
   - Mutual recursion support

3. **Currying**:
   - Partial application
   - Multi-argument functions

**Success Criteria**:
- Execute: `let rec fact n = if n <= 1 then 1 else n * fact (n - 1)`
- Execute: `let add x y = x + y; let inc = add 1`
- Proper tail-call optimization (bonus)

#### Milestone 2.2: Data Structures (Week 5)

**Tasks**:
1. **Tuples**:
   - `MakeTuple(u8)` instruction
   - Tuple destructuring in patterns

2. **Lists**:
   - Cons-cell representation
   - `::` operator, `[]` literals
   - List operations: head, tail, map, filter

3. **Arrays** (bonus):
   - `Vec<Value>` backing
   - Index access, update

**Success Criteria**:
- Execute: `let pair = (1, "hello")`
- Execute: `let nums = [1; 2; 3]`
- List comprehensions (bonus)

#### Milestone 2.3: Pattern Matching (Week 6)

**Tasks**:
1. **Match Expressions**:
   - Compile patterns to decision trees
   - Support literals, variables, wildcards
   - Tuple and list patterns

2. **Instructions**:
   - `MatchTag`, `GetField`, `Destruct`
   - Jump-based dispatch

**Success Criteria**:
- Execute:
  ```fsharp
  match xs with
  | [] -> 0
  | x :: xs -> x + sum xs
  ```

#### Milestone 2.4: Type System (Week 7)

**Tasks**:
1. **Type Inference**:
   - Hindley-Milner algorithm
   - Unification, generalization
   - Polymorphic types

2. **Type Checking**:
   - Annotate AST with inferred types
   - Reject ill-typed programs before compilation

**Success Criteria**:
- Infer: `let id x = x` → `'a -> 'a`
- Reject: `1 + "hello"` with clear error

**Phase 2 Deliverables**:
- Closures and recursive functions
- Tuples and lists
- Pattern matching
- Type inference
- 150+ tests

---

### Phase 3: Advanced Features (Weeks 8-11)

**Goal**: Add records, discriminated unions, and full host interop.

#### Milestone 3.1: Records (Week 8)

**Tasks**:
1. **Record Types**:
   - `type TabInfo = { Title: string; Index: int }`
   - Record construction, field access

2. **Bytecode**:
   - `MakeRecord(TypeId, u8)`
   - `GetField(FieldId)`

**Success Criteria**:
- Execute: `let tab = { Title = "main"; Index = 0 }`
- Execute: `tab.Title` → `"main"`

#### Milestone 3.2: Discriminated Unions (Week 9)

**Tasks**:
1. **DU Types**:
   - `type Direction = Left | Right | Up | Down`
   - `type Option<'a> = None | Some of 'a`

2. **Pattern Matching over DUs**:
   - Tag-based dispatch
   - Payload extraction

**Success Criteria**:
- Execute:
  ```fsharp
  match dir with
  | Left -> "left"
  | Right -> "right"
  ```

#### Milestone 3.3: Host Interop (Week 10)

**Tasks**:
1. **Built-in Functions**:
   - `Value::BuiltinFn`
   - Registration API: `engine.register_builtin("print", builtin_print)`

2. **Host→Script Calls**:
   - `engine.call("format_title", &[tab_info])?`

3. **Script→Host Calls**:
   - Access registered functions from scripts

**Success Criteria**:
- Register and call Rust functions from scripts
- Pass complex types (records) across boundary
- Error propagation

#### Milestone 3.4: Hot-Reload (Week 11)

**Tasks**:
1. **File Watching**:
   - Detect `.fsrs` file changes
   - Recompile on modification

2. **State Preservation**:
   - Preserve global values across reloads
   - GC handling during reload

**Success Criteria**:
- Modify script, see changes without restart
- No memory leaks

**Phase 3 Deliverables**:
- Records and discriminated unions
- Full host interop
- Hot-reload support
- Terminal emulator demo use case
- 250+ tests

---

### Phase 4: Production Ready (Weeks 12-16)

**Goal**: Optimize, polish, and prepare for real-world use.

#### Milestone 4.1: Performance (Weeks 12-13)

**Tasks**:
1. **Optimizations**:
   - Computed goto dispatch (25% speedup)
   - NaN boxing for Value representation
   - Inline caching for field access
   - Bytecode peephole optimizer

2. **Benchmarking**:
   - Compare against Lua, Rhai, Gluon
   - Target: 5-10M ops/sec
   - Startup time < 5ms

**Success Criteria**:
- Lua-comparable performance
- Documented performance characteristics

#### Milestone 4.2: Error Messages (Week 14)

**Tasks**:
1. **Rich Diagnostics**:
   - Colorful error messages (miette crate)
   - Source snippets with carets
   - Suggestions for common errors

2. **Stack Traces**:
   - Full call stack on runtime errors
   - Debug symbols

**Success Criteria**:
- Helpful error messages like Elm/Rust
- User testing with newcomers

#### Milestone 4.3: Module System (Week 15)

**Tasks**:
1. **Multi-File Support**:
   - Module declarations
   - Import/open statements
   - Symbol resolution

2. **Standard Library**:
   - List, String, Option modules
   - Essential functions

**Success Criteria**:
- Multi-module projects
- Clear module boundaries

#### Milestone 4.4: Documentation & Polish (Week 16)

**Tasks**:
1. **Documentation**:
   - Language reference
   - Embedding guide
   - API docs
   - Tutorial series

2. **Examples**:
   - Terminal config (WezTerm-style)
   - Plugin system
   - Game scripting

3. **Tooling**:
   - VS Code syntax highlighting
   - REPL (bonus)

**Phase 4 Deliverables**:
- Production-quality VM
- Comprehensive documentation
- Example applications
- 400+ tests
- v1.0.0-rc1 release

---

## Future Enhancements (Post-v1.0)

### Computation Expressions

- CE syntax desugaring
- Builder pattern support
- Domain-specific DSLs

### Async Workflows

- `async { ... }` syntax
- Integration with Tokio
- Async host functions

### Advanced Type Features

- Type aliases
- Phantom types
- GADTs (bonus)

### Editor Tooling

- Language Server Protocol (LSP)
- Go-to-definition
- Auto-completion
- Inline errors

### WASM Backend

- Compile to WebAssembly
- Browser embedding

---

## Success Metrics

### Technical Metrics

- **Performance**: 5-10M ops/sec, < 5ms startup
- **Memory**: < 1MB baseline, efficient GC
- **Compatibility**: Linux, macOS, Windows
- **Test Coverage**: > 80%

### Quality Metrics

- **Error Messages**: Ranked "Helpful" by 90% of users
- **Documentation**: Complete API coverage
- **Stability**: Zero crashes in production use

### Adoption Metrics

- **Target**: 3 real-world applications using FSRS
- **Community**: Active GitHub discussions
- **Examples**: 10+ example projects

---

## Risk Assessment

### High Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Performance not Lua-comparable | High | Early benchmarking, NaN boxing, computed goto |
| Type inference too complex | High | Start simple, iterative refinement |
| GC causes latency spikes | Medium | Incremental GC, tunable thresholds |

### Medium Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Hot-reload edge cases | Medium | Comprehensive testing, clear limitations |
| Host interop complexity | Medium | Design review, prototype early |
| Pattern matching bugs | Medium | Extensive test suite |

### Low Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Parser complexity | Low | Well-understood problem, many references |
| Build system issues | Low | Use standard Cargo workspace |

---

## Technical Decisions

### Architecture Decisions

1. **Stack-based VM**: Simpler codegen, proven design (OCaml ZINC, Python)
2. **Hybrid GC**: Ref-counting + cycle detection, predictable pauses
3. **Rust enums for Value**: Simplicity first, NaN boxing if needed
4. **Rhai-inspired API**: Zero-boilerplate, automatic marshalling

### Language Decisions

1. **F# subset, not full F#**: Pragmatic scope, embeddable
2. **Eager evaluation**: Simpler implementation, predictable performance
3. **Curried functions**: Functional style, partial application
4. **No typeclasses (v1)**: Keep type system simple

### Tooling Decisions

1. **Just + Nushell**: Cross-platform, powerful scripting
2. **Cargo workspace**: Standard Rust tooling
3. **Tarpaulin for coverage**: Integrated, works with Cargo

---

## Development Guidelines

### Code Quality

- **Rustfmt**: Consistent formatting
- **Clippy**: Zero warnings policy
- **Tests**: Required for all PRs
- **Docs**: Public APIs must have docs

### Performance

- **Benchmark regressions**: CI checks
- **Profiling**: Regular flamegraph analysis
- **Memory**: Valgrind/Miri checks

### Process

- **Phased development**: No big-bang, incremental progress
- **Claude Code assistance**: Leverage AI for implementation
- **Weekly reviews**: Assess progress, adjust plans

---

## Getting Started

### For Developers

1. Read `docs/SETUP.md` for environment setup
2. Run `just bootstrap` to initialize
3. Start with Phase 1, Milestone 1.1 tasks
4. Use `docs/CLAUDE_CODE_NOTES.md` for detailed prompts

### For Users (Post-v1.0)

1. Install: `cargo install fsrs`
2. Follow tutorial: `docs/TUTORIAL.md`
3. See examples: `examples/`

---

## Resources

- **Repository**: https://github.com/raibid-labs/fsrs
- **Documentation**: `docs/`
- **Discussions**: GitHub Discussions
- **Issue Tracker**: GitHub Issues

---

**Next Review**: End of Phase 1 (Week 3)
**Contact**: raibid-labs team