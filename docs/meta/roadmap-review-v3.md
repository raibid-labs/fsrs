# ROADMAP_REVIEW_V3.md

## 1. Executive Summary
**Date:** 2025-11-19
**Auditor:** Principal Compiler Architect
**Scope:** `fsrs-frontend`, `fsrs-vm`, `fsrs-demo`

**CRITICAL ALERT:** The FSRS VM is currently **functionally broken** for any non-trivial program. While the frontend can parse and compile code, the VM **cannot execute function calls**.

**Key Findings:**
1.  **VM `Call` Instruction Missing:** The VM's main loop does not implement `Instruction::Call`.
2.  **Closure Value Missing:** `Value` enum has no variant for functions or closures.
3.  **Dead Type Inference:** The inference engine is implemented but disconnected (stubbed out).
4.  **HOF Blocker:** Native functions (`HostFn`) cannot call back into the VM, making `List.map` impossible to implement in Rust.

## 2. Critical Fixes Required (P0)

### Epic: VM Function Execution
**Goal:** Make function calls work.

#### Task: Implement `Value::Closure`
**Labels:** `priority:critical`, `area:vm`
**Context:**
*   Modify `rust/crates/fsrs-vm/src/value.rs`.
*   Add `Value::Closure(Rc<Closure>)`.
*   Define `Closure` struct (chunk, upvalues, arity).

#### Task: Implement `Instruction::Call`
**Labels:** `priority:critical`, `area:vm`
**Context:**
*   Modify `rust/crates/fsrs-vm/src/vm.rs`.
*   Implement `Instruction::Call(arg_count)`:
    1.  Peek stack to find callee.
    2.  Check if `Value::Closure` or `Value::HostFn`.
    3.  If Closure: Push `Frame`, update `ip`.
    4.  If HostFn: Call Rust function, push result.

## 3. Architectural Debt

### RFD-006: Re-entrant Host Functions
**Labels:** `type:rfd`, `status:proposed`, `area:vm`

#### Context
To implement `List.map`, the Rust implementation of `map` needs to call the user-provided FSRS function.
Current `HostFn` signature: `Fn(&[Value]) -> Result<Value>`.
Required signature: `Fn(&mut Vm, &[Value]) -> Result<Value>`.

#### Recommendation
Refactor `HostFn` (and `StdlibFn`) to take `&mut Vm`. This allows native functions to:
1.  Push arguments to the stack.
2.  Call `vm.call_value()`.
3.  Retrieve results.

### RFD-007: Integrate Type Inference
**Labels:** `type:rfd`, `status:proposed`, `area:frontend`

#### Context
`inference.rs` is dead code. `Compiler::type_check` is a stub.

#### Recommendation
Wire them up. In `compiler.rs`, `compile_with_options` should instantiate `TypeInference`, run `infer()`, and error out if types mismatch.

## 4. Summary of Status
*   **Parsing:** ✅ Excellent (Modules, Records, DUs supported)
*   **Compilation:** ⚠️ Good (Emits bytecode, but inference disconnected)
*   **Runtime (Values):** ⚠️ Mixed (Records/DUs good, Closures missing)
*   **Runtime (Execution):** ❌ Broken (No function calls)

**Immediate Action:** Stop adding features. Fix `Instruction::Call` and `Value::Closure`.
