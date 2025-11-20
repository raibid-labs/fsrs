// Integration Tests for Host Interop API
// Tests the complete host function registration and calling pipeline

use fsrs_vm::{HostRegistry, Value, VmError};

#[test]
fn test_register_simple_function() {
    let mut registry = HostRegistry::new();
    registry.register_fn1("increment", |v| {
        let n = v
            .as_int()
            .ok_or_else(|| VmError::Runtime("Expected int".into()))?;
        Ok(Value::Int(n + 1))
    });

    let result = registry.call("increment", &[Value::Int(41)]).unwrap();
    assert_eq!(result, Value::Int(42));
}

#[test]
fn test_register_string_function() {
    let mut registry = HostRegistry::new();
    registry.register_fn1("uppercase", |v| {
        let s = v
            .as_str()
            .ok_or_else(|| VmError::Runtime("Expected string".into()))?;
        Ok(Value::Str(s.to_uppercase()))
    });

    let result = registry
        .call("uppercase", &[Value::Str("hello".to_string())])
        .unwrap();
    assert_eq!(result, Value::Str("HELLO".to_string()));
}

#[test]
fn test_register_binary_function() {
    let mut registry = HostRegistry::new();
    registry.register_fn2("max", |a, b| {
        let x = a
            .as_int()
            .ok_or_else(|| VmError::Runtime("Expected int".into()))?;
        let y = b
            .as_int()
            .ok_or_else(|| VmError::Runtime("Expected int".into()))?;
        Ok(Value::Int(x.max(y)))
    });

    let result = registry
        .call("max", &[Value::Int(10), Value::Int(20)])
        .unwrap();
    assert_eq!(result, Value::Int(20));
}

#[test]
fn test_host_function_with_list() {
    let mut registry = HostRegistry::new();
    registry.register_fn1("sum", |v| {
        let list = v
            .list_to_vec()
            .ok_or_else(|| VmError::Runtime("Expected list".into()))?;
        let sum: i64 = list.iter().filter_map(|v| v.as_int()).sum();
        Ok(Value::Int(sum))
    });

    let list = Value::vec_to_cons(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
    let result = registry.call("sum", &[list]).unwrap();
    assert_eq!(result, Value::Int(6));
}

#[test]
fn test_multiple_host_functions() {
    let mut registry = HostRegistry::new();

    registry.register_fn1("double", |v| {
        let n = v
            .as_int()
            .ok_or_else(|| VmError::Runtime("Expected int".into()))?;
        Ok(Value::Int(n * 2))
    });

    registry.register_fn1("triple", |v| {
        let n = v
            .as_int()
            .ok_or_else(|| VmError::Runtime("Expected int".into()))?;
        Ok(Value::Int(n * 3))
    });

    let doubled = registry.call("double", &[Value::Int(5)]).unwrap();
    let tripled = registry.call("triple", &[doubled]).unwrap();

    assert_eq!(tripled, Value::Int(30));
}

#[test]
fn test_ternary_function() {
    let mut registry = HostRegistry::new();
    registry.register_fn3("clamp", |min_v, val_v, max_v| {
        let min = min_v
            .as_int()
            .ok_or_else(|| VmError::Runtime("Expected int".into()))?;
        let val = val_v
            .as_int()
            .ok_or_else(|| VmError::Runtime("Expected int".into()))?;
        let max = max_v
            .as_int()
            .ok_or_else(|| VmError::Runtime("Expected int".into()))?;
        Ok(Value::Int(val.clamp(min, max)))
    });

    let result = registry
        .call("clamp", &[Value::Int(0), Value::Int(-5), Value::Int(10)])
        .unwrap();
    assert_eq!(result, Value::Int(0));

    let result = registry
        .call("clamp", &[Value::Int(0), Value::Int(15), Value::Int(10)])
        .unwrap();
    assert_eq!(result, Value::Int(10));

    let result = registry
        .call("clamp", &[Value::Int(0), Value::Int(5), Value::Int(10)])
        .unwrap();
    assert_eq!(result, Value::Int(5));
}

#[test]
fn test_list_generation() {
    let mut registry = HostRegistry::new();
    registry.register_fn1("range", |v| {
        let n = v
            .as_int()
            .ok_or_else(|| VmError::Runtime("Expected int".into()))?;
        if n < 0 {
            return Err(VmError::Runtime("Range must be non-negative".into()));
        }
        let values: Vec<Value> = (1..=n).map(Value::Int).collect();
        Ok(Value::vec_to_cons(values))
    });

    let result = registry.call("range", &[Value::Int(5)]).unwrap();
    let expected = Value::vec_to_cons(vec![
        Value::Int(1),
        Value::Int(2),
        Value::Int(3),
        Value::Int(4),
        Value::Int(5),
    ]);
    assert_eq!(result, expected);
}

#[test]
fn test_string_list_processing() {
    let mut registry = HostRegistry::new();
    registry.register_fn1("concat", |v| {
        let list = v
            .list_to_vec()
            .ok_or_else(|| VmError::Runtime("Expected list".into()))?;
        let strings: Vec<String> = list
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();
        Ok(Value::Str(strings.join(" ")))
    });

    let list = Value::vec_to_cons(vec![
        Value::Str("Hello".to_string()),
        Value::Str("World".to_string()),
    ]);
    let result = registry.call("concat", &[list]).unwrap();
    assert_eq!(result, Value::Str("Hello World".to_string()));
}

#[test]
fn test_type_conversion_error() {
    let mut registry = HostRegistry::new();
    registry.register_fn1("double", |v| {
        let n = v
            .as_int()
            .ok_or_else(|| VmError::Runtime("Expected int".into()))?;
        Ok(Value::Int(n * 2))
    });

    let result = registry.call("double", &[Value::Bool(true)]);
    assert!(result.is_err());
}

#[test]
fn test_arity_mismatch() {
    let mut registry = HostRegistry::new();
    registry.register_fn2("add", |a, b| {
        let x = a
            .as_int()
            .ok_or_else(|| VmError::Runtime("Expected int".into()))?;
        let y = b
            .as_int()
            .ok_or_else(|| VmError::Runtime("Expected int".into()))?;
        Ok(Value::Int(x + y))
    });

    // Call with wrong number of arguments
    let result = registry.call("add", &[Value::Int(1)]);
    assert!(result.is_err());

    let result = registry.call("add", &[Value::Int(1), Value::Int(2), Value::Int(3)]);
    assert!(result.is_err());
}

#[test]
fn test_nullary_function() {
    let mut registry = HostRegistry::new();
    registry.register_fn0("get_answer", || Ok(Value::Int(42)));

    let result = registry.call("get_answer", &[]).unwrap();
    assert_eq!(result, Value::Int(42));
}

#[test]
fn test_function_returning_list() {
    let mut registry = HostRegistry::new();
    registry.register_fn0("get_primes", || {
        let primes = vec![
            Value::Int(2),
            Value::Int(3),
            Value::Int(5),
            Value::Int(7),
            Value::Int(11),
        ];
        Ok(Value::vec_to_cons(primes))
    });

    let result = registry.call("get_primes", &[]).unwrap();
    let list = result.list_to_vec().unwrap();
    assert_eq!(list.len(), 5);
    assert_eq!(list[0], Value::Int(2));
    assert_eq!(list[4], Value::Int(11));
}

#[test]
fn test_function_with_bool() {
    let mut registry = HostRegistry::new();
    registry.register_fn1("is_even", |v| {
        let n = v
            .as_int()
            .ok_or_else(|| VmError::Runtime("Expected int".into()))?;
        Ok(Value::Bool(n % 2 == 0))
    });

    let result = registry.call("is_even", &[Value::Int(42)]).unwrap();
    assert_eq!(result, Value::Bool(true));

    let result = registry.call("is_even", &[Value::Int(43)]).unwrap();
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_chained_host_calls() {
    let mut registry = HostRegistry::new();

    registry.register_fn1("double", |v| {
        let n = v
            .as_int()
            .ok_or_else(|| VmError::Runtime("Expected int".into()))?;
        Ok(Value::Int(n * 2))
    });

    registry.register_fn1("increment", |v| {
        let n = v
            .as_int()
            .ok_or_else(|| VmError::Runtime("Expected int".into()))?;
        Ok(Value::Int(n + 1))
    });

    // double(increment(5)) = double(6) = 12
    let step1 = registry.call("increment", &[Value::Int(5)]).unwrap();
    let step2 = registry.call("double", &[step1]).unwrap();
    assert_eq!(step2, Value::Int(12));
}
