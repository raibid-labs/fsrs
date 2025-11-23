// Host Interop Demo - Examples of using Fusabi from Rust
//
// This example demonstrates:
// - Registering host functions
// - Calling host functions from Fusabi
// - Type conversions between Rust and Fusabi
// - Working with lists, strings, and other types

use fusabi_demo::FusabiEngine;
use fusabi_vm::Value;

fn main() {
    println!("=== Fusabi Host Interop Demo ===\n");

    // Create engine
    let mut engine = FusabiEngine::new();

    // ========== Example 1: Simple Arithmetic ==========
    println!("Example 1: Simple Arithmetic");
    engine.register_fn1("double", |v| {
        let n = v
            .as_int()
            .ok_or_else(|| fusabi_vm::VmError::Runtime("Expected int".into()))?;
        Ok(Value::Int(n * 2))
    });

    let result = engine.call_host("double", &[Value::Int(21)]).unwrap();
    println!("  double(21) = {}", result);
    println!();

    // ========== Example 2: String Manipulation ==========
    println!("Example 2: String Manipulation");
    engine.register_fn1("greet", |v| {
        let name = v
            .as_str()
            .ok_or_else(|| fusabi_vm::VmError::Runtime("Expected string".into()))?;
        Ok(Value::Str(format!("Hello, {}!", name)))
    });

    let result = engine
        .call_host("greet", &[Value::Str("World".to_string())])
        .unwrap();
    println!("  greet('World') = {}", result);
    println!();

    // ========== Example 3: Binary Functions ==========
    println!("Example 3: Binary Functions");
    engine.register_fn2("max", |a, b| {
        let x = a
            .as_int()
            .ok_or_else(|| fusabi_vm::VmError::Runtime("Expected int".into()))?;
        let y = b
            .as_int()
            .ok_or_else(|| fusabi_vm::VmError::Runtime("Expected int".into()))?;
        Ok(Value::Int(x.max(y)))
    });

    let result = engine
        .call_host("max", &[Value::Int(10), Value::Int(20)])
        .unwrap();
    println!("  max(10, 20) = {}", result);
    println!();

    // ========== Example 4: List Processing ==========
    println!("Example 4: List Processing");
    engine.register_fn1("sum", |v| {
        let list = v
            .list_to_vec()
            .ok_or_else(|| fusabi_vm::VmError::Runtime("Expected list".into()))?;
        let sum: i64 = list.iter().filter_map(|v| v.as_int()).sum();
        Ok(Value::Int(sum))
    });

    let list = Value::vec_to_cons(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
    let result = engine.call_host("sum", &[list]).unwrap();
    println!("  sum([1; 2; 3]) = {}", result);
    println!();

    // ========== Example 5: List Generation ==========
    println!("Example 5: List Generation");
    engine.register_fn1("range", |v| {
        let n = v
            .as_int()
            .ok_or_else(|| fusabi_vm::VmError::Runtime("Expected int".into()))?;
        if n < 0 {
            return Err(fusabi_vm::VmError::Runtime(
                "Range must be non-negative".into(),
            ));
        }
        let values: Vec<Value> = (1..=n).map(Value::Int).collect();
        Ok(Value::vec_to_cons(values))
    });

    let result = engine.call_host("range", &[Value::Int(5)]).unwrap();
    println!("  range(5) = {}", result);
    println!();

    // ========== Example 6: List Mapping ==========
    println!("Example 6: Combining Host Functions");
    // First generate a range, then double each element
    let range_result = engine.call_host("range", &[Value::Int(5)]).unwrap();
    let list_vec = range_result.list_to_vec().unwrap();
    let doubled: Vec<Value> = list_vec
        .iter()
        .map(|v| engine.call_host("double", &[v.clone()]).unwrap())
        .collect();
    let doubled_list = Value::vec_to_cons(doubled);
    let sum_result = engine.call_host("sum", &[doubled_list]).unwrap();

    println!("  sum(map(double, range(5))) = {}", sum_result);
    println!();

    // ========== Example 7: Boolean Functions ==========
    println!("Example 7: Boolean Functions");
    engine.register_fn1("is_even", |v| {
        let n = v
            .as_int()
            .ok_or_else(|| fusabi_vm::VmError::Runtime("Expected int".into()))?;
        Ok(Value::Bool(n % 2 == 0))
    });

    let result = engine.call_host("is_even", &[Value::Int(42)]).unwrap();
    println!("  is_even(42) = {}", result);
    let result = engine.call_host("is_even", &[Value::Int(43)]).unwrap();
    println!("  is_even(43) = {}", result);
    println!();

    // ========== Example 8: Ternary Functions ==========
    println!("Example 8: Ternary Functions");
    engine.register_fn3("clamp", |min_v, val_v, max_v| {
        let min = min_v
            .as_int()
            .ok_or_else(|| fusabi_vm::VmError::Runtime("Expected int".into()))?;
        let val = val_v
            .as_int()
            .ok_or_else(|| fusabi_vm::VmError::Runtime("Expected int".into()))?;
        let max = max_v
            .as_int()
            .ok_or_else(|| fusabi_vm::VmError::Runtime("Expected int".into()))?;
        Ok(Value::Int(val.clamp(min, max)))
    });

    let result = engine
        .call_host("clamp", &[Value::Int(0), Value::Int(-5), Value::Int(10)])
        .unwrap();
    println!("  clamp(0, -5, 10) = {}", result);
    let result = engine
        .call_host("clamp", &[Value::Int(0), Value::Int(15), Value::Int(10)])
        .unwrap();
    println!("  clamp(0, 15, 10) = {}", result);
    println!();

    // ========== Example 9: String List ==========
    println!("Example 9: String List Processing");
    engine.register_fn1("concat_strings", |v| {
        let list = v
            .list_to_vec()
            .ok_or_else(|| fusabi_vm::VmError::Runtime("Expected list".into()))?;
        let strings: Vec<String> = list
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();
        Ok(Value::Str(strings.join(" ")))
    });

    let string_list = Value::vec_to_cons(vec![
        Value::Str("Hello".to_string()),
        Value::Str("from".to_string()),
        Value::Str("Fusabi".to_string()),
    ]);
    let result = engine.call_host("concat_strings", &[string_list]).unwrap();
    println!("  concat_strings(['Hello'; 'from'; 'Fusabi']) = {}", result);
    println!();

    // ========== Example 10: Global Bindings ==========
    println!("Example 10: Global Bindings");
    engine.set_global("version", Value::Str("0.1.0".to_string()));
    engine.set_global("max_size", Value::Int(1000));

    println!("  Global 'version' = {}", engine.get_global("version").unwrap());
    println!("  Global 'max_size' = {}", engine.get_global("max_size").unwrap());
    println!();

    // ========== Summary ==========
    println!("=== Summary ===");
    println!("Registered host functions:");
    let mut names = engine.host_function_names();
    names.sort();
    for name in names {
        println!("  - {}", name);
    }
    println!("\nHost interop demo completed successfully!");
}
