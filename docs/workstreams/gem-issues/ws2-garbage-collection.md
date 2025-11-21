# Workstream 2: VM Core - Garbage Collection

## Status
🔴 Blocked by WS1 (or can run carefully in parallel with coordination)

## Overview
Implement mark-and-sweep garbage collection to reclaim memory from reference cycles that `Rc<RefCell<T>>` cannot handle (e.g., recursive records, circular closures). This ensures Fusabi doesn't leak memory in long-running applications.

## Objectives
- [ ] Create `Trace` trait for traversing object graphs
- [ ] Implement `Trace` for all `Value` variants
- [ ] Create `GcHeap` allocator to replace direct `Rc::new`
- [ ] Add GC roots tracking in VM (stack, globals, upvalues)
- [ ] Implement `Vm::collect_garbage()` with mark and sweep phases
- [ ] Add object header with color bit for marking
- [ ] Integrate GC trigger logic (alloc threshold, periodic)

## Agent Assignment
**Suggested Agent Type**: `backend-architect`, `rust-pro`, `coder`
**Skill Requirements**: Rust unsafe, memory management, GC algorithms, systems programming

## Dependencies
- **WS1 (HOF Support)**: Recommended to complete first to avoid conflicts in `vm.rs`
- **Alternative**: Can run in parallel with careful git coordination

## Tasks

### Task 2.1: Define Trace Trait
**Description**: Create a `Trace` trait for traversing object references during mark phase.

**Deliverables**:
- `trait Trace` with `fn trace(&self, tracer: &mut Tracer)`
- `struct Tracer` with `fn mark(&mut self, value: &Value)`
- Design for traversing nested structures

**Files to Create/Modify**:
- `rust/crates/fusabi-vm/src/gc.rs` (new file)

**Implementation**:
```rust
// fusabi-vm/src/gc.rs

/// Trait for types that can participate in garbage collection.
pub trait Trace {
    /// Trace all references held by this value.
    fn trace(&self, tracer: &mut Tracer);
}

/// Tracer used during the mark phase of GC.
pub struct Tracer {
    marked: HashSet<*const GcBox<dyn Trace>>,
}

impl Tracer {
    pub fn new() -> Self {
        Self {
            marked: HashSet::new(),
        }
    }

    /// Mark a value as reachable.
    pub fn mark<T: Trace>(&mut self, ptr: *const GcBox<T>) {
        if self.marked.insert(ptr as *const GcBox<dyn Trace>) {
            // First time seeing this object, trace its children
            unsafe { (*ptr).trace(self); }
        }
    }
}
```

**Validation**:
```bash
cd rust/crates/fusabi-vm
cargo build
# Should compile without errors
```

---

### Task 2.2: Implement Trace for Value Types
**Description**: Implement `Trace` for `Value`, `Record`, `Variant`, `Closure`, and `Vec<Value>`.

**Deliverables**:
- `impl Trace for Value`
- `impl Trace for Record`
- `impl Trace for Variant`
- `impl Trace for Closure`
- `impl Trace for Vec<Value>`

**Files to Create/Modify**:
- `rust/crates/fusabi-vm/src/gc.rs`
- `rust/crates/fusabi-vm/src/value.rs` (add Trace impls)

**Implementation**:
```rust
impl Trace for Value {
    fn trace(&self, tracer: &mut Tracer) {
        match self {
            Value::Record(r) => tracer.mark(r.as_ptr()),
            Value::Variant(v) => tracer.mark(v.as_ptr()),
            Value::Closure(c) => tracer.mark(c.as_ptr()),
            Value::List(l) => {
                for item in l.borrow().iter() {
                    item.trace(tracer);
                }
            }
            Value::Array(a) => {
                for item in a.borrow().iter() {
                    item.trace(tracer);
                }
            }
            // Primitive types don't need tracing
            Value::Int(_) | Value::Float(_) | Value::Bool(_)
            | Value::String(_) | Value::Unit => {}
        }
    }
}

impl Trace for Record {
    fn trace(&self, tracer: &mut Tracer) {
        for (_, value) in self.fields.iter() {
            value.trace(tracer);
        }
    }
}

impl Trace for Closure {
    fn trace(&self, tracer: &mut Tracer) {
        for upvalue in self.upvalues.iter() {
            upvalue.trace(tracer);
        }
    }
}
```

**Validation**:
```rust
#[test]
fn test_trace_simple_value() {
    let mut tracer = Tracer::new();
    let val = Value::Int(42);
    val.trace(&mut tracer);
    // Should not panic
}

#[test]
fn test_trace_nested_record() {
    let mut tracer = Tracer::new();
    let record = Value::Record(/* nested record */);
    record.trace(&mut tracer);
    // Should trace all nested values
}
```

---

### Task 2.3: Create GcHeap Allocator
**Description**: Create a central `GcHeap` to manage all GC-managed allocations, replacing direct `Rc::new`.

**Deliverables**:
- `struct GcHeap` with allocation tracking
- `GcHeap::alloc<T: Trace>(value: T) -> Gc<T>`
- Object header with mark bit
- List of all allocated objects for sweep phase

**Files to Create/Modify**:
- `rust/crates/fusabi-vm/src/gc.rs`

**Implementation**:
```rust
/// GC-managed pointer (replaces Rc for GC-managed objects)
pub struct Gc<T: ?Sized> {
    ptr: NonNull<GcBox<T>>,
}

/// Header for GC-managed objects
struct GcBox<T: ?Sized> {
    header: GcHeader,
    data: T,
}

struct GcHeader {
    marked: Cell<bool>,
}

/// Global heap for GC-managed objects
pub struct GcHeap {
    objects: Vec<*mut GcBox<dyn Trace>>,
    bytes_allocated: usize,
    threshold: usize,
}

impl GcHeap {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            bytes_allocated: 0,
            threshold: 1024 * 1024, // 1MB initial threshold
        }
    }

    pub fn alloc<T: Trace + 'static>(&mut self, value: T) -> Gc<T> {
        let size = std::mem::size_of::<GcBox<T>>();
        self.bytes_allocated += size;

        let boxed = Box::new(GcBox {
            header: GcHeader {
                marked: Cell::new(false),
            },
            data: value,
        });

        let ptr = Box::into_raw(boxed);
        self.objects.push(ptr as *mut GcBox<dyn Trace>);

        Gc {
            ptr: unsafe { NonNull::new_unchecked(ptr) },
        }
    }

    pub fn should_collect(&self) -> bool {
        self.bytes_allocated > self.threshold
    }
}
```

**Validation**:
```rust
#[test]
fn test_gc_heap_alloc() {
    let mut heap = GcHeap::new();
    let val = heap.alloc(42);
    assert_eq!(*val, 42);
}
```

---

### Task 2.4: Integrate GC Roots in VM
**Description**: Add GC roots tracking to VM (stack, globals, upvalues).

**Deliverables**:
- Add `gc_heap: GcHeap` to `Vm` struct
- Ensure stack, globals, and upvalues are treated as roots
- Method to collect all roots: `fn collect_roots(&self) -> Vec<Value>`

**Files to Create/Modify**:
- `rust/crates/fusabi-vm/src/vm.rs`

**Implementation**:
```rust
pub struct Vm {
    // ... existing fields
    pub(crate) gc_heap: GcHeap,
}

impl Vm {
    fn collect_roots(&self) -> Vec<&Value> {
        let mut roots = Vec::new();

        // Stack is always a root
        for value in &self.stack {
            roots.push(value);
        }

        // Globals are roots
        for (_, value) in &self.globals {
            roots.push(value);
        }

        // Open upvalues are roots
        for upvalue in &self.open_upvalues {
            roots.push(&upvalue.value);
        }

        roots
    }
}
```

**Validation**:
```rust
#[test]
fn test_collect_roots() {
    let mut vm = Vm::new();
    vm.stack.push(Value::Int(42));
    vm.globals.insert("x".into(), Value::Int(99));

    let roots = vm.collect_roots();
    assert_eq!(roots.len(), 2);
}
```

---

### Task 2.5: Implement Mark Phase
**Description**: Implement the mark phase of GC that traverses from roots and marks reachable objects.

**Deliverables**:
- `fn mark(&mut self, roots: &[&Value])`
- Traverse all roots using `Trace` trait
- Set mark bit on reachable objects

**Files to Create/Modify**:
- `rust/crates/fusabi-vm/src/gc.rs`

**Implementation**:
```rust
impl GcHeap {
    fn mark(&mut self, roots: &[&Value]) {
        let mut tracer = Tracer::new();

        // Mark all roots
        for root in roots {
            root.trace(&mut tracer);
        }

        // tracer.marked now contains all reachable objects
    }
}
```

**Validation**:
```rust
#[test]
fn test_mark_phase() {
    let mut heap = GcHeap::new();
    let val1 = heap.alloc(42);
    let val2 = heap.alloc(99);

    let roots = vec![&Value::from(val1)];
    heap.mark(&roots);

    // val1 should be marked, val2 should not
}
```

---

### Task 2.6: Implement Sweep Phase
**Description**: Implement the sweep phase that deallocates unmarked objects.

**Deliverables**:
- `fn sweep(&mut self)`
- Iterate all allocated objects
- Deallocate objects without mark bit
- Reset mark bits on surviving objects

**Files to Create/Modify**:
- `rust/crates/fusabi-vm/src/gc.rs`

**Implementation**:
```rust
impl GcHeap {
    fn sweep(&mut self) {
        self.objects.retain(|&ptr| {
            unsafe {
                let obj = &*ptr;
                if obj.header.marked.get() {
                    // Object is reachable, keep it and reset mark
                    obj.header.marked.set(false);
                    true
                } else {
                    // Object is unreachable, deallocate it
                    drop(Box::from_raw(ptr));
                    self.bytes_allocated -= std::mem::size_of_val(&*ptr);
                    false
                }
            }
        });
    }
}
```

**Validation**:
```rust
#[test]
fn test_sweep_phase() {
    let mut heap = GcHeap::new();
    let val1 = heap.alloc(42); // Reachable
    let val2 = heap.alloc(99); // Unreachable

    let roots = vec![&Value::from(val1)];
    heap.mark(&roots);

    let before = heap.objects.len();
    heap.sweep();
    let after = heap.objects.len();

    assert_eq!(before, 2);
    assert_eq!(after, 1); // Only val1 survives
}
```

---

### Task 2.7: Implement Vm::collect_garbage
**Description**: Combine mark and sweep into `Vm::collect_garbage()` method.

**Deliverables**:
- `pub fn collect_garbage(&mut self)`
- Call mark phase with roots
- Call sweep phase
- Log GC stats (objects collected, bytes freed)

**Files to Create/Modify**:
- `rust/crates/fusabi-vm/src/vm.rs`

**Implementation**:
```rust
impl Vm {
    pub fn collect_garbage(&mut self) {
        let roots = self.collect_roots();

        let before = self.gc_heap.bytes_allocated;

        self.gc_heap.mark(&roots);
        self.gc_heap.sweep();

        let after = self.gc_heap.bytes_allocated;
        let freed = before - after;

        log::debug!(
            "GC collected {} bytes ({} -> {} bytes allocated)",
            freed,
            before,
            after
        );
    }
}
```

**Validation**:
```rust
#[test]
fn test_collect_garbage() {
    let mut vm = Vm::new();

    // Create some garbage
    for i in 0..100 {
        let _ = vm.gc_heap.alloc(i);
    }

    // Only keep one value on stack
    vm.stack.push(Value::Int(42));

    let before = vm.gc_heap.objects.len();
    vm.collect_garbage();
    let after = vm.gc_heap.objects.len();

    assert!(after < before); // Should have collected garbage
}
```

---

### Task 2.8: Add GC Trigger Logic
**Description**: Add logic to trigger GC automatically based on allocation threshold.

**Deliverables**:
- Check `gc_heap.should_collect()` after allocations
- Call `collect_garbage()` when threshold exceeded
- Adjust threshold after GC (e.g., 2x current allocation)
- Configuration option to disable/tune GC

**Files to Create/Modify**:
- `rust/crates/fusabi-vm/src/vm.rs`
- `rust/crates/fusabi-vm/src/gc.rs`

**Implementation**:
```rust
impl Vm {
    fn maybe_collect(&mut self) {
        if self.gc_heap.should_collect() {
            self.collect_garbage();

            // Adjust threshold: 2x current allocation or min 1MB
            let new_threshold = (self.gc_heap.bytes_allocated * 2).max(1024 * 1024);
            self.gc_heap.threshold = new_threshold;
        }
    }
}

// Call after allocations:
impl Value {
    pub fn new_record(fields: HashMap<String, Value>, heap: &mut GcHeap) -> Self {
        let record = heap.alloc(Record { fields });
        Value::Record(record)
    }
}
```

**Validation**:
```rust
#[test]
fn test_gc_auto_trigger() {
    let mut vm = Vm::new();
    vm.gc_heap.threshold = 100; // Small threshold for testing

    // Allocate until GC triggers
    for i in 0..1000 {
        let _ = vm.gc_heap.alloc(i);
        // GC should trigger automatically
    }

    // Should have collected garbage multiple times
}
```

---

### Task 2.9: Add Comprehensive Tests
**Description**: Add unit tests and stress tests for GC.

**Deliverables**:
- Unit tests for mark, sweep, full GC
- Test for cyclic references (the main GC use case)
- Stress test with large allocations
- Test for GC during VM execution

**Files to Create/Modify**:
- `rust/crates/fusabi-vm/tests/gc_tests.rs`
- `examples/tests/gc_stress.fsx`

**Test Cases**:
1. Simple GC: allocate, make garbage, collect
2. Cyclic references: create cycle, collect
3. Deep nesting: nested records, collect
4. Large allocations: stress test
5. GC during execution: run F# script with GC

**Validation**:
```bash
cd rust/crates/fusabi-vm
cargo test --test gc_tests
# All GC tests pass

cargo run -- run examples/tests/gc_stress.fsx
# Script completes without memory leaks
```

---

## Definition of Done
- [ ] `Trace` trait defined and implemented for all Value types
- [ ] `GcHeap` allocator replaces direct `Rc::new`
- [ ] VM tracks GC roots (stack, globals, upvalues)
- [ ] Mark phase implemented and tested
- [ ] Sweep phase implemented and tested
- [ ] `Vm::collect_garbage()` combines mark + sweep
- [ ] Automatic GC triggering based on threshold
- [ ] Comprehensive test suite passing
- [ ] Cyclic reference test proves GC works
- [ ] No memory leaks in stress tests
- [ ] Documentation updated
- [ ] PR ready for review

## Agent Coordination Hooks
```bash
# BEFORE Work:
npx claude-flow@alpha hooks pre-task --description "ws2-garbage-collection"
npx claude-flow@alpha hooks session-restore --session-id "swarm-fusabi-gem-ws2"

# DURING Work:
npx claude-flow@alpha hooks post-edit --file "rust/crates/fusabi-vm/src/gc.rs" --memory-key "swarm/fusabi-gem/ws2/gc-implementation"
npx claude-flow@alpha hooks post-edit --file "rust/crates/fusabi-vm/src/vm.rs" --memory-key "swarm/fusabi-gem/ws2/vm-integration"
npx claude-flow@alpha hooks notify --message "GC mark-and-sweep complete"

# AFTER Work:
npx claude-flow@alpha hooks post-task --task-id "ws2-complete"
npx claude-flow@alpha hooks session-end --export-metrics true
```

## Estimated Effort
**Duration**: 4-5 days
**Complexity**: High

## References
- [Baby's First Garbage Collector](http://journal.stuffwithstuff.com/2013/12/08/babys-first-garbage-collector/)
- [Crafting Interpreters - Garbage Collection](http://craftinginterpreters.com/garbage-collection.html)
- [Rust Book - Unsafe](https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html)

## Notes
- **Unsafe Required**: GC implementation requires unsafe code for pointer manipulation
- **Safety**: Carefully document all unsafe blocks with safety invariants
- **Performance**: GC adds overhead, but prevents memory leaks in long-running apps
- **Alternative**: Could use a conservative GC library (e.g., `gc` crate) but custom GC gives more control
- **Future Work**:
  - Incremental GC (avoid stop-the-world pauses)
  - Generational GC (optimize for short-lived objects)
  - Concurrent GC (parallel marking)

## Conflict Warnings
⚠️ **File Conflicts with WS1**:
- Both WS1 and WS2 modify `fusabi-vm/src/vm.rs`
- **Recommendation**: Complete WS1 first, then start WS2
- **Alternative**: Coordinate closely if running in parallel
  - WS1 agent focuses on `Instruction::Call` and `HostFn` logic
  - WS2 agent focuses on `GcHeap` integration and `collect_garbage` method
  - Merge WS1 first, then WS2 rebases on updated main
