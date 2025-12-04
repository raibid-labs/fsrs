# Changelog

All notable changes to Fusabi will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2025-11-23

### Documentation Fixes
- **Rebranding**: Updated crate-level READMEs to correct "FSRS" to "Fusabi".
- **Metadata**: Updated crate descriptions and documentation links for crates.io.

## [0.1.0] - 2025-11-23

### Initial Public Release (Fusabi)

### Features
- **Host Interop (WS1)**: 
  - Re-entrant host functions (`List.map` works natively)
  - `NativeFn` value type for partial application
  - Ergonomic `HostRegistry` API for embedding
- **Standard Library (WS3)**:
  - `List`, `String`, `Option` modules in global scope
  - `print`, `printfn`, `id`, `ignore`, `fst`, `snd` core functions
  - Pipeline operator `|>`
  - Implicit module access (`List.length` instead of `List.List.length`)
- **Bytecode Serialization (WS4)**:
  - `.fzb` binary format with magic bytes and versioning
  - `fus grind` command to compile scripts
  - `fus run` supports executing `.fzb` files directly
- **Language Core**:
  - F# dialect with Let-bindings, Recursion, Currying
  - Pattern Matching (literals, tuples, variables, wildcards)
  - Data Types: Records, Discriminated Unions, Lists (`::`), Arrays (`[| |]`), Tuples
- **Tooling**:
  - `fus` CLI for running and compiling
  - `justfile` for build automation

### Changes
- **Rebranding**: Project renamed from FSRS to Fusabi
- **Architecture**: Split into `fusabi-frontend` (compiler), `fusabi-vm` (runtime), and `fusabi` (CLI)

[0.1.0]: https://github.com/fusabi-lang/fusabi/releases/tag/v0.1.0
