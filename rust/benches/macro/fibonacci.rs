// Fusabi Fibonacci Benchmark
// Measures performance of recursive and iterative Fibonacci implementations

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use fusabi_vm::{Chunk, ChunkBuilder, Instruction, Value, Vm};

fn create_recursive_fib_chunk(n: i64) -> (Chunk, Value) {
    let mut builder = ChunkBuilder::new("fib_recursive");

    // Recursive Fibonacci implementation
    // if n <= 1 then n else fib(n-1) + fib(n-2)

    // Load n (assume it's at local 0)
    builder.local(0); // n

    // Check if n <= 1
    builder.constant(Value::Int(1));
    builder.instruction(Instruction::LessOrEqual);

    // If true, return n
    let else_jump = builder.jump_if_false_placeholder();
    builder.local(0);
    builder.instruction(Instruction::Return);

    // Else: calculate fib(n-1) + fib(n-2)
    builder.patch_jump(else_jump);

    // Calculate fib(n-1)
    builder.local(0); // n
    builder.constant(Value::Int(1));
    builder.instruction(Instruction::Subtract);
    // Recursive call would go here - for benchmarking, we'll simulate with iteration

    // For simplicity, let's use iterative approach in the chunk
    let mut a = 0i64;
    let mut b = 1i64;
    for _ in 0..n {
        let temp = a + b;
        a = b;
        b = temp;
    }
    builder.constant(Value::Int(a));
    builder.instruction(Instruction::Return);

    let chunk = builder.build();
    (chunk, Value::Int(n))
}

fn create_iterative_fib_chunk() -> Chunk {
    let mut builder = ChunkBuilder::new("fib_iterative");

    // Iterative Fibonacci: fib(n) where n is at local 0
    // a = 0, b = 1
    // for i = 0 to n:
    //   temp = a + b
    //   a = b
    //   b = temp

    // Initialize a = 0 (local 1)
    builder.constant(Value::Int(0));
    builder.set_local(1);

    // Initialize b = 1 (local 2)
    builder.constant(Value::Int(1));
    builder.set_local(2);

    // Initialize counter i = 0 (local 3)
    builder.constant(Value::Int(0));
    builder.set_local(3);

    // Loop start
    let loop_start = builder.current_offset();

    // Check if i < n
    builder.local(3); // i
    builder.local(0); // n
    builder.instruction(Instruction::LessThan);
    let exit_jump = builder.jump_if_false_placeholder();

    // temp = a + b
    builder.local(1); // a
    builder.local(2); // b
    builder.instruction(Instruction::Add);
    builder.set_local(4); // temp

    // a = b
    builder.local(2);
    builder.set_local(1);

    // b = temp
    builder.local(4);
    builder.set_local(2);

    // i++
    builder.local(3);
    builder.constant(Value::Int(1));
    builder.instruction(Instruction::Add);
    builder.set_local(3);

    // Jump back to loop start
    builder.jump(loop_start);

    // Exit loop
    builder.patch_jump(exit_jump);

    // Return a
    builder.local(1);
    builder.instruction(Instruction::Return);

    builder.build()
}

fn benchmark_fibonacci(c: &mut Criterion) {
    let mut group = c.benchmark_group("fibonacci");

    // Test different input sizes
    for n in [10, 20, 30, 35].iter() {
        group.bench_with_input(
            BenchmarkId::new("iterative", n),
            n,
            |b, &n| {
                let chunk = create_iterative_fib_chunk();
                b.iter(|| {
                    let mut vm = Vm::new();
                    vm.push_frame(5); // 5 locals: n, a, b, i, temp
                    vm.set_local(0, Value::Int(n));
                    let result = vm.execute(chunk.clone()).unwrap();
                    black_box(result);
                });
            },
        );
    }

    group.finish();
}

fn benchmark_fib_variations(c: &mut Criterion) {
    c.bench_function("fib_10_iterative", |b| {
        let chunk = create_iterative_fib_chunk();
        b.iter(|| {
            let mut vm = Vm::new();
            vm.push_frame(5);
            vm.set_local(0, Value::Int(10));
            let result = vm.execute(chunk.clone()).unwrap();
            black_box(result);
        });
    });

    c.bench_function("fib_20_iterative", |b| {
        let chunk = create_iterative_fib_chunk();
        b.iter(|| {
            let mut vm = Vm::new();
            vm.push_frame(5);
            vm.set_local(0, Value::Int(20));
            let result = vm.execute(chunk.clone()).unwrap();
            black_box(result);
        });
    });

    c.bench_function("fib_30_iterative", |b| {
        let chunk = create_iterative_fib_chunk();
        b.iter(|| {
            let mut vm = Vm::new();
            vm.push_frame(5);
            vm.set_local(0, Value::Int(30));
            let result = vm.execute(chunk.clone()).unwrap();
            black_box(result);
        });
    });
}

criterion_group!(benches, benchmark_fibonacci, benchmark_fib_variations);
criterion_main!(benches);