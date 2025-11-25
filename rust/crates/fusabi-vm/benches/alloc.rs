// Benchmarks for memory allocation performance
// Tests creation of various data structures

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fusabi_vm::{Chunk, ChunkBuilder, Instruction, Value, Vm};

fn bench_record_allocation(c: &mut Criterion) {
    c.bench_function("alloc/records_10k", |b| {
        let mut builder = ChunkBuilder::new("record_alloc_benchmark");

        let name_key = builder.add_constant(Value::String("name".into()));
        let age_key = builder.add_constant(Value::String("age".into()));
        let name_val = builder.add_constant(Value::String("Alice".into()));
        let age_val = builder.add_constant(Value::Int(30));

        // Create 10,000 records
        for _ in 0..10_000 {
            // Push field-value pairs
            builder.add_instruction(Instruction::LoadConst(name_key));
            builder.add_instruction(Instruction::LoadConst(name_val));
            builder.add_instruction(Instruction::LoadConst(age_key));
            builder.add_instruction(Instruction::LoadConst(age_val));
            builder.add_instruction(Instruction::MakeRecord(2));
            builder.add_instruction(Instruction::Pop);
        }

        // Return dummy value
        let ret_idx = builder.add_constant(Value::Unit);
        builder.add_instruction(Instruction::LoadConst(ret_idx));
        builder.add_instruction(Instruction::Return);
        let chunk = builder.build();

        b.iter(|| {
            let mut vm = Vm::new();
            black_box(vm.execute(chunk.clone()).unwrap());
        });
    });
}

fn bench_array_allocation(c: &mut Criterion) {
    c.bench_function("alloc/arrays_10k", |b| {
        let mut builder = ChunkBuilder::new("array_alloc_benchmark");

        let val1 = builder.add_constant(Value::Int(1));
        let val2 = builder.add_constant(Value::Int(2));
        let val3 = builder.add_constant(Value::Int(3));

        // Create 10,000 small arrays
        for _ in 0..10_000 {
            builder.add_instruction(Instruction::LoadConst(val1));
            builder.add_instruction(Instruction::LoadConst(val2));
            builder.add_instruction(Instruction::LoadConst(val3));
            builder.add_instruction(Instruction::MakeArray(3));
            builder.add_instruction(Instruction::Pop);
        }

        // Return dummy value
        let ret_idx = builder.add_constant(Value::Unit);
        builder.add_instruction(Instruction::LoadConst(ret_idx));
        builder.add_instruction(Instruction::Return);
        let chunk = builder.build();

        b.iter(|| {
            let mut vm = Vm::new();
            black_box(vm.execute(chunk.clone()).unwrap());
        });
    });
}

fn bench_closure_allocation(c: &mut Criterion) {
    c.bench_function("alloc/closures_10k", |b| {
        // Create a simple function chunk
        let mut inner_fn = ChunkBuilder::new("closure_fn");
        let const_idx = inner_fn.add_constant(Value::Int(42));
        inner_fn.add_instruction(Instruction::LoadConst(const_idx));
        inner_fn.add_instruction(Instruction::Return);

        let mut builder = ChunkBuilder::new("closure_alloc_benchmark");

        // Add the function as a constant
        let fn_idx = builder.add_constant(Value::Chunk(inner_fn.build()));

        // Create 10,000 closures (with 0 upvalues)
        for _ in 0..10_000 {
            builder.add_instruction(Instruction::MakeClosure(fn_idx, 0));
            builder.add_instruction(Instruction::Pop);
        }

        // Return dummy value
        let ret_idx = builder.add_constant(Value::Unit);
        builder.add_instruction(Instruction::LoadConst(ret_idx));
        builder.add_instruction(Instruction::Return);
        let chunk = builder.build();

        b.iter(|| {
            let mut vm = Vm::new();
            black_box(vm.execute(chunk.clone()).unwrap());
        });
    });
}

fn bench_list_allocation(c: &mut Criterion) {
    c.bench_function("alloc/lists_10k", |b| {
        let mut builder = ChunkBuilder::new("list_alloc_benchmark");

        let val1 = builder.add_constant(Value::Int(1));
        let val2 = builder.add_constant(Value::Int(2));
        let val3 = builder.add_constant(Value::Int(3));

        // Create 10,000 small lists
        for _ in 0..10_000 {
            builder.add_instruction(Instruction::LoadConst(val1));
            builder.add_instruction(Instruction::LoadConst(val2));
            builder.add_instruction(Instruction::LoadConst(val3));
            builder.add_instruction(Instruction::MakeList(3));
            builder.add_instruction(Instruction::Pop);
        }

        // Return dummy value
        let ret_idx = builder.add_constant(Value::Unit);
        builder.add_instruction(Instruction::LoadConst(ret_idx));
        builder.add_instruction(Instruction::Return);
        let chunk = builder.build();

        b.iter(|| {
            let mut vm = Vm::new();
            black_box(vm.execute(chunk.clone()).unwrap());
        });
    });
}

fn bench_tuple_allocation(c: &mut Criterion) {
    c.bench_function("alloc/tuples_10k", |b| {
        let mut builder = ChunkBuilder::new("tuple_alloc_benchmark");

        let val1 = builder.add_constant(Value::Int(1));
        let val2 = builder.add_constant(Value::String("test".into()));
        let val3 = builder.add_constant(Value::Bool(true));

        // Create 10,000 tuples
        for _ in 0..10_000 {
            builder.add_instruction(Instruction::LoadConst(val1));
            builder.add_instruction(Instruction::LoadConst(val2));
            builder.add_instruction(Instruction::LoadConst(val3));
            builder.add_instruction(Instruction::MakeTuple(3));
            builder.add_instruction(Instruction::Pop);
        }

        // Return dummy value
        let ret_idx = builder.add_constant(Value::Unit);
        builder.add_instruction(Instruction::LoadConst(ret_idx));
        builder.add_instruction(Instruction::Return);
        let chunk = builder.build();

        b.iter(|| {
            let mut vm = Vm::new();
            black_box(vm.execute(chunk.clone()).unwrap());
        });
    });
}

fn bench_string_allocation(c: &mut Criterion) {
    c.bench_function("alloc/strings_10k", |b| {
        let mut builder = ChunkBuilder::new("string_alloc_benchmark");

        // Create 10,000 strings
        for i in 0..10_000 {
            let str_val = builder.add_constant(Value::String(format!("string_{}", i)));
            builder.add_instruction(Instruction::LoadConst(str_val));
            builder.add_instruction(Instruction::Pop);
        }

        // Return dummy value
        let ret_idx = builder.add_constant(Value::Unit);
        builder.add_instruction(Instruction::LoadConst(ret_idx));
        builder.add_instruction(Instruction::Return);
        let chunk = builder.build();

        b.iter(|| {
            let mut vm = Vm::new();
            black_box(vm.execute(chunk.clone()).unwrap());
        });
    });
}

fn bench_mixed_allocation(c: &mut Criterion) {
    c.bench_function("alloc/mixed_10k", |b| {
        let mut builder = ChunkBuilder::new("mixed_alloc_benchmark");

        let int_val = builder.add_constant(Value::Int(42));
        let str_val = builder.add_constant(Value::String("test".into()));
        let name_key = builder.add_constant(Value::String("name".into()));

        // Create a mix of data structures
        for i in 0..10_000 {
            match i % 5 {
                0 => {
                    // Array
                    builder.add_instruction(Instruction::LoadConst(int_val));
                    builder.add_instruction(Instruction::LoadConst(int_val));
                    builder.add_instruction(Instruction::MakeArray(2));
                }
                1 => {
                    // List
                    builder.add_instruction(Instruction::LoadConst(int_val));
                    builder.add_instruction(Instruction::LoadConst(int_val));
                    builder.add_instruction(Instruction::MakeList(2));
                }
                2 => {
                    // Tuple
                    builder.add_instruction(Instruction::LoadConst(int_val));
                    builder.add_instruction(Instruction::LoadConst(str_val));
                    builder.add_instruction(Instruction::MakeTuple(2));
                }
                3 => {
                    // Record
                    builder.add_instruction(Instruction::LoadConst(name_key));
                    builder.add_instruction(Instruction::LoadConst(str_val));
                    builder.add_instruction(Instruction::MakeRecord(1));
                }
                _ => {
                    // Just a string
                    builder.add_instruction(Instruction::LoadConst(str_val));
                }
            }
            builder.add_instruction(Instruction::Pop);
        }

        // Return dummy value
        let ret_idx = builder.add_constant(Value::Unit);
        builder.add_instruction(Instruction::LoadConst(ret_idx));
        builder.add_instruction(Instruction::Return);
        let chunk = builder.build();

        b.iter(|| {
            let mut vm = Vm::new();
            black_box(vm.execute(chunk.clone()).unwrap());
        });
    });
}

criterion_group!(
    benches,
    bench_record_allocation,
    bench_array_allocation,
    bench_closure_allocation,
    bench_list_allocation,
    bench_tuple_allocation,
    bench_string_allocation,
    bench_mixed_allocation
);
criterion_main!(benches);
