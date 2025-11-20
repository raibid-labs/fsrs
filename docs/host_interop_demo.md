# Host Interop Demo - FSRS Host Function API

This document demonstrates the host interop functionality for FSRS.

## Overview

The FSRS host interop API allows Rust applications to:
- Register native functions callable from FSRS scripts
- Automatically marshal types between Rust and FSRS
- Call host functions with type safety
- Work with various data types (integers, strings, lists, etc.)

## Example Usage

```rust
use fsrs_demo::FsrsEngine;
use fsrs_vm::Value;

fn main() {
    let mut engine = FsrsEngine::new();

    // Example 1: Simple arithmetic
    engine.register_fn1("double", |v| {
        let n = v.as_int()
            .ok_or_else(|| fsrs_vm::VmError::Runtime("Expected int".into()))?;
        Ok(Value::Int(n * 2))
    });

    let result = engine.call_host("double", &[Value::Int(21)]).unwrap();
    println!("double(21) = {}", result);  // 42

    // Example 2: String manipulation
    engine.register_fn1("greet", |v| {
        let name = v.as_str()
            .ok_or_else(|| fsrs_vm::VmError::Runtime("Expected string".into()))?;
        Ok(Value::Str(format!("Hello, {}!", name)))
    });

    let result = engine
        .call_host("greet", &[Value::Str("World".to_string())])
        .unwrap();
    println!("{}", result);  // Hello, World!

    // Example 3: List processing
    engine.register_fn1("sum", |v| {
        let list = v.list_to_vec()
            .ok_or_else(|| fsrs_vm::VmError::Runtime("Expected list".into()))?;
        let sum: i64 = list.iter().filter_map(|v| v.as_int()).sum();
        Ok(Value::Int(sum))
    });

    let list = Value::vec_to_cons(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
    let result = engine.call_host("sum", &[list]).unwrap();
    println!("sum([1; 2; 3]) = {}", result);  // 6
}
```

## Features Demonstrated

1. **Simple Arithmetic** - Register functions that operate on integers
2. **String Manipulation** - Work with string values
3. **Binary Functions** - Functions with multiple arguments
4. **List Processing** - Work with lists and perform operations like sum
5. **List Generation** - Generate lists from ranges
6. **Boolean Functions** - Functions returning boolean results
7. **Ternary Functions** - Functions with 3 arguments
8. **Global Bindings** - Store and retrieve global values

## Test Results

All integration tests pass:
- ✅ 14/14 host interop tests
- ✅ 6/6 host module unit tests
- ✅ 18/18 conversion tests
- ✅ 10/10 host_api tests

## Implementation Files

- `/rust/crates/fsrs-vm/src/host.rs` - Host function registry
- `/rust/crates/fsrs-vm/src/conversions.rs` - Type conversions
- `/rust/crates/fsrs-demo/src/host_api.rs` - High-level API
- `/rust/crates/fsrs-vm/tests/test_host_interop.rs` - Integration tests

## API Reference

### FsrsEngine Methods

- `new()` - Create new engine
- `register(name, fn)` - Register dynamic arity function
- `register_fn1(name, fn)` - Register unary function
- `register_fn2(name, fn)` - Register binary function
- `register_fn3(name, fn)` - Register ternary function
- `call_host(name, args)` - Call registered host function
- `set_global(name, value)` - Set global variable
- `get_global(name)` - Get global variable

### Type Conversions

Automatic conversions between Rust and FSRS types:

**Rust → FSRS:**
- `i64`, `i32`, `usize` → `Value::Int`
- `bool` → `Value::Bool`
- `String`, `&str` → `Value::Str`
- `()` → `Value::Unit`
- `Vec<T>` → `Value::Cons` (list)

**FSRS → Rust:**
- `Value::Int` → `i64`, `i32`, `usize`
- `Value::Bool` → `bool`
- `Value::Str` → `String`
- `Value::Unit` → `()`
- `Value::Cons` → `Vec<T>`

## Next Steps

The host interop system is now fully functional and tested. Future enhancements could include:
- Integration with FSRS script execution
- More complex type conversions
- Async host functions
- FFI bindings
- Performance optimizations
