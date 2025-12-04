# Parallel Orchestration Plan V11

**Date**: 2025-12-03
**Status**: Active

## Work Streams

Based on Audit V11 findings, noting that Task A (fpm manifest) was completed in V9/V10:

### Stream A: Documentation Cleanup - PR #189
**Branch**: `chore/docs-cleanup`
**Deliverable**: Reorganized `docs/` folder

Clean up docs/ for the "Pull-based" aggregation:
- Keep `STDLIB_REFERENCE.md` at root (primary reference)
- Move specs to `docs/design/` (language-spec, vm-design, ABI, etc.)
- Move meta/project docs to `docs/meta/` (retrospectives, CI notes)
- Archive old issue docs to `docs/archive/`

**Dependencies**: None

---

### Stream B: Async Parser Support - PR #190
**Branch**: `feat/ce-parser`
**Deliverable**: Updated `rust/crates/fusabi-frontend/src/parser.rs`

Implement computation expression parsing:
- Parse `async { ... }` blocks
- Parse `let!`, `do!`, `return`, `return!`, `yield`, `yield!` statements
- Build CEStatement and ComputationExpr AST nodes
- Basic desugaring stub

**Dependencies**: V10 lexer tokens (completed)

---

## Execution Order

```
┌─────────────────────────────────────────────────────────────┐
│                    PARALLEL EXECUTION                        │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Stream A ─────────────────────►                            │
│  (Docs Cleanup)                                             │
│                                                             │
│  Stream B ─────────────────────►                            │
│  (CE Parser)                                                │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## PR Merge Strategy

Both streams are independent - merge as completed.

## Success Criteria

- [ ] `docs/` folder organized for Pull-based aggregation
- [ ] `STDLIB_REFERENCE.md` prominent at docs root
- [ ] Parser recognizes `async { }` blocks
- [ ] Parser builds CEStatement nodes
- [ ] All PRs merged, no open issues
- [ ] Release v0.28.0
