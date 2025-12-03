# Roadmap: System Integration & Documentation Fixes

**Date**: 2025-12-02
**Status**: Active

## Phase 1: Documentation Completeness (Immediate)
The documentation generator needs to be modernized and updated for the new modules.

1.  **Rewrite `scripts/gen-docs.sh` to NuShell**:
    - Create `scripts/gen-docs.nu`.
    - Port the regex parsing logic from the Bash/Awk script to idiomatic NuShell.
    - Ensure it includes **ALL** modules: `Array`, `List`, `Map`, `Option`, `String`, `JSON`, `Result`, `Math`, `Process`, `Time`, `Url`, `Config`, `Events`, `TerminalInfo`, `TerminalControl`, `Commands`, `UIFormatting`.
    - Add descriptions for all new modules.
    - Delete `scripts/gen-docs.sh`.
2.  **Regenerate Docs**: Run `nu scripts/gen-docs.nu` and verify `docs/STDLIB_REFERENCE.md` is complete.

## Phase 2: System Integration Testing
We have a lot of new power (Process, FS, Terminal). We need to verify it works in the runtime.

1.  **Create `examples/system_demo.fsx`**:
    - Use `Process.runShell` to echo something.
    - Use `Time.now` to print the time.
    - Use `Math.sin` to calculate something.
    - Use `Result.bind` to chain operations.
    - Use `Url.parse` to check a URL.
2.  **Run the script**: Ensure it executes without panic.

## Phase 3: Feature Freeze & Polish
With this massive expansion, we should freeze new feature additions and focus on stability.
- Ensure error messages from `Process` or `File` operations are clean (Fusabi exceptions vs Rust panics).
- Ensure type safety in the new modules.

## Instructions for Developer/Agent
1.  **Port doc script to NuShell**. Ensure it captures all new modules.
2.  **Write the system demo**. This proves the features work.