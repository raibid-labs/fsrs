// Comprehensive VM dispatch benchmarks for issue #203
// Measures VM performance across different workload patterns

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use fusabi_vm::{ChunkBuilder, Instruction, Value, Vm};

// =============================================================================
// 1. ARITHMETIC-HEAVY WORKLOADS
// =============================================================================

fn bench_arithmetic_add_chain(c: &mut Criterion) {
    let mut group = c.benchmark_group("dispatch/arithmetic");

    for size in [100, 1000, 10_000] {
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::new("add_chain", size), &size, |b, &size| {
            let mut builder = ChunkBuilder::new("add_chain");
            let one_idx = builder.add_constant(Value::Int(1));
            builder.add_instruction(Instruction::LoadConst(one_idx));

            for _ in 0..size {
                builder.add_instruction(Instruction::LoadConst(one_idx));
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

    group.finish();
}

fn bench_arithmetic_mixed(c: &mut Criterion) {
    let mut group = c.benchmark_group("dispatch/arithmetic");

    for size in [100, 1000, 5000] {
        group.throughput(Throughput::Elements(size as u64 * 3));
        group.bench_with_input(BenchmarkId::new("mixed_ops", size), &size, |b, &size| {
            let mut builder = ChunkBuilder::new("mixed_arithmetic");
            let val_idx = builder.add_constant(Value::Int(100));
            let two_idx = builder.add_constant(Value::Int(2));
            builder.add_instruction(Instruction::LoadConst(val_idx));

            for _ in 0..size {
                builder.add_instruction(Instruction::LoadConst(two_idx));
                builder.add_instruction(Instruction::Add);
                builder.add_instruction(Instruction::LoadConst(two_idx));
                builder.add_instruction(Instruction::Sub);
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

    group.finish();
}

fn bench_arithmetic_loop_simulation(c: &mut Criterion) {
    c.bench_function("dispatch/arithmetic/loop_with_accumulator", |b| {
        let mut builder = ChunkBuilder::new("loop_accumulator");

        let zero_idx = builder.add_constant(Value::Int(0));
        let one_idx = builder.add_constant(Value::Int(1));
        let limit_idx = builder.add_constant(Value::Int(1000));

        builder.add_instruction(Instruction::LoadConst(zero_idx));
        builder.add_instruction(Instruction::StoreLocal(0));
        builder.add_instruction(Instruction::LoadConst(zero_idx));
        builder.add_instruction(Instruction::StoreLocal(1));

        let loop_start = builder.instruction_count();

        builder.add_instruction(Instruction::LoadLocal(0));
        builder.add_instruction(Instruction::LoadConst(limit_idx));
        builder.add_instruction(Instruction::Lt);
        builder.add_instruction(Instruction::JumpIfFalse(10));

        builder.add_instruction(Instruction::LoadLocal(1));
        builder.add_instruction(Instruction::LoadLocal(0));
        builder.add_instruction(Instruction::Add);
        builder.add_instruction(Instruction::StoreLocal(1));

        builder.add_instruction(Instruction::LoadLocal(0));
        builder.add_instruction(Instruction::LoadConst(one_idx));
        builder.add_instruction(Instruction::Add);
        builder.add_instruction(Instruction::StoreLocal(0));

        let jump_back = loop_start as i16 - builder.instruction_count() as i16 - 1;
        builder.add_instruction(Instruction::Jump(jump_back));

        builder.add_instruction(Instruction::LoadLocal(1));
        builder.add_instruction(Instruction::Return);

        let chunk = builder.build();

        b.iter(|| {
            let mut vm = Vm::new();
            black_box(vm.execute(chunk.clone()).unwrap());
        });
    });
}

// =============================================================================
// 2. FUNCTION CALL OVERHEAD
// =============================================================================

fn bench_recursive_fibonacci(c: &mut Criterion) {
    let mut group = c.benchmark_group("dispatch/function_calls");

    for n in [5, 10, 15, 20] {
        group.bench_with_input(BenchmarkId::new("fibonacci_recursive", n), &n, |b, &n| {
            let mut fib_fn = ChunkBuilder::new("fib");
            let zero_idx = fib_fn.add_constant(Value::Int(0));
            let one_idx = fib_fn.add_constant(Value::Int(1));
            let two_idx = fib_fn.add_constant(Value::Int(2));

            fib_fn.add_instruction(Instruction::LoadLocal(0));
            fib_fn.add_instruction(Instruction::LoadConst(two_idx));
            fib_fn.add_instruction(Instruction::Lt);
            fib_fn.add_instruction(Instruction::JumpIfFalse(3));

            fib_fn.add_instruction(Instruction::LoadLocal(0));
            fib_fn.add_instruction(Instruction::Return);
            fib_fn.add_instruction(Instruction::Jump(0));

            fib_fn.add_instruction(Instruction::LoadLocal(1));
            fib_fn.add_instruction(Instruction::LoadLocal(0));
            fib_fn.add_instruction(Instruction::LoadConst(one_idx));
            fib_fn.add_instruction(Instruction::Sub);
            fib_fn.add_instruction(Instruction::Call(1));

            fib_fn.add_instruction(Instruction::LoadLocal(1));
            fib_fn.add_instruction(Instruction::LoadLocal(0));
            fib_fn.add_instruction(Instruction::LoadConst(two_idx));
            fib_fn.add_instruction(Instruction::Sub);
            fib_fn.add_instruction(Instruction::Call(1));

            fib_fn.add_instruction(Instruction::Add);
            fib_fn.add_instruction(Instruction::Return);

            let mut builder = ChunkBuilder::new("fib_benchmark");
            let fn_idx = builder.add_constant(Value::Chunk(fib_fn.build()));
            builder.add_instruction(Instruction::MakeClosure(fn_idx, 0));
            builder.add_instruction(Instruction::StoreLocal(0));

            builder.add_instruction(Instruction::LoadLocal(0));
            builder.add_instruction(Instruction::Dup);
            let n_idx = builder.add_constant(Value::Int(n));
            builder.add_instruction(Instruction::LoadConst(n_idx));
            builder.add_instruction(Instruction::Call(2));
            builder.add_instruction(Instruction::Return);

            let chunk = builder.build();

            b.iter(|| {
                let mut vm = Vm::new();
                black_box(vm.execute(chunk.clone()).unwrap());
            });
        });
    }

    group.finish();
}

fn bench_nested_calls(c: &mut Criterion) {
    let mut group = c.benchmark_group("dispatch/function_calls");

    for depth in [5, 10, 20, 50] {
        group.bench_with_input(
            BenchmarkId::new("nested_calls", depth),
            &depth,
            |b, &depth| {
                let mut inner_fn = ChunkBuilder::new("inner");
                let const_idx = inner_fn.add_constant(Value::Int(42));
                inner_fn.add_instruction(Instruction::LoadConst(const_idx));
                inner_fn.add_instruction(Instruction::Return);

                let mut current_chunk = inner_fn.build();

                for i in 0..depth {
                    let mut wrapper = ChunkBuilder::new(&format!("wrapper_{}", i));
                    let fn_idx = wrapper.add_constant(Value::Chunk(current_chunk));
                    wrapper.add_instruction(Instruction::LoadConst(fn_idx));
                    wrapper.add_instruction(Instruction::Call(0));
                    wrapper.add_instruction(Instruction::Return);
                    current_chunk = wrapper.build();
                }

                let mut builder = ChunkBuilder::new("nested_call_benchmark");
                for _ in 0..100 {
                    let fn_idx = builder.add_constant(Value::Chunk(current_chunk.clone()));
                    builder.add_instruction(Instruction::LoadConst(fn_idx));
                    builder.add_instruction(Instruction::Call(0));
                    builder.add_instruction(Instruction::Pop);
                }

                let ret_idx = builder.add_constant(Value::Unit);
                builder.add_instruction(Instruction::LoadConst(ret_idx));
                builder.add_instruction(Instruction::Return);
                let chunk = builder.build();

                b.iter(|| {
                    let mut vm = Vm::new();
                    black_box(vm.execute(chunk.clone()).unwrap());
                });
            },
        );
    }

    group.finish();
}

fn bench_call_return_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("dispatch/function_calls");

    for iterations in [100, 1000, 10_000] {
        group.throughput(Throughput::Elements(iterations as u64));
        group.bench_with_input(
            BenchmarkId::new("call_return", iterations),
            &iterations,
            |b, &iterations| {
                let mut simple_fn = ChunkBuilder::new("simple");
                let const_idx = simple_fn.add_constant(Value::Int(1));
                simple_fn.add_instruction(Instruction::LoadConst(const_idx));
                simple_fn.add_instruction(Instruction::Return);

                let mut builder = ChunkBuilder::new("call_return_benchmark");
                let fn_idx = builder.add_constant(Value::Chunk(simple_fn.build()));
                builder.add_instruction(Instruction::MakeClosure(fn_idx, 0));
                builder.add_instruction(Instruction::StoreLocal(0));

                for _ in 0..iterations {
                    builder.add_instruction(Instruction::LoadLocal(0));
                    builder.add_instruction(Instruction::Call(0));
                    builder.add_instruction(Instruction::Pop);
                }

                let ret_idx = builder.add_constant(Value::Unit);
                builder.add_instruction(Instruction::LoadConst(ret_idx));
                builder.add_instruction(Instruction::Return);
                let chunk = builder.build();

                b.iter(|| {
                    let mut vm = Vm::new();
                    black_box(vm.execute(chunk.clone()).unwrap());
                });
            },
        );
    }

    group.finish();
}

// =============================================================================
// 3. STACK OPERATIONS
// =============================================================================

fn bench_push_pop_intensive(c: &mut Criterion) {
    let mut group = c.benchmark_group("dispatch/stack");

    for size in [100, 1000, 10_000] {
        group.throughput(Throughput::Elements(size as u64 * 2));
        group.bench_with_input(BenchmarkId::new("push_pop", size), &size, |b, &size| {
            let mut builder = ChunkBuilder::new("push_pop");
            let val_idx = builder.add_constant(Value::Int(42));

            for _ in 0..size {
                builder.add_instruction(Instruction::LoadConst(val_idx));
                builder.add_instruction(Instruction::Pop);
            }

            builder.add_instruction(Instruction::LoadConst(val_idx));
            builder.add_instruction(Instruction::Return);
            let chunk = builder.build();

            b.iter(|| {
                let mut vm = Vm::new();
                black_box(vm.execute(chunk.clone()).unwrap());
            });
        });
    }

    group.finish();
}

fn bench_local_variable_access(c: &mut Criterion) {
    let mut group = c.benchmark_group("dispatch/stack");

    for num_locals in [4, 16, 64] {
        group.bench_with_input(
            BenchmarkId::new("local_access", num_locals),
            &num_locals,
            |b, &num_locals| {
                let mut builder = ChunkBuilder::new("local_access");
                let val_idx = builder.add_constant(Value::Int(1));

                for i in 0..num_locals.min(255) as u8 {
                    builder.add_instruction(Instruction::LoadConst(val_idx));
                    builder.add_instruction(Instruction::StoreLocal(i));
                }

                for _ in 0..1000 {
                    for i in 0..num_locals.min(255) as u8 {
                        builder.add_instruction(Instruction::LoadLocal(i));
                        builder.add_instruction(Instruction::Pop);
                    }
                }

                builder.add_instruction(Instruction::LoadLocal(0));
                builder.add_instruction(Instruction::Return);
                let chunk = builder.build();

                b.iter(|| {
                    let mut vm = Vm::new();
                    black_box(vm.execute(chunk.clone()).unwrap());
                });
            },
        );
    }

    group.finish();
}

fn bench_dup_operations(c: &mut Criterion) {
    c.bench_function("dispatch/stack/dup_intensive", |b| {
        let mut builder = ChunkBuilder::new("dup_intensive");
        let val_idx = builder.add_constant(Value::Int(1));

        builder.add_instruction(Instruction::LoadConst(val_idx));

        for _ in 0..1000 {
            builder.add_instruction(Instruction::Dup);
            builder.add_instruction(Instruction::Pop);
        }

        builder.add_instruction(Instruction::Return);
        let chunk = builder.build();

        b.iter(|| {
            let mut vm = Vm::new();
            black_box(vm.execute(chunk.clone()).unwrap());
        });
    });
}

// =============================================================================
// 4. COLLECTION OPERATIONS
// =============================================================================

fn bench_list_construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("dispatch/collections");

    for size in [3, 10, 50, 100] {
        group.bench_with_input(
            BenchmarkId::new("list_construction", size),
            &size,
            |b, &size| {
                let mut builder = ChunkBuilder::new("list_construction");
                let val_idx = builder.add_constant(Value::Int(42));

                for _ in 0..100 {
                    for _ in 0..size {
                        builder.add_instruction(Instruction::LoadConst(val_idx));
                    }
                    builder.add_instruction(Instruction::MakeList(size as u16));
                    builder.add_instruction(Instruction::Pop);
                }

                let ret_idx = builder.add_constant(Value::Unit);
                builder.add_instruction(Instruction::LoadConst(ret_idx));
                builder.add_instruction(Instruction::Return);
                let chunk = builder.build();

                b.iter(|| {
                    let mut vm = Vm::new();
                    black_box(vm.execute(chunk.clone()).unwrap());
                });
            },
        );
    }

    group.finish();
}

fn bench_list_traversal(c: &mut Criterion) {
    c.bench_function("dispatch/collections/list_traversal", |b| {
        let mut builder = ChunkBuilder::new("list_traversal");

        for i in 0..100 {
            let val_idx = builder.add_constant(Value::Int(i));
            builder.add_instruction(Instruction::LoadConst(val_idx));
        }
        builder.add_instruction(Instruction::MakeList(100));
        builder.add_instruction(Instruction::StoreLocal(0));

        let zero_idx = builder.add_constant(Value::Int(0));
        builder.add_instruction(Instruction::LoadConst(zero_idx));
        builder.add_instruction(Instruction::StoreLocal(1));

        let loop_start = builder.instruction_count();

        builder.add_instruction(Instruction::LoadLocal(0));
        builder.add_instruction(Instruction::IsNil);
        builder.add_instruction(Instruction::JumpIfFalse(3));

        builder.add_instruction(Instruction::LoadLocal(1));
        builder.add_instruction(Instruction::Return);
        builder.add_instruction(Instruction::Jump(0));

        builder.add_instruction(Instruction::LoadLocal(0));
        builder.add_instruction(Instruction::ListHead);
        builder.add_instruction(Instruction::LoadLocal(1));
        builder.add_instruction(Instruction::Add);
        builder.add_instruction(Instruction::StoreLocal(1));

        builder.add_instruction(Instruction::LoadLocal(0));
        builder.add_instruction(Instruction::ListTail);
        builder.add_instruction(Instruction::StoreLocal(0));

        let jump_back = loop_start as i16 - builder.instruction_count() as i16 - 1;
        builder.add_instruction(Instruction::Jump(jump_back));

        let chunk = builder.build();

        b.iter(|| {
            let mut vm = Vm::new();
            black_box(vm.execute(chunk.clone()).unwrap());
        });
    });
}

fn bench_array_access(c: &mut Criterion) {
    let mut group = c.benchmark_group("dispatch/collections");

    for size in [10, 100, 1000] {
        group.bench_with_input(
            BenchmarkId::new("array_random_access", size),
            &size,
            |b, &size| {
                let mut builder = ChunkBuilder::new("array_access");

                for i in 0..size {
                    let val_idx = builder.add_constant(Value::Int(i as i64));
                    builder.add_instruction(Instruction::LoadConst(val_idx));
                }
                builder.add_instruction(Instruction::MakeArray(size as u16));
                builder.add_instruction(Instruction::StoreLocal(0));

                for i in 0..1000 {
                    builder.add_instruction(Instruction::LoadLocal(0));
                    let idx = builder.add_constant(Value::Int((i % size) as i64));
                    builder.add_instruction(Instruction::LoadConst(idx));
                    builder.add_instruction(Instruction::ArrayGet);
                    builder.add_instruction(Instruction::Pop);
                }

                let ret_idx = builder.add_constant(Value::Unit);
                builder.add_instruction(Instruction::LoadConst(ret_idx));
                builder.add_instruction(Instruction::Return);
                let chunk = builder.build();

                b.iter(|| {
                    let mut vm = Vm::new();
                    black_box(vm.execute(chunk.clone()).unwrap());
                });
            },
        );
    }

    group.finish();
}

fn bench_array_sequential_access(c: &mut Criterion) {
    c.bench_function("dispatch/collections/array_sequential", |b| {
        let mut builder = ChunkBuilder::new("array_sequential");

        for i in 0..100 {
            let val_idx = builder.add_constant(Value::Int(i));
            builder.add_instruction(Instruction::LoadConst(val_idx));
        }
        builder.add_instruction(Instruction::MakeArray(100));
        builder.add_instruction(Instruction::StoreLocal(0));

        let zero_idx = builder.add_constant(Value::Int(0));
        let one_idx = builder.add_constant(Value::Int(1));
        let limit_idx = builder.add_constant(Value::Int(100));

        builder.add_instruction(Instruction::LoadConst(zero_idx));
        builder.add_instruction(Instruction::StoreLocal(1));
        builder.add_instruction(Instruction::LoadConst(zero_idx));
        builder.add_instruction(Instruction::StoreLocal(2));

        let loop_start = builder.instruction_count();

        builder.add_instruction(Instruction::LoadLocal(1));
        builder.add_instruction(Instruction::LoadConst(limit_idx));
        builder.add_instruction(Instruction::Lt);
        builder.add_instruction(Instruction::JumpIfFalse(12));

        builder.add_instruction(Instruction::LoadLocal(0));
        builder.add_instruction(Instruction::LoadLocal(1));
        builder.add_instruction(Instruction::ArrayGet);
        builder.add_instruction(Instruction::LoadLocal(2));
        builder.add_instruction(Instruction::Add);
        builder.add_instruction(Instruction::StoreLocal(2));

        builder.add_instruction(Instruction::LoadLocal(1));
        builder.add_instruction(Instruction::LoadConst(one_idx));
        builder.add_instruction(Instruction::Add);
        builder.add_instruction(Instruction::StoreLocal(1));

        let jump_back = loop_start as i16 - builder.instruction_count() as i16 - 1;
        builder.add_instruction(Instruction::Jump(jump_back));

        builder.add_instruction(Instruction::LoadLocal(2));
        builder.add_instruction(Instruction::Return);

        let chunk = builder.build();

        b.iter(|| {
            let mut vm = Vm::new();
            black_box(vm.execute(chunk.clone()).unwrap());
        });
    });
}

// =============================================================================
// 5. CONTROL FLOW
// =============================================================================

fn bench_conditional_jumps(c: &mut Criterion) {
    let mut group = c.benchmark_group("dispatch/control_flow");

    for num_branches in [10, 100, 1000] {
        group.throughput(Throughput::Elements(num_branches as u64));
        group.bench_with_input(
            BenchmarkId::new("conditional_jumps", num_branches),
            &num_branches,
            |b, &num_branches| {
                let mut builder = ChunkBuilder::new("conditional_jumps");
                let true_idx = builder.add_constant(Value::Bool(true));
                let false_idx = builder.add_constant(Value::Bool(false));
                let one_idx = builder.add_constant(Value::Int(1));

                builder.add_instruction(Instruction::LoadConst(one_idx));
                builder.add_instruction(Instruction::StoreLocal(0));

                for i in 0..num_branches {
                    let cond_idx = if i % 2 == 0 { true_idx } else { false_idx };
                    builder.add_instruction(Instruction::LoadConst(cond_idx));
                    builder.add_instruction(Instruction::JumpIfFalse(4));

                    builder.add_instruction(Instruction::LoadLocal(0));
                    builder.add_instruction(Instruction::LoadConst(one_idx));
                    builder.add_instruction(Instruction::Add);
                    builder.add_instruction(Instruction::StoreLocal(0));
                }

                builder.add_instruction(Instruction::LoadLocal(0));
                builder.add_instruction(Instruction::Return);
                let chunk = builder.build();

                b.iter(|| {
                    let mut vm = Vm::new();
                    black_box(vm.execute(chunk.clone()).unwrap());
                });
            },
        );
    }

    group.finish();
}

fn bench_pattern_matching_int(c: &mut Criterion) {
    c.bench_function("dispatch/control_flow/pattern_match_int", |b| {
        let mut builder = ChunkBuilder::new("pattern_match_int");

        let test_values: Vec<i64> = (0..100).collect();
        let result_idx = builder.add_constant(Value::Int(0));

        builder.add_instruction(Instruction::LoadConst(result_idx));
        builder.add_instruction(Instruction::StoreLocal(0));

        for val in test_values {
            let val_idx = builder.add_constant(Value::Int(val));
            builder.add_instruction(Instruction::LoadConst(val_idx));
            builder.add_instruction(Instruction::StoreLocal(1));

            builder.add_instruction(Instruction::LoadLocal(1));
            builder.add_instruction(Instruction::CheckInt(0));
            builder.add_instruction(Instruction::JumpIfFalse(4));
            let zero_result = builder.add_constant(Value::Int(0));
            builder.add_instruction(Instruction::LoadConst(zero_result));
            builder.add_instruction(Instruction::StoreLocal(0));
            builder.add_instruction(Instruction::Jump(15));

            builder.add_instruction(Instruction::LoadLocal(1));
            builder.add_instruction(Instruction::CheckInt(1));
            builder.add_instruction(Instruction::JumpIfFalse(4));
            let one_result = builder.add_constant(Value::Int(1));
            builder.add_instruction(Instruction::LoadConst(one_result));
            builder.add_instruction(Instruction::StoreLocal(0));
            builder.add_instruction(Instruction::Jump(8));

            builder.add_instruction(Instruction::LoadLocal(1));
            builder.add_instruction(Instruction::CheckInt(2));
            builder.add_instruction(Instruction::JumpIfFalse(4));
            let two_result = builder.add_constant(Value::Int(2));
            builder.add_instruction(Instruction::LoadConst(two_result));
            builder.add_instruction(Instruction::StoreLocal(0));
            builder.add_instruction(Instruction::Jump(1));

            builder.add_instruction(Instruction::Pop);
        }

        builder.add_instruction(Instruction::LoadLocal(0));
        builder.add_instruction(Instruction::Return);
        let chunk = builder.build();

        b.iter(|| {
            let mut vm = Vm::new();
            black_box(vm.execute(chunk.clone()).unwrap());
        });
    });
}

fn bench_pattern_matching_tuple(c: &mut Criterion) {
    c.bench_function("dispatch/control_flow/pattern_match_tuple", |b| {
        let mut builder = ChunkBuilder::new("pattern_match_tuple");

        let zero_idx = builder.add_constant(Value::Int(0));
        builder.add_instruction(Instruction::LoadConst(zero_idx));
        builder.add_instruction(Instruction::StoreLocal(0));

        for i in 0..100 {
            let elem1 = builder.add_constant(Value::Int(i));
            let elem2 = builder.add_constant(Value::Int(i + 1));
            builder.add_instruction(Instruction::LoadConst(elem1));
            builder.add_instruction(Instruction::LoadConst(elem2));
            builder.add_instruction(Instruction::MakeTuple(2));
            builder.add_instruction(Instruction::StoreLocal(1));

            builder.add_instruction(Instruction::LoadLocal(1));
            builder.add_instruction(Instruction::CheckTupleLen(2));
            builder.add_instruction(Instruction::JumpIfFalse(7));

            builder.add_instruction(Instruction::LoadLocal(1));
            builder.add_instruction(Instruction::GetTupleElem(0));
            builder.add_instruction(Instruction::LoadLocal(1));
            builder.add_instruction(Instruction::GetTupleElem(1));
            builder.add_instruction(Instruction::Add);
            builder.add_instruction(Instruction::LoadLocal(0));
            builder.add_instruction(Instruction::Add);
            builder.add_instruction(Instruction::StoreLocal(0));
        }

        builder.add_instruction(Instruction::LoadLocal(0));
        builder.add_instruction(Instruction::Return);
        let chunk = builder.build();

        b.iter(|| {
            let mut vm = Vm::new();
            black_box(vm.execute(chunk.clone()).unwrap());
        });
    });
}

fn bench_tight_loop(c: &mut Criterion) {
    let mut group = c.benchmark_group("dispatch/control_flow");

    for iterations in [1000, 10_000, 100_000] {
        group.throughput(Throughput::Elements(iterations as u64));
        group.bench_with_input(
            BenchmarkId::new("tight_loop", iterations),
            &iterations,
            |b, &iterations| {
                let mut builder = ChunkBuilder::new("tight_loop");

                let zero_idx = builder.add_constant(Value::Int(0));
                let one_idx = builder.add_constant(Value::Int(1));
                let limit_idx = builder.add_constant(Value::Int(iterations));

                builder.add_instruction(Instruction::LoadConst(zero_idx));
                builder.add_instruction(Instruction::StoreLocal(0));

                let loop_start = builder.instruction_count();

                builder.add_instruction(Instruction::LoadLocal(0));
                builder.add_instruction(Instruction::LoadConst(limit_idx));
                builder.add_instruction(Instruction::Lt);
                builder.add_instruction(Instruction::JumpIfFalse(6));

                builder.add_instruction(Instruction::LoadLocal(0));
                builder.add_instruction(Instruction::LoadConst(one_idx));
                builder.add_instruction(Instruction::Add);
                builder.add_instruction(Instruction::StoreLocal(0));

                let jump_back = loop_start as i16 - builder.instruction_count() as i16 - 1;
                builder.add_instruction(Instruction::Jump(jump_back));

                builder.add_instruction(Instruction::LoadLocal(0));
                builder.add_instruction(Instruction::Return);
                let chunk = builder.build();

                b.iter(|| {
                    let mut vm = Vm::new();
                    black_box(vm.execute(chunk.clone()).unwrap());
                });
            },
        );
    }

    group.finish();
}

// =============================================================================
// BASELINE: INSTRUCTION DISPATCH OVERHEAD
// =============================================================================

fn bench_dispatch_baseline(c: &mut Criterion) {
    let mut group = c.benchmark_group("dispatch/baseline");

    for size in [1000, 10_000, 100_000] {
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(
            BenchmarkId::new("noop_sequence", size),
            &size,
            |b, &size| {
                let mut builder = ChunkBuilder::new("noop_sequence");
                let val_idx = builder.add_constant(Value::Int(1));

                builder.add_instruction(Instruction::LoadConst(val_idx));
                for _ in 0..size {
                    builder.add_instruction(Instruction::Dup);
                    builder.add_instruction(Instruction::Pop);
                }

                builder.add_instruction(Instruction::Return);
                let chunk = builder.build();

                b.iter(|| {
                    let mut vm = Vm::new();
                    black_box(vm.execute(chunk.clone()).unwrap());
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_arithmetic_add_chain,
    bench_arithmetic_mixed,
    bench_arithmetic_loop_simulation,
    bench_recursive_fibonacci,
    bench_nested_calls,
    bench_call_return_overhead,
    bench_push_pop_intensive,
    bench_local_variable_access,
    bench_dup_operations,
    bench_list_construction,
    bench_list_traversal,
    bench_array_access,
    bench_array_sequential_access,
    bench_conditional_jumps,
    bench_pattern_matching_int,
    bench_pattern_matching_tuple,
    bench_tight_loop,
    bench_dispatch_baseline,
);

criterion_main!(benches);
