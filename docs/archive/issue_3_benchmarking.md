# Issue 3: [Benchmark] Create Comprehensive Benchmarking Suite

**Labels:** `infrastructure`, `performance`

## Context
We need to measure Fusabi against Rhai, Rune, and Lua to track performance regressions and verify our "Lua-class" claims.

## Implementation Plan
**Objective:** Set up `criterion` and `hyperfine` benchmarks.

1.  **Micro-benchmarks** (`rust/crates/fusabi-vm/benches/`):
    * Add `criterion` dev-dependency.
    * Create `op_dispatch.rs`: Benchmark a tight loop of `Add`, `Sub`, `Call`.
    * Create `alloc.rs`: Benchmark creating 10k Records vs 10k Tuples.

2.  **Macro-benchmarks** (`examples/benchmarks/`):
    * Create `fib.fsx` (Recursion test).
    * Create `sieve.fsx` (List/Array manipulation).
    * Create `binary_trees.fsx` (GC pressure).

3.  **Comparison Harness**:
    * Create a `bench_compare` binary in the workspace.
    * Implement the *exact same* `fib` and `sieve` algorithms in Rhai and Rune (add them as deps).
    * Run all three and print a markdown table of results.

4.  **CI Integration**:
    * Add a `just benchmark` recipe that runs these and fails if performance drops >10% (optional).
