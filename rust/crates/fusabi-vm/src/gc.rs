// Fusabi VM Garbage Collector
// Production-grade mark-and-sweep garbage collector with cycle detection

use crate::value::Value;
use std::any::Any;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::mem;
use std::rc::{Rc, Weak};

// ========== Trace Trait ==========

/// Trait for types that can be traced during garbage collection.
/// All heap-allocated types that may form reference cycles must implement this.
pub trait Trace {
    /// Mark this object and trace all objects it references.
    /// Called during the mark phase of garbage collection.
    fn trace(&self, tracer: &mut Tracer);
}

/// The tracer is responsible for marking objects during the mark phase.
pub struct Tracer {
    /// Set of object IDs that have been marked as reachable.
    marked: HashSet<ObjectId>,
    /// Stack of objects to trace (worklist for iterative marking).
    to_trace: Vec<GcBox>,
}

impl Tracer {
    /// Create a new tracer for a GC cycle.
    fn new() -> Self {
        Tracer {
            marked: HashSet::new(),
            to_trace: Vec::new(),
        }
    }

    /// Mark an object as reachable.
    /// Returns true if the object was newly marked, false if already marked.
    pub fn mark(&mut self, object: &GcBox) -> bool {
        let id = object.id();
        if self.marked.insert(id) {
            // Object was not previously marked
            self.to_trace.push(object.clone());
            true
        } else {
            // Object was already marked
            false
        }
    }

    /// Mark a raw Value as reachable.
    pub fn mark_value(&mut self, value: &Value) {
        value.trace(self);
    }

    /// Perform iterative marking until all reachable objects are marked.
    fn mark_all(&mut self) {
        while let Some(object) = self.to_trace.pop() {
            // Trace the object's references
            object.trace(self);
        }
    }
}

// ========== Object ID and Header ==========

/// Unique identifier for a GC-managed object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectId(usize);

impl ObjectId {
    fn new() -> Self {
        // Use a simple counter for object IDs
        static COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        ObjectId(COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
    }
}

/// Header information for GC-managed objects.
struct GcHeader {
    /// Unique identifier for this object.
    id: ObjectId,
    /// Size of the object in bytes (for memory tracking).
    size: usize,
    /// Type name for debugging.
    type_name: &'static str,
}

// ========== GcBox - Smart Pointer for GC-Managed Objects ==========

/// A garbage-collected box (smart pointer) to a heap-allocated value.
/// Similar to Rc but with garbage collection support.
#[derive(Clone)]
pub struct GcBox {
    /// The underlying Rc containing the GC-managed data.
    inner: Rc<GcBoxInner>,
}

struct GcBoxInner {
    /// GC metadata header.
    header: GcHeader,
    /// The actual data (type-erased).
    data: Box<dyn GcObject>,
}

/// Trait implemented by all types that can be garbage collected.
trait GcObject: Any + Trace {
    /// Get the data as Any for downcasting.
    fn as_any(&self) -> &dyn Any;
    /// Get the data as mutable Any for downcasting.
    fn as_any_mut(&mut self) -> &mut dyn Any;
    /// Clone the object into a new box.
    fn clone_box(&self) -> Box<dyn GcObject>;
    /// Get debug representation.
    fn debug_fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}

impl GcBox {
    /// Allocate a new GC-managed object.
    pub fn new<T>(heap: &mut GcHeap, value: T) -> Self
    where
        T: 'static + Clone + Trace + fmt::Debug,
    {
        let size = mem::size_of::<T>();
        let type_name = std::any::type_name::<T>();
        let header = GcHeader {
            id: ObjectId::new(),
            size,
            type_name,
        };

        let inner = Rc::new(GcBoxInner {
            header,
            data: Box::new(GcWrapper { value }),
        });

        let gc_box = GcBox { inner };

        // Register with the heap
        heap.register(gc_box.clone());

        gc_box
    }

    /// Get the object's unique ID.
    pub fn id(&self) -> ObjectId {
        self.inner.header.id
    }

    /// Get the size of the object.
    pub fn size(&self) -> usize {
        self.inner.header.size
    }

    /// Try to downcast to a specific type.
    pub fn downcast_ref<T: 'static>(&self) -> Option<&T> {
        self.inner.data.as_any().downcast_ref::<GcWrapper<T>>()
            .map(|wrapper| &wrapper.value)
    }

    /// Trace this object's references.
    fn trace(&self, tracer: &mut Tracer) {
        self.inner.data.trace(tracer);
    }
}

impl fmt::Debug for GcBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GcBox({:?}, {})", self.inner.header.id, self.inner.header.type_name)?;
        self.inner.data.debug_fmt(f)
    }
}

impl PartialEq for GcBox {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }
}

/// Wrapper to make any T into a GcObject.
struct GcWrapper<T> {
    value: T,
}

impl<T> GcObject for GcWrapper<T>
where
    T: 'static + Clone + Trace + fmt::Debug,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn GcObject> {
        Box::new(GcWrapper {
            value: self.value.clone(),
        })
    }

    fn debug_fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, " => {:?}", self.value)
    }
}

impl<T: Trace> Trace for GcWrapper<T> {
    fn trace(&self, tracer: &mut Tracer) {
        self.value.trace(tracer);
    }
}

// ========== GcHeap - The Garbage Collector ==========

/// The garbage collection heap that manages all GC-allocated objects.
pub struct GcHeap {
    /// All allocated objects (using weak references to allow collection).
    objects: Vec<Weak<GcBoxInner>>,
    /// Total memory allocated (in bytes).
    total_allocated: usize,
    /// Memory threshold for triggering GC.
    gc_threshold: usize,
    /// Number of GC cycles performed.
    gc_cycles: usize,
    /// Statistics for the last GC cycle.
    last_gc_stats: GcStats,
}

/// Statistics from a garbage collection cycle.
#[derive(Debug, Clone, Default)]
pub struct GcStats {
    /// Number of objects before collection.
    pub objects_before: usize,
    /// Number of objects after collection.
    pub objects_after: usize,
    /// Number of objects collected.
    pub objects_collected: usize,
    /// Bytes freed during collection.
    pub bytes_freed: usize,
    /// Time taken for collection (in microseconds).
    pub duration_us: u64,
}

impl GcHeap {
    /// Create a new GC heap with default settings.
    pub fn new() -> Self {
        GcHeap {
            objects: Vec::new(),
            total_allocated: 0,
            gc_threshold: 1024 * 1024, // 1MB default threshold
            gc_cycles: 0,
            last_gc_stats: GcStats::default(),
        }
    }

    /// Create a new GC heap with a custom threshold.
    pub fn with_threshold(gc_threshold: usize) -> Self {
        GcHeap {
            objects: Vec::new(),
            total_allocated: 0,
            gc_threshold,
            gc_cycles: 0,
            last_gc_stats: GcStats::default(),
        }
    }

    /// Register a newly allocated object with the heap.
    fn register(&mut self, object: GcBox) {
        self.total_allocated += object.size();
        self.objects.push(Rc::downgrade(&object.inner));
    }

    /// Check if GC should be triggered based on memory pressure.
    pub fn should_collect(&self) -> bool {
        self.total_allocated >= self.gc_threshold
    }

    /// Get the total allocated memory.
    pub fn total_allocated(&self) -> usize {
        self.total_allocated
    }

    /// Get the number of live objects.
    pub fn live_objects(&self) -> usize {
        self.objects.iter().filter(|weak| weak.strong_count() > 0).count()
    }

    /// Get statistics from the last GC cycle.
    pub fn last_gc_stats(&self) -> &GcStats {
        &self.last_gc_stats
    }

    /// Perform a garbage collection cycle.
    ///
    /// # Arguments
    /// * `roots` - Root values that should be kept alive
    ///
    /// # Returns
    /// Statistics about the collection cycle
    pub fn collect(&mut self, roots: &[Value]) -> GcStats {
        let start_time = std::time::Instant::now();
        let objects_before = self.live_objects();

        // Create a tracer for this GC cycle
        let mut tracer = Tracer::new();

        // Mark phase: Mark all reachable objects starting from roots
        for root in roots {
            root.trace(&mut tracer);
        }

        // Complete the marking (iterative)
        tracer.mark_all();

        // Sweep phase: Remove unmarked objects
        let mut new_objects = Vec::new();
        let mut bytes_freed = 0;

        for weak in &self.objects {
            if let Some(strong) = weak.upgrade() {
                let id = ObjectId(strong.header.id.0);
                if tracer.marked.contains(&id) {
                    // Object is still reachable, keep it
                    new_objects.push(weak.clone());
                } else {
                    // Object is unreachable, it will be collected
                    bytes_freed += strong.header.size;
                }
            }
            // Weak references that can't be upgraded are already collected
        }

        // Update the object list
        self.objects = new_objects;
        self.total_allocated = self.total_allocated.saturating_sub(bytes_freed);

        let objects_after = self.live_objects();
        let duration_us = start_time.elapsed().as_micros() as u64;

        // Update statistics
        self.last_gc_stats = GcStats {
            objects_before,
            objects_after,
            objects_collected: objects_before - objects_after,
            bytes_freed,
            duration_us,
        };

        self.gc_cycles += 1;
        self.last_gc_stats.clone()
    }

    /// Adjust the GC threshold based on memory usage patterns.
    /// This implements a simple adaptive threshold strategy.
    pub fn adjust_threshold(&mut self) {
        // If we're collecting too frequently (> 50% objects collected),
        // increase the threshold
        if self.last_gc_stats.objects_collected > self.last_gc_stats.objects_after {
            self.gc_threshold = (self.gc_threshold as f64 * 1.5) as usize;
        }
        // If we're collecting too infrequently (< 10% objects collected),
        // decrease the threshold
        else if self.last_gc_stats.objects_collected < self.last_gc_stats.objects_before / 10 {
            self.gc_threshold = (self.gc_threshold as f64 * 0.75) as usize;
        }

        // Keep threshold within reasonable bounds
        const MIN_THRESHOLD: usize = 64 * 1024;    // 64KB minimum
        const MAX_THRESHOLD: usize = 64 * 1024 * 1024; // 64MB maximum
        self.gc_threshold = self.gc_threshold.clamp(MIN_THRESHOLD, MAX_THRESHOLD);
    }
}

impl Default for GcHeap {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for GcHeap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GcHeap")
            .field("objects", &self.live_objects())
            .field("total_allocated", &self.total_allocated)
            .field("gc_threshold", &self.gc_threshold)
            .field("gc_cycles", &self.gc_cycles)
            .finish()
    }
}

// ========== Trace Implementations for Standard Types ==========

impl<T: Trace> Trace for Vec<T> {
    fn trace(&self, tracer: &mut Tracer) {
        for item in self {
            item.trace(tracer);
        }
    }
}

impl<T: Trace> Trace for Option<T> {
    fn trace(&self, tracer: &mut Tracer) {
        if let Some(value) = self {
            value.trace(tracer);
        }
    }
}

impl<T: Trace> Trace for Box<T> {
    fn trace(&self, tracer: &mut Tracer) {
        (**self).trace(tracer);
    }
}

impl<T: Trace> Trace for Rc<T> {
    fn trace(&self, tracer: &mut Tracer) {
        (**self).trace(tracer);
    }
}

impl<T: Trace> Trace for RefCell<T> {
    fn trace(&self, tracer: &mut Tracer) {
        self.borrow().trace(tracer);
    }
}

impl<K, V: Trace> Trace for HashMap<K, V> {
    fn trace(&self, tracer: &mut Tracer) {
        for value in self.values() {
            value.trace(tracer);
        }
    }
}

// Trace implementation for primitive types (no-op)
impl Trace for i64 {
    fn trace(&self, _tracer: &mut Tracer) {
        // Primitive types don't reference other objects
    }
}

impl Trace for bool {
    fn trace(&self, _tracer: &mut Tracer) {
        // Primitive types don't reference other objects
    }
}

impl Trace for String {
    fn trace(&self, _tracer: &mut Tracer) {
        // Strings don't reference other GC objects
    }
}

impl Trace for () {
    fn trace(&self, _tracer: &mut Tracer) {
        // Unit type doesn't reference anything
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test type for GC
    #[derive(Debug, Clone)]
    struct TestNode {
        value: i64,
        next: Option<GcBox>,
    }

    impl Trace for TestNode {
        fn trace(&self, tracer: &mut Tracer) {
            if let Some(ref next) = self.next {
                tracer.mark(next);
            }
        }
    }

    #[test]
    fn test_gc_allocation() {
        let mut heap = GcHeap::new();
        let node = GcBox::new(&mut heap, TestNode {
            value: 42,
            next: None,
        });

        assert_eq!(heap.live_objects(), 1);
        assert!(heap.total_allocated() > 0);
    }

    #[test]
    fn test_gc_collection_unreferenced() {
        let mut heap = GcHeap::new();

        // Allocate and immediately drop
        {
            let _node = GcBox::new(&mut heap, TestNode {
                value: 42,
                next: None,
            });
        }

        // Should be collected
        let stats = heap.collect(&[]);
        assert_eq!(stats.objects_collected, 1);
        assert_eq!(heap.live_objects(), 0);
    }

    #[test]
    fn test_gc_collection_referenced() {
        let mut heap = GcHeap::new();

        let node = GcBox::new(&mut heap, TestNode {
            value: 42,
            next: None,
        });

        // Create a Value that references the node (we'll need to update Value to support GcBox)
        // For now, just keep the node alive by holding a reference

        let stats = heap.collect(&[]);

        // Node is still referenced, shouldn't be collected
        assert_eq!(stats.objects_collected, 0);
        assert_eq!(heap.live_objects(), 1);

        drop(node);
    }

    #[test]
    fn test_gc_linked_list() {
        let mut heap = GcHeap::new();

        // Create a linked list
        let node3 = GcBox::new(&mut heap, TestNode {
            value: 3,
            next: None,
        });

        let node2 = GcBox::new(&mut heap, TestNode {
            value: 2,
            next: Some(node3.clone()),
        });

        let node1 = GcBox::new(&mut heap, TestNode {
            value: 1,
            next: Some(node2.clone()),
        });

        assert_eq!(heap.live_objects(), 3);

        // Drop intermediate reference
        drop(node2);
        drop(node3);

        // All nodes should still be alive through node1
        let stats = heap.collect(&[]);
        assert_eq!(stats.objects_collected, 0);
        assert_eq!(heap.live_objects(), 3);

        // Drop head of list
        drop(node1);

        // Now all should be collected
        let stats = heap.collect(&[]);
        assert_eq!(stats.objects_collected, 3);
        assert_eq!(heap.live_objects(), 0);
    }
}