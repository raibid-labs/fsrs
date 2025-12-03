# Parallel Orchestration Plan V7

**Date**: 2025-12-03
**Status**: Active

## Work Streams

Based on Audit V7 findings, the following work can be executed:

### Stream A: CI Documentation Integrity - PR #176
**Branch**: `feat/ci-docs-check`
**Deliverable**: Updated `.github/workflows/ci.yml`

Add CI job to enforce documentation freshness:
- Install Nushell
- Run `nu scripts/gen-docs.nu`
- Assert `git diff --exit-code docs/STDLIB_REFERENCE.md`
- Fail build if docs are stale

**Dependencies**: None

---

### Stream B: Script Module - PR #177
**Branch**: `feat/script-module`
**Deliverable**: `rust/crates/fusabi-vm/src/stdlib/script.rs`

Implement dynamic code evaluation:
- `Script.eval : string -> Result<Value, string>`
- Execute Fusabi code in current VM context
- Return result or error message

**Dependencies**: None

---

### Stream C: REPL Example - PR #178
**Branch**: `feat/repl-example`
**Deliverable**: `examples/repl.fsx`

Self-hosted REPL demonstrating:
- Interactive loop using Console module
- Dynamic evaluation using Script.eval
- Error handling and display
- Help commands

**Dependencies**: Stream B must complete first

---

## Execution Order

```
┌─────────────────────────────────────────────────────────────┐
│                    PARALLEL EXECUTION                        │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Stream A ─────────────────────►                            │
│  (CI Docs Check)                                            │
│                                                             │
│  Stream B ─────────────────────► Stream C                   │
│  (Script.eval)                   (REPL)                     │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## PR Merge Strategy

1. Stream A can be merged independently
2. Stream B must merge before Stream C starts
3. Stream C merges last

## Success Criteria

- [ ] CI fails when `STDLIB_REFERENCE.md` is stale
- [ ] `Script.eval` executes Fusabi code dynamically
- [ ] `examples/repl.fsx` provides interactive Fusabi experience
- [ ] All PRs merged, no open issues
- [ ] New release cut (v0.24.0)
