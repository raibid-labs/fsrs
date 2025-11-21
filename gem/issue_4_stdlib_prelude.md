# Issue 4: [Stdlib] Implement Implicit Prelude & Core Operators

**Labels:** `feature`, `area:frontend`, `area:stdlib`

## Context
Users currently have to `open List` manually. Operators like `|>` (pipe) are missing or treated as generic binary ops without proper precedence.

## Implementation Plan
**Objective:** Auto-import core functions and fix operators.

1.  **Core Module** (`fusabi-vm/src/stdlib/core.rs`):
    * Implement `print`, `printfn` (wrappers over println), `id`, `ignore`, `fst`, `snd`.
    * Register these in `StdlibRegistry` under a generic scope (not `Core.print`, just `print`).

2.  **Implicit Open** (`fusabi-frontend/src/compiler.rs`):
    * Modify `Compiler::compile_program`. Before parsing/compiling the user module, inject the bindings from the `Core` module into the initial `Compiler` scope.

3.  **Pipeline Operator** (`fusabi-frontend/src/lexer.rs` & `parser.rs`):
    * Add `Token::PipeRight` (`|>`).
    * In `Parser`, add a specific rule for `|>` with precedence *lower* than function application but *higher* than assignment.
    * Desugar `a |> f` to `f a`.
