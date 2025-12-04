# Claude Code Configuration - Fusabi (Functional Scripting for Rust)

## 🚨 CRITICAL: CONCURRENT EXECUTION & FILE MANAGEMENT

**ABSOLUTE RULES**:
1. ALL operations MUST be concurrent/parallel in a single message
2. **NEVER save working files to the root folder**
3. ALWAYS organize files in appropriate subdirectories
4. **USE CLAUDE CODE'S TASK TOOL** for spawning agents concurrently

### ⚡ GOLDEN RULE: "1 MESSAGE = ALL RELATED OPERATIONS"

**MANDATORY PATTERNS**:
- **TodoWrite**: ALWAYS batch ALL todos in ONE call (5-10+ todos minimum)
- **Task tool (Claude Code)**: ALWAYS spawn ALL agents in ONE message with full instructions
- **File operations**: ALWAYS batch ALL reads/writes/edits in ONE message
- **Bash commands**: ALWAYS batch ALL terminal operations in ONE message

## Project Overview

**Fusabi** is a Mini-F# dialect with an embeddable Rust bytecode VM designed to replace Lua-style scripting. Key features:

- **Mini-F# Front-end**: Parser, type inference (Hindley-Milner), bytecode compiler
- **Bytecode VM**: Stack-based interpreter with GC, closures, pattern matching
- **Host Interop**: Lua-class embedding API for Rust applications
- **Hot-Reload**: Development-friendly script reloading

### Tech Stack
- **Language**: Rust (2021 edition)
- **Build**: Cargo workspace + Just + Nushell
- **Testing**: cargo test + tarpaulin (coverage)
- **Target Use Case**: Terminal emulator configs, plugin systems, embedded scripting
- **CLI Binary**: `fus` command for running Fusabi scripts

## 📁 File Organization Rules

**Directory Structure:**
```
/rust/crates
  /fusabi-frontend  - Parser, typechecker, bytecode compiler
  /fusabi-vm        - Bytecode VM runtime
  /fusabi-demo      - Demo host application
/tests              - Integration tests
/examples           - Example .fus scripts
/docs               - Documentation and design docs
/scripts            - Nushell automation scripts
/.github            - CI/CD workflows
```

**NEVER save to root folder. Use these directories:**
- `/rust/crates/fusabi-*/src` - Rust source code
- `/tests` - Test files
- `/docs` - Documentation
- `/examples` - Example scripts
- `/scripts` - Nushell automation

## 🛠️ Just Commands (via justfile)

### Quick Reference
```bash
just           # Show all available commands
just build     # Build all crates
just test      # Run test suite
just demo      # Run demo host
just watch     # Watch mode for development
just check     # Run all quality checks (fmt, lint, test)
```

### Development Workflow
```bash
just setup     # Initial project setup
just bootstrap # Bootstrap environment
just dev       # Development mode (watch + test)
just fmt       # Format code
just lint      # Run clippy
```

### Specialized Commands
```bash
just build-crate CRATE   # Build specific crate
just test-crate CRATE    # Test specific crate
just demo-example NAME   # Run specific example
just docs                # Generate and open docs
```

## 🐚 Nushell Scripts

All automation scripts are in `/scripts/*.nu`:
- `build.nu` - Build orchestration with error handling
- `test.nu` - Testing with filtering and reporting
- `bootstrap.nu` - Project setup and validation

## Code Style & Best Practices

### Rust Code
- Use `rustfmt` for formatting (zero tolerance)
- Follow Rust API guidelines
- Keep modules focused: < 500 lines per file
- Document public APIs with `///`
- Use `clippy` with zero warnings policy
- Prefer `Result<T, E>` over panics

### Mini-F# Scripts
- Follow F# style guide
- Type annotations for clarity
- Document script functions
- Examples in `examples/` directory

### Testing Strategy
- **Unit tests**: Test individual functions/modules
- **Integration tests**: Test crate interactions
- **Example scripts**: Validate language features
- **Target**: > 80% coverage

## 🎯 Development Phases

### Current: Phase 1 - MVP (Weeks 1-3)
Focus on core language and basic VM:

1. **Week 1**: AST, lexer, parser for Mini-F# subset
2. **Week 2**: VM foundation (values, bytecode, interpreter)
3. **Week 3**: End-to-end integration (compile + execute)

See `docs/roadmap.md` for complete phase breakdown.

### Implementation Guidance

When working on a specific component, follow this pattern:

**AST Work** (`rust/crates/fusabi-frontend/src/ast.rs`):
- Define types: `Literal`, `BinOp`, `Expr`
- Start minimal, extend as needed
- Document variant meanings

**Lexer Work** (`rust/crates/fusabi-frontend/src/lexer.rs`):
- Token stream with position tracking
- Handle keywords, operators, literals
- Clear error messages

**Parser Work** (`rust/crates/fusabi-frontend/src/parser.rs`):
- Recursive-descent for expressions
- Operator precedence handling
- Error recovery

**VM Work** (`rust/crates/fusabi-vm/src/`):
- `value.rs`: Value enum representation
- `bytecode.rs`: Instruction definitions
- `vm.rs`: Interpreter loop
- `gc.rs`: Garbage collector (Phase 2+)

## 🤖 Agent Execution Pattern

When using Claude Code's Task tool for parallel work:

```javascript
// Single message with all agents spawned concurrently
[Parallel Agent Execution]:
  Task("Parser implementation", "Implement recursive-descent parser for Phase 1 subset", "backend-architect")
  Task("VM interpreter", "Implement bytecode interpreter loop", "backend-architect")
  Task("Test suite", "Create comprehensive tests for lexer and parser", "test-writer-fixer")
  Task("Examples", "Create example .fus scripts for testing", "backend-architect")

  // Batch ALL todos
  TodoWrite { todos: [...10 todos for current milestone...] }

  // Batch file operations (if doing yourself)
  Write "rust/crates/fusabi-frontend/src/lexer.rs"
  Write "rust/crates/fusabi-vm/src/value.rs"
  Write "tests/integration_test.rs"
```

## 📋 Project-Specific Guidelines

### AST Design
- Keep core AST minimal (extensible later)
- Support: literals, variables, let, functions, if/then/else
- Phase 1: No records, DUs, or modules yet

### Bytecode VM
- Stack-based (like OCaml ZINC, Python)
- Start simple: LoadConst, Add, Sub, Mul, Div, Return
- Extend incrementally: Call, Jump, MatchTag

### Type System (Phase 2)
- Hindley-Milner inference
- Start with: int, bool, string, arrow types
- Add polymorphism: `'a -> 'a`

### Host Interop (Phase 3)
- Rhai-inspired API (zero boilerplate)
- `engine.register_function("name", rust_fn)`
- Automatic type marshalling

## 🔧 Build System

### Cargo Workspace
```toml
[workspace]
members = [
  "crates/fusabi-frontend",
  "crates/fusabi-vm",
  "crates/fusabi-demo"
]
resolver = "2"
```

### Just + Nushell Integration
- Justfile: User-facing command interface
- Nushell scripts: Complex implementation logic
- Cross-platform compatible
- Rich error handling

## 🚀 Getting Started

```bash
# Initial setup
just bootstrap    # Setup and validate environment

# Development
just dev          # Start watch mode
just test         # Run tests
just demo         # Run demo

# Quality checks
just check        # fmt + lint + test
just fmt          # Format code
just lint         # Run clippy
```

## 📚 Documentation Structure

- **roadmap.md**: Phased development plan with milestones
- **setup.md**: Environment setup and prerequisites
- **01-overview.md**: High-level architecture
- **02-language-spec.md**: Mini-F# language specification
- **03-vm-design.md**: VM architecture and bytecode
- **claude-code-notes.md**: Detailed implementation tasks
- **research-notes.md**: VM and embedding research
- **host-interop.md**: Host API design
- **nushell-patterns.md**: Scripting patterns
- **toc.md**: Documentation index

## 🎯 Phase-Specific Focus

### Phase 1 (Current): Core MVP
- **Priority**: Get something working end-to-end
- **Scope**: Minimal - integers, arithmetic, if/then/else
- **Tests**: Basic unit and integration tests
- **Documentation**: Implementation notes

### Phase 2: Language Features
- **Priority**: Functions, closures, data structures
- **Scope**: Let-rec, tuples, lists, pattern matching
- **Tests**: Comprehensive language feature tests
- **Documentation**: Language reference

### Phase 3: Advanced Features
- **Priority**: Records, DUs, host interop, hot-reload
- **Scope**: Production-ready embedding
- **Tests**: Real-world use case tests
- **Documentation**: Embedding guide

### Phase 4: Production Polish
- **Priority**: Performance, error messages, tooling
- **Scope**: v1.0 release preparation
- **Tests**: Performance benchmarks
- **Documentation**: Complete user guides

## 🔗 Integration Tips

1. **Use Just for all commands** - Don't run cargo directly
2. **Leverage Nushell for scripting** - Rich data handling
3. **Follow fusabi-lang patterns** - Consistency across projects
4. **Use Task tool for parallelism** - Maximum efficiency
5. **Batch all operations** - Single message principle
6. **Organize files properly** - Never dump in root

## 📊 Quality Standards

### Code Quality
- **Formatting**: rustfmt, zero deviations
- **Linting**: clippy, zero warnings
- **Testing**: > 80% coverage
- **Documentation**: All public APIs documented

### Performance Targets
- **Execution**: 5-10M ops/sec
- **Startup**: < 5ms for simple scripts
- **Memory**: < 1MB baseline
- **GC pauses**: < 10ms

### Error Messages
- **Clarity**: User understands what's wrong
- **Location**: Line/column information
- **Suggestions**: Hint at fixes when possible
- **Examples**: Show correct usage

## Support & Resources

- **Repository**: https://github.com/fusabi-lang/fusabi
- **Issues**: GitHub Issues for bugs/features
- **Discussions**: GitHub Discussions for questions
- **Documentation**: Complete in `docs/` directory

---

**Remember**: All operations in parallel, proper file organization, Task tool for agents!
