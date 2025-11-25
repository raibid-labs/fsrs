# Issue 61: [VM] Implement Mark-and-Sweep Garbage Collection

**Labels:** `enhancement`, `priority:high`, `area:vm`

## Context
The current memory management model uses `Rc<RefCell<T>>` (reference counting), which is fast but suffers from memory leaks when reference cycles are created (e.g., a recursive record that points to itself, or two closures closing over each other). To support long-running applications and complex data structures, we need a Mark-and-Sweep Garbage Collector (GC) to detect and reclaim these cycles.

## Implementation Plan

### 1. Trace Trait Infrastructure (`fusabi-vm/src/gc.rs`)
*   **Define `Trace` Trait:**
    ```rust
    pub trait Trace {
        fn trace(&self, tracer: &mut Tracer);
    }
    ```
*   **Implement `Trace`:**
    *   Implement for all heap-allocated values: `Value`, `Record`, `Variant`, `Closure`.
    *   Implement for containers: `Vec<T>`, `HashMap<K, V>`.

### 2. GcHeap & Object Header
*   **Object Header:**
    *   Add a `GcHeader` to all manageable objects containing a `mark_bit` (Color: White/Grey/Black).
*   **GcHeap Structure:**
    *   Create a `GcHeap` struct that owns all allocated objects (likely using `Vec<Box<dyn Trace>>` or similar arena-like structure).
    *   Replace direct `Rc::new` calls with `vm.heap.allocate(...)`.

### 3. VM Integration (`fusabi-vm/src/vm.rs`)
*   **Roots:**
    *   The GC must know all "roots" (reachable objects).
    *   Roots include: `vm.stack`, `vm.globals`, `vm.upvalues`, and `vm.frames`.
*   **Allocation & Trigger:**
    *   Check heap size on every allocation.
    *   Trigger `collect_garbage()` if threshold is exceeded (e.g., every 1MB allocated).

### 4. Collection Logic
*   **Phase 1: Mark**
    *   Start from all Roots.
    *   Push to a worklist (grey set).
    *   While worklist not empty: pop, mark black, trace children (push to worklist).
*   **Phase 2: Sweep**
    *   Iterate through all objects in `GcHeap`.
    *   If object is White (unmarked): Drop/Deallocate.
    *   If object is Black (marked): Reset to White for next cycle.

### 5. Testing
*   **Cycle Test:**
    *   Create a recursive record `let rec x = { next = x }`.
    *   Drop reference to `x`.
    *   Force GC.
    *   Assert that `x` is collected (heap size decreases).
