# Issue 2: [VM] Implement Mark-and-Sweep Garbage Collection

**Labels:** `enhancement`, `priority:high`, `area:vm`

## Context
The current `Rc<RefCell<T>>` model leaks memory if reference cycles are created (e.g., recursive records). We need a mechanism to reclaim these cycles.

## Implementation Plan
**Objective:** Implement a basic Mark-and-Sweep GC to run alongside Rc.

1.  **Trace Trait** (`fusabi-vm/src/gc.rs`):
    * Create `trait Trace { fn trace(&self, tracer: &mut Tracer); }`.
    * Implement `Trace` for `Value`, `Record`, `Variant`, `Closure`, `Vec<Value>`.

2.  **VM Integration** (`fusabi-vm/src/vm.rs`):
    * Add `gc_roots: Vec<Value>` to `Vm`.
    * Ensure the `stack`, `globals`, and `upvalues` are treated as roots.

3.  **Allocator** (`fusabi-vm/src/value.rs`):
    * Instead of `Rc::new`, use a central `GcHeap` struct in the VM to allocate objects.
    * Objects should implement a header (color bit for marking).

4.  **Collection Cycle**:
    * Implement `Vm::collect_garbage()`:
        * **Mark:** Traverse all roots, set "Marked" bit.
        * **Sweep:** Iterate all allocated objects in the `GcHeap`. If not marked, drop/deallocate. If marked, reset bit.
