# Parallel Orchestration Plan V10

**Date**: 2025-12-03
**Status**: Active

## Work Streams

Based on Audit V10 findings, the following work can be executed in parallel:

### Stream A: FPM Build/Run Commands - PR #185
**Branch**: `feat/fpm-build-run`
**Deliverable**: Updated `rust/crates/fusabi-pm/src/cli.rs`

Implement the placeholder `fpm build` and `fpm run` commands:
- `fpm build` - Compile main.fsx to bytecode
- `fpm run` - Execute the project via VM
- Integration with fusabi compiler and VM

**Dependencies**: None

---

### Stream B: CE Lexer Support - PR #186
**Branch**: `feat/ce-lexer`
**Deliverable**: Updated `rust/crates/fusabi-frontend/src/lexer.rs`

Add computation expression keywords to the lexer:
- `async` keyword
- `let!` token (let-bang)
- `do!` token (do-bang)
- `return` keyword (already may exist)
- `return!` token
- `yield` and `yield!` tokens

**Dependencies**: None

---

### Stream C: CE AST Nodes - PR #187
**Branch**: `feat/ce-ast`
**Deliverable**: Updated `rust/crates/fusabi-frontend/src/ast.rs`

Add AST nodes for computation expressions:
- `ComputationExpr { builder: String, body: Vec<CEStatement> }`
- `CEStatement` enum: `LetBang`, `DoBang`, `Return`, `ReturnBang`, `Yield`

**Dependencies**: None (AST can be defined before parser)

---

## Execution Order

```
┌─────────────────────────────────────────────────────────────┐
│                    PARALLEL EXECUTION                        │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Stream A ─────────────────────►                            │
│  (FPM Build/Run)                                            │
│                                                             │
│  Stream B ─────────────────────►                            │
│  (CE Lexer Tokens)                                          │
│                                                             │
│  Stream C ─────────────────────►                            │
│  (CE AST Nodes)                                             │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## PR Merge Strategy

All streams are independent - merge as completed.

## Success Criteria

- [ ] `fpm build` compiles Fusabi projects
- [ ] `fpm run` executes Fusabi projects
- [ ] Lexer recognizes CE keywords (`async`, `let!`, `do!`, etc.)
- [ ] AST has `ComputationExpr` node defined
- [ ] All PRs merged, no open issues
- [ ] Release v0.27.0

## Future Work (Post-V10)

- CE Parser implementation
- CE desugaring in compiler
- Async runtime in VM
- Async stdlib module
