// Benchmarks for instruction dispatch performance
// Tests tight loops of arithmetic and control flow operations

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fusabi_vm::{Chunk, ChunkBuilder, Instruction, Value, Vm};

fn bench_add_operations(c: &mut Criterion) {
    c.bench_function("op_dispatch/add_1000", |b| {
        // Build bytecode for: 1 + 1 + 1 + ... (1000 additions)
        let mut builder = ChunkBuilder::new("add_benchmark");

        // Load initial value
        let const_idx = builder.add_constant(Value::Int(1));
        builder.add_instruction(Instruction::LoadConst(const_idx));

        // Chain 1000 additions
        for _ in 0..1000 {
            builder.add_instruction(Instruction::LoadConst(const_idx));
            builder.add_instruction(Instruction::Add);
        }

        builder.add_instruction(Instruction::Return);
        let chunk = builder.build();

        b.iter(|| {
            let mut vm = Vm::new();
            black_box(vm.execute(chunk.clone()).unwrap());
        });
    });
}

fn bench_sub_operations(c: &mut Criterion) {
    c.bench_function("op_dispatch/sub_1000", |b| {
        let mut builder = ChunkBuilder::new("sub_benchmark");

        // Start with a large number
        let start_idx = builder.add_constant(Value::Int(10000));
        builder.add_instruction(Instruction::LoadConst(start_idx));

        let one_idx = builder.add_constant(Value::Int(1));

        // Chain 1000 subtractions
        for _ in 0..1000 {
            builder.add_instruction(Instruction::LoadConst(one_idx));
            builder.add_instruction(Instruction::Sub);
        }

        builder.add_instruction(Instruction::Return);
        let chunk = builder.build();

        b.iter(|| {
            let mut vm = Vm::new();
            black_box(vm.execute(chunk.clone()).unwrap());
        });
    });
}

fn bench_mul_operations(c: &mut Criterion) {
    c.bench_function("op_dispatch/mul_1000", |b| {
        let mut builder = ChunkBuilder::new("mul_benchmark");

        let two_idx = builder.add_constant(Value::Int(2));
        builder.add_instruction(Instruction::LoadConst(two_idx));

        // Chain 1000 multiplications (will overflow but that's ok for benchmarking)
        for _ in 0..1000 {
            builder.add_instruction(Instruction::LoadConst(two_idx));
            builder.add_instruction(Instruction::Mul);
        }

        builder.add_instruction(Instruction::Return);
        let chunk = builder.build();

        b.iter(|| {
            let mut vm = Vm::new();
            black_box(vm.execute(chunk.clone()).unwrap());
        });
    });
}

fn bench_comparison_operations(c: &mut Criterion) {
    c.bench_function("op_dispatch/comparison_1000", |b| {
        let mut builder = ChunkBuilder::new("comparison_benchmark");

        let val1_idx = builder.add_constant(Value::Int(42));
        let val2_idx = builder.add_constant(Value::Int(43));

        // Chain 1000 comparisons
        for _ in 0..1000 {
            builder.add_instruction(Instruction::LoadConst(val1_idx));
            builder.add_instruction(Instruction::LoadConst(val2_idx));
            builder.add_instruction(Instruction::Lt);
            builder.add_instruction(Instruction::Pop);
        }

        // Return a dummy value
        builder.add_instruction(Instruction::LoadConst(val1_idx));
        builder.add_instruction(Instruction::Return);
        let chunk = builder.build();

        b.iter(|| {
            let mut vm = Vm::new();
            black_box(vm.execute(chunk.clone()).unwrap());
        });
    });
}

fn bench_call_operations(c: &mut Criterion) {
    c.bench_function("op_dispatch/call_100", |b| {
        // Create a simple function that returns 42
        let mut inner_fn = ChunkBuilder::new("simple_fn");
        let const_idx = inner_fn.add_constant(Value::Int(42));
        inner_fn.add_instruction(Instruction::LoadConst(const_idx));
        inner_fn.add_instruction(Instruction::Return);

        let mut builder = ChunkBuilder::new("call_benchmark");

        // Add the function as a constant
        let fn_idx = builder.add_constant(Value::Chunk(inner_fn.build()));

        // Call it 100 times
        for _ in 0..100 {
            builder.add_instruction(Instruction::LoadConst(fn_idx));
            builder.add_instruction(Instruction::Call(0));
            builder.add_instruction(Instruction::Pop);
        }

        // Return a dummy value
        let ret_idx = builder.add_constant(Value::Int(1));
        builder.add_instruction(Instruction::LoadConst(ret_idx));
        builder.add_instruction(Instruction::Return);
        let chunk = builder.build();

        b.iter(|| {
            let mut vm = Vm::new();
            black_box(vm.execute(chunk.clone()).unwrap());
        });
    });
}

fn bench_mixed_operations(c: &mut Criterion) {
    c.bench_function("op_dispatch/mixed_ops_1000", |b| {
        let mut builder = ChunkBuilder::new("mixed_benchmark");

        let val_idx = builder.add_constant(Value::Int(10));
        builder.add_instruction(Instruction::LoadConst(val_idx));

        // Mix of different operations
        for i in 0..1000 {
            builder.add_instruction(Instruction::LoadConst(val_idx));
            match i % 4 {
                0 => builder.add_instruction(Instruction::Add),
                1 => builder.add_instruction(Instruction::Sub),
                2 => builder.add_instruction(Instruction::Mul),
                _ => {
                    builder.add_instruction(Instruction::Dup);
                    builder.add_instruction(Instruction::Lt);
                    builder.add_instruction(Instruction::Pop);
                }
            }
        }

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
    bench_add_operations,
    bench_sub_operations,
    bench_mul_operations,
    bench_comparison_operations,
    bench_call_operations,
    bench_mixed_operations
);
criterion_main!(benches);
