# Workstream 4: Serialization & Performance

## Status
🟡 Ready to Start (parallel with WS1-WS3)

## Overview
Implement bytecode serialization to `.fzb` files for faster startup times, and create a comprehensive benchmarking suite comparing Fusabi against Rhai, Rune, and Lua. This validates Fusabi's "Lua-class performance" claim.

## Objectives
**Bytecode Serialization**:
- [ ] Add serde serialization for `Chunk`, `Instruction`, `Value`
- [ ] Define `.fzb` file format with magic bytes
- [ ] Implement `fus grind` CLI command to compile to bytecode
- [ ] Update `fus run` to auto-detect and load `.fzb` files

**Benchmarking**:
- [ ] Set up Criterion micro-benchmarks for VM operations
- [ ] Create macro-benchmarks (fib, sieve, binary_trees)
- [ ] Implement comparison harness for Rhai, Rune, Lua
- [ ] Integrate benchmarks into CI with regression detection

## Agent Assignment
**Suggested Agent Type**: `performance-engineer`, `rust-pro`, `coder`
**Skill Requirements**: Rust serde, benchmarking, performance analysis, systems programming

## Dependencies
- None (can run in parallel with WS1-WS3)
- Benchmarks benefit from having WS1 complete for HOF benchmarks

## Tasks

### Part A: Bytecode Serialization

#### Task 4A.1: Add Serde to VM Crate
**Description**: Add serde feature and derive Serialize/Deserialize for bytecode structures.

**Deliverables**:
- Add `serde` feature to `fusabi-vm/Cargo.toml`
- Derive `Serialize, Deserialize` for `Chunk`, `Instruction`
- Handle `Value` serialization (skip runtime values like closures)

**Files to Create/Modify**:
- `rust/crates/fusabi-vm/Cargo.toml`
- `rust/crates/fusabi-vm/src/bytecode.rs`
- `rust/crates/fusabi-vm/src/value.rs`

**Implementation**:
```toml
# fusabi-vm/Cargo.toml

[features]
default = []
serde = ["dep:serde", "dep:bincode"]

[dependencies]
serde = { version = "1.0", features = ["derive"], optional = true }
bincode = { version = "1.3", optional = true }
```

```rust
// fusabi-vm/src/bytecode.rs

#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Chunk {
    pub instructions: Vec<Instruction>,
    pub constants: Vec<Value>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Instruction {
    LoadConst(usize),
    LoadGlobal(String),
    StoreGlobal(String),
    // ... all other instructions
}
```

**Challenge**: `Value` contains `Rc<RefCell<T>>` which cannot be serialized directly. **Solution**:
- Only serialize *constant* values (Int, Float, String, Bool, Unit)
- Skip closures, records with runtime data
- Or implement custom serde for `Rc` (serialize the data, not the pointer)

**Validation**:
```bash
cd rust/crates/fusabi-vm
cargo build --features serde
# Should compile
```

---

#### Task 4A.2: Define .fzb File Format
**Description**: Define the `.fzb` bytecode file format with magic bytes and versioning.

**Deliverables**:
- Magic bytes: `FZB\x01` (4 bytes)
- Version field (u8)
- Serialized `Chunk` data (bincode)
- Documentation of file format in `docs/ABI.md`

**Files to Create/Modify**:
- `rust/crates/fusabi-vm/src/lib.rs`
- `docs/ABI.md` (WS6 will complete, but add structure here)

**Implementation**:
```rust
// fusabi-vm/src/lib.rs

pub const FZB_MAGIC: &[u8] = b"FZB\x01";
pub const FZB_VERSION: u8 = 1;

#[cfg(feature = "serde")]
pub fn serialize_chunk(chunk: &Chunk) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut bytes = Vec::new();

    // Write magic bytes
    bytes.extend_from_slice(FZB_MAGIC);

    // Write version
    bytes.push(FZB_VERSION);

    // Serialize chunk with bincode
    let chunk_bytes = bincode::serialize(chunk)?;
    bytes.extend_from_slice(&chunk_bytes);

    Ok(bytes)
}

#[cfg(feature = "serde")]
pub fn deserialize_chunk(bytes: &[u8]) -> Result<Chunk, Box<dyn std::error::Error>> {
    // Check magic bytes
    if !bytes.starts_with(FZB_MAGIC) {
        return Err("Invalid magic bytes".into());
    }

    // Check version
    if bytes[4] != FZB_VERSION {
        return Err(format!("Unsupported version: {}", bytes[4]).into());
    }

    // Deserialize chunk
    let chunk: Chunk = bincode::deserialize(&bytes[5..])?;
    Ok(chunk)
}
```

**Validation**:
```rust
#[test]
fn test_serialize_deserialize_chunk() {
    let chunk = Chunk {
        instructions: vec![Instruction::LoadConst(0), Instruction::Return],
        constants: vec![Value::Int(42)],
        name: Some("test".into()),
    };

    let bytes = serialize_chunk(&chunk).unwrap();
    let restored = deserialize_chunk(&bytes).unwrap();

    assert_eq!(chunk, restored);
}
```

---

#### Task 4A.3: Implement `fus grind` CLI Command
**Description**: Add `grind` subcommand to compile `.fsx` to `.fzb`.

**Deliverables**:
- `fus grind <file.fsx>` command
- Compiles source to bytecode
- Writes to `<file.fzb>`
- Success message with file size

**Files to Create/Modify**:
- `rust/fusabi/src/main.rs`

**Implementation**:
```rust
// fusabi/src/main.rs

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.get(1).map(String::as_str) {
        Some("grind") => {
            if let Some(file) = args.get(2) {
                grind_command(file);
            } else {
                eprintln!("Usage: fus grind <file.fsx>");
            }
        }
        // ... other commands
    }
}

#[cfg(feature = "serde")]
fn grind_command(file_path: &str) {
    // Read source file
    let source = fs::read_to_string(file_path)
        .expect("Failed to read source file");

    // Parse and compile
    let ast = Parser::parse(&source).expect("Parse error");
    let mut compiler = Compiler::new();
    let chunk = compiler.compile_program(&ast).expect("Compile error");

    // Serialize to .fzb
    let bytes = fusabi_vm::serialize_chunk(&chunk).expect("Serialization error");

    // Write to file
    let output_path = file_path.replace(".fsx", ".fzb");
    fs::write(&output_path, &bytes).expect("Failed to write output file");

    println!("Compiled {} ({} bytes) -> {}", file_path, bytes.len(), output_path);
}
```

**Validation**:
```bash
# Create test file
echo 'printfn "Hello, Fusabi!"' > test.fsx

# Compile to bytecode
cargo run -- grind test.fsx
# Output: Compiled test.fsx (123 bytes) -> test.fzb

# Check file exists
ls -lh test.fzb
```

---

#### Task 4A.4: Update `fus run` to Auto-detect .fzb
**Description**: Modify `run` command to detect `.fzb` magic bytes and deserialize instead of parsing.

**Deliverables**:
- Check for magic bytes in file
- If `.fzb`, deserialize and run directly
- If `.fsx`, parse and compile as before
- Performance improvement message

**Files to Create/Modify**:
- `rust/fusabi/src/main.rs`

**Implementation**:
```rust
// fusabi/src/main.rs

fn run_command(file_path: &str) {
    let bytes = fs::read(file_path).expect("Failed to read file");

    let chunk = if bytes.starts_with(fusabi_vm::FZB_MAGIC) {
        // Bytecode file, deserialize directly
        println!("Loading pre-compiled bytecode...");
        fusabi_vm::deserialize_chunk(&bytes).expect("Deserialization error")
    } else {
        // Source file, parse and compile
        let source = String::from_utf8(bytes).expect("Invalid UTF-8");
        let ast = Parser::parse(&source).expect("Parse error");
        let mut compiler = Compiler::new();
        compiler.compile_program(&ast).expect("Compile error")
    };

    // Execute bytecode
    let mut vm = Vm::new();
    vm.execute(&chunk).expect("Runtime error");
}
```

**Validation**:
```bash
# Run .fsx file (parse + compile)
time cargo run -- run test.fsx
# Output: Hello, Fusabi!
# Time: ~50ms (parse + compile + run)

# Run .fzb file (deserialize only)
time cargo run -- run test.fzb
# Output: Loading pre-compiled bytecode...
#         Hello, Fusabi!
# Time: ~10ms (deserialize + run)

# Should be significantly faster!
```

---

### Part B: Benchmarking Suite

#### Task 4B.1: Set Up Criterion Micro-benchmarks
**Description**: Add Criterion benchmarks for core VM operations.

**Deliverables**:
- Add `criterion` dev-dependency
- Benchmark: Instruction dispatch (tight loop of Add/Sub/Mul)
- Benchmark: Value allocation (create 10k records vs tuples)
- Benchmark: Function calls (native vs script)

**Files to Create/Modify**:
- `rust/crates/fusabi-vm/Cargo.toml`
- `rust/crates/fusabi-vm/benches/op_dispatch.rs`
- `rust/crates/fusabi-vm/benches/alloc.rs`
- `rust/crates/fusabi-vm/benches/call.rs`

**Implementation**:
```toml
# fusabi-vm/Cargo.toml

[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }

[[bench]]
name = "op_dispatch"
harness = false

[[bench]]
name = "alloc"
harness = false
```

```rust
// fusabi-vm/benches/op_dispatch.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fusabi_vm::*;

fn bench_arithmetic_ops(c: &mut Criterion) {
    let mut vm = Vm::new();
    let chunk = /* bytecode for: 1 + 2 + 3 + ... + 1000 */;

    c.bench_function("arithmetic_ops", |b| {
        b.iter(|| {
            vm.reset();
            vm.execute(black_box(&chunk)).unwrap();
        });
    });
}

criterion_group!(benches, bench_arithmetic_ops);
criterion_main!(benches);
```

**Validation**:
```bash
cd rust/crates/fusabi-vm
cargo bench
# Should run benchmarks and generate HTML report in target/criterion/
```

---

#### Task 4B.2: Create Macro-benchmarks
**Description**: Implement standard benchmark programs in Fusabi.

**Deliverables**:
- `examples/benchmarks/fib.fsx` - Recursive Fibonacci (tests call overhead)
- `examples/benchmarks/sieve.fsx` - Sieve of Eratosthenes (tests list ops)
- `examples/benchmarks/binary_trees.fsx` - Binary tree allocation (tests GC pressure)

**Files to Create/Modify**:
- `examples/benchmarks/fib.fsx`
- `examples/benchmarks/sieve.fsx`
- `examples/benchmarks/binary_trees.fsx`

**Example**:
```fsharp
// examples/benchmarks/fib.fsx
let rec fib n =
    if n <= 1 then n
    else fib (n - 1) + fib (n - 2)

let result = fib 30
printfn "fib(30) = %d" result
```

```fsharp
// examples/benchmarks/sieve.fsx
let sieve n =
    let rec filter primes candidates =
        match candidates with
        | [] -> primes
        | p :: rest ->
            let filtered = List.filter (fun x -> x % p <> 0) rest
            filter (p :: primes) filtered

    filter [] (List.range 2 n)

let primes = sieve 10000
printfn "Found %d primes" (List.length primes)
```

**Validation**:
```bash
cargo run -- run examples/benchmarks/fib.fsx
# Output: fib(30) = 832040
# (verify correctness)

cargo run -- run examples/benchmarks/sieve.fsx
# Output: Found 1229 primes
# (verify correctness)
```

---

#### Task 4B.3: Implement Comparison Harness
**Description**: Create a benchmark runner that compares Fusabi against Rhai, Rune, and Lua.

**Deliverables**:
- New binary `bench_compare` in workspace
- Implement same benchmarks in Rhai, Rune, Lua
- Run all and print comparison table
- Markdown output for documentation

**Files to Create/Modify**:
- `rust/bench_compare/Cargo.toml` (new crate)
- `rust/bench_compare/src/main.rs`
- `rust/bench_compare/benches/fib.rhai`
- `rust/bench_compare/benches/fib.rune`
- `rust/bench_compare/benches/fib.lua`

**Implementation**:
```toml
# rust/bench_compare/Cargo.toml

[package]
name = "bench_compare"
version = "0.1.0"
edition = "2021"

[dependencies]
fusabi = { path = "../fusabi" }
rhai = "1.16"
rune = "0.13"
mlua = "0.9"
criterion = "0.5"
```

```rust
// rust/bench_compare/src/main.rs

use std::time::Instant;

fn main() {
    println!("| Language | fib(30) | sieve(10k) | binary_trees(10) |");
    println!("|----------|---------|------------|------------------|");

    // Fusabi
    let fib_time = bench_fusabi_fib();
    let sieve_time = bench_fusabi_sieve();
    let trees_time = bench_fusabi_trees();
    println!("| Fusabi   | {:.2}ms | {:.2}ms    | {:.2}ms          |", fib_time, sieve_time, trees_time);

    // Rhai
    let fib_time = bench_rhai_fib();
    let sieve_time = bench_rhai_sieve();
    let trees_time = bench_rhai_trees();
    println!("| Rhai     | {:.2}ms | {:.2}ms    | {:.2}ms          |", fib_time, sieve_time, trees_time);

    // Similar for Rune and Lua
    // ...
}

fn bench_fusabi_fib() -> f64 {
    let source = include_str!("../benches/fib.fsx");
    let start = Instant::now();
    fusabi::run(source).unwrap();
    start.elapsed().as_secs_f64() * 1000.0
}
```

**Validation**:
```bash
cd rust/bench_compare
cargo run --release
# Should print comparison table
```

---

#### Task 4B.4: CI Integration
**Description**: Add benchmarks to CI with performance regression detection.

**Deliverables**:
- GitHub Actions workflow for benchmarks
- Store baseline performance
- Fail CI if performance drops >10%
- Optional: Upload results to GitHub Pages

**Files to Create/Modify**:
- `.github/workflows/benchmark.yml`
- `justfile` (add `benchmark` recipe)

**Implementation**:
```yaml
# .github/workflows/benchmark.yml

name: Benchmark

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Run benchmarks
        run: cargo bench --all

      - name: Compare with baseline
        run: |
          # Logic to compare with stored baseline
          # Fail if >10% regression
```

**Validation**:
```bash
# Test locally
just benchmark
# Should run all benchmarks
```

---

## Definition of Done

**Bytecode Serialization**:
- [ ] Serde feature added to `fusabi-vm`
- [ ] `Chunk` and `Instruction` serializable
- [ ] `.fzb` file format defined with magic bytes
- [ ] `fus grind` command compiles to bytecode
- [ ] `fus run` auto-detects and loads `.fzb` files
- [ ] Serialization tests passing
- [ ] Startup time improvement measured and documented

**Benchmarking**:
- [ ] Criterion micro-benchmarks for VM ops
- [ ] Macro-benchmarks (fib, sieve, binary_trees) in Fusabi
- [ ] Comparison harness for Rhai, Rune, Lua
- [ ] Benchmarks integrated into CI
- [ ] Performance comparison table generated
- [ ] Documentation updated with performance claims
- [ ] PR ready for review

## Agent Coordination Hooks
```bash
# BEFORE Work:
npx claude-flow@alpha hooks pre-task --description "ws4-serialization-performance"
npx claude-flow@alpha hooks session-restore --session-id "swarm-fusabi-gem-ws4"

# DURING Work:
npx claude-flow@alpha hooks post-edit --file "rust/crates/fusabi-vm/src/bytecode.rs" --memory-key "swarm/fusabi-gem/ws4/serde-impl"
npx claude-flow@alpha hooks post-edit --file "rust/fusabi/src/main.rs" --memory-key "swarm/fusabi-gem/ws4/grind-command"
npx claude-flow@alpha hooks post-edit --file "rust/crates/fusabi-vm/benches/op_dispatch.rs" --memory-key "swarm/fusabi-gem/ws4/benchmarks"
npx claude-flow@alpha hooks notify --message "Serialization and benchmarking complete"

# AFTER Work:
npx claude-flow@alpha hooks post-task --task-id "ws4-complete"
npx claude-flow@alpha hooks session-end --export-metrics true
```

## Estimated Effort
**Duration**: 4-5 days (2 days serialization, 2-3 days benchmarking)
**Complexity**: Medium

## References
- [Serde Documentation](https://serde.rs/)
- [Bincode Format](https://github.com/bincode-org/bincode)
- [Criterion Benchmarking](https://bheisler.github.io/criterion.rs/book/)
- [Language Benchmarks Game](https://benchmarksgame-team.pages.debian.net/benchmarksgame/)

## Notes
- **Serialization Challenges**: `Rc` cannot be serialized directly. Solution: Only serialize constant values in `Chunk.constants`, skip runtime-generated closures.
- **Performance**: `.fzb` files should load 5-10x faster than parsing source
- **Benchmarking**: Fusabi should be within 2-3x of Lua performance (the gold standard for embedded scripting)
- **Future Work**:
  - JIT compilation for hot loops
  - Profile-guided optimization
  - SIMD for vector operations

## File Conflicts
- **No conflicts** with other workstreams
- Safe to run in parallel with WS1-WS3
