# Audit V9 Summary

**Date**: 2025-12-02
**Status**: **Assessment Complete**

## 1. Codebase Assessment
*   **Stdlib**: Excellent coverage. `Console`, `Process`, `Time`, etc., make Fusabi a viable scripting language.
*   **REPL**: `examples/repl.fsx` structure is correct, but it relies on `Script.eval` which currently returns a "Not Available" error because the VM crate cannot depend on the Compiler crate.
*   **Infrastructure**: CI is robust with docs freshness checks.

## 2. Critical Fix: `Script.eval`
To make the REPL work, the `fusabi` crate (which has access to both VM and Compiler) must override the stub implementation of `Script.eval`.

**Plan**:
1.  In `rust/crates/fusabi/src/lib.rs`, implement a `host_script_eval` function that calls `Compiler::compile_to_chunk`.
2.  Update `run_source_with_options` to register this function:
    ```rust
    fusabi_vm::stdlib::register_stdlib(&mut vm);
    // Override Script.eval with the real one
    vm.register_host_function("Script.eval", host_script_eval);
    ```

## 3. Next Steps: Implementation Instructions

### Task A: Enable Dynamic Evaluation (Fix REPL)
*   **File**: `rust/crates/fusabi/src/lib.rs`
*   **Action**: Implement `script_eval_impl` that uses `fusabi_frontend` to compile the string argument into a `Chunk`, then calls `vm.execute(chunk)`.
*   **Action**: Register this function in `run_source_with_options` and `run_file_with_options` to overwrite the default stub.

### Task B: Package Manager Scaffolding (`fpm`)
*   **Action**: Create new crate `rust/crates/fusabi-pm` (or `fpm`).
*   **Action**: Define `Manifest` struct for `fusabi.toml` using `serde` and `toml`.
    ```toml
    [package]
    name = "my-lib"
    version = "0.1.0"
    ```
*   **Action**: Implement `fpm init` to generate this file.

### Task C: Async Design (RFC)
*   **Action**: Create `docs/design/RFC-002-ASYNC-CE.md`.
*   **Content**: Detail how `async { let! x = ... }` desugars to `Async.Bind(..., fun x -> ...)`. Define the `Async` module signature (`Return`, `Bind`, `Delay`, `Zero`).

### Task D: Community Repository
*   **Instruction**: Create a new GitHub repository `fusabi-lang/fusabi-community`.
*   **Content**: Move `examples/commander.fsx` there as a "Showcase" project. Start porting simple F# utils (like `FSharp.Data` JSON traversal) as single-file modules.
