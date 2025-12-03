# Roadmap V7: Ecosystem & Polish (Revised)

**Date**: 2025-12-02
**Status**: Active

We have a solid VM and Stdlib. Now we build the ecosystem and ensure our documentation artifact is robust for the central aggregator.

## Phase 1: Documentation Integrity (Immediate)
We must ensure `docs/STDLIB_REFERENCE.md` never drifts from the code, as it is the source of truth for the central docs site.

1.  **CI Check**: Update `.github/workflows/ci.yml`.
    - Add a step to install Nushell.
    - Run `nu scripts/gen-docs.nu`.
    - Assert `git diff --exit-code docs/STDLIB_REFERENCE.md`. Fail if docs are stale.

## Phase 2: The "Fusabi REPL" (Dogfooding II)
We need a self-hosted REPL to verify the language's interactivity.
1.  **Implement `Script` Module**:
    - `Script.eval : string -> Result<Value, string>` (Executes Fusabi code in the current VM context).
2.  **Build `examples/repl.fsx`**:
    - Loop: `Console.write "> "`, `Console.readLine`, `Script.eval`, `print result`.
    - Handle errors gracefully.

## Phase 3: Community Initialization
1.  **Create Repo**: `fusabi-community`.
2.  **Manual Port**: Port a simple library (e.g. `FSharp.Control.Async` or a JSON utility) to verify cross-repo sharing (via git submodule or copy-paste for now).

## Instructions for Developer (Claude)
1.  **Update CI**: Modify `ci.yml` to enforce documentation freshness.
2.  **Implement `Script.eval`**: This is the key enabler for the REPL.
3.  **Write REPL**: Create `examples/repl.fsx`.