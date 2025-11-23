# Issue 1: [Architecture] Enable Re-entrant Host Functions (HOF Support)

**Labels:** `architecture`, `priority:critical`, `area:vm`, `blocked:stdlib`

## Context
Currently, `HostFn` (native Rust functions) has the signature `Fn(&[Value]) -> Result<Value>`. This prevents native functions from calling back into the VM (e.g., to execute a lambda passed to `List.map`). This blocks the implementation of higher-order functions in the Standard Library.

## Implementation Plan
**Objective:** Refactor `HostFn` to allow re-entrancy.

1.  **Refactor Type Definitions** (`fusabi-vm/src/host.rs`):
    * Change `HostFn` signature to: `type HostFn = Box<dyn Fn(&mut VmContext, &[Value]) -> Result<Value, VmError>>;`
    * Define `VmContext` struct that exposes a safe subset of `Vm` operations (push, pop, call) without exposing the entire `Vm` struct to avoid borrow checker hell (if possible) OR simply pass `&mut Vm`. *Note: Passing `&mut Vm` is easier but requires the Vm to be exception-safe.*

2.  **Update Registry** (`fusabi-vm/src/host.rs`):
    * Update `HostRegistry::call` to accept `&mut Vm`.

3.  **Update VM Loop** (`fusabi-vm/src/vm.rs`):
    * In `Instruction::Call`, when dispatching a `HostFn`, pass `self` (the VM instance).
    * *Constraint:* You may need to wrap the VM state in a `RefCell` or split the VM into `State` and `Executor` to allow the callback to mutate the stack while the outer loop is borrowing it.

4.  **Add Helper API**:
    * Add `Vm::call_closure(closure: Value, args: &[Value]) -> Result<Value>` to allow the host function to invoke a script closure easily.

5.  **Fix Stdlib**:
    * Update all existing stdlib functions (`list.rs`, `string.rs`) to match the new signature (ignore the `vm` arg for now).
    * Implement `List.map` in `list.rs` using the new capability.
