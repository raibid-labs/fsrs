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
        return Err(VmError::Runtime(format!("List.map expects 2 arguments, got {}", args.len())));
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

    let elements = list.list_to_vec().ok_or(VmError::Runtime("Malformed list".into()))?;
    let mut mapped_elements = Vec::new();
    
    for elem in elements {
        let result = vm.call_value(func.clone(), &[elem])?;
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
}
