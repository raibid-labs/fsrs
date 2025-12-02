# Roadmap: Post-Stdlib Expansion

**Date**: 2025-12-02
**Status**: Active

With the Standard Library Foundation (List, Array, Map, Option, String) complete, we move to system integration and robustness.

## Phase 1: Integration Testing (Immediate)
While unit tests exist in Rust, we need to verify these functions work in the actual Fusabi environment (end-to-end).

1.  **Run the Demo**: Execute `examples/stdlib_demo.fsx` using the Fusabi CLI (if built) or `cargo run` to ensure it executes without error.
2.  **New Test Scripts**: Create focused `.fsx` test scripts for edge cases:
    - `tests/arrays_bounds.fsx`: Verify `Array.get/set` throw runtime errors as expected.
    - `tests/hof_stress.fsx`: Test nested closures and recursion with `List.fold`.

## Phase 2: Global "Prelude" Polish
Ensure the developer experience matches F#.

1.  **Implicit Imports**: Users currently have to access `List.map`. In F#, `List` is auto-opened or available. Verify if `open List` works or if we need to implement it.
2.  **Print Functions**: Ensure `print` / `printfn` are robust. Currently they might just be `Debug` implementations. We should make them `Display` friendly for strings.

## Phase 3: Missing Core Features
1.  **Result Module**: `Result<'T, 'E>` is standard in F# for error handling (more idiomatic than throwing exceptions).
    - Implement `Result` DU variants (`Ok`, `Error`).
    - Implement `Result` module (`map`, `bind`, `mapError`).
2.  **Math Module**: Basic arithmetic is present, but functions like `Math.sqrt`, `Math.sin`, `Math.pow` are needed for any scientific work.

## Phase 4: Documentation Generation
Implement the "Single Source of Truth" documentation strategy proposed in the first audit.
- Create a script `scripts/gen-docs.sh` (or `.nu`) to parse `///` comments from `stdlib/*.rs` and generate `docs/STDLIB_REFERENCE.md`.

## Recommendation for Next Session
Focus on **Phase 1 (Integration Testing)** and **Phase 4 (Docs Generation)**.
