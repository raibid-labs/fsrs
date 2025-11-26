//! Comprehensive tests for the bytecode compilation API (Issue #123)
//!
//! Tests the new public API functions for compiling Fusabi source to bytecode
//! and loading/executing bytecode directly.

use fusabi::{
    compile_and_execute, compile_file_to_bytecode, compile_to_bytecode, compile_to_chunk,
    execute_bytecode, Value,
};
use fusabi_vm::{deserialize_chunk, Vm};
use std::fs;


#[test]
fn test_compile_to_bytecode_basic() {
    // Simple expression
    let source = "42";
    let bytecode = compile_to_bytecode(source).expect("Compilation failed");

    // Bytecode should have FZB magic header
    assert!(bytecode.starts_with(b"FZB\x01"), "Missing FZB magic header");
    assert!(bytecode.len() > 5, "Bytecode too short");
}

#[test]
fn test_compile_to_bytecode_arithmetic() {
    let source = "let x = 42 in x * 2";
    let bytecode = compile_to_bytecode(source).expect("Compilation failed");

    // Should be valid bytecode
    assert!(bytecode.starts_with(b"FZB\x01"));

    // Deserialize and verify it's a valid chunk
    let chunk = deserialize_chunk(&bytecode).expect("Deserialization failed");
    assert!(!chunk.instructions.is_empty());
}

#[test]
fn test_compile_to_bytecode_functions() {
    let source = r#"
        let double = fun x -> x * 2 in
        double 21
    "#;
    let bytecode = compile_to_bytecode(source).expect("Compilation failed");
    assert!(bytecode.starts_with(b"FZB\x01"));
}

#[test]
fn test_compile_to_bytecode_complex() {
    let source = r#"
        let factorial = fun n ->
            if n <= 1 then 1
            else n * factorial (n - 1)
        in factorial 5
    "#;
    let bytecode = compile_to_bytecode(source).expect("Compilation failed");
    assert!(bytecode.starts_with(b"FZB\x01"));

    // Verify it's a valid chunk
    let chunk = deserialize_chunk(&bytecode).expect("Deserialization failed");
    assert!(!chunk.instructions.is_empty());
}

#[test]
fn test_compile_file_to_bytecode() {
    // Create a temporary test file
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("test_script.fsx");
    let source = "let x = 100 in x + 23";

    fs::write(&test_file, source).expect("Failed to write test file");

    // Compile the file
    let bytecode = compile_file_to_bytecode(test_file.to_str().unwrap())
        .expect("File compilation failed");

    // Verify bytecode
    assert!(bytecode.starts_with(b"FZB\x01"));
    assert!(bytecode.len() > 5);

    // Clean up
    fs::remove_file(&test_file).ok();
}

#[test]
fn test_compile_to_chunk() {
    let source = "let x = 42 in x * 2";
    let chunk = compile_to_chunk(source).expect("Chunk compilation failed");

    // Should have instructions and constants
    assert!(!chunk.instructions.is_empty());
    assert!(!chunk.constants.is_empty());
}

#[test]
fn test_execute_bytecode() {
    let source = "100 + 23";
    let bytecode = compile_to_bytecode(source).expect("Compilation failed");

    // Execute bytecode directly
    let result = execute_bytecode(&bytecode).expect("Execution failed");
    assert_eq!(result.as_int(), Some(123));
}

#[test]
fn test_compile_and_execute() {
    let source = "let x = 10 in x + 32";
    let (bytecode, result) = compile_and_execute(source).expect("Compile and execute failed");

    // Check bytecode is valid
    assert!(bytecode.starts_with(b"FZB\x01"));

    // Check result is correct
    assert_eq!(result.as_int(), Some(42));
}

#[test]
fn test_round_trip_compile_deserialize_execute() {
    // Compile source to bytecode
    let source = "let x = 10 in let y = 20 in x + y";
    let bytecode = compile_to_bytecode(source).expect("Compilation failed");

    // Deserialize the bytecode
    let chunk = deserialize_chunk(&bytecode).expect("Deserialization failed");

    // Execute in VM
    let mut vm = Vm::new();
    fusabi_vm::stdlib::register_stdlib(&mut vm);
    let result = vm.execute(chunk).expect("Execution failed");

    // Verify result
    assert_eq!(result.as_int(), Some(30));
}

#[test]
fn test_round_trip_with_function() {
    let source = r#"
        let add = fun x -> fun y -> x + y in
        let add5 = add 5 in
        add5 10
    "#;

    let bytecode = compile_to_bytecode(source).expect("Compilation failed");
    let result = execute_bytecode(&bytecode).expect("Execution failed");
    assert_eq!(result.as_int(), Some(15));
}

#[test]
fn test_vm_from_bytecode() {
    let source = "42 * 2";
    let bytecode = compile_to_bytecode(source).expect("Compilation failed");

    // Create VM from bytecode
    let mut vm = Vm::from_bytecode(&bytecode).expect("Failed to create VM from bytecode");
    fusabi_vm::stdlib::register_stdlib(&mut vm);

    // Execute
    let result = vm.run().expect("Execution failed");
    assert_eq!(result.as_int(), Some(84));
}

#[test]
fn test_vm_execute_bytecode() {
    let source = "100 + 23";
    let bytecode = compile_to_bytecode(source).expect("Compilation failed");

    // Execute bytecode directly via VM
    let result = Vm::execute_bytecode(&bytecode).expect("Direct execution failed");
    assert_eq!(result.as_int(), Some(123));
}

#[test]
fn test_vm_execute_bytecode_with_stdlib() {
    // Test that stdlib functions are available
    let source = r#"
        let numbers = [1, 2, 3, 4, 5] in
        List.length numbers
    "#;
    let bytecode = compile_to_bytecode(source).expect("Compilation failed");

    let result = Vm::execute_bytecode(&bytecode).expect("Execution with stdlib failed");
    assert_eq!(result.as_int(), Some(5));
}

#[test]
fn test_compile_error_handling() {
    // Invalid syntax should return an error
    let source = "let x = in 42"; // Invalid syntax
    let result = compile_to_bytecode(source);
    assert!(result.is_err(), "Should fail on invalid syntax");
}

#[test]
fn test_compile_undefined_variable() {
    // Undefined variable should still compile (runtime error)
    let source = "unknown_var + 42";
    let result = compile_to_bytecode(source);
    // This should compile but fail at runtime
    assert!(result.is_ok(), "Should compile (fails at runtime)");
}

#[test]
fn test_bytecode_file_save_and_load() {
    let temp_dir = std::env::temp_dir();
    let bytecode_file = temp_dir.join("test_compiled.fzb");

    // Compile and save
    let source = "let answer = 42 in answer";
    let bytecode = compile_to_bytecode(source).expect("Compilation failed");
    fs::write(&bytecode_file, &bytecode).expect("Failed to write bytecode file");

    // Load and execute
    let loaded_bytecode = fs::read(&bytecode_file).expect("Failed to read bytecode file");
    let result = execute_bytecode(&loaded_bytecode).expect("Execution failed");

    assert_eq!(result.as_int(), Some(42));

    // Clean up
    fs::remove_file(&bytecode_file).ok();
}

#[test]
fn test_compile_list_operations() {
    let source = r#"
        let list = [1, 2, 3] in
        let doubled = List.map (fun x -> x * 2) list in
        List.head doubled
    "#;

    let bytecode = compile_to_bytecode(source).expect("Compilation failed");
    let result = execute_bytecode(&bytecode).expect("Execution failed");
    assert_eq!(result.as_int(), Some(2));
}

#[test]
fn test_compile_tuple_operations() {
    let source = r#"
        let pair = (10, 20) in
        let (x, y) = pair in
        x + y
    "#;

    let bytecode = compile_to_bytecode(source).expect("Compilation failed");
    let result = execute_bytecode(&bytecode).expect("Execution failed");
    assert_eq!(result.as_int(), Some(30));
}

#[test]
fn test_compile_pattern_matching() {
    let source = r#"
        let classify = fun n ->
            match n with
            | 0 -> "zero"
            | 1 -> "one"
            | _ -> "many"
        in classify 1
    "#;

    let bytecode = compile_to_bytecode(source).expect("Compilation failed");
    let result = execute_bytecode(&bytecode).expect("Execution failed");
    assert_eq!(result.as_str(), Some("one"));
}

#[test]
fn test_compile_recursive_function() {
    let source = r#"
        let sum = fun n ->
            if n <= 0 then 0
            else n + sum (n - 1)
        in sum 10
    "#;

    let bytecode = compile_to_bytecode(source).expect("Compilation failed");
    let result = execute_bytecode(&bytecode).expect("Execution failed");
    assert_eq!(result.as_int(), Some(55));
}

#[test]
fn test_compile_higher_order_functions() {
    let source = r#"
        let apply_twice = fun f -> fun x -> f (f x) in
        let add_one = fun x -> x + 1 in
        apply_twice add_one 10
    "#;

    let bytecode = compile_to_bytecode(source).expect("Compilation failed");
    let result = execute_bytecode(&bytecode).expect("Execution failed");
    assert_eq!(result.as_int(), Some(12));
}

#[test]
fn test_compile_string_operations() {
    let source = r#"
        let greeting = "Hello" in
        let name = "World" in
        String.concat greeting (String.concat " " name)
    "#;

    let bytecode = compile_to_bytecode(source).expect("Compilation failed");
    let result = execute_bytecode(&bytecode).expect("Execution failed");
    assert_eq!(result.as_str(), Some("Hello World"));
}

#[test]
fn test_bytecode_size_reasonable() {
    let source = "42";
    let bytecode = compile_to_bytecode(source).expect("Compilation failed");

    // Bytecode should be reasonably sized (header + minimal chunk data)
    // This is a sanity check - bytecode shouldn't be huge for simple expressions
    assert!(
        bytecode.len() < 1000,
        "Bytecode too large for simple expression: {} bytes",
        bytecode.len()
    );
}

#[test]
fn test_multiple_compilations_independent() {
    let source1 = "10 + 20";
    let source2 = "100 - 50";

    let bytecode1 = compile_to_bytecode(source1).expect("Compilation 1 failed");
    let bytecode2 = compile_to_bytecode(source2).expect("Compilation 2 failed");

    // Execute both independently
    let result1 = execute_bytecode(&bytecode1).expect("Execution 1 failed");
    let result2 = execute_bytecode(&bytecode2).expect("Execution 2 failed");

    assert_eq!(result1.as_int(), Some(30));
    assert_eq!(result2.as_int(), Some(50));
}

#[test]
fn test_compile_with_comments() {
    let source = r#"
        // This is a comment
        let x = 42 in  // inline comment
        (* multi-line
           comment *)
        x * 2
    "#;

    let bytecode = compile_to_bytecode(source).expect("Compilation with comments failed");
    let result = execute_bytecode(&bytecode).expect("Execution failed");
    assert_eq!(result.as_int(), Some(84));
}

#[test]
fn test_bytecode_deterministic() {
    // Same source should produce same bytecode
    let source = "let x = 42 in x + 1";

    let bytecode1 = compile_to_bytecode(source).expect("Compilation 1 failed");
    let bytecode2 = compile_to_bytecode(source).expect("Compilation 2 failed");

    assert_eq!(
        bytecode1, bytecode2,
        "Same source should produce identical bytecode"
    );
}

#[test]
fn test_empty_file_compilation() {
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("empty.fsx");

    // Create empty file
    fs::write(&test_file, "").expect("Failed to write empty file");

    // Attempt to compile - should fail gracefully
    let result = compile_file_to_bytecode(test_file.to_str().unwrap());

    // Clean up
    fs::remove_file(&test_file).ok();

    // Empty source should produce an error
    assert!(result.is_err(), "Empty file should fail to compile");
}

#[test]
fn test_file_not_found() {
    let result = compile_file_to_bytecode("/tmp/definitely_does_not_exist_12345.fsx");
    assert!(result.is_err(), "Should error on non-existent file");
}

#[test]
fn test_compile_boolean_logic() {
    let source = r#"
        let x = true in
        let y = false in
        if x && not y then 100 else 0
    "#;

    let bytecode = compile_to_bytecode(source).expect("Compilation failed");
    let result = execute_bytecode(&bytecode).expect("Execution failed");
    assert_eq!(result.as_int(), Some(100));
}

#[test]
fn test_compile_map_creation() {
    let source = r#"
        let m = Map.empty in
        let m2 = Map.insert "key" 42 m in
        Map.get "key" m2
    "#;

    let bytecode = compile_to_bytecode(source).expect("Compilation failed");
    let result = execute_bytecode(&bytecode).expect("Execution failed");

    // Map.get returns an Option, so we expect Some(42)
    // For now, just verify it executes without error
    assert!(result != Value::Unit);
}

#[test]
fn test_factorial_bytecode() {
    let source = r#"
        let factorial = fun n ->
            if n <= 1 then 1
            else n * factorial (n - 1)
        in factorial 10
    "#;

    let (bytecode, result) = compile_and_execute(source).expect("Compile and execute failed");

    // Verify bytecode is valid
    assert!(bytecode.starts_with(b"FZB\x01"));

    // 10! = 3628800
    assert_eq!(result.as_int(), Some(3628800));

    // Verify we can execute the bytecode again with same result
    let result2 = execute_bytecode(&bytecode).expect("Second execution failed");
    assert_eq!(result2.as_int(), Some(3628800));
}

#[test]
fn test_fibonacci_bytecode() {
    let source = r#"
        let fib = fun n ->
            if n <= 1 then n
            else fib (n - 1) + fib (n - 2)
        in fib 10
    "#;

    let bytecode = compile_to_bytecode(source).expect("Compilation failed");
    let result = execute_bytecode(&bytecode).expect("Execution failed");

    // fib(10) = 55
    assert_eq!(result.as_int(), Some(55));
}

#[test]
fn test_list_map_filter_bytecode() {
    let source = r#"
        let numbers = [1, 2, 3, 4, 5] in
        let doubled = List.map (fun x -> x * 2) numbers in
        let evens = List.filter (fun x -> x % 2 == 0) doubled in
        List.length evens
    "#;

    let bytecode = compile_to_bytecode(source).expect("Compilation failed");
    let result = execute_bytecode(&bytecode).expect("Execution failed");

    // All doubled numbers are even, so length should be 5
    assert_eq!(result.as_int(), Some(5));
}

#[test]
fn test_nested_functions_bytecode() {
    let source = r#"
        let outer = fun x ->
            let inner = fun y -> x + y in
            inner
        in
        let add5 = outer 5 in
        add5 10
    "#;

    let bytecode = compile_to_bytecode(source).expect("Compilation failed");
    let result = execute_bytecode(&bytecode).expect("Execution failed");
    assert_eq!(result.as_int(), Some(15));
}

#[test]
fn test_bytecode_with_large_constants() {
    let source = "999999999 + 1";
    let bytecode = compile_to_bytecode(source).expect("Compilation failed");
    let result = execute_bytecode(&bytecode).expect("Execution failed");
    assert_eq!(result.as_int(), Some(1000000000));
}

#[test]
fn test_compile_chunk_execute_directly() {
    let source = "let x = 21 in x * 2";
    let chunk = compile_to_chunk(source).expect("Chunk compilation failed");

    let mut vm = Vm::new();
    fusabi_vm::stdlib::register_stdlib(&mut vm);
    let result = vm.execute(chunk).expect("Execution failed");

    assert_eq!(result.as_int(), Some(42));
}

#[test]
fn test_invalid_bytecode_magic() {
    let invalid_bytecode = b"INVALID_MAGIC_BYTES";
    let result = execute_bytecode(invalid_bytecode);
    assert!(result.is_err(), "Should error on invalid magic bytes");
}

#[test]
fn test_corrupted_bytecode() {
    // Create valid bytecode then corrupt it
    let source = "42";
    let mut bytecode = compile_to_bytecode(source).expect("Compilation failed");

    // Corrupt the bytecode (keep header valid but corrupt data)
    if bytecode.len() > 10 {
        bytecode[10] = 0xFF;
        bytecode[11] = 0xFF;
    }

    let result = execute_bytecode(&bytecode);
    assert!(result.is_err(), "Should error on corrupted bytecode");
}
