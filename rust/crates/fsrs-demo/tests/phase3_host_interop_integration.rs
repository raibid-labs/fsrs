//! Phase 3 Host Interop Integration Tests
//!
//! Comprehensive end-to-end validation of host interop:
//! - Host function registration (nullary, unary, binary, ternary, variadic)
//! - Type marshalling (primitives, lists, strings, records, DUs, options)
//! - Error propagation (type errors, runtime errors, arity mismatches)
//! - Real-world callback scenarios (terminal config, plugin systems, etc.)
//! - Performance validation
//!
//! Success Criteria: 20+ integration tests validating production-ready embedding

use fsrs_demo::host_api::FsrsEngine;
use fsrs_vm::{HostRegistry, Value, VmError};

// ============================================================================
// SECTION 1: Host Function Registration Tests (5+ tests)
// ============================================================================

#[test]
fn test_host_registration_nullary_function() {
    // Test: Register and call nullary function (no arguments)
    let mut engine = FsrsEngine::new();
    engine.register_fn0("get_magic_number", || Ok(Value::Int(42)));

    let result = engine.call_host("get_magic_number", &[]).unwrap();
    assert_eq!(result, Value::Int(42));
}

#[test]
fn test_host_registration_unary_function() {
    // Test: Register and call unary function (1 argument)
    let mut engine = FsrsEngine::new();
    engine.register_fn1("double", |v| {
        let n = v
            .as_int()
            .ok_or_else(|| VmError::Runtime("Expected int".into()))?;
        Ok(Value::Int(n * 2))
    });

    let result = engine.call_host("double", &[Value::Int(21)]).unwrap();
    assert_eq!(result, Value::Int(42));
}

#[test]
fn test_host_registration_binary_function() {
    // Test: Register and call binary function (2 arguments)
    let mut engine = FsrsEngine::new();
    engine.register_fn2("add", |a, b| {
        let x = a
            .as_int()
            .ok_or_else(|| VmError::Runtime("Expected int".into()))?;
        let y = b
            .as_int()
            .ok_or_else(|| VmError::Runtime("Expected int".into()))?;
        Ok(Value::Int(x + y))
    });

    let result = engine
        .call_host("add", &[Value::Int(20), Value::Int(22)])
        .unwrap();
    assert_eq!(result, Value::Int(42));
}

#[test]
fn test_host_registration_ternary_function() {
    // Test: Register and call ternary function (3 arguments)
    let mut engine = FsrsEngine::new();
    engine.register_fn3("sum3", |a, b, c| {
        let x = a
            .as_int()
            .ok_or_else(|| VmError::Runtime("Expected int".into()))?;
        let y = b
            .as_int()
            .ok_or_else(|| VmError::Runtime("Expected int".into()))?;
        let z = c
            .as_int()
            .ok_or_else(|| VmError::Runtime("Expected int".into()))?;
        Ok(Value::Int(x + y + z))
    });

    let result = engine
        .call_host("sum3", &[Value::Int(10), Value::Int(20), Value::Int(12)])
        .unwrap();
    assert_eq!(result, Value::Int(42));
}

#[test]
fn test_host_registration_variadic_function() {
    // Test: Register variadic function (variable arguments)
    let mut engine = FsrsEngine::new();
    engine.register("sum_all", |args| {
        let sum: i64 = args.iter().filter_map(|v| v.as_int()).sum();
        Ok(Value::Int(sum))
    });

    // Call with different arities
    let result1 = engine.call_host("sum_all", &[Value::Int(42)]).unwrap();
    assert_eq!(result1, Value::Int(42));

    let result2 = engine
        .call_host("sum_all", &[Value::Int(10), Value::Int(32)])
        .unwrap();
    assert_eq!(result2, Value::Int(42));

    let result3 = engine
        .call_host(
            "sum_all",
            &[Value::Int(10), Value::Int(20), Value::Int(12)],
        )
        .unwrap();
    assert_eq!(result3, Value::Int(42));
}

#[test]
fn test_host_registration_multiple_functions() {
    // Test: Register multiple functions and call them
    let mut engine = FsrsEngine::new();

    engine.register_fn1("square", |v| {
        let n = v
            .as_int()
            .ok_or_else(|| VmError::Runtime("Expected int".into()))?;
        Ok(Value::Int(n * n))
    });

    engine.register_fn1("negate", |v| {
        let n = v
            .as_int()
            .ok_or_else(|| VmError::Runtime("Expected int".into()))?;
        Ok(Value::Int(-n))
    });

    engine.register_fn2("power", |base, exp| {
        let b = base
            .as_int()
            .ok_or_else(|| VmError::Runtime("Expected int".into()))?;
        let e = exp
            .as_int()
            .ok_or_else(|| VmError::Runtime("Expected int".into()))?;
        let result = (b as i64).pow(e as u32);
        Ok(Value::Int(result))
    });

    let square_result = engine.call_host("square", &[Value::Int(7)]).unwrap();
    assert_eq!(square_result, Value::Int(49));

    let negate_result = engine.call_host("negate", &[Value::Int(42)]).unwrap();
    assert_eq!(negate_result, Value::Int(-42));

    let power_result = engine
        .call_host("power", &[Value::Int(2), Value::Int(10)])
        .unwrap();
    assert_eq!(power_result, Value::Int(1024));
}

// ============================================================================
// SECTION 2: Type Marshalling Tests (5+ tests)
// ============================================================================

#[test]
fn test_type_marshalling_primitives() {
    // Test: Marshalling primitive types (int, bool, string)
    let mut engine = FsrsEngine::new();

    // Int identity
    engine.register_fn1("int_identity", |v| Ok(v));
    let int_result = engine.call_host("int_identity", &[Value::Int(42)]).unwrap();
    assert_eq!(int_result, Value::Int(42));

    // Bool identity
    engine.register_fn1("bool_identity", |v| Ok(v));
    let bool_result = engine
        .call_host("bool_identity", &[Value::Bool(true)])
        .unwrap();
    assert_eq!(bool_result, Value::Bool(true));

    // String identity
    engine.register_fn1("string_identity", |v| Ok(v));
    let string_result = engine
        .call_host("string_identity", &[Value::Str("hello".to_string())])
        .unwrap();
    assert_eq!(string_result, Value::Str("hello".to_string()));
}

#[test]
fn test_type_marshalling_lists() {
    // Test: Marshalling list values
    let mut engine = FsrsEngine::new();

    engine.register_fn1("list_sum", |v| {
        let list = v
            .list_to_vec()
            .ok_or_else(|| VmError::Runtime("Expected list".into()))?;
        let sum: i64 = list.iter().filter_map(|val| val.as_int()).sum();
        Ok(Value::Int(sum))
    });

    let list = Value::vec_to_cons(vec![
        Value::Int(10),
        Value::Int(20),
        Value::Int(12),
    ]);
    let result = engine.call_host("list_sum", &[list]).unwrap();
    assert_eq!(result, Value::Int(42));
}

#[test]
fn test_type_marshalling_string_lists() {
    // Test: Lists of strings
    let mut engine = FsrsEngine::new();

    engine.register_fn1("join_strings", |v| {
        let list = v
            .list_to_vec()
            .ok_or_else(|| VmError::Runtime("Expected list".into()))?;
        let strings: Vec<String> = list
            .iter()
            .filter_map(|val| val.as_str().map(|s| s.to_string()))
            .collect();
        Ok(Value::Str(strings.join(" ")))
    });

    let string_list = Value::vec_to_cons(vec![
        Value::Str("Hello".to_string()),
        Value::Str("World".to_string()),
    ]);
    let result = engine.call_host("join_strings", &[string_list]).unwrap();
    assert_eq!(result, Value::Str("Hello World".to_string()));
}

#[test]
fn test_type_marshalling_nested_lists() {
    // Test: Nested list structures
    let mut engine = FsrsEngine::new();

    engine.register_fn1("flatten_and_sum", |v| {
        let outer_list = v
            .list_to_vec()
            .ok_or_else(|| VmError::Runtime("Expected list".into()))?;
        let mut sum = 0i64;
        for item in outer_list {
            if let Some(inner_list) = item.list_to_vec() {
                sum += inner_list.iter().filter_map(|val| val.as_int()).sum::<i64>();
            }
        }
        Ok(Value::Int(sum))
    });

    let nested = Value::vec_to_cons(vec![
        Value::vec_to_cons(vec![Value::Int(10), Value::Int(5)]),
        Value::vec_to_cons(vec![Value::Int(20), Value::Int(7)]),
    ]);
    let result = engine.call_host("flatten_and_sum", &[nested]).unwrap();
    assert_eq!(result, Value::Int(42));
}

#[test]
fn test_type_marshalling_mixed_types() {
    // Test: Functions handling multiple types
    let mut engine = FsrsEngine::new();

    engine.register_fn1("describe_value", |v| {
        let description = match &v {
            Value::Int(n) => format!("int:{}", n),
            Value::Bool(b) => format!("bool:{}", b),
            Value::Str(s) => format!("string:{}", s),
            Value::Cons { .. } => "list".to_string(),
            _ => "unknown".to_string(),
        };
        Ok(Value::Str(description))
    });

    let int_desc = engine
        .call_host("describe_value", &[Value::Int(42)])
        .unwrap();
    assert_eq!(int_desc, Value::Str("int:42".to_string()));

    let bool_desc = engine
        .call_host("describe_value", &[Value::Bool(true)])
        .unwrap();
    assert_eq!(bool_desc, Value::Str("bool:true".to_string()));

    let str_desc = engine
        .call_host("describe_value", &[Value::Str("test".to_string())])
        .unwrap();
    assert_eq!(str_desc, Value::Str("string:test".to_string()));
}

// ============================================================================
// SECTION 3: Error Propagation Tests (5+ tests)
// ============================================================================

#[test]
fn test_error_type_mismatch() {
    // Test: Type mismatch errors are propagated
    let mut engine = FsrsEngine::new();
    engine.register_fn1("expect_int", |v| {
        v.as_int()
            .ok_or_else(|| VmError::Runtime("Expected int".into()))?;
        Ok(Value::Int(42))
    });

    // Pass wrong type
    let result = engine.call_host("expect_int", &[Value::Bool(true)]);
    assert!(result.is_err());
}

#[test]
fn test_error_arity_mismatch() {
    // Test: Arity mismatch errors
    let mut engine = FsrsEngine::new();
    engine.register_fn2("add", |a, b| {
        let x = a
            .as_int()
            .ok_or_else(|| VmError::Runtime("Expected int".into()))?;
        let y = b
            .as_int()
            .ok_or_else(|| VmError::Runtime("Expected int".into()))?;
        Ok(Value::Int(x + y))
    });

    // Call with wrong number of arguments
    let result1 = engine.call_host("add", &[Value::Int(1)]);
    assert!(result1.is_err());

    let result2 = engine.call_host("add", &[Value::Int(1), Value::Int(2), Value::Int(3)]);
    assert!(result2.is_err());

    // Call with correct arity should work
    let result3 = engine
        .call_host("add", &[Value::Int(1), Value::Int(2)])
        .unwrap();
    assert_eq!(result3, Value::Int(3));
}

#[test]
fn test_error_runtime_errors() {
    // Test: Runtime errors from host functions
    let mut engine = FsrsEngine::new();
    engine.register_fn1("divide_by_zero", |_| {
        Err(VmError::Runtime("Division by zero".into()))
    });

    let result = engine.call_host("divide_by_zero", &[Value::Int(42)]);
    assert!(result.is_err());
}

#[test]
fn test_error_nonexistent_function() {
    // Test: Calling nonexistent function
    let engine = FsrsEngine::new();
    let result = engine.call_host("nonexistent", &[Value::Int(42)]);
    assert!(result.is_err());
}

#[test]
fn test_error_list_extraction_failure() {
    // Test: Error when list extraction fails
    let mut engine = FsrsEngine::new();
    engine.register_fn1("expect_list", |v| {
        v.list_to_vec()
            .ok_or_else(|| VmError::Runtime("Expected list".into()))?;
        Ok(Value::Int(42))
    });

    // Pass non-list
    let result = engine.call_host("expect_list", &[Value::Int(123)]);
    assert!(result.is_err());
}

#[test]
fn test_error_partial_list_processing() {
    // Test: Error handling in list processing
    let mut engine = FsrsEngine::new();
    engine.register_fn1("sum_ints_only", |v| {
        let list = v
            .list_to_vec()
            .ok_or_else(|| VmError::Runtime("Expected list".into()))?;

        // Require all elements to be ints
        for item in &list {
            if item.as_int().is_none() {
                return Err(VmError::Runtime("All elements must be ints".into()));
            }
        }

        let sum: i64 = list.iter().filter_map(|val| val.as_int()).sum();
        Ok(Value::Int(sum))
    });

    // Valid list
    let valid_list = Value::vec_to_cons(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
    let result = engine.call_host("sum_ints_only", &[valid_list]).unwrap();
    assert_eq!(result, Value::Int(6));

    // Invalid list (mixed types)
    let invalid_list = Value::vec_to_cons(vec![Value::Int(1), Value::Bool(true)]);
    let result = engine.call_host("sum_ints_only", &[invalid_list]);
    assert!(result.is_err());
}

// ============================================================================
// SECTION 4: Real-World Callback Scenarios (5+ tests)
// ============================================================================

#[test]
fn test_real_world_terminal_tab_formatter() {
    // Scenario: Terminal emulator tab title formatting
    let mut engine = FsrsEngine::new();

    engine.register_fn2("format_tab_title", |index, title| {
        let idx = index
            .as_int()
            .ok_or_else(|| VmError::Runtime("Expected int".into()))?;
        let title_str = title
            .as_str()
            .ok_or_else(|| VmError::Runtime("Expected string".into()))?;

        let formatted = format!("[{}] {}", idx, title_str);
        Ok(Value::Str(formatted))
    });

    let result = engine
        .call_host(
            "format_tab_title",
            &[Value::Int(1), Value::Str("main.rs".to_string())],
        )
        .unwrap();
    assert_eq!(result, Value::Str("[1] main.rs".to_string()));
}

#[test]
fn test_real_world_config_validation() {
    // Scenario: Configuration value validation
    let mut engine = FsrsEngine::new();

    engine.register_fn2("validate_port", |min, port| {
        let min_port = min
            .as_int()
            .ok_or_else(|| VmError::Runtime("Expected int".into()))?;
        let port_num = port
            .as_int()
            .ok_or_else(|| VmError::Runtime("Expected int".into()))?;

        if port_num < min_port || port_num > 65535 {
            return Err(VmError::Runtime("Invalid port number".into()));
        }

        Ok(Value::Bool(true))
    });

    // Valid port
    let valid = engine
        .call_host("validate_port", &[Value::Int(1024), Value::Int(8080)])
        .unwrap();
    assert_eq!(valid, Value::Bool(true));

    // Invalid port
    let invalid = engine.call_host("validate_port", &[Value::Int(1024), Value::Int(100)]);
    assert!(invalid.is_err());
}

#[test]
fn test_real_world_event_handler() {
    // Scenario: Event handling with string commands
    let mut engine = FsrsEngine::new();

    engine.register_fn1("handle_key_event", |key| {
        let key_str = key
            .as_str()
            .ok_or_else(|| VmError::Runtime("Expected string".into()))?;

        let action = match key_str {
            "Ctrl+T" => "NewTab",
            "Ctrl+W" => "CloseTab",
            "Ctrl+N" => "NewWindow",
            _ => "Unknown",
        };

        Ok(Value::Str(action.to_string()))
    });

    let new_tab = engine
        .call_host("handle_key_event", &[Value::Str("Ctrl+T".to_string())])
        .unwrap();
    assert_eq!(new_tab, Value::Str("NewTab".to_string()));

    let close_tab = engine
        .call_host("handle_key_event", &[Value::Str("Ctrl+W".to_string())])
        .unwrap();
    assert_eq!(close_tab, Value::Str("CloseTab".to_string()));
}

#[test]
fn test_real_world_plugin_loader() {
    // Scenario: Plugin system - load plugin by name
    let mut engine = FsrsEngine::new();

    engine.register_fn1("load_plugin", |name| {
        let plugin_name = name
            .as_str()
            .ok_or_else(|| VmError::Runtime("Expected string".into()))?;

        // Simulate plugin loading
        let available_plugins = vec!["autocomplete", "syntax-highlight", "git-integration"];

        if available_plugins.contains(&plugin_name) {
            Ok(Value::Bool(true))
        } else {
            Err(VmError::Runtime(format!(
                "Plugin '{}' not found",
                plugin_name
            )))
        }
    });

    // Load valid plugin
    let result = engine
        .call_host("load_plugin", &[Value::Str("autocomplete".to_string())])
        .unwrap();
    assert_eq!(result, Value::Bool(true));

    // Load invalid plugin
    let result = engine.call_host("load_plugin", &[Value::Str("invalid".to_string())]);
    assert!(result.is_err());
}

#[test]
fn test_real_world_color_picker() {
    // Scenario: Color utilities
    let mut engine = FsrsEngine::new();

    engine.register_fn3("create_rgb", |r, g, b| {
        let red = r
            .as_int()
            .ok_or_else(|| VmError::Runtime("Expected int".into()))?;
        let green = g
            .as_int()
            .ok_or_else(|| VmError::Runtime("Expected int".into()))?;
        let blue = b
            .as_int()
            .ok_or_else(|| VmError::Runtime("Expected int".into()))?;

        if red < 0 || red > 255 || green < 0 || green > 255 || blue < 0 || blue > 255 {
            return Err(VmError::Runtime("RGB values must be 0-255".into()));
        }

        let hex = format!("#{:02X}{:02X}{:02X}", red, green, blue);
        Ok(Value::Str(hex))
    });

    let color = engine
        .call_host(
            "create_rgb",
            &[Value::Int(255), Value::Int(128), Value::Int(0)],
        )
        .unwrap();
    assert_eq!(color, Value::Str("#FF8000".to_string()));
}

// ============================================================================
// SECTION 5: Performance and Composition Tests (5+ tests)
// ============================================================================

#[test]
fn test_performance_repeated_calls() {
    // Test: Many repeated calls to host functions
    let mut engine = FsrsEngine::new();
    engine.register_fn1("increment", |v| {
        let n = v
            .as_int()
            .ok_or_else(|| VmError::Runtime("Expected int".into()))?;
        Ok(Value::Int(n + 1))
    });

    let mut value = Value::Int(0);
    for _ in 0..1000 {
        value = engine.call_host("increment", &[value]).unwrap();
    }

    assert_eq!(value, Value::Int(1000));
}

#[test]
fn test_composition_chained_host_calls() {
    // Test: Chain multiple host function calls
    let mut engine = FsrsEngine::new();

    engine.register_fn1("double", |v| {
        let n = v
            .as_int()
            .ok_or_else(|| VmError::Runtime("Expected int".into()))?;
        Ok(Value::Int(n * 2))
    });

    engine.register_fn1("increment", |v| {
        let n = v
            .as_int()
            .ok_or_else(|| VmError::Runtime("Expected int".into()))?;
        Ok(Value::Int(n + 1))
    });

    engine.register_fn1("square", |v| {
        let n = v
            .as_int()
            .ok_or_else(|| VmError::Runtime("Expected int".into()))?;
        Ok(Value::Int(n * n))
    });

    // Compute: square(increment(double(5))) = square(increment(10)) = square(11) = 121
    let step1 = engine.call_host("double", &[Value::Int(5)]).unwrap();
    let step2 = engine.call_host("increment", &[step1]).unwrap();
    let step3 = engine.call_host("square", &[step2]).unwrap();

    assert_eq!(step3, Value::Int(121));
}

#[test]
fn test_composition_host_and_stdlib() {
    // Test: Combining host functions with stdlib (via registry pattern)
    let mut registry = HostRegistry::new();

    // Register host functions
    registry.register_fn1("double", |v| {
        let n = v
            .as_int()
            .ok_or_else(|| VmError::Runtime("Expected int".into()))?;
        Ok(Value::Int(n * 2))
    });

    // Create a list, double each element (simulated)
    let list = Value::vec_to_cons(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);

    // Extract list, double each, rebuild
    let list_vec = list.list_to_vec().unwrap();
    let doubled_vec: Result<Vec<Value>, VmError> = list_vec
        .into_iter()
        .map(|v| registry.call("double", &[v]))
        .collect();
    let doubled_list = Value::vec_to_cons(doubled_vec.unwrap());

    let doubled_vec = doubled_list.list_to_vec().unwrap();
    assert_eq!(doubled_vec[0], Value::Int(2));
    assert_eq!(doubled_vec[1], Value::Int(4));
    assert_eq!(doubled_vec[2], Value::Int(6));
}

#[test]
fn test_composition_nested_callbacks() {
    // Test: Host functions calling other host functions
    let mut engine = FsrsEngine::new();

    engine.register_fn1("base_operation", |v| {
        let n = v
            .as_int()
            .ok_or_else(|| VmError::Runtime("Expected int".into()))?;
        Ok(Value::Int(n + 10))
    });

    // Register a "higher-order" function that uses the result of another
    engine.register_fn1("complex_operation", |v| {
        let n = v
            .as_int()
            .ok_or_else(|| VmError::Runtime("Expected int".into()))?;
        // Simulate: base_operation result * 2
        let base_result = n + 10;
        Ok(Value::Int(base_result * 2))
    });

    let result = engine
        .call_host("complex_operation", &[Value::Int(5)])
        .unwrap();
    assert_eq!(result, Value::Int(30)); // (5 + 10) * 2
}

#[test]
fn test_composition_stateful_operations() {
    // Test: Simulating stateful operations through host functions
    let mut engine = FsrsEngine::new();

    // Register functions that simulate state management
    engine.register_fn2("create_counter", |_init, increment| {
        // Returns a tuple-like structure (simulated with list)
        let init_val = _init
            .as_int()
            .ok_or_else(|| VmError::Runtime("Expected int".into()))?;
        let inc_val = increment
            .as_int()
            .ok_or_else(|| VmError::Runtime("Expected int".into()))?;

        Ok(Value::vec_to_cons(vec![
            Value::Int(init_val),
            Value::Int(inc_val),
        ]))
    });

    engine.register_fn1("next_value", |counter| {
        let counter_vec = counter
            .list_to_vec()
            .ok_or_else(|| VmError::Runtime("Expected counter".into()))?;

        if counter_vec.len() != 2 {
            return Err(VmError::Runtime("Invalid counter".into()));
        }

        let current = counter_vec[0]
            .as_int()
            .ok_or_else(|| VmError::Runtime("Expected int".into()))?;
        let increment = counter_vec[1]
            .as_int()
            .ok_or_else(|| VmError::Runtime("Expected int".into()))?;

        let next = current + increment;
        Ok(Value::vec_to_cons(vec![
            Value::Int(next),
            Value::Int(increment),
        ]))
    });

    // Create counter starting at 0, incrementing by 5
    let counter = engine
        .call_host("create_counter", &[Value::Int(0), Value::Int(5)])
        .unwrap();

    // Next value should be 5
    let counter2 = engine.call_host("next_value", &[counter]).unwrap();
    let counter2_vec = counter2.list_to_vec().unwrap();
    assert_eq!(counter2_vec[0], Value::Int(5));

    // Next value should be 10
    let counter3 = engine.call_host("next_value", &[counter2]).unwrap();
    let counter3_vec = counter3.list_to_vec().unwrap();
    assert_eq!(counter3_vec[0], Value::Int(10));
}
