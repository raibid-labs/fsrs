// Bytecode Compilation API Example
//
// This example demonstrates how to use Fusabi's bytecode compilation API
// for production deployments, caching, and optimized execution.
//
// Usage:
//   cargo run --example bytecode_compilation

use fusabi::{compile_file_to_bytecode, compile_to_bytecode};
use fusabi_vm::{deserialize_chunk, serialize_chunk, Vm};
use std::fs;
use std::path::Path;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Fusabi Bytecode Compilation API Demo ===\n");

    // Example 1: Compile source code to bytecode
    example_1_compile_source()?;

    // Example 2: Save and load bytecode files
    example_2_save_load_bytecode()?;

    // Example 3: Compile file directly to bytecode
    example_3_compile_file()?;

    // Example 4: Performance comparison (source vs bytecode)
    example_4_performance_comparison()?;

    // Example 5: Bytecode caching pattern
    example_5_bytecode_caching()?;

    Ok(())
}

/// Example 1: Compile source code to bytecode
fn example_1_compile_source() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Example 1: Compile Source to Bytecode ---");

    let source = r#"
        let factorial n =
            if n <= 1 then 1
            else n * factorial (n - 1)
        in
        factorial 10
    "#;

    // Compile source to bytecode
    let bytecode = compile_to_bytecode(source)?;
    println!("  Compiled {} bytes of bytecode", bytecode.len());

    // Execute the bytecode
    let chunk = deserialize_chunk(&bytecode)?;
    let mut vm = Vm::new();
    fusabi_vm::stdlib::register_stdlib(&mut vm);

    let result = vm.execute(chunk)?;
    println!("  Result: {:?}", result);
    println!();

    Ok(())
}

/// Example 2: Save and load bytecode files
fn example_2_save_load_bytecode() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Example 2: Save and Load Bytecode Files ---");

    let source = r#"
        let fibonacci n =
            let rec fib_iter a b count =
                if count = 0 then a
                else fib_iter b (a + b) (count - 1)
            in
            fib_iter 0 1 n
        in
        fibonacci 15
    "#;

    // Compile to bytecode
    let bytecode = compile_to_bytecode(source)?;
    println!("  Compiled {} bytes", bytecode.len());

    // Save to .fzb file
    let bytecode_path = "/tmp/fibonacci.fzb";
    fs::write(bytecode_path, &bytecode)?;
    println!("  Saved bytecode to {}", bytecode_path);

    // Load and execute bytecode
    let loaded_bytecode = fs::read(bytecode_path)?;
    let chunk = deserialize_chunk(&loaded_bytecode)?;

    let mut vm = Vm::new();
    fusabi_vm::stdlib::register_stdlib(&mut vm);
    let result = vm.execute(chunk)?;
    println!("  Loaded and executed: {:?}", result);
    println!();

    Ok(())
}

/// Example 3: Compile file directly to bytecode
fn example_3_compile_file() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Example 3: Compile File to Bytecode ---");

    // First, create a temporary source file
    let source_path = "/tmp/example.fsx";
    let source = r#"
        // Example pipeline transformation
        let pipeline_transform data =
            data
            |> List.map (fun x -> x * 2)
            |> List.filter (fun x -> x > 10)
            |> List.fold (fun acc x -> acc + x) 0
        in
        pipeline_transform [1; 2; 3; 4; 5; 6; 7; 8; 9; 10]
    "#;
    fs::write(source_path, source)?;

    // Compile file to bytecode
    let bytecode = compile_file_to_bytecode(source_path)?;
    println!("  Compiled {} bytes from {}", bytecode.len(), source_path);

    // Execute the bytecode
    let chunk = deserialize_chunk(&bytecode)?;
    let mut vm = Vm::new();
    fusabi_vm::stdlib::register_stdlib(&mut vm);
    let result = vm.execute(chunk)?;
    println!("  Result: {:?}", result);
    println!();

    Ok(())
}

/// Example 4: Performance comparison (source vs bytecode)
fn example_4_performance_comparison() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Example 4: Performance Comparison ---");

    let source = r#"
        let sum_squares n =
            let rec loop i acc =
                if i > n then acc
                else loop (i + 1) (acc + i * i)
            in
            loop 1 0
        in
        sum_squares 100
    "#;

    // Compile once
    let compile_start = Instant::now();
    let bytecode = compile_to_bytecode(source)?;
    let compile_time = compile_start.elapsed();
    println!("  Compilation time: {:?}", compile_time);

    // Execute from bytecode 100 times
    let exec_start = Instant::now();
    for _ in 0..100 {
        let chunk = deserialize_chunk(&bytecode)?;
        let mut vm = Vm::new();
        fusabi_vm::stdlib::register_stdlib(&mut vm);
        let _result = vm.execute(chunk)?;
    }
    let exec_time = exec_start.elapsed();
    println!("  100 executions from bytecode: {:?}", exec_time);
    println!("  Average per execution: {:?}", exec_time / 100);
    println!();

    Ok(())
}

/// Example 5: Bytecode caching pattern (production use case)
fn example_5_bytecode_caching() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Example 5: Bytecode Caching Pattern ---");
    println!("  (Hibana-style observability agent pattern)");

    // Simulate a production caching scenario
    let pipeline_configs = vec![
        ("filter_logs", r#"
            let filter_severity logs =
                logs |> List.filter (fun log ->
                    log.severity = "ERROR" || log.severity = "WARN"
                )
            in
            filter_severity
        "#),
        ("aggregate_metrics", r#"
            let aggregate metrics =
                metrics |> List.fold (fun acc m ->
                    acc + m.value
                ) 0.0
            in
            aggregate
        "#),
        ("transform_events", r#"
            let transform events =
                events |> List.map (fun e -> {
                    timestamp = e.time;
                    message = e.msg;
                    level = e.severity
                })
            in
            transform
        "#),
    ];

    // Cache directory
    let cache_dir = "/tmp/fusabi_cache";
    fs::create_dir_all(cache_dir)?;

    for (name, source) in &pipeline_configs {
        let cache_path = format!("{}/{}.fzb", cache_dir, name);

        // Check if cached bytecode exists
        let bytecode = if Path::new(&cache_path).exists() {
            println!("  [CACHE HIT] Loading {} from cache", name);
            fs::read(&cache_path)?
        } else {
            println!("  [CACHE MISS] Compiling {} to bytecode", name);
            let bytecode = compile_to_bytecode(source)?;
            fs::write(&cache_path, &bytecode)?;
            println!("  [CACHED] Saved {} ({} bytes)", name, bytecode.len());
            bytecode
        };

        // Verify bytecode is valid
        let chunk = deserialize_chunk(&bytecode)?;
        println!("  [VERIFIED] {} bytecode is valid ({} instructions)",
                 name, chunk.instructions.len());
    }

    println!("\n  Bytecode cache location: {}", cache_dir);
    println!("  Production pattern: Check cache -> Compile if miss -> Execute");
    println!();

    Ok(())
}

// Additional helper functions for production scenarios

/// Helper: Compile with error handling
#[allow(dead_code)]
fn compile_with_fallback(source: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    match compile_to_bytecode(source) {
        Ok(bytecode) => Ok(bytecode),
        Err(e) => {
            eprintln!("Compilation failed: {}", e);
            // In production, you might:
            // 1. Log the error
            // 2. Return a default/safe bytecode
            // 3. Use a backup configuration
            Err(e.into())
        }
    }
}

/// Helper: Validate bytecode before caching
#[allow(dead_code)]
fn validate_and_cache(
    bytecode: &[u8],
    cache_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Verify bytecode is valid by deserializing
    let chunk = deserialize_chunk(bytecode)?;

    // Additional validation checks
    if chunk.instructions.is_empty() {
        return Err("Empty bytecode chunk".into());
    }

    // Save to cache
    fs::write(cache_path, bytecode)?;
    Ok(())
}

/// Helper: Load bytecode with version checking
#[allow(dead_code)]
fn load_bytecode_safe(path: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let bytecode = fs::read(path)?;

    // Verify magic bytes
    if !bytecode.starts_with(fusabi_vm::FZB_MAGIC) {
        return Err("Invalid bytecode file: bad magic bytes".into());
    }

    // Verify version
    if bytecode.len() < 5 {
        return Err("Invalid bytecode file: too short".into());
    }

    if bytecode[4] != fusabi_vm::FZB_VERSION {
        return Err(format!(
            "Bytecode version mismatch: found {}, expected {}",
            bytecode[4],
            fusabi_vm::FZB_VERSION
        )
        .into());
    }

    Ok(bytecode)
}

/// Helper: Memory-based bytecode cache (for hot paths)
#[allow(dead_code)]
mod memory_cache {
    use std::collections::HashMap;
    use std::sync::{Arc, RwLock};

    pub struct BytecodeCache {
        cache: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    }

    impl BytecodeCache {
        pub fn new() -> Self {
            BytecodeCache {
                cache: Arc::new(RwLock::new(HashMap::new())),
            }
        }

        pub fn get(&self, key: &str) -> Option<Vec<u8>> {
            self.cache.read().ok()?.get(key).cloned()
        }

        pub fn set(&self, key: String, bytecode: Vec<u8>) {
            if let Ok(mut cache) = self.cache.write() {
                cache.insert(key, bytecode);
            }
        }

        pub fn clear(&self) {
            if let Ok(mut cache) = self.cache.write() {
                cache.clear();
            }
        }

        pub fn size(&self) -> usize {
            self.cache.read().ok().map(|c| c.len()).unwrap_or(0)
        }
    }
}

// Production deployment patterns:
//
// 1. Development Mode (Hot Reload):
//    - Keep .fsx files in source control
//    - Load and execute directly (no bytecode)
//    - Fast iteration, slower startup
//
// 2. Production Mode (AOT Compilation):
//    - Compile .fsx to .fzb during build/deployment
//    - Ship only .fzb files
//    - Faster startup, optimized execution
//
// 3. Hybrid Mode (JIT Cache):
//    - Ship .fsx files
//    - Compile to bytecode on first run
//    - Cache bytecode in memory or disk
//    - Best of both: flexibility + performance
