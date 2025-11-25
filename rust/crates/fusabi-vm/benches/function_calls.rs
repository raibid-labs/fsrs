// Benchmarks for function call performance
// Tests native functions, closures, and recursion

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fusabi_vm::{Chunk, ChunkBuilder, HostFn, Instruction, Value, Vm};
use std::rc::Rc;

fn bench_native_function_calls(c: &mut Criterion) {
    c.bench_function("function_calls/native_1000", |b| {
        b.iter(|| {
            let mut vm = Vm::new();

            // Register a simple native function
            vm.register_host_fn(
                "test_fn",
                HostFn::new(|_args| Ok(Value::Int(42))),
            );

            let mut builder = ChunkBuilder::new("native_call_benchmark");

            // Call the native function 1000 times
            for _ in 0..1000 {
                let fn_name = builder.add_constant(Value::String("test_fn".into()));
                builder.add_instruction(Instruction::LoadGlobal(fn_name));
                builder.add_instruction(Instruction::Call(0));
                builder.add_instruction(Instruction::Pop);
            }

            // Return dummy value
            let ret_idx = builder.add_constant(Value::Unit);
            builder.add_instruction(Instruction::LoadConst(ret_idx));
            builder.add_instruction(Instruction::Return);
            let chunk = builder.build();

            black_box(vm.execute(chunk).unwrap());
        });
    });
}

fn bench_native_function_with_args(c: &mut Criterion) {
    c.bench_function("function_calls/native_with_args_1000", |b| {
        b.iter(|| {
            let mut vm = Vm::new();

            // Register a native function that takes 2 arguments
            vm.register_host_fn(
                "add",
                HostFn::new(|args| {
                    if args.len() != 2 {
                        return Err(fusabi_vm::VmError::Runtime(
                            "Expected 2 arguments".into(),
                        ));
                    }
                    let a = args[0].as_int().unwrap_or(0);
                    let b = args[1].as_int().unwrap_or(0);
                    Ok(Value::Int(a + b))
                }),
            );

            let mut builder = ChunkBuilder::new("native_call_args_benchmark");

            let arg1 = builder.add_constant(Value::Int(10));
            let arg2 = builder.add_constant(Value::Int(20));

            // Call the native function 1000 times with arguments
            for _ in 0..1000 {
                let fn_name = builder.add_constant(Value::String("add".into()));
                builder.add_instruction(Instruction::LoadGlobal(fn_name));
                builder.add_instruction(Instruction::LoadConst(arg1));
                builder.add_instruction(Instruction::LoadConst(arg2));
                builder.add_instruction(Instruction::Call(2));
                builder.add_instruction(Instruction::Pop);
            }

            // Return dummy value
            let ret_idx = builder.add_constant(Value::Unit);
            builder.add_instruction(Instruction::LoadConst(ret_idx));
            builder.add_instruction(Instruction::Return);
            let chunk = builder.build();

            black_box(vm.execute(chunk).unwrap());
        });
    });
}

fn bench_closure_calls(c: &mut Criterion) {
    c.bench_function("function_calls/closure_1000", |b| {
        // Create a simple closure
        let mut inner_fn = ChunkBuilder::new("simple_closure");
        let const_idx = inner_fn.add_constant(Value::Int(42));
        inner_fn.add_instruction(Instruction::LoadConst(const_idx));
        inner_fn.add_instruction(Instruction::Return);

        let mut builder = ChunkBuilder::new("closure_call_benchmark");

        // Create closure once
        let fn_idx = builder.add_constant(Value::Chunk(inner_fn.build()));
        builder.add_instruction(Instruction::MakeClosure(fn_idx, 0));
        builder.add_instruction(Instruction::StoreLocal(0));

        // Call it 1000 times
        for _ in 0..1000 {
            builder.add_instruction(Instruction::LoadLocal(0));
            builder.add_instruction(Instruction::Call(0));
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

fn bench_closure_with_args(c: &mut Criterion) {
    c.bench_function("function_calls/closure_with_args_1000", |b| {
        // Create a closure that adds two arguments
        let mut inner_fn = ChunkBuilder::new("add_closure");
        inner_fn.add_instruction(Instruction::LoadLocal(0)); // First arg
        inner_fn.add_instruction(Instruction::LoadLocal(1)); // Second arg
        inner_fn.add_instruction(Instruction::Add);
        inner_fn.add_instruction(Instruction::Return);

        let mut builder = ChunkBuilder::new("closure_args_call_benchmark");

        // Create closure once
        let fn_idx = builder.add_constant(Value::Chunk(inner_fn.build()));
        builder.add_instruction(Instruction::MakeClosure(fn_idx, 0));
        builder.add_instruction(Instruction::StoreLocal(0));

        let arg1 = builder.add_constant(Value::Int(10));
        let arg2 = builder.add_constant(Value::Int(20));

        // Call it 1000 times with arguments
        for _ in 0..1000 {
            builder.add_instruction(Instruction::LoadLocal(0));
            builder.add_instruction(Instruction::LoadConst(arg1));
            builder.add_instruction(Instruction::LoadConst(arg2));
            builder.add_instruction(Instruction::Call(2));
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

fn bench_recursive_calls(c: &mut Criterion) {
    c.bench_function("function_calls/recursive_factorial_10", |b| {
        // Factorial function (iterative countdown)
        // factorial(n) = if n <= 1 then 1 else n * factorial(n-1)
        let mut factorial_fn = ChunkBuilder::new("factorial");

        // Load n
        factorial_fn.add_instruction(Instruction::LoadLocal(0));
        let one_idx = factorial_fn.add_constant(Value::Int(1));
        factorial_fn.add_instruction(Instruction::LoadConst(one_idx));

        // Check if n <= 1
        factorial_fn.add_instruction(Instruction::Lte);
        factorial_fn.add_instruction(Instruction::JumpIfFalse(3)); // Skip to else branch

        // Then branch: return 1
        factorial_fn.add_instruction(Instruction::LoadConst(one_idx));
        factorial_fn.add_instruction(Instruction::Return);
        factorial_fn.add_instruction(Instruction::Jump(0)); // Placeholder, will be filled

        // Else branch: n * factorial(n-1)
        // Load n
        factorial_fn.add_instruction(Instruction::LoadLocal(0));

        // Load factorial function (assuming it's in local 1)
        factorial_fn.add_instruction(Instruction::LoadLocal(1));

        // Calculate n - 1
        factorial_fn.add_instruction(Instruction::LoadLocal(0));
        factorial_fn.add_instruction(Instruction::LoadConst(one_idx));
        factorial_fn.add_instruction(Instruction::Sub);

        // Call factorial(n-1)
        factorial_fn.add_instruction(Instruction::Call(1));

        // Multiply n * factorial(n-1)
        factorial_fn.add_instruction(Instruction::Mul);
        factorial_fn.add_instruction(Instruction::Return);

        let mut builder = ChunkBuilder::new("recursive_call_benchmark");

        // Create factorial closure
        let fn_idx = builder.add_constant(Value::Chunk(factorial_fn.build()));
        builder.add_instruction(Instruction::MakeClosure(fn_idx, 0));
        builder.add_instruction(Instruction::StoreLocal(0));

        // Call factorial(10) 100 times
        let arg = builder.add_constant(Value::Int(10));
        for _ in 0..100 {
            builder.add_instruction(Instruction::LoadLocal(0));
            builder.add_instruction(Instruction::Dup); // For the recursive call
            builder.add_instruction(Instruction::LoadConst(arg));
            builder.add_instruction(Instruction::Call(2)); // Pass fn and arg
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

fn bench_tail_call_optimization(c: &mut Criterion) {
    c.bench_function("function_calls/tail_call_1000", |b| {
        // Simple tail-recursive countdown
        let mut countdown_fn = ChunkBuilder::new("countdown");

        // Load n
        countdown_fn.add_instruction(Instruction::LoadLocal(0));
        let zero_idx = countdown_fn.add_constant(Value::Int(0));
        countdown_fn.add_instruction(Instruction::LoadConst(zero_idx));

        // Check if n <= 0
        countdown_fn.add_instruction(Instruction::Lte);
        countdown_fn.add_instruction(Instruction::JumpIfFalse(3)); // Skip to else branch

        // Then branch: return 0
        countdown_fn.add_instruction(Instruction::LoadConst(zero_idx));
        countdown_fn.add_instruction(Instruction::Return);
        countdown_fn.add_instruction(Instruction::Jump(0)); // Placeholder

        // Else branch: tail call countdown(n-1)
        countdown_fn.add_instruction(Instruction::LoadLocal(0));
        let one_idx = countdown_fn.add_constant(Value::Int(1));
        countdown_fn.add_instruction(Instruction::LoadConst(one_idx));
        countdown_fn.add_instruction(Instruction::Sub);
        countdown_fn.add_instruction(Instruction::TailCall(1));

        let mut builder = ChunkBuilder::new("tail_call_benchmark");

        // Create countdown closure
        let fn_idx = builder.add_constant(Value::Chunk(countdown_fn.build()));
        builder.add_instruction(Instruction::MakeClosure(fn_idx, 0));
        builder.add_instruction(Instruction::StoreLocal(0));

        // Call countdown(1000)
        builder.add_instruction(Instruction::LoadLocal(0));
        let arg = builder.add_constant(Value::Int(1000));
        builder.add_instruction(Instruction::LoadConst(arg));
        builder.add_instruction(Instruction::Call(1));
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
    bench_native_function_calls,
    bench_native_function_with_args,
    bench_closure_calls,
    bench_closure_with_args,
    bench_recursive_calls,
    bench_tail_call_optimization
);
criterion_main!(benches);
