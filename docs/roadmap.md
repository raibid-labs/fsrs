# Fusabi Project Roadmap

**Version**: 0.2.0-alpha
**Status**: Phase 3 (Advanced Features)
**Last Updated**: November 22, 2025

---

## Executive Summary

Fusabi (Functional Scripting for Rust) is an experimental Mini-F# dialect with an embeddable Rust bytecode VM. The goal is to provide F#-style developer ergonomics (records, DUs, pattern matching, pipelines) in a small, eager, expression-oriented language suitable for embedded scripting and configuration.

### Vision

- **F#-Style Ergonomics**: Records, discriminated unions, pattern matching, pipelines, and simple modules
- **Embedded Scripting**: Lua-class bytecode VM implemented entirely in Rust
- **Host Integration**: Designed for Rust host applications with hot-path callbacks
- **Performance**: Target 5-10M operations/second with Lua-comparable startup time

---

## Current State (November 2025)

### Completed Features
- **Core Language**: Literals, Variables, Let-bindings, If/Then/Else
- **Functions**: Lambdas, Closures, Currying, Recursion (Let-Rec)
- **Data Structures**: Tuples, Lists (Cons), Arrays (Mutable), Records, Discriminated Unions
- **Pattern Matching**: Full support for literals, tuples, DUs, and nesting
- **Type System**: Hindley-Milner Type Inference
- **Modules**: Basic module support and imports
- **Testing**: Comprehensive test suite for all implemented features

### In Progress / Planned (Gem Issues)
- **WS1: Re-entrant Host Functions**: allowing host functions to call back into VM (Critical for Stdlib)
- **WS2: Garbage Collection**: Cycle detection for `Rc<RefCell>`
- **WS3: Stdlib Prelude**: Expanded standard library (List.map, etc.)
- **WS4: Bytecode Serialization**: `.fzb` format support
- **WS5: MCP Server**: AI Agent integration
- **WS6: Documentation & Examples**: Expanded guides and usage examples
- **WS7: Branding**: Visual identity and CLI polish

---

## Development Phases

### Phase 1: MVP - Core Language & Interpreter (Completed)
- Core AST, Lexer, Parser
- Basic VM (Integers, Booleans, Strings)
- End-to-end execution

### Phase 2: Language Features (Completed)
- **Functions**: Closures, Let-Rec, Currying
- **Data Structures**: Tuples, Lists, Arrays
- **Pattern Matching**: Match expressions, destruction
- **Type System**: Inference engine

### Phase 3: Advanced Features (In Progress)
- **Records & DUs**: Implemented
- **Host Interop**: Basic support implemented. Needs re-entrancy (WS1).
- **Module System**: Basic support implemented.

### Phase 4: Production Ready (Next)
- **Performance**: Benchmarking and optimizations
- **Tooling**: Language Server, Debugger
- **Ecosystem**: Package manager

---

## Active Workstreams (Gem Issues)

See `docs/workstreams/gem-issues/00-overview.md` for detailed breakdown.

1. **WS1**: VM Core - Re-entrant Host Functions
2. **WS2**: VM Core - Garbage Collection
3. **WS3**: Frontend - Stdlib Prelude & Operators
4. **WS4**: Serialization & Performance
5. **WS5**: Ecosystem - MCP Server
6. **WS6**: Examples & Documentation
7. **WS7**: Branding & Visual Identity
