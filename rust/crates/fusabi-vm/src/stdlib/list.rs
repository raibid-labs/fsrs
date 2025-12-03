// Fusabi List Standard Library
// Provides common list operations for cons-based lists

use crate::value::Value;
use crate::vm::{Vm, VmError};

/// List.length : 'a list -> int
/// Returns the number of elements in a list
pub fn list_length(list: &Value) -> Result<Value, VmError> {
    match list {
        Value::Nil => Ok(Value::Int(0)),
        Value::Cons { tail, .. } => {
            let tail_len = list_length(tail)?;
            if let Value::Int(n) = tail_len {
                Ok(Value::Int(n + 1))
            } else {
                Err(VmError::Runtime("Expected int from length".to_string()))
            }
        }
        _ => Err(VmError::TypeMismatch {
            expected: "list",
            got: list.type_name(),
        }),
    }
}

/// List.head : 'a list -> 'a
/// Returns the first element of a list
/// Throws error if list is empty
pub fn list_head(list: &Value) -> Result<Value, VmError> {
    match list {
        Value::Cons { head, .. } => Ok((**head).clone()),
        Value::Nil => Err(VmError::EmptyList),
        _ => Err(VmError::TypeMismatch {
            expected: "list",
            got: list.type_name(),
        }),
    }
}

/// List.tail : 'a list -> 'a list
/// Returns the list without its first element
/// Throws error if list is empty
pub fn list_tail(list: &Value) -> Result<Value, VmError> {
    match list {
        Value::Cons { tail, .. } => Ok((**tail).clone()),
        Value::Nil => Err(VmError::EmptyList),
        _ => Err(VmError::TypeMismatch {
            expected: "list",
            got: list.type_name(),
        }),
    }
}

/// List.reverse : 'a list -> 'a list
/// Returns a list with elements in reverse order
pub fn list_reverse(list: &Value) -> Result<Value, VmError> {
    let mut acc = Value::Nil;
    let mut current = list.clone();

    loop {
        match current {
            Value::Nil => return Ok(acc),
            Value::Cons { head, tail } => {
                acc = Value::Cons {
                    head: head.clone(),
                    tail: Box::new(acc),
                };
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
}

/// List.isEmpty : 'a list -> bool
/// Returns true if the list is empty (Nil)
pub fn list_is_empty(list: &Value) -> Result<Value, VmError> {
    match list {
        Value::Nil => Ok(Value::Bool(true)),
        Value::Cons { .. } => Ok(Value::Bool(false)),
        _ => Err(VmError::TypeMismatch {
            expected: "list",
            got: list.type_name(),
        }),
    }
}

/// List.append : 'a list -> 'a list -> 'a list
/// Concatenates two lists
pub fn list_append(list1: &Value, list2: &Value) -> Result<Value, VmError> {
    match list1 {
        Value::Nil => Ok(list2.clone()),
        Value::Cons { head, tail } => {
            let appended_tail = list_append(tail, list2)?;
            Ok(Value::Cons {
                head: head.clone(),
                tail: Box::new(appended_tail),
            })
        }
        _ => Err(VmError::TypeMismatch {
            expected: "list",
            got: list1.type_name(),
        }),
    }
}

/// List.concat : 'a list list -> 'a list
/// Concatenates a list of lists into a single list
pub fn list_concat(lists: &Value) -> Result<Value, VmError> {
    let mut result = Value::Nil;
    let mut current = lists.clone();

    // Collect all lists first (to avoid reversing)
    let mut all_lists = Vec::new();
    loop {
        match current {
            Value::Nil => break,
            Value::Cons { head, tail } => {
                all_lists.push((*head).clone());
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

    // Concatenate in reverse order
    for list in all_lists.iter().rev() {
        result = list_append(list, &result)?;
    }

    Ok(result)
}

/// List.map : ('a -> 'b) -> 'a list -> 'b list
/// Applies a function to each element of the list
pub fn list_map(vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::Runtime(format!(
            "List.map expects 2 arguments, got {}",
            args.len()
        )));
    }

    let func = &args[0];
    let list = &args[1];

    // Verify list type
    if !matches!(list, Value::Nil | Value::Cons { .. }) {
        return Err(VmError::TypeMismatch {
            expected: "list",
            got: list.type_name(),
        });
    }

    let elements = list
        .list_to_vec()
        .ok_or(VmError::Runtime("Malformed list".into()))?;
    let mut mapped_elements = Vec::new();

    for elem in elements {
        let result = vm.call_value(func.clone(), &[elem])?;
        mapped_elements.push(result);
    }

    Ok(Value::vec_to_cons(mapped_elements))
}

/// List.iter : ('a -> unit) -> 'a list -> unit
/// Calls a function on each element for side effects, returns Unit
pub fn list_iter(vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::Runtime(format!(
            "List.iter expects 2 arguments, got {}",
            args.len()
        )));
    }

    let func = &args[0];
    let list = &args[1];

    // Verify list type
    if !matches!(list, Value::Nil | Value::Cons { .. }) {
        return Err(VmError::TypeMismatch {
            expected: "list",
            got: list.type_name(),
        });
    }

    let elements = list
        .list_to_vec()
        .ok_or(VmError::Runtime("Malformed list".into()))?;

    for elem in elements {
        vm.call_value(func.clone(), &[elem])?;
    }

    Ok(Value::Unit)
}

/// List.filter : ('a -> bool) -> 'a list -> 'a list
/// Returns list of elements where predicate returns true
pub fn list_filter(vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::Runtime(format!(
            "List.filter expects 2 arguments, got {}",
            args.len()
        )));
    }

    let func = &args[0];
    let list = &args[1];

    // Verify list type
    if !matches!(list, Value::Nil | Value::Cons { .. }) {
        return Err(VmError::TypeMismatch {
            expected: "list",
            got: list.type_name(),
        });
    }

    let elements = list
        .list_to_vec()
        .ok_or(VmError::Runtime("Malformed list".into()))?;
    let mut filtered_elements = Vec::new();

    for elem in elements {
        let result = vm.call_value(func.clone(), &[elem.clone()])?;
        match result {
            Value::Bool(true) => filtered_elements.push(elem),
            Value::Bool(false) => {}
            _ => {
                return Err(VmError::TypeMismatch {
                    expected: "bool",
                    got: result.type_name(),
                })
            }
        }
    }

    Ok(Value::vec_to_cons(filtered_elements))
}

/// List.fold : ('a -> 'b -> 'a) -> 'a -> 'b list -> 'a
/// Applies folder function to accumulator and each element
pub fn list_fold(vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 3 {
        return Err(VmError::Runtime(format!(
            "List.fold expects 3 arguments, got {}",
            args.len()
        )));
    }

    let func = &args[0];
    let init = &args[1];
    let list = &args[2];

    // Verify list type
    if !matches!(list, Value::Nil | Value::Cons { .. }) {
        return Err(VmError::TypeMismatch {
            expected: "list",
            got: list.type_name(),
        });
    }

    let elements = list
        .list_to_vec()
        .ok_or(VmError::Runtime("Malformed list".into()))?;
    let mut acc = init.clone();

    for elem in elements {
        // Curried function: first apply acc, then elem
        // func acc elem => (func acc) elem
        let partial = vm.call_value(func.clone(), &[acc])?;
        acc = vm.call_value(partial, &[elem])?;
    }

    Ok(acc)
}

/// List.exists : ('a -> bool) -> 'a list -> bool
/// Returns true if any element satisfies the predicate
pub fn list_exists(vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::Runtime(format!(
            "List.exists expects 2 arguments, got {}",
            args.len()
        )));
    }

    let func = &args[0];
    let list = &args[1];

    // Verify list type
    if !matches!(list, Value::Nil | Value::Cons { .. }) {
        return Err(VmError::TypeMismatch {
            expected: "list",
            got: list.type_name(),
        });
    }

    let elements = list
        .list_to_vec()
        .ok_or(VmError::Runtime("Malformed list".into()))?;

    for elem in elements {
        let result = vm.call_value(func.clone(), &[elem])?;
        match result {
            Value::Bool(true) => return Ok(Value::Bool(true)),
            Value::Bool(false) => {}
            _ => {
                return Err(VmError::TypeMismatch {
                    expected: "bool",
                    got: result.type_name(),
                })
            }
        }
    }

    Ok(Value::Bool(false))
}

/// List.find : ('a -> bool) -> 'a list -> 'a
/// Returns first element satisfying predicate
/// Throws error if not found
pub fn list_find(vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::Runtime(format!(
            "List.find expects 2 arguments, got {}",
            args.len()
        )));
    }

    let func = &args[0];
    let list = &args[1];

    // Verify list type
    if !matches!(list, Value::Nil | Value::Cons { .. }) {
        return Err(VmError::TypeMismatch {
            expected: "list",
            got: list.type_name(),
        });
    }

    let elements = list
        .list_to_vec()
        .ok_or(VmError::Runtime("Malformed list".into()))?;

    for elem in elements {
        let result = vm.call_value(func.clone(), &[elem.clone()])?;
        match result {
            Value::Bool(true) => return Ok(elem),
            Value::Bool(false) => {}
            _ => {
                return Err(VmError::TypeMismatch {
                    expected: "bool",
                    got: result.type_name(),
                })
            }
        }
    }

    Err(VmError::Runtime(
        "List.find: no element satisfies predicate".to_string(),
    ))
}

/// List.tryFind : ('a -> bool) -> 'a list -> 'a option
/// Returns Some(elem) if found, None otherwise
pub fn list_try_find(vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::Runtime(format!(
            "List.tryFind expects 2 arguments, got {}",
            args.len()
        )));
    }

    let func = &args[0];
    let list = &args[1];

    // Verify list type
    if !matches!(list, Value::Nil | Value::Cons { .. }) {
        return Err(VmError::TypeMismatch {
            expected: "list",
            got: list.type_name(),
        });
    }

    let elements = list
        .list_to_vec()
        .ok_or(VmError::Runtime("Malformed list".into()))?;

    for elem in elements {
        let result = vm.call_value(func.clone(), &[elem.clone()])?;
        match result {
            Value::Bool(true) => {
                return Ok(Value::Variant {
                    type_name: "Option".to_string(),
                    variant_name: "Some".to_string(),
                    fields: vec![elem],
                })
            }
            Value::Bool(false) => {}
            _ => {
                return Err(VmError::TypeMismatch {
                    expected: "bool",
                    got: result.type_name(),
                })
            }
        }
    }

    Ok(Value::Variant {
        type_name: "Option".to_string(),
        variant_name: "None".to_string(),
        fields: vec![],
    })
}

/// List.nth : int -> 'a list -> 'a option
/// Returns the element at the given index, or None if out of bounds
pub fn list_nth(index: &Value, list: &Value) -> Result<Value, VmError> {
    // Check index is Int
    let idx = match index {
        Value::Int(n) => *n,
        _ => {
            return Err(VmError::TypeMismatch {
                expected: "int",
                got: index.type_name(),
            })
        }
    };

    // If negative, return None
    if idx < 0 {
        return Ok(Value::Variant {
            type_name: "Option".to_string(),
            variant_name: "None".to_string(),
            fields: vec![],
        });
    }

    // Verify list type
    if !matches!(list, Value::Nil | Value::Cons { .. }) {
        return Err(VmError::TypeMismatch {
            expected: "list",
            got: list.type_name(),
        });
    }

    // Walk the list to the index
    let mut current = list.clone();
    let mut current_idx = 0i64;

    loop {
        match current {
            Value::Nil => {
                // Reached end of list before finding index
                return Ok(Value::Variant {
                    type_name: "Option".to_string(),
                    variant_name: "None".to_string(),
                    fields: vec![],
                });
            }
            Value::Cons { head, tail } => {
                if current_idx == idx {
                    // Found the element
                    return Ok(Value::Variant {
                        type_name: "Option".to_string(),
                        variant_name: "Some".to_string(),
                        fields: vec![(*head).clone()],
                    });
                }
                current_idx += 1;
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
}

/// List.mapi : (int -> 'a -> 'b) -> 'a list -> 'b list
/// Maps with index - needs VM for callback execution
pub fn list_mapi(vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::Runtime(format!(
            "List.mapi expects 2 arguments, got {}",
            args.len()
        )));
    }

    let func = &args[0];
    let list = &args[1];

    // Verify list type
    if !matches!(list, Value::Nil | Value::Cons { .. }) {
        return Err(VmError::TypeMismatch {
            expected: "list",
            got: list.type_name(),
        });
    }

    let elements = list
        .list_to_vec()
        .ok_or(VmError::Runtime("Malformed list".into()))?;
    let mut mapped_elements = Vec::new();

    for (index, elem) in elements.iter().enumerate() {
        // Call func with index and element
        // Since the function is curried (int -> 'a -> 'b), we need to:
        // 1. Call func with index to get a partial function
        // 2. Call the partial with elem to get the result
        let index_val = Value::Int(index as i64);
        let partial = vm.call_value(func.clone(), &[index_val])?;
        let result = vm.call_value(partial, &[elem.clone()])?;
        mapped_elements.push(result);
    }

    Ok(Value::vec_to_cons(mapped_elements))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_length_empty() {
        let list = Value::Nil;
        let result = list_length(&list).unwrap();
        assert_eq!(result, Value::Int(0));
    }

    #[test]
    fn test_list_length_single() {
        let list = Value::vec_to_cons(vec![Value::Int(42)]);
        let result = list_length(&list).unwrap();
        assert_eq!(result, Value::Int(1));
    }

    #[test]
    fn test_list_length_multiple() {
        let list = Value::vec_to_cons(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        let result = list_length(&list).unwrap();
        assert_eq!(result, Value::Int(3));
    }

    #[test]
    fn test_list_head_success() {
        let list = Value::vec_to_cons(vec![Value::Int(42), Value::Int(100)]);
        let result = list_head(&list).unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_list_head_empty_error() {
        let list = Value::Nil;
        let result = list_head(&list);
        assert!(matches!(result, Err(VmError::EmptyList)));
    }

    #[test]
    fn test_list_tail_success() {
        let list = Value::vec_to_cons(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        let result = list_tail(&list).unwrap();
        let expected = Value::vec_to_cons(vec![Value::Int(2), Value::Int(3)]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_list_tail_empty_error() {
        let list = Value::Nil;
        let result = list_tail(&list);
        assert!(matches!(result, Err(VmError::EmptyList)));
    }

    #[test]
    fn test_list_reverse_empty() {
        let list = Value::Nil;
        let result = list_reverse(&list).unwrap();
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_list_reverse_single() {
        let list = Value::vec_to_cons(vec![Value::Int(42)]);
        let result = list_reverse(&list).unwrap();
        assert_eq!(result, list);
    }

    #[test]
    fn test_list_reverse_multiple() {
        let list = Value::vec_to_cons(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        let result = list_reverse(&list).unwrap();
        let expected = Value::vec_to_cons(vec![Value::Int(3), Value::Int(2), Value::Int(1)]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_list_is_empty_true() {
        let list = Value::Nil;
        let result = list_is_empty(&list).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_list_is_empty_false() {
        let list = Value::vec_to_cons(vec![Value::Int(1)]);
        let result = list_is_empty(&list).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_list_append_empty_left() {
        let list1 = Value::Nil;
        let list2 = Value::vec_to_cons(vec![Value::Int(1), Value::Int(2)]);
        let result = list_append(&list1, &list2).unwrap();
        assert_eq!(result, list2);
    }

    #[test]
    fn test_list_append_empty_right() {
        let list1 = Value::vec_to_cons(vec![Value::Int(1), Value::Int(2)]);
        let list2 = Value::Nil;
        let result = list_append(&list1, &list2).unwrap();
        assert_eq!(result, list1);
    }

    #[test]
    fn test_list_append_both_nonempty() {
        let list1 = Value::vec_to_cons(vec![Value::Int(1), Value::Int(2)]);
        let list2 = Value::vec_to_cons(vec![Value::Int(3), Value::Int(4)]);
        let result = list_append(&list1, &list2).unwrap();
        let expected = Value::vec_to_cons(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
            Value::Int(4),
        ]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_list_concat_empty() {
        let lists = Value::Nil;
        let result = list_concat(&lists).unwrap();
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_list_concat_single_list() {
        let inner = Value::vec_to_cons(vec![Value::Int(1), Value::Int(2)]);
        let lists = Value::vec_to_cons(vec![inner.clone()]);
        let result = list_concat(&lists).unwrap();
        assert_eq!(result, inner);
    }

    #[test]
    fn test_list_concat_multiple_lists() {
        let list1 = Value::vec_to_cons(vec![Value::Int(1), Value::Int(2)]);
        let list2 = Value::vec_to_cons(vec![Value::Int(3), Value::Int(4)]);
        let list3 = Value::vec_to_cons(vec![Value::Int(5)]);
        let lists = Value::vec_to_cons(vec![list1, list2, list3]);
        let result = list_concat(&lists).unwrap();
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
    fn test_list_type_error() {
        let not_list = Value::Int(42);
        assert!(list_length(&not_list).is_err());
        assert!(list_head(&not_list).is_err());
        assert!(list_tail(&not_list).is_err());
        assert!(list_reverse(&not_list).is_err());
        assert!(list_is_empty(&not_list).is_err());
    }

    // Tests for list_iter
    #[test]
    fn test_list_iter_empty() {
        use crate::chunk::Chunk;
        use crate::closure::Closure;
        use std::sync::Arc;

        let mut vm = Vm::new();
        let chunk = Chunk::new();
        let closure = Closure::new(chunk);
        let func = Value::Closure(Arc::new(closure));
        let list = Value::Nil;
        let result = list_iter(&mut vm, &[func, list]).unwrap();
        assert_eq!(result, Value::Unit);
    }

    #[test]
    fn test_list_iter_wrong_arg_count() {
        let mut vm = Vm::new();
        let result = list_iter(&mut vm, &[Value::Int(1)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_iter_type_error() {
        use crate::chunk::Chunk;
        use crate::closure::Closure;
        use std::sync::Arc;

        let mut vm = Vm::new();
        let chunk = Chunk::new();
        let closure = Closure::new(chunk);
        let func = Value::Closure(Arc::new(closure));
        let not_list = Value::Int(42);
        let result = list_iter(&mut vm, &[func, not_list]);
        assert!(matches!(
            result,
            Err(VmError::TypeMismatch {
                expected: "list",
                ..
            })
        ));
    }

    // Tests for list_filter
    #[test]
    fn test_list_filter_empty() {
        use crate::chunk::Chunk;
        use crate::closure::Closure;
        use std::sync::Arc;

        let mut vm = Vm::new();
        let chunk = Chunk::new();
        let closure = Closure::new(chunk);
        let func = Value::Closure(Arc::new(closure));
        let list = Value::Nil;
        let result = list_filter(&mut vm, &[func, list]).unwrap();
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_list_filter_wrong_arg_count() {
        let mut vm = Vm::new();
        let result = list_filter(&mut vm, &[Value::Int(1)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_filter_type_error() {
        use crate::chunk::Chunk;
        use crate::closure::Closure;
        use std::sync::Arc;

        let mut vm = Vm::new();
        let chunk = Chunk::new();
        let closure = Closure::new(chunk);
        let func = Value::Closure(Arc::new(closure));
        let not_list = Value::Int(42);
        let result = list_filter(&mut vm, &[func, not_list]);
        assert!(matches!(
            result,
            Err(VmError::TypeMismatch {
                expected: "list",
                ..
            })
        ));
    }

    // Tests for list_fold
    #[test]
    fn test_list_fold_empty() {
        use crate::chunk::Chunk;
        use crate::closure::Closure;
        use std::sync::Arc;

        let mut vm = Vm::new();
        let chunk = Chunk::new();
        let closure = Closure::new(chunk);
        let func = Value::Closure(Arc::new(closure));
        let init = Value::Int(0);
        let list = Value::Nil;
        let result = list_fold(&mut vm, &[func, init.clone(), list]).unwrap();
        assert_eq!(result, init);
    }

    #[test]
    fn test_list_fold_wrong_arg_count() {
        let mut vm = Vm::new();
        let result = list_fold(&mut vm, &[Value::Int(1), Value::Int(2)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_fold_type_error() {
        use crate::chunk::Chunk;
        use crate::closure::Closure;
        use std::sync::Arc;

        let mut vm = Vm::new();
        let chunk = Chunk::new();
        let closure = Closure::new(chunk);
        let func = Value::Closure(Arc::new(closure));
        let init = Value::Int(0);
        let not_list = Value::Int(42);
        let result = list_fold(&mut vm, &[func, init, not_list]);
        assert!(matches!(
            result,
            Err(VmError::TypeMismatch {
                expected: "list",
                ..
            })
        ));
    }

    // Tests for list_exists
    #[test]
    fn test_list_exists_empty() {
        use crate::chunk::Chunk;
        use crate::closure::Closure;
        use std::sync::Arc;

        let mut vm = Vm::new();
        let chunk = Chunk::new();
        let closure = Closure::new(chunk);
        let func = Value::Closure(Arc::new(closure));
        let list = Value::Nil;
        let result = list_exists(&mut vm, &[func, list]).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_list_exists_wrong_arg_count() {
        let mut vm = Vm::new();
        let result = list_exists(&mut vm, &[Value::Int(1)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_exists_type_error() {
        use crate::chunk::Chunk;
        use crate::closure::Closure;
        use std::sync::Arc;

        let mut vm = Vm::new();
        let chunk = Chunk::new();
        let closure = Closure::new(chunk);
        let func = Value::Closure(Arc::new(closure));
        let not_list = Value::Int(42);
        let result = list_exists(&mut vm, &[func, not_list]);
        assert!(matches!(
            result,
            Err(VmError::TypeMismatch {
                expected: "list",
                ..
            })
        ));
    }

    // Tests for list_find
    #[test]
    fn test_list_find_empty_error() {
        use crate::chunk::Chunk;
        use crate::closure::Closure;
        use std::sync::Arc;

        let mut vm = Vm::new();
        let chunk = Chunk::new();
        let closure = Closure::new(chunk);
        let func = Value::Closure(Arc::new(closure));
        let list = Value::Nil;
        let result = list_find(&mut vm, &[func, list]);
        assert!(matches!(result, Err(VmError::Runtime(_))));
    }

    #[test]
    fn test_list_find_wrong_arg_count() {
        let mut vm = Vm::new();
        let result = list_find(&mut vm, &[Value::Int(1)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_find_type_error() {
        use crate::chunk::Chunk;
        use crate::closure::Closure;
        use std::sync::Arc;

        let mut vm = Vm::new();
        let chunk = Chunk::new();
        let closure = Closure::new(chunk);
        let func = Value::Closure(Arc::new(closure));
        let not_list = Value::Int(42);
        let result = list_find(&mut vm, &[func, not_list]);
        assert!(matches!(
            result,
            Err(VmError::TypeMismatch {
                expected: "list",
                ..
            })
        ));
    }

    // Tests for list_try_find
    #[test]
    fn test_list_try_find_empty() {
        use crate::chunk::Chunk;
        use crate::closure::Closure;
        use std::sync::Arc;

        let mut vm = Vm::new();
        let chunk = Chunk::new();
        let closure = Closure::new(chunk);
        let func = Value::Closure(Arc::new(closure));
        let list = Value::Nil;
        let result = list_try_find(&mut vm, &[func, list]).unwrap();
        assert!(matches!(
            result,
            Value::Variant {
                variant_name,
                ..
            } if variant_name == "None"
        ));
    }

    #[test]
    fn test_list_try_find_wrong_arg_count() {
        let mut vm = Vm::new();
        let result = list_try_find(&mut vm, &[Value::Int(1)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_try_find_type_error() {
        use crate::chunk::Chunk;
        use crate::closure::Closure;
        use std::sync::Arc;

        let mut vm = Vm::new();
        let chunk = Chunk::new();
        let closure = Closure::new(chunk);
        let func = Value::Closure(Arc::new(closure));
        let not_list = Value::Int(42);
        let result = list_try_find(&mut vm, &[func, not_list]);
        assert!(matches!(
            result,
            Err(VmError::TypeMismatch {
                expected: "list",
                ..
            })
        ));
    }

    #[test]
    fn test_list_try_find_returns_none_variant() {
        use crate::chunk::Chunk;
        use crate::closure::Closure;
        use std::sync::Arc;

        let mut vm = Vm::new();
        let chunk = Chunk::new();
        let closure = Closure::new(chunk);
        let func = Value::Closure(Arc::new(closure));
        let list = Value::Nil;
        let result = list_try_find(&mut vm, &[func, list]).unwrap();

        match result {
            Value::Variant {
                type_name,
                variant_name,
                fields,
            } => {
                assert_eq!(type_name, "Option");
                assert_eq!(variant_name, "None");
                assert_eq!(fields.len(), 0);
            }
            _ => panic!("Expected Variant value"),
        }
    }

    // Tests for list_nth
    #[test]
    fn test_list_nth_empty() {
        let list = Value::Nil;
        let index = Value::Int(0);
        let result = list_nth(&index, &list).unwrap();
        assert!(matches!(
            result,
            Value::Variant {
                variant_name,
                ..
            } if variant_name == "None"
        ));
    }

    #[test]
    fn test_list_nth_negative_index() {
        let list = Value::vec_to_cons(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        let index = Value::Int(-1);
        let result = list_nth(&index, &list).unwrap();
        assert!(matches!(
            result,
            Value::Variant {
                variant_name,
                ..
            } if variant_name == "None"
        ));
    }

    #[test]
    fn test_list_nth_out_of_bounds() {
        let list = Value::vec_to_cons(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        let index = Value::Int(10);
        let result = list_nth(&index, &list).unwrap();
        assert!(matches!(
            result,
            Value::Variant {
                variant_name,
                ..
            } if variant_name == "None"
        ));
    }

    #[test]
    fn test_list_nth_first_element() {
        let list = Value::vec_to_cons(vec![Value::Int(42), Value::Int(100), Value::Int(200)]);
        let index = Value::Int(0);
        let result = list_nth(&index, &list).unwrap();
        match result {
            Value::Variant {
                variant_name,
                fields,
                ..
            } if variant_name == "Some" => {
                assert_eq!(fields.len(), 1);
                assert_eq!(fields[0], Value::Int(42));
            }
            _ => panic!("Expected Some variant"),
        }
    }

    #[test]
    fn test_list_nth_middle_element() {
        let list = Value::vec_to_cons(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        let index = Value::Int(1);
        let result = list_nth(&index, &list).unwrap();
        match result {
            Value::Variant {
                variant_name,
                fields,
                ..
            } if variant_name == "Some" => {
                assert_eq!(fields.len(), 1);
                assert_eq!(fields[0], Value::Int(2));
            }
            _ => panic!("Expected Some variant"),
        }
    }

    #[test]
    fn test_list_nth_last_element() {
        let list = Value::vec_to_cons(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        let index = Value::Int(2);
        let result = list_nth(&index, &list).unwrap();
        match result {
            Value::Variant {
                variant_name,
                fields,
                ..
            } if variant_name == "Some" => {
                assert_eq!(fields.len(), 1);
                assert_eq!(fields[0], Value::Int(3));
            }
            _ => panic!("Expected Some variant"),
        }
    }

    #[test]
    fn test_list_nth_type_error_index() {
        let list = Value::vec_to_cons(vec![Value::Int(1)]);
        let index = Value::Str("not an int".to_string());
        let result = list_nth(&index, &list);
        assert!(matches!(
            result,
            Err(VmError::TypeMismatch {
                expected: "int",
                ..
            })
        ));
    }

    #[test]
    fn test_list_nth_type_error_list() {
        let not_list = Value::Int(42);
        let index = Value::Int(0);
        let result = list_nth(&index, &not_list);
        assert!(matches!(
            result,
            Err(VmError::TypeMismatch {
                expected: "list",
                ..
            })
        ));
    }

    // Tests for list_mapi
    #[test]
    fn test_list_mapi_empty() {
        use crate::chunk::Chunk;
        use crate::closure::Closure;
        use std::sync::Arc;

        let mut vm = Vm::new();
        let chunk = Chunk::new();
        let closure = Closure::new(chunk);
        let func = Value::Closure(Arc::new(closure));
        let list = Value::Nil;
        let result = list_mapi(&mut vm, &[func, list]).unwrap();
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_list_mapi_wrong_arg_count() {
        let mut vm = Vm::new();
        let result = list_mapi(&mut vm, &[Value::Int(1)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_mapi_type_error() {
        use crate::chunk::Chunk;
        use crate::closure::Closure;
        use std::sync::Arc;

        let mut vm = Vm::new();
        let chunk = Chunk::new();
        let closure = Closure::new(chunk);
        let func = Value::Closure(Arc::new(closure));
        let not_list = Value::Int(42);
        let result = list_mapi(&mut vm, &[func, not_list]);
        assert!(matches!(
            result,
            Err(VmError::TypeMismatch {
                expected: "list",
                ..
            })
        ));
    }
}
