# Workstream 1: VM Core - Re-entrant Host Functions

## Status
🟢 Complete

## Overview
Refactor `HostFn` (native Rust functions) to allow re-entrancy, enabling native functions to call back into the VM. This is **critical** for implementing higher-order functions like `List.map`, `List.filter`, and `List.fold` in the standard library.

## Objectives
- [x] Refactor `HostFn` type signature to accept `VmContext` or `&mut Vm`
- [x] Update VM loop to pass VM instance to host functions
- [x] Implement helper API: `Vm::call_closure(closure, args)`
- [x] Update all existing stdlib functions to new signature
- [x] Implement `List.map` as proof of concept
- [x] Ensure exception safety and borrow checker compatibility

## Summary of Changes
- Introduced `NativeFn` value variant to support partial application and native function handles.
- Updated `HostRegistry` to store `Rc<HostFn>` where `HostFn` is `dyn Fn(&mut Vm, &[Value]) -> Result<Value, VmError>`.
- Implemented re-entrancy in `vm.call_value` and `HostRegistry::call`, allowing host functions to invoke closures via the VM.
- Implemented `List.map` in `stdlib/list.rs` using this new capability.
- Added `repro_map.fsx` demonstrating successful `List.map` usage with a closure.

## Agent Assignment
**Suggested Agent Type**: `backend-architect`, `rust-pro`, `coder`
**Skill Requirements**: Rust ownership/borrowing, VM architecture, systems programming

## Dependencies
- None (foundation workstream, can start immediately)

## Tasks

### Task 1.1: Refactor HostFn Type Definition
**Description**: Change `HostFn` signature to allow VM access.

**Current Signature**:
```rust
type HostFn = Box<dyn Fn(&[Value]) -> Result<Value, VmError>>;
```

**New Signature (Option A - Full VM access)**:
```rust
type HostFn = Box<dyn Fn(&mut Vm, &[Value]) -> Result<Value, VmError>>;
```

**New Signature (Option B - Restricted context)**:
```rust
pub struct VmContext<'a> {
    vm: &'a mut Vm,
}

impl<'a> VmContext<'a> {
    pub fn push(&mut self, value: Value) { ... }
    pub fn pop(&mut self) -> Option<Value> { ... }
    pub fn call(&mut self, closure: Value, args: &[Value]) -> Result<Value, VmError> { ... }
}

type HostFn = Box<dyn Fn(&mut VmContext, &[Value]) -> Result<Value, VmError>>;
```

**Deliverables**:
- Updated `HostFn` type in `fusabi-vm/src/host.rs`
- Choose between Option A (simpler) or Option B (safer abstraction)
- Consider using `RefCell` or splitting VM into `State` + `Executor` if borrow checker issues arise

**Files to Create/Modify**:
- `rust/crates/fusabi-vm/src/host.rs`

**Validation**:
```bash
cd rust/crates/fusabi-vm
cargo build
# Should compile with new HostFn signature
```

---

### Task 1.2: Update HostRegistry
**Description**: Modify `HostRegistry::call` to accept `&mut Vm` and pass it to host functions.

**Deliverables**:
- Updated `HostRegistry::call(name, vm: &mut Vm, args: &[Value])`
- Updated `HostRegistry::register` to accept new `HostFn` signature
- Ensure all registry operations work with new signature

**Files to Create/Modify**:
- `rust/crates/fusabi-vm/src/host.rs`

**Validation**:
```bash
cd rust/crates/fusabi-vm
cargo test --lib host
# All host registry tests pass
```

---

### Task 1.3: Update VM Instruction Loop
**Description**: Modify `Instruction::Call` in VM loop to pass `self` (VM instance) to host functions.

**Deliverables**:
- Updated `match instruction` arm for `Instruction::Call` in `vm.rs`
- Pass `self` (or `&mut self`) to `HostRegistry::call`
- Handle borrow checker issues (may need `RefCell` wrapper or VM state split)

**Files to Create/Modify**:
- `rust/crates/fusabi-vm/src/vm.rs`

**Constraint**: If borrow checker complains about mutable borrow while VM loop is borrowing, consider:
1. Using `RefCell<VmState>` for interior mutability
2. Splitting VM into `VmState` and `VmExecutor`
3. Using unsafe (last resort, requires careful safety documentation)

**Validation**:
```bash
cd rust/crates/fusabi-vm
cargo test --lib vm
# VM tests pass with new calling convention
```

---

### Task 1.4: Add Helper API for Calling Closures
**Description**: Implement `Vm::call_closure` to allow host functions to easily invoke script closures.

**Deliverables**:
- `pub fn call_closure(&mut self, closure: Value, args: &[Value]) -> Result<Value, VmError>`
- Extract closure's chunk, upvalues, and parameter count
- Push args onto stack
- Execute closure bytecode
- Return result value

**Files to Create/Modify**:
- `rust/crates/fusabi-vm/src/vm.rs`

**Implementation Sketch**:
```rust
impl Vm {
    pub fn call_closure(&mut self, closure: Value, args: &[Value]) -> Result<Value, VmError> {
        // Extract closure data
        let closure_data = match closure {
            Value::Closure(c) => c,
            _ => return Err(VmError::TypeError("Expected closure".into())),
        };

        // Push args onto stack
        for arg in args {
            self.stack.push(arg.clone());
        }

        // Call the closure (similar to Instruction::Call logic)
        // ...

        // Pop and return result
        self.stack.pop().ok_or(VmError::StackUnderflow)
    }
}
```

**Validation**:
```rust
#[test]
fn test_call_closure_from_rust() {
    let mut vm = Vm::new();

    // Create a closure: fn(x) { x + 1 }
    let closure = /* ... */;

    let result = vm.call_closure(closure, &[Value::Int(5)]).unwrap();
    assert_eq!(result, Value::Int(6));
}
```

---

### Task 1.5: Update Existing Stdlib Functions
**Description**: Update all existing standard library functions to match new `HostFn` signature.

**Deliverables**:
- Update `fusabi-vm/src/stdlib/list.rs` functions
- Update `fusabi-vm/src/stdlib/string.rs` functions
- Update `fusabi-vm/src/stdlib/math.rs` functions (if exists)
- For functions that don't need VM access, ignore the `vm` parameter

**Example Update**:
```rust
// OLD:
pub fn list_len(args: &[Value]) -> Result<Value, VmError> {
    // ...
}

// NEW:
pub fn list_len(_vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    // ...
}
```

**Files to Create/Modify**:
- `rust/crates/fusabi-vm/src/stdlib/list.rs`
- `rust/crates/fusabi-vm/src/stdlib/string.rs`
- `rust/crates/fusabi-vm/src/stdlib/math.rs` (if exists)

**Validation**:
```bash
cd rust/crates/fusabi-vm
cargo test --lib stdlib
# All stdlib tests pass with new signature
```

---

### Task 1.6: Implement List.map as Proof of Concept
**Description**: Implement `List.map` using the new re-entrant capability.

**Deliverables**:
- `pub fn list_map(vm: &mut Vm, args: &[Value]) -> Result<Value, VmError>`
- Takes `[list, fn]` as arguments
- Iterates over list elements
- Calls `vm.call_closure(fn, &[element])` for each element
- Collects results into new list

**Implementation**:
```rust
pub fn list_map(vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::ArityMismatch { expected: 2, got: args.len() });
    }

    let list = match &args[0] {
        Value::List(l) => l,
        _ => return Err(VmError::TypeError("Expected list".into())),
    };

    let func = &args[1];

    let mut mapped = Vec::new();
    for item in list.iter() {
        let result = vm.call_closure(func.clone(), &[item.clone()])?;
        mapped.push(result);
    }

    Ok(Value::List(Rc::new(RefCell::new(mapped))))
}
```

**Register in stdlib**:
```rust
// In stdlib/list.rs
pub fn register_list_functions(registry: &mut HostRegistry) {
    registry.register("List.map", Box::new(list_map));
    // ...
}
```

**Files to Create/Modify**:
- `rust/crates/fusabi-vm/src/stdlib/list.rs`

**Validation**:
```bash
# Create test F# script: examples/test_map.fsx
let nums = [1; 2; 3; 4; 5]
let doubled = List.map (fun x -> x * 2) nums
printfn "%A" doubled
# Should output: [2; 4; 6; 8; 10]

cargo run -- run examples/test_map.fsx
```

---

### Task 1.7: Add Comprehensive Tests
**Description**: Add unit tests and integration tests for re-entrant host functions.

**Deliverables**:
- Unit tests for `Vm::call_closure`
- Unit tests for `List.map`
- Integration tests with F# scripts using `List.map`, `List.filter`, `List.fold`
- Edge case tests (empty list, closure that throws, etc.)

**Files to Create/Modify**:
- `rust/crates/fusabi-vm/tests/host_reentrant.rs`
- `examples/tests/hof_tests.fsx`

**Test Cases**:
1. `List.map` with simple function
2. `List.map` with closure capturing variables
3. `List.map` on empty list
4. `List.map` with function that throws error
5. Nested `List.map` (map inside map)
6. `List.filter` (once implemented)

**Validation**:
```bash
cd rust/crates/fusabi-vm
cargo test
# All tests pass

cd ../..
cargo run -- run examples/tests/hof_tests.fsx
# All F# integration tests pass
```

---

## Definition of Done
- [ ] `HostFn` type refactored to accept VM context
- [ ] `HostRegistry::call` updated to pass VM
- [ ] VM loop updated to pass `self` to host functions
- [ ] `Vm::call_closure` helper API implemented
- [ ] All existing stdlib functions updated to new signature
- [ ] `List.map` implemented and working
- [ ] Comprehensive test suite passing
- [ ] No borrow checker violations
- [ ] No unsafe code (or minimal with safety documentation)
- [ ] Documentation updated (inline docs + comments)
- [ ] PR ready for review

## Agent Coordination Hooks
```bash
# BEFORE Work:
npx claude-flow@alpha hooks pre-task --description "ws1-hof-support"
npx claude-flow@alpha hooks session-restore --session-id "swarm-fusabi-gem-ws1"

# DURING Work:
npx claude-flow@alpha hooks post-edit --file "rust/crates/fusabi-vm/src/host.rs" --memory-key "swarm/fusabi-gem/ws1/hostfn-refactor"
npx claude-flow@alpha hooks post-edit --file "rust/crates/fusabi-vm/src/vm.rs" --memory-key "swarm/fusabi-gem/ws1/vm-update"
npx claude-flow@alpha hooks post-edit --file "rust/crates/fusabi-vm/src/stdlib/list.rs" --memory-key "swarm/fusabi-gem/ws1/list-map"
npx claude-flow@alpha hooks notify --message "List.map implemented and tested"

# AFTER Work:
npx claude-flow@alpha hooks post-task --task-id "ws1-complete"
npx claude-flow@alpha hooks session-end --export-metrics true
```

## Estimated Effort
**Duration**: 3-4 days
**Complexity**: Medium-High

## References
- [Crafting Interpreters - Closures](http://craftinginterpreters.com/closures.html)
- [Rust Book - Ownership & Borrowing](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html)
- [RefCell Interior Mutability](https://doc.rust-lang.org/book/ch15-05-interior-mutability.html)

## Notes
- **Borrow Checker Challenges**: The main challenge is that the VM loop holds a mutable borrow, and host functions need to borrow the VM mutably as well. Solutions:
  1. Use `RefCell<VmState>` for interior mutability
  2. Split VM into separate state and executor structs
  3. Carefully structure code to ensure no overlapping borrows
- **Exception Safety**: Ensure that if a host function panics or returns an error, the VM state remains consistent
- **Performance**: Measure performance impact of additional indirection (should be minimal)
- **Future Work**: Consider implementing `List.filter`, `List.fold`, `List.reduce` after `List.map` proves the pattern

## Conflict Warnings
⚠️ **File Conflicts with WS2**:
- Both WS1 and WS2 modify `fusabi-vm/src/vm.rs`
- **Recommendation**: Complete WS1 first, then start WS2
- **Alternative**: Coordinate closely if running in parallel (one agent focuses on `Instruction::Call` logic, other on GC logic)
