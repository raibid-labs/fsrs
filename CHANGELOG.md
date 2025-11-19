# Changelog

All notable changes to FSRS will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Records AST and lexer support (Layer 1 complete)
- Discriminated unions work in progress
- Type expression parsing for record types

## [0.2.0-alpha] - 2025-11-18

### Added
- **Pattern Matching** (#27): Full pattern matching support with 95% test coverage
  - Literal patterns (int, bool, string)
  - Variable binding patterns
  - Wildcard patterns (_)
  - Tuple patterns (simple and nested)
  - 93+ tests, 4 example scripts
  - Complete documentation in language spec
- **Arrays** (#26): Full array support with immutable update semantics
  - Array literals: `[|1; 2; 3|]`
  - Array indexing: `arr.[0]`
  - Immutable updates: `arr.[0] <- 99`
  - Array.length operation
  - 122+ tests, 4 example scripts
- **Lists** (#25): Functional cons-cell lists
  - List literals: `[1; 2; 3]`
  - Cons operator: `1 :: [2; 3]`
  - Head/tail operations
  - 81+ tests, 4 example scripts
- **Tuples** (#24): Heterogeneous fixed-size collections
  - Tuple syntax: `(1, "hello", true)`
  - Nested tuples: `(1, (2, 3))`
  - Pattern matching over tuples
  - 70+ tests, examples included
- **Currying** (#23): Automatic desugaring of multi-parameter functions
- **Let-Rec** (#22): Recursive function bindings
- **Closures** (#21): Full closure support with upvalue capture

### Changed
- Test suite grew from 477 to 697+ tests (+46%)
- Documentation updated with complete examples
- Language spec updated for all new features

### Fixed
- Pattern matching stack management for variable binding
- Tuple pattern destructuring issues
- Various clippy warnings across codebase

## [0.1.0-alpha] - 2025-11-17

### Added
- Initial project structure
- Core AST definitions
- Lexer and tokenizer
- Basic parser
- Bytecode instruction set
- Stack-based VM interpreter
- Value representation (Int, Bool, String, Unit)
- Basic arithmetic and comparison operations
- If/then/else expressions
- Let bindings
- Function application
- Test infrastructure
- CI/CD with pre-commit/pre-push hooks
- Comprehensive documentation

### Infrastructure
- Rust workspace with 3 crates (fsrs-frontend, fsrs-vm, fsrs-demo)
- Just + Nushell build automation
- GitHub Actions CI/CD
- Claude Code configuration

## [0.0.1] - 2025-11-16

### Added
- Repository initialization
- Documentation structure
- Design documents
- Roadmap planning

[Unreleased]: https://github.com/raibid-labs/fsrs/compare/v0.2.0-alpha...HEAD
[0.2.0-alpha]: https://github.com/raibid-labs/fsrs/compare/v0.1.0-alpha...v0.2.0-alpha
[0.1.0-alpha]: https://github.com/raibid-labs/fsrs/compare/v0.0.1...v0.1.0-alpha
[0.0.1]: https://github.com/raibid-labs/fsrs/releases/tag/v0.0.1
