# Audit V11 Summary

**Date**: 2025-12-02
**Status**: **Ready for Expansion**

## 1. `fusabi-community` Roadmap
I have defined the structure for the community repo. It will act as the **Registry** and **Monorepo**.
*   **Action**: Launch a new Claude session for `fusabi-community` initialization.
*   **Task**: Move `commander` there.

## 2. `fusabi` Repo (Current) Next Steps
The scaffolding for `fusabi-pm` exists but is empty.

**Task A: Package Manager Logic**
*   **File**: `rust/crates/fusabi-pm/src/manifest.rs`
*   **Goal**: Implement `serde` structs for `[package]` and `[dependencies]`.
*   **Goal**: Implement `init` command logic to create a default `fusabi.toml`.

**Task B: Documentation Cleanup**
*   **Goal**: Move `docs/*.md` (excluding `STDLIB_REFERENCE.md`) into `docs/design/` or `docs/meta/` to make the "Pull-based" aggregation cleaner. `STDLIB_REFERENCE.md` should remain discoverable.

**Task C: Async Compiler Support**
*   **Goal**: Start implementing the `async` keyword in the Lexer/Parser.

## Instructions for Claude Code
1.  **Implement `fusabi-pm` Manifest**: Write the code to parse/generate `fusabi.toml`.
2.  **Cleanup Docs**: Organize the `docs/` folder.
3.  **Start Async**: Add `async` token to `Lexer`.
