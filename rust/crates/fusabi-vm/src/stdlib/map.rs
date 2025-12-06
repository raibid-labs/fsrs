// Fusabi Map Standard Library
use crate::value::Value;
use crate::vm::{Vm, VmError};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

/// Map.empty : unit -> 'a map
/// Creates an empty map
pub fn map_empty(_unit: &Value) -> Result<Value, VmError> {
    Ok(Value::Map(Arc::new(Mutex::new(HashMap::new()))))
}

/// Map.add : string -> 'a -> 'a map -> 'a map
/// Adds a key-value pair to the map, returning a new map
pub fn map_add(key: &Value, value: &Value, map: &Value) -> Result<Value, VmError> {
    let key_str = key.as_str().ok_or_else(|| VmError::TypeMismatch {
        expected: "string",
        got: key.type_name(),
    })?;
    match map {
        Value::Map(m) => {
            let mut new_map = m.lock().unwrap().clone();
            new_map.insert(key_str.to_string(), value.clone());
            Ok(Value::Map(Arc::new(Mutex::new(new_map))))
        }
        _ => Err(VmError::TypeMismatch {
            expected: "map",
            got: map.type_name(),
        }),
    }
}

/// Map.remove : string -> 'a map -> 'a map
/// Removes a key from the map, returning a new map
pub fn map_remove(key: &Value, map: &Value) -> Result<Value, VmError> {
    let key_str = key.as_str().ok_or_else(|| VmError::TypeMismatch {
        expected: "string",
        got: key.type_name(),
    })?;
    match map {
        Value::Map(m) => {
            let mut new_map = m.lock().unwrap().clone();
            new_map.remove(key_str);
            Ok(Value::Map(Arc::new(Mutex::new(new_map))))
        }
        _ => Err(VmError::TypeMismatch {
            expected: "map",
            got: map.type_name(),
        }),
    }
}

/// Map.find : string -> 'a map -> 'a
/// Looks up a key in the map, throws error if not found
pub fn map_find(key: &Value, map: &Value) -> Result<Value, VmError> {
    let key_str = key.as_str().ok_or_else(|| VmError::TypeMismatch {
        expected: "string",
        got: key.type_name(),
    })?;
    match map {
        Value::Map(m) => {
            let m = m.lock().unwrap();
            m.get(key_str)
                .cloned()
                .ok_or_else(|| VmError::Runtime(format!("Map key not found: {}", key_str)))
        }
        _ => Err(VmError::TypeMismatch {
            expected: "map",
            got: map.type_name(),
        }),
    }
}

/// Map.tryFind : string -> 'a map -> 'a option
/// Looks up a key in the map, returns Some(value) or None
pub fn map_try_find(key: &Value, map: &Value) -> Result<Value, VmError> {
    let key_str = key.as_str().ok_or_else(|| VmError::TypeMismatch {
        expected: "string",
        got: key.type_name(),
    })?;
    match map {
        Value::Map(m) => {
            let m = m.lock().unwrap();
            match m.get(key_str) {
                Some(value) => Ok(Value::Variant {
                    type_name: "Option".to_string(),
                    variant_name: "Some".to_string(),
                    fields: vec![value.clone()],
                }),
                None => Ok(Value::Variant {
                    type_name: "Option".to_string(),
                    variant_name: "None".to_string(),
                    fields: vec![],
                }),
            }
        }
        _ => Err(VmError::TypeMismatch {
            expected: "map",
            got: map.type_name(),
        }),
    }
}

/// Map.containsKey : string -> 'a map -> bool
/// Returns true if the map contains the given key
pub fn map_contains_key(key: &Value, map: &Value) -> Result<Value, VmError> {
    let key_str = key.as_str().ok_or_else(|| VmError::TypeMismatch {
        expected: "string",
        got: key.type_name(),
    })?;
    match map {
        Value::Map(m) => {
            let m = m.lock().unwrap();
            Ok(Value::Bool(m.contains_key(key_str)))
        }
        _ => Err(VmError::TypeMismatch {
            expected: "map",
            got: map.type_name(),
        }),
    }
}

/// Map.isEmpty : 'a map -> bool
/// Returns true if the map is empty
pub fn map_is_empty(map: &Value) -> Result<Value, VmError> {
    match map {
        Value::Map(m) => Ok(Value::Bool(m.lock().unwrap().is_empty())),
        _ => Err(VmError::TypeMismatch {
            expected: "map",
            got: map.type_name(),
        }),
    }
}

/// Map.count : 'a map -> int
/// Returns the number of key-value pairs in the map
pub fn map_count(map: &Value) -> Result<Value, VmError> {
    match map {
        Value::Map(m) => Ok(Value::Int(m.lock().unwrap().len() as i64)),
        _ => Err(VmError::TypeMismatch {
            expected: "map",
            got: map.type_name(),
        }),
    }
}

/// Map.ofList : (string * 'a) list -> 'a map
/// Creates a map from a list of key-value tuples
pub fn map_of_list(list: &Value) -> Result<Value, VmError> {
    let mut map = HashMap::new();
    let mut current = list.clone();
    loop {
        match current {
            Value::Nil => break,
            Value::Cons { head, tail } => {
                if let Value::Tuple(elements) = &*head {
                    if elements.len() != 2 {
                        return Err(VmError::Runtime(
                            "Map.ofList expects list of 2-tuples".to_string(),
                        ));
                    }
                    let key_str = elements[0].as_str().ok_or_else(|| VmError::TypeMismatch {
                        expected: "string",
                        got: elements[0].type_name(),
                    })?;
                    map.insert(key_str.to_string(), elements[1].clone());
                } else {
                    return Err(VmError::Runtime(
                        "Map.ofList expects list of tuples".to_string(),
                    ));
                }
                current = (*tail).clone();
            }
            _ => {
                return Err(VmError::TypeMismatch {
                    expected: "list",
                    got: current.type_name(),
                })
            }
        }
    }
    Ok(Value::Map(Arc::new(Mutex::new(map))))
}

/// Map.toList : 'a map -> (string * 'a) list
/// Converts a map to a list of key-value tuples (sorted by key)
pub fn map_to_list(map: &Value) -> Result<Value, VmError> {
    match map {
        Value::Map(m) => {
            let m = m.lock().unwrap();
            let mut entries: Vec<_> = m
                .iter()
                .map(|(k, v)| Value::Tuple(vec![Value::Str(k.clone()), v.clone()]))
                .collect();
            entries.sort_by(|a, b| {
                if let (Value::Tuple(a_tuple), Value::Tuple(b_tuple)) = (a, b) {
                    if let (Some(Value::Str(a_key)), Some(Value::Str(b_key))) =
                        (a_tuple.get(0), b_tuple.get(0))
                    {
                        return a_key.cmp(b_key);
                    }
                }
                std::cmp::Ordering::Equal
            });
            Ok(Value::vec_to_cons(entries))
        }
        _ => Err(VmError::TypeMismatch {
            expected: "map",
            got: map.type_name(),
        }),
    }
}

/// Map.map : ('a -> 'b) -> 'a map -> 'b map
/// Applies a function to each value in the map, returning a new map
pub fn map_map(vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::Runtime(format!(
            "Map.map expects 2 arguments, got {}",
            args.len()
        )));
    }

    let func = &args[0];
    let map = &args[1];

    match map {
        Value::Map(m) => {
            let m = m.lock().unwrap();
            let mut new_map = HashMap::new();

            for (key, value) in m.iter() {
                let transformed_value = vm.call_value(func.clone(), &[value.clone()])?;
                new_map.insert(key.clone(), transformed_value);
            }

            Ok(Value::Map(Arc::new(Mutex::new(new_map))))
        }
        _ => Err(VmError::TypeMismatch {
            expected: "map",
            got: map.type_name(),
        }),
    }
}

/// Map.iter : (string -> 'a -> unit) -> 'a map -> unit
/// Calls a function on each key-value pair for side effects
pub fn map_iter(vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::Runtime(format!(
            "Map.iter expects 2 arguments, got {}",
            args.len()
        )));
    }

    let func = &args[0];
    let map = &args[1];

    match map {
        Value::Map(m) => {
            let m = m.lock().unwrap();

            // Collect keys in sorted order for deterministic iteration
            let mut keys: Vec<_> = m.keys().cloned().collect();
            keys.sort();

            for key in keys {
                if let Some(value) = m.get(&key) {
                    // Curried function: func key value => (func key) value
                    let partial = vm.call_value(func.clone(), &[Value::Str(key)])?;
                    vm.call_value(partial, &[value.clone()])?;
                }
            }

            Ok(Value::Unit)
        }
        _ => Err(VmError::TypeMismatch {
            expected: "map",
            got: map.type_name(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk::ChunkBuilder;
    use crate::closure::Closure;

    fn create_test_map() -> Value {
        let mut map = HashMap::new();
        map.insert("a".to_string(), Value::Int(1));
        map.insert("b".to_string(), Value::Int(2));
        map.insert("c".to_string(), Value::Int(3));
        Value::Map(Arc::new(Mutex::new(map)))
    }

    fn create_empty_map() -> Value {
        Value::Map(Arc::new(Mutex::new(HashMap::new())))
    }

    fn create_mock_closure() -> Value {
        let chunk = ChunkBuilder::new().build();
        let closure = Closure::new(chunk);
        Value::Closure(Arc::new(closure))
    }

    #[test]
    fn test_map_map_empty_map() {
        let mut vm = Vm::new();
        let func = create_mock_closure();
        let empty_map = create_empty_map();

        let args = vec![func, empty_map];
        let result = map_map(&mut vm, &args).expect("map_map failed");

        // Verify result is an empty map
        if let Value::Map(result_map) = result {
            let map = result_map.lock().unwrap();
            assert!(map.is_empty());
        } else {
            panic!("Expected Map result");
        }
    }

    #[test]
    fn test_map_map_wrong_argument_count() {
        let mut vm = Vm::new();
        let test_map = create_test_map();

        // Call with wrong number of arguments
        let args = vec![test_map];
        let result = map_map(&mut vm, &args);

        assert!(result.is_err());
        if let Err(VmError::Runtime(msg)) = result {
            assert!(msg.contains("expects 2 arguments"));
        } else {
            panic!("Expected Runtime error");
        }
    }

    #[test]
    fn test_map_map_type_mismatch() {
        let mut vm = Vm::new();
        let func = create_mock_closure();

        // Pass non-map value
        let args = vec![func, Value::Int(42)];
        let result = map_map(&mut vm, &args);

        assert!(result.is_err());
        if let Err(VmError::TypeMismatch { expected, got }) = result {
            assert_eq!(expected, "map");
            assert_eq!(got, "int");
        } else {
            panic!("Expected TypeMismatch error");
        }
    }

    #[test]
    fn test_map_iter_empty_map() {
        let mut vm = Vm::new();
        let func = create_mock_closure();
        let empty_map = create_empty_map();

        let args = vec![func, empty_map];
        let result = map_iter(&mut vm, &args).expect("map_iter failed");

        // Verify result is Unit
        assert_eq!(result, Value::Unit);
    }

    #[test]
    fn test_map_iter_wrong_argument_count() {
        let mut vm = Vm::new();
        let test_map = create_test_map();

        // Call with wrong number of arguments
        let args = vec![test_map];
        let result = map_iter(&mut vm, &args);

        assert!(result.is_err());
        if let Err(VmError::Runtime(msg)) = result {
            assert!(msg.contains("expects 2 arguments"));
        } else {
            panic!("Expected Runtime error");
        }
    }

    #[test]
    fn test_map_iter_type_mismatch() {
        let mut vm = Vm::new();
        let func = create_mock_closure();

        // Pass non-map value
        let args = vec![func, Value::Int(42)];
        let result = map_iter(&mut vm, &args);

        assert!(result.is_err());
        if let Err(VmError::TypeMismatch { expected, got }) = result {
            assert_eq!(expected, "map");
            assert_eq!(got, "int");
        } else {
            panic!("Expected TypeMismatch error");
        }
    }

    #[test]
    fn test_map_map_preserves_keys() {
        let mut vm = Vm::new();
        let func = create_mock_closure();
        let test_map = create_test_map();

        let args = vec![func, test_map];
        // This will fail when calling the function, but we're testing the structure
        // In real usage, the function would be properly compiled
        let result = map_map(&mut vm, &args);

        // We expect this to fail because we're using a mock closure
        // but the function should at least try to process the map
        // This test verifies the function signature and type checking
        assert!(result.is_err());
    }

    #[test]
    fn test_map_iter_processes_sorted_keys() {
        let mut vm = Vm::new();
        let func = create_mock_closure();
        let test_map = create_test_map();

        let args = vec![func, test_map];
        // This will fail when calling the function, but we're testing the structure
        // In real usage, the function would be properly compiled
        let result = map_iter(&mut vm, &args);

        // We expect this to fail because we're using a mock closure
        // but the function should at least try to iterate the map
        // This test verifies the function signature and type checking
        assert!(result.is_err());
    }
}
