# Audit V14 Summary

**Date**: 2025-12-02
**Status**: **Assessment Complete**

## 1. `fusabi` Repo Assessment
*   **Async Compiler**: `rust/crates/fusabi-frontend/src/compiler.rs` now has `compile_computation_expr` and `desugar_ce_statements`.
    *   **Analysis**: The desugaring logic looks correct (`let!` -> `Bind`, `return` -> `Return`, etc.). It generates MethodCall expressions on the builder object.
    *   **Missing**: The `Async` module itself in `stdlib`. The compiler generates calls to `Bind`, `Return`, etc., but the `Async` builder object doesn't exist in the runtime yet to handle these calls.
*   **Overall Health**: Core language features (Async CE support) are landing in the compiler.

## 2. `fusabi-community` Repo Assessment
*   **Structure**: The repo structure is correct (`packages/`, `registry/`, `tools/`).
*   **Packages**:
    *   `packages/commander`: Exists (migrated from examples).
    *   `packages/json`: Exists (scaffolded).
*   **Registry**: `registry/index.toml` exists.
*   **Verdict**: The community repo is initialized and ready for growth.

## 3. The "Ultrathink" Roadmap

We are now at the stage where the language is feature-complete enough for real work, but the ecosystem is young.

### Track A: Fusabi Core (The Engine)
1.  **Async Runtime**: The compiler emits `Async.Bind`. We need `stdlib/async.rs` to implement the `Async` builder and the `Async` type (likely wrapping a Rust Future or a state machine).
2.  **LSP (Language Server)**: This is the next big DX leap. `tree-sitter-fusabi` -> `fusabi-lsp`.
3.  **Optimization**: The VM is a simple stack machine. JIT or optimized bytecode (superinstructions) would be the next performance step.

### Track B: Fusabi Community (The Ecosystem)
1.  **CI for Packages**: We need a standardized GitHub Action in `fusabi-community` that builds and tests *every* package in `packages/` on PRs.
2.  **Registry Automation**: `fpm publish` should eventually automate updating `registry/index.toml`.
3.  **Standard Lib Extensions**: Things that don't belong in `fusabi-vm` (e.g. HTTP client, SQLite driver) should live here.

## Instructions for Claude (Fusabi Repo)
1.  **Implement `Async` Runtime**: Create `stdlib/async.rs` with the `Async` builder native functions (`Bind`, `Return`, `Zero`, `Delay`, `Combine`) to back the compiler's desugaring.
2.  **Register `Async`**: Add it to `stdlib/mod.rs` and `vm.globals`.

## Instructions for Claude (Fusabi Community Repo)
(To be executed in the other session)
1.  **Setup CI**: Create `.github/workflows/test-packages.yml`.
2.  **Expand `json`**: Implement actual JSON combinators in `packages/json/src/lib.fsx`.
