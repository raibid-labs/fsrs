// Simple benchmarks using the fusabi library directly
// Tests end-to-end performance with F# scripts

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fusabi::Engine;

fn bench_fib_simple(c: &mut Criterion) {
    c.bench_function("e2e/fib_15", |b| {
        let source = r#"
let rec fib n =
    if n <= 1 then
        n
    else
        fib (n - 1) + fib (n - 2)

fib 15
"#;
        b.iter(|| {
            let mut engine = Engine::new();
            black_box(engine.eval(source).unwrap());
        });
    });
}

fn bench_factorial(c: &mut Criterion) {
    c.bench_function("e2e/factorial_10", |b| {
        let source = r#"
let rec factorial n =
    if n <= 1 then
        1
    else
        n * factorial (n - 1)

factorial 10
"#;
        b.iter(|| {
            let mut engine = Engine::new();
            black_box(engine.eval(source).unwrap());
        });
    });
}

fn bench_ackermann(c: &mut Criterion) {
    c.bench_function("e2e/ackermann_3_6", |b| {
        let source = r#"
let rec ack m n =
    if m = 0 then
        n + 1
    else if n = 0 then
        ack (m - 1) 1
    else
        ack (m - 1) (ack m (n - 1))

ack 3 6
"#;
        b.iter(|| {
            let mut engine = Engine::new();
            black_box(engine.eval(source).unwrap());
        });
    });
}

fn bench_list_sum(c: &mut Criterion) {
    c.bench_function("e2e/list_sum_100", |b| {
        let source = r#"
let rec sum lst =
    match lst with
    | [] -> 0
    | head :: tail -> head + sum tail

let rec range start end_val =
    if start > end_val then
        []
    else
        start :: range (start + 1) end_val

let numbers = range 1 100
sum numbers
"#;
        b.iter(|| {
            let mut engine = Engine::new();
            black_box(engine.eval(source).unwrap());
        });
    });
}

fn bench_array_ops(c: &mut Criterion) {
    c.bench_function("e2e/array_create_and_sum", |b| {
        let source = r#"
let arr = [| 1; 2; 3; 4; 5; 6; 7; 8; 9; 10 |]
let len = 10

let rec sum idx acc =
    if idx >= len then
        acc
    else
        sum (idx + 1) (acc + arr.[idx])

sum 0 0
"#;
        b.iter(|| {
            let mut engine = Engine::new();
            black_box(engine.eval(source).unwrap());
        });
    });
}

criterion_group!(
    benches,
    bench_fib_simple,
    bench_factorial,
    bench_ackermann,
    bench_list_sum,
    bench_array_ops
);
criterion_main!(benches);
