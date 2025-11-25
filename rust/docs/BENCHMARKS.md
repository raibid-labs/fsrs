# Fusabi Benchmarking Suite

This document describes the comprehensive benchmarking suite for Fusabi, including how to run benchmarks, interpret results, and performance characteristics.

## Overview

The Fusabi benchmarking suite consists of three main components:

1. **Micro-benchmarks**: Low-level VM performance tests
2. **Macro-benchmarks**: Real-world F# script benchmarks
3. **Comparison benchmarks**: Fusabi vs other embedded languages (Rhai, Lua)

## Running Benchmarks

### Prerequisites

- Rust toolchain (1.70+)
- `just` command runner (optional, but recommended)
- Criterion.rs (automatically installed as dev-dependency)

### Quick Start

```bash
# Run all benchmarks
just benchmark

# Run only comparison benchmarks (fastest)
just benchmark-quick

# Run micro-benchmarks only
just benchmark-micro

# Run specific micro-benchmark
just benchmark-ops     # Operation dispatch
just benchmark-alloc   # Memory allocation
just benchmark-calls   # Function calls

# Run macro-benchmarks
just benchmark-macro

# Run comparison benchmarks
just benchmark-compare
```

### Without Just

```bash
# Micro-benchmarks
cargo bench -p fusabi-vm

# Specific benchmark
cargo bench -p fusabi-vm --bench op_dispatch

# Comparison benchmarks
cargo run --release -p fusabi-bench-compare

# Macro-benchmarks
cargo run --release --bin fusabi -- examples/benchmarks/fib.fsx
```

## Benchmark Categories

### 1. Micro-Benchmarks (`crates/fusabi-vm/benches/`)

These benchmarks test low-level VM performance:

#### Operation Dispatch (`op_dispatch.rs`)

Tests the performance of instruction dispatch and execution:

- **add_1000**: Chain of 1000 integer additions
- **sub_1000**: Chain of 1000 integer subtractions
- **mul_1000**: Chain of 1000 integer multiplications
- **comparison_1000**: Chain of 1000 comparison operations
- **call_100**: 100 function calls to a simple function
- **mixed_ops_1000**: Mix of different operations

**Key Metric**: Instructions per second

#### Memory Allocation (`alloc.rs`)

Tests allocation performance for various data structures:

- **records_10k**: Allocate 10,000 records
- **arrays_10k**: Allocate 10,000 arrays
- **closures_10k**: Allocate 10,000 closures
- **lists_10k**: Allocate 10,000 lists
- **tuples_10k**: Allocate 10,000 tuples
- **strings_10k**: Allocate 10,000 strings
- **mixed_10k**: Mix of different allocations

**Key Metric**: Allocations per second, GC pressure

#### Function Calls (`function_calls.rs`)

Tests function call overhead and recursion:

- **native_1000**: 1000 calls to native Rust function
- **native_with_args_1000**: 1000 calls to native function with arguments
- **closure_1000**: 1000 calls to a closure
- **closure_with_args_1000**: 1000 calls to closure with arguments
- **recursive_factorial_10**: 100 calls to factorial(10)
- **tail_call_1000**: Tail-recursive countdown from 1000

**Key Metric**: Call overhead, recursion performance

### 2. Macro-Benchmarks (`examples/benchmarks/`)

Real-world F# scripts testing combined performance:

#### Fibonacci (`fib.fsx`)

Recursive Fibonacci calculation (fib 25).

**Tests**: Function call overhead, recursion, stack management

```fsharp
let rec fib n =
    if n <= 1 then n
    else fib (n - 1) + fib (n - 2)

fib 25
```

#### Sieve of Eratosthenes (`sieve.fsx`)

Find all prime numbers up to 1000.

**Tests**: List operations, filtering, recursion

```fsharp
let rec sieve lst =
    match lst with
    | [] -> []
    | prime :: rest ->
        let is_not_multiple n = n % prime <> 0
        prime :: sieve (filter is_not_multiple rest)
```

#### Binary Trees (`binary_trees.fsx`)

Create and traverse binary trees of depth 10.

**Tests**: Allocation pressure, GC performance, discriminated unions

```fsharp
type Tree =
    | Leaf of int
    | Node of Tree * int * Tree
```

#### Takeuchi Function (`tak.fsx`)

Deep recursive function with multiple recursive calls.

**Tests**: Deep recursion, call stack performance

```fsharp
let rec tak x y z =
    if y >= x then z
    else tak (tak (x-1) y z) (tak (y-1) z x) (tak (z-1) x y)
```

### 3. Comparison Benchmarks (`crates/fusabi-bench-compare/`)

Compare Fusabi against other embedded scripting languages:

- **Rhai**: Rust-native scripting language
- **Lua**: Industry-standard embedded language (via mlua)

**Benchmarks**:

1. **Fibonacci (20)**: Recursive function performance
2. **Sieve (100)**: List/array operations
3. **Ackermann (3, 7)**: Deep recursion
4. **Array Operations (1000)**: Array creation and manipulation

## Interpreting Results

### Criterion Output

Criterion provides detailed statistics:

- **Time**: Mean execution time with confidence intervals
- **Thrpt**: Throughput (operations per second)
- **R²**: Goodness of fit (closer to 1.0 is better)
- **Outliers**: Number of outlier measurements

Example output:
```
op_dispatch/add_1000    time:   [12.345 µs 12.456 µs 12.567 µs]
                        change: [-2.3% -1.5% -0.7%] (p = 0.00 < 0.05)
                        Performance has improved.
```

### Comparison Results

The comparison harness outputs a table showing relative performance:

```
╔════════════════════════════════════════════════════════════════╗
║                     Summary Table                             ║
╠════════════════════════════════╦═══════════╦═══════════╦═══════════╣
║ Benchmark                      ║  Fusabi   ║   Rhai    ║    Lua    ║
╠════════════════════════════════╬═══════════╬═══════════╬═══════════╣
║ Fibonacci (fib 20)             ║   2.45 ms ║   3.12 ms ║   1.89 ms ║
║ Sieve (primes to 100)          ║   5.67 ms ║   7.23 ms ║   4.21 ms ║
...
```

**Relative Performance** is shown as multiplier (Fusabi = 1.00x):
- < 1.00x: Slower than Fusabi
- > 1.00x: Faster than Fusabi

## Performance Characteristics

### Expected Performance

Based on VM architecture, expected performance characteristics:

**Strengths**:
- Stack-based bytecode execution
- Efficient instruction dispatch
- Low function call overhead
- Good closure performance

**Trade-offs**:
- Interpreted bytecode (not JIT compiled)
- Reference counting GC (predictable but not the fastest)
- Immutable data structures (safer but may have copy overhead)

### Typical Results

Fusabi typically performs:
- **vs Rhai**: Competitive (both are Rust-native VMs)
- **vs Lua**: Slower than LuaJIT, competitive with standard Lua
- **Recursion**: Good performance due to tail-call optimization
- **Allocation**: Fast for small objects, good GC performance

## Regression Testing

### Baseline Files

Criterion saves baseline measurements in `target/criterion/`.

To save a baseline:
```bash
cargo bench -p fusabi-vm -- --save-baseline my-baseline
```

To compare against a baseline:
```bash
cargo bench -p fusabi-vm -- --baseline my-baseline
```

### CI Integration (Optional)

To fail CI on performance regression > 10%:

```bash
cargo bench -p fusabi-vm -- --baseline main
# Parse output and fail if any benchmark regressed > 10%
```

## Contributing

When making performance-sensitive changes:

1. Run benchmarks before and after changes
2. Save a baseline before changes: `cargo bench -- --save-baseline before`
3. Make your changes
4. Compare: `cargo bench -- --baseline before`
5. Include benchmark results in PR description

### Adding New Benchmarks

**Micro-benchmark**:
1. Add to appropriate file in `crates/fusabi-vm/benches/`
2. Follow criterion patterns (use `black_box`)
3. Update `[[bench]]` section in `Cargo.toml` if new file

**Macro-benchmark**:
1. Add `.fsx` file to `examples/benchmarks/`
2. Update `justfile` if needed
3. Document expected behavior

**Comparison benchmark**:
1. Add implementations to `crates/fusabi-bench-compare/src/benchmarks.rs`
2. Add to main harness in `main.rs`
3. Ensure equivalent algorithms across all languages

## Troubleshooting

### Benchmarks Take Too Long

- Use `just benchmark-quick` for faster results
- Reduce iterations in comparison harness
- Run specific benchmarks only

### Inconsistent Results

- Close other applications
- Run on AC power (not battery)
- Increase warmup iterations
- Check for thermal throttling

### Comparison Failures

- Ensure all dependencies are installed
- Check Lua vendored feature is enabled
- Verify syntax is correct for each language

## References

- [Criterion.rs Documentation](https://bheisler.github.io/criterion.rs/book/)
- [Rhai Documentation](https://rhai.rs/)
- [MLua Documentation](https://docs.rs/mlua/)
- [Computer Language Benchmarks Game](https://benchmarksgame-team.pages.debian.net/benchmarksgame/)
