// Fusabi Array Standard Library
// Provides operations for mutable arrays

use crate::value::Value;
use crate::vm::{Vm, VmError};
use std::sync::{Arc, Mutex};

/// Array.length : 'a array -> int
/// Returns the number of elements in the array
pub fn array_length(arr: &Value) -> Result<Value, VmError> {
    match arr {
        Value::Array(vec) => {
            let vec = vec.lock().unwrap();
            Ok(Value::Int(vec.len() as i64))
        }
        _ => Err(VmError::TypeMismatch {
            expected: "array",
            got: arr.type_name(),
        }),
    }
}

/// Array.isEmpty : 'a array -> bool
/// Returns true if the array is empty
pub fn array_is_empty(arr: &Value) -> Result<Value, VmError> {
    match arr {
        Value::Array(vec) => {
            let vec = vec.lock().unwrap();
            Ok(Value::Bool(vec.is_empty()))
        }
        _ => Err(VmError::TypeMismatch {
            expected: "array",
            got: arr.type_name(),
        }),
    }
}

/// Array.get : int -> 'a array -> 'a
/// Safe array indexing - throws error if index is out of bounds
pub fn array_get(index: &Value, arr: &Value) -> Result<Value, VmError> {
    let idx = index.as_int().ok_or_else(|| VmError::TypeMismatch {
        expected: "int",
        got: index.type_name(),
    })?;

    match arr {
        Value::Array(vec) => {
            let vec = vec.lock().unwrap();

            if idx < 0 {
                return Err(VmError::Runtime(format!(
                    "Array index out of bounds: index {} is negative",
                    idx
                )));
            }

            let idx_usize = idx as usize;
            if idx_usize >= vec.len() {
                return Err(VmError::Runtime(format!(
                    "Array index out of bounds: index {} >= length {}",
                    idx,
                    vec.len()
                )));
            }

            Ok(vec[idx_usize].clone())
        }
        _ => Err(VmError::TypeMismatch {
            expected: "array",
            got: arr.type_name(),
        }),
    }
}

/// Array.set : int -> 'a -> 'a array -> unit
/// Mutates array in place by setting the element at the given index
/// Throws error if index is out of bounds
pub fn array_set(index: &Value, value: &Value, arr: &Value) -> Result<Value, VmError> {
    let idx = index.as_int().ok_or_else(|| VmError::TypeMismatch {
        expected: "int",
        got: index.type_name(),
    })?;

    match arr {
        Value::Array(vec) => {
            let mut vec = vec.lock().unwrap();

            if idx < 0 {
                return Err(VmError::Runtime(format!(
                    "Array index out of bounds: index {} is negative",
                    idx
                )));
            }

            let idx_usize = idx as usize;
            if idx_usize >= vec.len() {
                return Err(VmError::Runtime(format!(
                    "Array index out of bounds: index {} >= length {}",
                    idx,
                    vec.len()
                )));
            }

            vec[idx_usize] = value.clone();
            Ok(Value::Unit)
        }
        _ => Err(VmError::TypeMismatch {
            expected: "array",
            got: arr.type_name(),
        }),
    }
}

/// Array.ofList : 'a list -> 'a array
/// Converts a cons list to an array
pub fn array_of_list(list: &Value) -> Result<Value, VmError> {
    let vec = list.list_to_vec().ok_or_else(|| VmError::TypeMismatch {
        expected: "list",
        got: list.type_name(),
    })?;

    Ok(Value::Array(Arc::new(Mutex::new(vec))))
}

/// Array.toList : 'a array -> 'a list
/// Converts an array to a cons list
pub fn array_to_list(arr: &Value) -> Result<Value, VmError> {
    match arr {
        Value::Array(vec) => {
            let vec = vec.lock().unwrap();
            Ok(Value::vec_to_cons(vec.clone()))
        }
        _ => Err(VmError::TypeMismatch {
            expected: "array",
            got: arr.type_name(),
        }),
    }
}

/// Array.init : int -> (int -> 'a) -> 'a array
/// Creates an array of given length by calling the function for each index
/// Takes (length: int, fn: int -> 'a) and creates array by calling fn for each index from 0 to length-1
pub fn array_init(vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::Runtime(format!(
            "Array.init expects 2 arguments, got {}",
            args.len()
        )));
    }

    let length = args[0].as_int().ok_or_else(|| VmError::TypeMismatch {
        expected: "int",
        got: args[0].type_name(),
    })?;

    if length < 0 {
        return Err(VmError::Runtime(format!(
            "Array.init requires non-negative length, got {}",
            length
        )));
    }

    let func = &args[1];
    let mut result = Vec::new();

    for i in 0..length {
        let value = vm.call_value(func.clone(), &[Value::Int(i)])?;
        result.push(value);
    }

    Ok(Value::Array(Arc::new(Mutex::new(result))))
}

/// Array.create : int -> 'a -> 'a array
/// Creates an array of given length filled with the specified value
pub fn array_create(length: &Value, value: &Value) -> Result<Value, VmError> {
    let len = length.as_int().ok_or_else(|| VmError::TypeMismatch {
        expected: "int",
        got: length.type_name(),
    })?;

    if len < 0 {
        return Err(VmError::Runtime(format!(
            "Array.create requires non-negative length, got {}",
            len
        )));
    }

    let vec = vec![value.clone(); len as usize];
    Ok(Value::Array(Arc::new(Mutex::new(vec))))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_array(elements: Vec<Value>) -> Value {
        Value::Array(Arc::new(Mutex::new(elements)))
    }

    #[test]
    fn test_array_length_empty() {
        let arr = make_array(vec![]);
        let result = array_length(&arr).unwrap();
        assert_eq!(result, Value::Int(0));
    }

    #[test]
    fn test_array_length_single() {
        let arr = make_array(vec![Value::Int(42)]);
        let result = array_length(&arr).unwrap();
        assert_eq!(result, Value::Int(1));
    }

    #[test]
    fn test_array_length_multiple() {
        let arr = make_array(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        let result = array_length(&arr).unwrap();
        assert_eq!(result, Value::Int(3));
    }

    #[test]
    fn test_array_length_type_error() {
        let not_array = Value::Int(42);
        let result = array_length(&not_array);
        assert!(matches!(result, Err(VmError::TypeMismatch { .. })));
    }

    #[test]
    fn test_array_is_empty_true() {
        let arr = make_array(vec![]);
        let result = array_is_empty(&arr).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_array_is_empty_false() {
        let arr = make_array(vec![Value::Int(1)]);
        let result = array_is_empty(&arr).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_array_is_empty_type_error() {
        let not_array = Value::Str("hello".to_string());
        let result = array_is_empty(&not_array);
        assert!(matches!(result, Err(VmError::TypeMismatch { .. })));
    }

    #[test]
    fn test_array_get_success() {
        let arr = make_array(vec![Value::Int(10), Value::Int(20), Value::Int(30)]);

        let result = array_get(&Value::Int(0), &arr).unwrap();
        assert_eq!(result, Value::Int(10));

        let result = array_get(&Value::Int(1), &arr).unwrap();
        assert_eq!(result, Value::Int(20));

        let result = array_get(&Value::Int(2), &arr).unwrap();
        assert_eq!(result, Value::Int(30));
    }

    #[test]
    fn test_array_get_out_of_bounds() {
        let arr = make_array(vec![Value::Int(10), Value::Int(20)]);

        let result = array_get(&Value::Int(2), &arr);
        assert!(matches!(result, Err(VmError::Runtime(_))));

        let result = array_get(&Value::Int(100), &arr);
        assert!(matches!(result, Err(VmError::Runtime(_))));
    }

    #[test]
    fn test_array_get_negative_index() {
        let arr = make_array(vec![Value::Int(10), Value::Int(20)]);
        let result = array_get(&Value::Int(-1), &arr);
        assert!(matches!(result, Err(VmError::Runtime(_))));
    }

    #[test]
    fn test_array_get_type_errors() {
        let arr = make_array(vec![Value::Int(10)]);

        // Non-int index
        let result = array_get(&Value::Str("0".to_string()), &arr);
        assert!(matches!(result, Err(VmError::TypeMismatch { .. })));

        // Non-array value
        let result = array_get(&Value::Int(0), &Value::Int(42));
        assert!(matches!(result, Err(VmError::TypeMismatch { .. })));
    }

    #[test]
    fn test_array_set_success() {
        let arr = make_array(vec![Value::Int(10), Value::Int(20), Value::Int(30)]);

        let result = array_set(&Value::Int(1), &Value::Int(100), &arr);
        assert_eq!(result.unwrap(), Value::Unit);

        // Verify the value was changed
        let value = array_get(&Value::Int(1), &arr).unwrap();
        assert_eq!(value, Value::Int(100));

        // Verify other values unchanged
        let value = array_get(&Value::Int(0), &arr).unwrap();
        assert_eq!(value, Value::Int(10));
        let value = array_get(&Value::Int(2), &arr).unwrap();
        assert_eq!(value, Value::Int(30));
    }

    #[test]
    fn test_array_set_out_of_bounds() {
        let arr = make_array(vec![Value::Int(10), Value::Int(20)]);

        let result = array_set(&Value::Int(2), &Value::Int(100), &arr);
        assert!(matches!(result, Err(VmError::Runtime(_))));

        let result = array_set(&Value::Int(100), &Value::Int(100), &arr);
        assert!(matches!(result, Err(VmError::Runtime(_))));
    }

    #[test]
    fn test_array_set_negative_index() {
        let arr = make_array(vec![Value::Int(10), Value::Int(20)]);
        let result = array_set(&Value::Int(-1), &Value::Int(100), &arr);
        assert!(matches!(result, Err(VmError::Runtime(_))));
    }

    #[test]
    fn test_array_set_type_errors() {
        let arr = make_array(vec![Value::Int(10)]);

        // Non-int index
        let result = array_set(&Value::Str("0".to_string()), &Value::Int(100), &arr);
        assert!(matches!(result, Err(VmError::TypeMismatch { .. })));

        // Non-array value
        let result = array_set(&Value::Int(0), &Value::Int(100), &Value::Int(42));
        assert!(matches!(result, Err(VmError::TypeMismatch { .. })));
    }

    #[test]
    fn test_array_of_list_empty() {
        let list = Value::Nil;
        let result = array_of_list(&list).unwrap();

        let length = array_length(&result).unwrap();
        assert_eq!(length, Value::Int(0));
    }

    #[test]
    fn test_array_of_list_single() {
        let list = Value::vec_to_cons(vec![Value::Int(42)]);
        let result = array_of_list(&list).unwrap();

        let length = array_length(&result).unwrap();
        assert_eq!(length, Value::Int(1));

        let value = array_get(&Value::Int(0), &result).unwrap();
        assert_eq!(value, Value::Int(42));
    }

    #[test]
    fn test_array_of_list_multiple() {
        let list = Value::vec_to_cons(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        let result = array_of_list(&list).unwrap();

        let length = array_length(&result).unwrap();
        assert_eq!(length, Value::Int(3));

        assert_eq!(array_get(&Value::Int(0), &result).unwrap(), Value::Int(1));
        assert_eq!(array_get(&Value::Int(1), &result).unwrap(), Value::Int(2));
        assert_eq!(array_get(&Value::Int(2), &result).unwrap(), Value::Int(3));
    }

    #[test]
    fn test_array_of_list_type_error() {
        let not_list = Value::Int(42);
        let result = array_of_list(&not_list);
        assert!(matches!(result, Err(VmError::TypeMismatch { .. })));
    }

    #[test]
    fn test_array_to_list_empty() {
        let arr = make_array(vec![]);
        let result = array_to_list(&arr).unwrap();
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_array_to_list_single() {
        let arr = make_array(vec![Value::Int(42)]);
        let result = array_to_list(&arr).unwrap();

        let expected = Value::vec_to_cons(vec![Value::Int(42)]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_array_to_list_multiple() {
        let arr = make_array(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        let result = array_to_list(&arr).unwrap();

        let expected = Value::vec_to_cons(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_array_to_list_type_error() {
        let not_array = Value::Str("hello".to_string());
        let result = array_to_list(&not_array);
        assert!(matches!(result, Err(VmError::TypeMismatch { .. })));
    }

    #[test]
    fn test_array_to_list_round_trip() {
        let original = Value::vec_to_cons(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        let arr = array_of_list(&original).unwrap();
        let result = array_to_list(&arr).unwrap();
        assert_eq!(result, original);
    }

    #[test]
    fn test_array_create_empty() {
        let arr = array_create(&Value::Int(0), &Value::Int(42)).unwrap();
        let length = array_length(&arr).unwrap();
        assert_eq!(length, Value::Int(0));
    }

    #[test]
    fn test_array_create_single() {
        let arr = array_create(&Value::Int(1), &Value::Int(42)).unwrap();
        let length = array_length(&arr).unwrap();
        assert_eq!(length, Value::Int(1));

        let value = array_get(&Value::Int(0), &arr).unwrap();
        assert_eq!(value, Value::Int(42));
    }

    #[test]
    fn test_array_create_multiple() {
        let arr = array_create(&Value::Int(5), &Value::Str("hello".to_string())).unwrap();
        let length = array_length(&arr).unwrap();
        assert_eq!(length, Value::Int(5));

        for i in 0..5 {
            let value = array_get(&Value::Int(i), &arr).unwrap();
            assert_eq!(value, Value::Str("hello".to_string()));
        }
    }

    #[test]
    fn test_array_create_negative_length() {
        let result = array_create(&Value::Int(-1), &Value::Int(42));
        assert!(matches!(result, Err(VmError::Runtime(_))));
    }

    #[test]
    fn test_array_create_type_error() {
        let result = array_create(&Value::Str("5".to_string()), &Value::Int(42));
        assert!(matches!(result, Err(VmError::TypeMismatch { .. })));
    }

    #[test]
    fn test_array_init_empty() {
        use crate::chunk::ChunkBuilder;
        let mut vm = Vm::new();
        let chunk = ChunkBuilder::new().build();
        let func = Value::Closure(Arc::new(crate::closure::Closure::new(chunk)));

        let result = array_init(&mut vm, &[Value::Int(0), func]);
        assert!(result.is_ok());

        let arr = result.unwrap();
        let length = array_length(&arr).unwrap();
        assert_eq!(length, Value::Int(0));
    }

    #[test]
    fn test_array_init_negative_length() {
        use crate::chunk::ChunkBuilder;
        let mut vm = Vm::new();
        let chunk = ChunkBuilder::new().build();
        let func = Value::Closure(Arc::new(crate::closure::Closure::new(chunk)));

        let result = array_init(&mut vm, &[Value::Int(-1), func]);
        assert!(matches!(result, Err(VmError::Runtime(_))));
    }

    #[test]
    fn test_array_init_wrong_arg_count() {
        let mut vm = Vm::new();

        let result = array_init(&mut vm, &[Value::Int(5)]);
        assert!(matches!(result, Err(VmError::Runtime(_))));

        let result = array_init(&mut vm, &[]);
        assert!(matches!(result, Err(VmError::Runtime(_))));
    }

    #[test]
    fn test_array_init_type_error() {
        use crate::chunk::ChunkBuilder;
        let mut vm = Vm::new();
        let chunk = ChunkBuilder::new().build();
        let func = Value::Closure(Arc::new(crate::closure::Closure::new(chunk)));

        let result = array_init(&mut vm, &[Value::Str("5".to_string()), func]);
        assert!(matches!(result, Err(VmError::TypeMismatch { .. })));
    }

    #[test]
    fn test_array_mutation() {
        let arr = make_array(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);

        // Mutate the array
        array_set(&Value::Int(0), &Value::Int(10), &arr).unwrap();
        array_set(&Value::Int(1), &Value::Int(20), &arr).unwrap();
        array_set(&Value::Int(2), &Value::Int(30), &arr).unwrap();

        // Verify all mutations
        assert_eq!(array_get(&Value::Int(0), &arr).unwrap(), Value::Int(10));
        assert_eq!(array_get(&Value::Int(1), &arr).unwrap(), Value::Int(20));
        assert_eq!(array_get(&Value::Int(2), &arr).unwrap(), Value::Int(30));
    }

    #[test]
    fn test_array_mixed_types() {
        let arr = make_array(vec![
            Value::Int(42),
            Value::Str("hello".to_string()),
            Value::Bool(true),
            Value::Unit,
        ]);

        assert_eq!(array_length(&arr).unwrap(), Value::Int(4));
        assert_eq!(array_get(&Value::Int(0), &arr).unwrap(), Value::Int(42));
        assert_eq!(
            array_get(&Value::Int(1), &arr).unwrap(),
            Value::Str("hello".to_string())
        );
        assert_eq!(array_get(&Value::Int(2), &arr).unwrap(), Value::Bool(true));
        assert_eq!(array_get(&Value::Int(3), &arr).unwrap(), Value::Unit);
    }
}
