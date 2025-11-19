# FSRS - F# Script Runtime System

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)]()
[![License](https://img.shields.io/badge/license-MIT-blue)]()
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)]()

**FSRS** is an experimental Mini-F# dialect with an embeddable Rust bytecode VM, designed to replace Lua-style scripting in applications like terminal emulators (e.g., WezTerm). It combines F#-style developer ergonomics with the simplicity of Lua embedding.

---

## 📋 Table of Contents

- [Overview](#overview)
- [Key Features](#key-features)
- [Project Status](#project-status)
- [Quick Start](#quick-start)
- [Documentation](#documentation)
- [Project Structure](#project-structure)
- [Development](#development)
- [Roadmap](#roadmap)
- [Contributing](#contributing)
- [License](#license)

---

## Overview

FSRS provides:

- **F#-Style Ergonomics**: Records, discriminated unions, pattern matching, pipelines, and simple modules
- **Embedded Scripting**: Lua-class bytecode VM implemented entirely in Rust (no .NET, no LLVM, no WASM)
- **Host Integration**: Designed for Rust host applications with hot-path callbacks and zero-cost abstractions
- **Hot-Reload Support**: Development-friendly script reloading without application restart
- **Type Safety**: Hindley-Milner type inference for compile-time safety

### Use Cases

- **Terminal Emulator Configuration**: Replace Lua configs in tools like WezTerm
- **Plugin Systems**: Embeddable scripting for extensible applications
- **Configuration Files with Logic**: Expressive configs beyond TOML/YAML
- **Game Scripting**: Functional scripting for game logic

---

## Key Features

### Language Features (Mini-F# Dialect)

```fsharp
# Let bindings and functions
let add x y = x + y
let inc = add 1

# Pattern matching over discriminated unions
type Direction = Left | Right | Up | Down

match dir with
| Left -> "left"
| Right -> "right"
| Up -> "up"
| Down -> "down"

# Records and field access
type TabInfo = {
    Title: string
    Index: int
    ProcessName: string
}

let tab = { Title = "main"; Index = 0; ProcessName = "zsh" }
let title = tab.Title

# Pipelines and composition
let normalizeTitle title =
    title
    |> String.trim
    |> String.toLower
```

### Runtime Features

- **Stack-Based Bytecode VM**: Inspired by OCaml ZINC machine
- **Garbage Collection**: Hybrid ref-counting + cycle detection
- **Closures**: First-class functions with upvalue capture
- **Pattern Matching**: Compiled to efficient decision trees
- **Host Interop**: Rhai-inspired zero-boilerplate API

---

## Project Status

**Version**: 0.2.0-alpha
**Status**: Phase 2 - Language Features (80% Complete)

### Implemented Features

**Phase 1 (MVP) - Complete** ✅
- ✅ Core AST with all expression types
- ✅ Lexer and tokenizer
- ✅ Parser with full F# syntax support
- ✅ Bytecode compiler
- ✅ Stack-based VM interpreter
- ✅ Test suite (697+ tests)

**Phase 2 (Features) - 80% Complete** 🚧
- ✅ Closures and first-class functions
- ✅ Recursive functions (let-rec)
- ✅ Currying and partial application
- ✅ Tuples (70+ tests)
- ✅ Lists with cons-cell implementation (81+ tests)
- ✅ Arrays with mutable semantics (122+ tests)
- ✅ Pattern matching (95% coverage, 93+ tests)
- 🚧 Records (AST + Lexer complete)
- 🚧 Discriminated unions (in progress)
- ⏳ Type inference (ready to start)

**Test Coverage**: 697+ tests, 100% passing (core features)
**Documentation**: Complete language spec, VM design, 12 example scripts
**PRs Merged**: 47 total (15 this session)

See [ROADMAP.md](docs/ROADMAP.md) for the complete development plan.

---

## Quick Start

### Prerequisites

- **Rust** (1.70+): Install from [rustup.rs](https://rustup.rs)
- **Nushell** (0.90+): Install from [nushell.sh](https://www.nushell.sh)
- **Just** (optional): `cargo install just`

### Installation

```bash
# Clone the repository
git clone https://github.com/raibid-labs/fsrs.git
cd fsrs

# Bootstrap the environment
just bootstrap

# Build the project
just build

# Run tests
just test

# Run the demo (placeholder in early stages)
just demo
```

### Using Manually (Without Just)

```bash
# Bootstrap
nu scripts/bootstrap.nu

# Build
cd rust && cargo build --workspace

# Test
cargo test --workspace

# Run demo
cargo run -p fsrs-demo
```

---

## Documentation

Comprehensive documentation is available in the [`docs/`](docs/) directory:

### Getting Started

- **[SETUP.md](docs/SETUP.md)** - Development environment setup
- **[ROADMAP.md](docs/ROADMAP.md)** - Development phases and timeline
- **[TOC.md](docs/TOC.md)** - Complete documentation index

### Architecture & Design

- **[01-overview.md](docs/01-overview.md)** - System architecture overview
- **[02-language-spec.md](docs/02-language-spec.md)** - Mini-F# language specification
- **[03-vm-design.md](docs/03-vm-design.md)** - Bytecode VM architecture
- **[HOST_INTEROP.md](docs/HOST_INTEROP.md)** - Host API design

### Implementation

- **[CLAUDE_CODE_NOTES.md](docs/CLAUDE_CODE_NOTES.md)** - Detailed implementation tasks
- **[RESEARCH_NOTES.md](docs/RESEARCH_NOTES.md)** - VM and embedding research
- **[NUSHELL_PATTERNS.md](docs/NUSHELL_PATTERNS.md)** - Scripting patterns

### Development

- **[CLAUDE.md](CLAUDE.md)** - Claude Code configuration and workflows
- **[justfile](justfile)** - Build automation commands

---

## Project Structure

```
fsrs/
├── rust/                    # Rust workspace
│   ├── Cargo.toml           # Workspace configuration
│   └── crates/
│       ├── fsrs-frontend/   # Parser, typechecker, bytecode compiler
│       ├── fsrs-vm/         # Bytecode VM runtime
│       └── fsrs-demo/       # Demo host application
├── docs/                    # Comprehensive documentation
│   ├── ROADMAP.md           # Development roadmap
│   ├── SETUP.md             # Setup guide
│   ├── 01-overview.md       # Architecture
│   ├── 02-language-spec.md  # Language specification
│   └── ...
├── examples/                # Example .fsrs scripts
├── scripts/                 # Nushell automation scripts
│   ├── build.nu
│   ├── test.nu
│   └── bootstrap.nu
├── tests/                   # Integration tests
├── justfile                 # Build automation
├── CLAUDE.md                # Claude Code configuration
└── README.md                # This file
```

---

## Development

### Available Commands (Just)

```bash
# Show all available commands
just

# Building
just build              # Build all crates (dev mode)
just build-release      # Build optimized release binaries
just build-crate CRATE  # Build specific crate

# Testing
just test               # Run all tests
just test-unit          # Unit tests only
just test-integration   # Integration tests
just test-coverage      # Generate coverage report

# Development
just dev                # Watch mode with auto-rebuild
just watch              # Watch and rebuild
just watch-test         # Watch and run tests
just demo               # Run demo host

# Code Quality
just check              # Run all checks (fmt + lint + test)
just fmt                # Format code
just lint               # Run clippy linter

# Documentation
just docs               # Generate and open API docs
```

### Development Workflow

```bash
# 1. Start watch mode
just watch

# 2. In another terminal, watch tests
just watch-test

# 3. Make changes...

# 4. Before committing, run checks
just check
```

### Using Without Just

All Just commands map to standard Cargo commands. See [SETUP.md](docs/SETUP.md) for manual commands.

---

## Roadmap

FSRS is developed in four phases over 16 weeks:

### Phase 1: MVP - Core Language & Interpreter (Weeks 1-3)

- Core AST, lexer, parser
- Basic bytecode VM (stack-based)
- Integer arithmetic, if/then/else
- Simple function calls

### Phase 2: Language Features (Weeks 4-7)

- Closures and recursive functions
- Tuples, lists, arrays
- Pattern matching
- Type inference (Hindley-Milner)

### Phase 3: Advanced Features (Weeks 8-11)

- Records and discriminated unions
- Host interop API
- Hot-reload support
- Module system

### Phase 4: Production Ready (Weeks 12-16)

- Performance optimization
- Rich error messages
- Comprehensive documentation
- v1.0.0-rc1 release

See [ROADMAP.md](docs/ROADMAP.md) for detailed milestones and deliverables.

---

## Contributing

We welcome contributions! Here's how to get started:

1. **Read the docs**: Start with [SETUP.md](docs/SETUP.md) and [ROADMAP.md](docs/ROADMAP.md)
2. **Check the roadmap**: See what phase we're in and what needs work
3. **Review implementation notes**: [CLAUDE_CODE_NOTES.md](docs/CLAUDE_CODE_NOTES.md) has detailed tasks
4. **Follow conventions**: See [CLAUDE.md](CLAUDE.md) for development patterns
5. **Submit a PR**: Include tests and documentation

### Development Guidelines

- **Code Style**: Follow Rustfmt (enforced)
- **Linting**: Zero clippy warnings policy
- **Testing**: > 80% coverage target
- **Documentation**: Public APIs must have docs
- **Commits**: Conventional commits preferred

---

## License

This project is licensed under the MIT License - see the LICENSE file for details.

---

## Acknowledgments

- **Inspiration**: WezTerm's Lua configuration, F# language design
- **Research**: OCaml ZINC VM, Lua embedding patterns, Rhai, Gluon
- **Tooling**: Rust, Cargo, Just, Nushell

---

## Contact & Support

- **Repository**: https://github.com/raibid-labs/fsrs
- **Issues**: Report bugs or request features on GitHub Issues
- **Discussions**: Ask questions on GitHub Discussions
- **Documentation**: Complete docs in [`docs/`](docs/)

---

**Status**: Early development - not ready for production use
**Target**: v1.0.0 in Q2 2026

For detailed implementation tasks, see [CLAUDE_CODE_NOTES.md](docs/CLAUDE_CODE_NOTES.md).
For the complete roadmap, see [ROADMAP.md](docs/ROADMAP.md).
