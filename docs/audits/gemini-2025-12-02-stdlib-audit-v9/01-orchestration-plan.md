# Parallel Orchestration Plan V9

**Date**: 2025-12-03
**Status**: Active

## Work Streams

Based on Audit V9 findings, the following work can be executed in parallel:

### Stream A: Enable Script.eval - PR #181
**Branch**: `feat/script-eval-impl`
**Deliverable**: Updated `rust/crates/fusabi/src/lib.rs`

Implement real Script.eval in the fusabi crate:
- Create `script_eval_impl` function using full compilation pipeline
- Override the VM stub by registering host function after stdlib init
- Enable dynamic code evaluation in REPL

**Implementation**:
```rust
fn script_eval_impl(vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    let code = args[0].as_str()?;
    // Compile using fusabi-frontend
    let chunk = compile_to_chunk(code)?;
    // Execute in VM
    vm.execute(chunk)
}
```

**Dependencies**: None

---

### Stream B: Package Manager Scaffolding - PR #182
**Branch**: `feat/fusabi-pm`
**Deliverable**: New `rust/crates/fusabi-pm/` crate

Create foundation for `fpm` package manager:
- Define `Manifest` struct for `fusabi.toml`
- Implement basic parsing with serde + toml
- Add `fpm init` command scaffold

**Dependencies**: None

---

### Stream C: Async RFC Document - PR #183
**Branch**: `docs/rfc-002-async`
**Deliverable**: `docs/design/RFC-002-ASYNC-CE.md`

Design specification for async computation expressions:
- Syntax: `async { let! x = ... }`
- Desugaring rules to `Async.Bind`, `Async.Return`
- Async module signature

**Dependencies**: None

---

## Execution Order

```
┌─────────────────────────────────────────────────────────────┐
│                    PARALLEL EXECUTION                        │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Stream A ─────────────────────►                            │
│  (Script.eval Implementation)                               │
│                                                             │
│  Stream B ─────────────────────►                            │
│  (Package Manager Scaffolding)                              │
│                                                             │
│  Stream C ─────────────────────►                            │
│  (Async RFC Document)                                       │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## PR Merge Strategy

All streams are independent - merge as completed.

## Success Criteria

- [ ] `Script.eval` works in REPL context
- [ ] `fusabi-pm` crate compiles and parses manifests
- [ ] RFC-002 provides complete async CE specification
- [ ] All PRs merged, no open issues
- [ ] Release v0.26.0
