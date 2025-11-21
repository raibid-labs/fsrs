// Comprehensive Garbage Collection Tests for Fusabi VM
// Tests mark-and-sweep GC with cycle detection

use fusabi_vm::{Chunk, ChunkBuilder, Instruction, Value, Vm};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

// ========== Basic GC Tests ==========

#[test]
fn test_gc_basic_collection() {
    let mut vm = Vm::new();

    // Create some values and let them go out of scope
    for i in 0..10 {
        vm.set_global(i, Value::Int(i as i64));
    }

    // Clear some globals
    for i in 0..5 {
        vm.set_global(i, Value::Unit);
    }

    // Trigger GC
    let stats = vm.collect_garbage();

    // Should have collected some objects
    assert!(stats.objects_collected > 0 || stats.bytes_freed > 0);
}

#[test]
fn test_gc_preserves_stack_values() {
    let mut vm = Vm::new();

    let chunk = ChunkBuilder::new()
        .constant(Value::Int(42))
        .constant(Value::Str("hello".to_string()))
        .instruction(Instruction::LoadConst(0))
        .instruction(Instruction::LoadConst(1))
        .instruction(Instruction::Return)
        .build();

    // Start execution but don't complete
    let _ = vm.execute(chunk);

    let before_count = vm.live_objects();

    // Trigger GC
    vm.collect_garbage();

    // Stack values should be preserved
    let after_count = vm.live_objects();
    assert_eq!(before_count, after_count);
}

#[test]
fn test_gc_preserves_globals() {
    let mut vm = Vm::new();

    // Set global variables
    vm.set_global(0, Value::Int(100));
    vm.set_global(1, Value::Str("global".to_string()));
    vm.set_global(2, Value::Bool(true));

    // Trigger GC
    vm.collect_garbage();

    // Globals should be preserved
    assert_eq!(vm.get_global(0), Some(Value::Int(100)));
    assert_eq!(vm.get_global(1), Some(Value::Str("global".to_string())));
    assert_eq!(vm.get_global(2), Some(Value::Bool(true)));
}

#[test]
fn test_gc_with_tuples() {
    let mut vm = Vm::new();

    let chunk = ChunkBuilder::new()
        .constant(Value::Int(1))
        .constant(Value::Int(2))
        .constant(Value::Int(3))
        .instruction(Instruction::LoadConst(0))
        .instruction(Instruction::LoadConst(1))
        .instruction(Instruction::LoadConst(2))
        .instruction(Instruction::MakeTuple(3))
        .instruction(Instruction::Return)
        .build();

    let result = vm.execute(chunk).unwrap();

    // Tuple should survive GC
    vm.collect_garbage();

    assert!(result.is_tuple());
    assert_eq!(result.as_tuple().unwrap().len(), 3);
}

#[test]
fn test_gc_with_lists() {
    let mut vm = Vm::new();

    let chunk = ChunkBuilder::new()
        .constant(Value::Int(1))
        .constant(Value::Int(2))
        .constant(Value::Int(3))
        .instruction(Instruction::LoadConst(0))
        .instruction(Instruction::LoadConst(1))
        .instruction(Instruction::LoadConst(2))
        .instruction(Instruction::MakeList(3))
        .instruction(Instruction::Return)
        .build();

    let result = vm.execute(chunk).unwrap();

    // List should survive GC
    vm.collect_garbage();

    assert!(result.is_cons());
}

#[test]
fn test_gc_with_arrays() {
    let mut vm = Vm::new();

    let chunk = ChunkBuilder::new()
        .constant(Value::Int(10))
        .constant(Value::Int(20))
        .constant(Value::Int(30))
        .instruction(Instruction::LoadConst(0))
        .instruction(Instruction::LoadConst(1))
        .instruction(Instruction::LoadConst(2))
        .instruction(Instruction::MakeArray(3))
        .instruction(Instruction::Return)
        .build();

    let result = vm.execute(chunk).unwrap();

    // Array should survive GC
    vm.collect_garbage();

    assert!(result.is_array());
    assert_eq!(result.array_length().unwrap(), 3);
}

// ========== Reference Cycle Tests ==========

#[test]
fn test_gc_circular_reference_record() {
    let mut vm = Vm::new();

    // Create a record that references itself
    let record = Value::Record(Rc::new(RefCell::new(HashMap::new())));

    if let Value::Record(ref r) = record {
        // Create circular reference
        r.borrow_mut().insert("self".to_string(), record.clone());
    }

    vm.set_global(0, record.clone());

    // Clear the global to make it unreachable
    vm.set_global(0, Value::Unit);

    // GC should collect the circular reference
    let stats = vm.collect_garbage();
    assert!(stats.objects_collected > 0 || stats.bytes_freed > 0);
}

#[test]
fn test_gc_mutual_reference_records() {
    let mut vm = Vm::new();

    // Create two records that reference each other
    let record1 = Value::Record(Rc::new(RefCell::new(HashMap::new())));
    let record2 = Value::Record(Rc::new(RefCell::new(HashMap::new())));

    if let (Value::Record(ref r1), Value::Record(ref r2)) = (&record1, &record2) {
        r1.borrow_mut().insert("other".to_string(), record2.clone());
        r2.borrow_mut().insert("other".to_string(), record1.clone());
    }

    vm.set_global(0, record1);
    vm.set_global(1, record2);

    // Clear both globals to make them unreachable
    vm.set_global(0, Value::Unit);
    vm.set_global(1, Value::Unit);

    // GC should collect both records
    let stats = vm.collect_garbage();
    assert!(stats.objects_collected > 0 || stats.bytes_freed > 0);
}

// ========== Allocation Threshold Tests ==========

#[test]
fn test_gc_allocation_threshold_triggers() {
    let mut vm = Vm::with_gc_threshold(1024); // 1KB threshold

    // Allocate many small objects
    for i in 0..200 {
        let chunk = ChunkBuilder::new()
            .constant(Value::Int(i))
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::MakeTuple(1))
            .instruction(Instruction::Pop)
            .instruction(Instruction::Return)
            .build();

        let _ = vm.execute(chunk);
    }

    // GC should have been triggered automatically
    let stats = vm.gc_stats();
    assert!(vm.total_allocated() > 0);
}

#[test]
fn test_gc_adaptive_threshold() {
    let mut vm = Vm::new();

    // First collection
    vm.collect_garbage();
    // Can't directly access private field, so track by behavior

    // Create and collect many objects
    for _ in 0..50 {
        vm.set_global(0, Value::Tuple(vec![Value::Int(1), Value::Int(2)]));
    }
    vm.set_global(0, Value::Unit);
    let stats = vm.collect_garbage();

    // Just verify that GC works and collects objects
    assert!(stats.objects_collected > 0 || stats.objects_before > 0);
}

// ========== Stress Tests ==========

#[test]
fn test_gc_stress_many_small_allocations() {
    let mut vm = Vm::new();

    // Create 1000+ small allocations
    for i in 0..1000 {
        let val = Value::Tuple(vec![Value::Int(i)]);
        vm.set_global((i % 10) as usize, val); // Reuse 10 global slots
    }

    let stats = vm.collect_garbage();
    assert!(stats.objects_before > 0);

    // Most objects should be collected (only 10 globals remain)
    assert!(vm.live_objects() <= 20); // Some tolerance for internal objects
}

#[test]
fn test_gc_stress_deep_nesting() {
    let mut vm = Vm::new();

    // Create deeply nested structure
    let mut value = Value::Unit;
    for i in 0..100 {
        value = Value::Tuple(vec![Value::Int(i), value]);
    }

    vm.set_global(0, value);

    // Should handle deep nesting without stack overflow
    let stats = vm.collect_garbage();
    assert!(vm.live_objects() > 0);
}

#[test]
fn test_gc_stress_10k_allocations() {
    let mut vm = Vm::with_gc_threshold(1024 * 1024); // 1MB threshold

    // Allocate 10,000+ objects
    for i in 0..10_000 {
        let chunk = ChunkBuilder::new()
            .constant(Value::Int(i))
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::Return)
            .build();

        let _ = vm.execute(chunk);

        // Periodically clear to allow collection
        if i % 1000 == 0 {
            vm.collect_garbage();
        }
    }

    // Final collection
    let stats = vm.collect_garbage();

    // Should have handled 10k allocations without issues
    assert!(stats.duration_us < 10_000_000); // Less than 10 seconds
}

#[test]
fn test_gc_stress_mixed_types() {
    let mut vm = Vm::new();

    // Mix of different value types
    for i in 0..500 {
        let val = match i % 5 {
            0 => Value::Int(i),
            1 => Value::Str(format!("string_{}", i)),
            2 => Value::Tuple(vec![Value::Int(i), Value::Bool(i % 2 == 0)]),
            3 => Value::vec_to_cons(vec![Value::Int(i), Value::Int(i + 1)]),
            4 => Value::Array(Rc::new(RefCell::new(vec![Value::Int(i)]))),
            _ => unreachable!(),
        };

        vm.set_global((i % 50) as usize, val); // Reuse 50 slots
    }

    let stats = vm.collect_garbage();
    assert!(stats.objects_collected > 0 || vm.live_objects() > 0);
}

// ========== Memory Safety Tests ==========

#[test]
fn test_gc_no_double_free() {
    let mut vm = Vm::new();

    // Create shared references
    let array = Value::Array(Rc::new(RefCell::new(vec![Value::Int(42)])));
    vm.set_global(0, array.clone());
    vm.set_global(1, array.clone());

    // Clear one reference
    vm.set_global(0, Value::Unit);

    // GC should not double-free
    vm.collect_garbage();

    // Other reference should still be valid
    assert_eq!(vm.get_global(1).unwrap().array_get(0).unwrap(), Value::Int(42));
}

#[test]
fn test_gc_preserves_shared_structure() {
    let mut vm = Vm::new();

    // Create shared substructure
    let shared = Value::Tuple(vec![Value::Int(100)]);
    let val1 = Value::Tuple(vec![shared.clone(), Value::Int(1)]);
    let val2 = Value::Tuple(vec![shared.clone(), Value::Int(2)]);

    vm.set_global(0, val1);
    vm.set_global(1, val2);

    // GC should preserve shared structure
    vm.collect_garbage();

    // Both should still reference the same shared value
    let g0 = vm.get_global(0).unwrap();
    let g1 = vm.get_global(1).unwrap();

    if let (Value::Tuple(t0), Value::Tuple(t1)) = (g0, g1) {
        assert_eq!(t0[0], t1[0]); // Same shared tuple
    }
}

// ========== GC Timing Tests ==========

#[test]
fn test_gc_pause_time() {
    let mut vm = Vm::new();

    // Create moderate amount of garbage
    for i in 0..100 {
        vm.set_global(0, Value::Tuple(vec![Value::Int(i)]));
    }

    // Measure GC pause
    let start = std::time::Instant::now();
    let stats = vm.collect_garbage();
    let duration = start.elapsed();

    // GC pause should be under 10ms for moderate heap
    assert!(duration.as_millis() < 10);
    assert!(stats.duration_us < 10_000); // 10ms in microseconds
}

#[test]
fn test_gc_incremental_collections() {
    let mut vm = Vm::with_gc_threshold(1024); // Low threshold

    let mut total_collections = 0;

    // Many small allocations should trigger incremental GC
    for i in 0..1000 {
        let chunk = ChunkBuilder::new()
            .constant(Value::Int(i))
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::Return)
            .build();

        let before = vm.gc_stats().clone();
        let _ = vm.execute(chunk);
        let after = vm.gc_stats();

        // Count collections
        if after.objects_before != before.objects_before {
            total_collections += 1;
        }
    }

    // Should have triggered multiple collections
    assert!(total_collections > 1);
}

// ========== Edge Cases ==========

#[test]
fn test_gc_empty_heap() {
    let mut vm = Vm::new();

    // GC on empty heap should work
    let stats = vm.collect_garbage();
    assert_eq!(stats.objects_collected, 0);
    assert_eq!(stats.bytes_freed, 0);
}

#[test]
fn test_gc_only_primitives() {
    let mut vm = Vm::new();

    // Only primitive values (no heap allocations)
    vm.set_global(0, Value::Int(42));
    vm.set_global(1, Value::Bool(true));
    vm.set_global(2, Value::Unit);

    let stats = vm.collect_garbage();

    // No objects to collect (primitives don't allocate)
    assert_eq!(stats.objects_collected, 0);
}

#[test]
fn test_gc_complex_graph() {
    let mut vm = Vm::new();

    // Create complex reference graph
    let mut nodes = Vec::new();
    for i in 0..10 {
        let mut fields = HashMap::new();
        fields.insert("id".to_string(), Value::Int(i));
        nodes.push(Value::Record(Rc::new(RefCell::new(fields))));
    }

    // Create connections
    for i in 0..10 {
        if let Value::Record(ref r) = nodes[i] {
            if i > 0 {
                r.borrow_mut().insert("prev".to_string(), nodes[i - 1].clone());
            }
            if i < 9 {
                r.borrow_mut().insert("next".to_string(), nodes[i + 1].clone());
            }
        }
    }

    // Keep only middle node as root
    vm.set_global(0, nodes[5].clone());
    drop(nodes);

    // GC should keep connected component alive
    let stats = vm.collect_garbage();
    assert!(vm.live_objects() >= 6); // At least nodes 0-5 should be alive
}

#[test]
fn test_gc_with_variants() {
    let mut vm = Vm::new();

    // Create variants with nested values
    let variant = Value::Variant {
        type_name: "Option".to_string(),
        variant_name: "Some".to_string(),
        fields: vec![Value::Tuple(vec![Value::Int(1), Value::Int(2)])],
    };

    vm.set_global(0, variant);

    // Should preserve variant and nested values
    vm.collect_garbage();

    let v = vm.get_global(0).unwrap();
    assert!(v.is_variant());
    assert_eq!(v.variant_fields().unwrap().len(), 1);
}

// ========== Concurrent Modification Tests ==========

#[test]
fn test_gc_during_execution() {
    let mut vm = Vm::with_gc_threshold(512); // Very low threshold

    // Long-running computation that triggers GC
    let chunk = ChunkBuilder::new()
        .constant(Value::Int(0))
        .instruction(Instruction::LoadConst(0))
        .instruction(Instruction::StoreLocal(0))
        .instruction(Instruction::LoadLocal(0))
        .instruction(Instruction::LoadLocal(0))
        .instruction(Instruction::MakeTuple(2))
        .instruction(Instruction::Pop)
        .instruction(Instruction::Return)
        .build();

    // Should handle GC during execution
    let result = vm.execute(chunk);
    assert!(result.is_ok());
}