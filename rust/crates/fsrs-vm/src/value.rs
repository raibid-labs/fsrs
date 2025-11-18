// FSRS VM Value Representation
// Defines runtime values for the bytecode VM

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

/// Runtime value representation for the FSRS VM
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// 64-bit signed integer
    Int(i64),
    /// Boolean value
    Bool(bool),
    /// Heap-allocated string
    Str(String),
    /// Unit type (void/null equivalent)
    Unit,
    /// Tuple of values (e.g., (1, 2), (x, "hello", true))
    Tuple(Vec<Value>),
    /// Cons cell for list construction (head :: tail)
    Cons { head: Box<Value>, tail: Box<Value> },
    /// Empty list []
    Nil,
    /// Mutable array with vector-based storage
    Array(Rc<RefCell<Vec<Value>>>),
}

impl Value {
    /// Returns the type name of the value as a string
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Int(_) => "int",
            Value::Bool(_) => "bool",
            Value::Str(_) => "string",
            Value::Unit => "unit",
            Value::Tuple(_) => "tuple",
            Value::Cons { .. } => "list",
            Value::Nil => "list",
            Value::Array(_) => "array",
        }
    }

    /// Attempts to extract an i64 from the value
    /// Returns Some(i64) if the value is Int, None otherwise
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Value::Int(n) => Some(*n),
            _ => None,
        }
    }

    /// Attempts to extract a bool from the value
    /// Returns Some(bool) if the value is Bool, None otherwise
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Attempts to extract a string reference from the value
    /// Returns Some(&str) if the value is Str, None otherwise
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Attempts to extract a tuple reference from the value
    /// Returns Some(&Vec<Value>) if the value is Tuple, None otherwise
    pub fn as_tuple(&self) -> Option<&Vec<Value>> {
        match self {
            Value::Tuple(elements) => Some(elements),
            _ => None,
        }
    }

    /// Attempts to extract cons cell components
    /// Returns Some((&Value, &Value)) if the value is Cons, None otherwise
    pub fn as_cons(&self) -> Option<(&Value, &Value)> {
        match self {
            Value::Cons { head, tail } => Some((head, tail)),
            _ => None,
        }
    }

    /// Checks if the value is "truthy" for conditional logic
    /// - Bool(false) and Unit are falsy
    /// - Int(0) is falsy
    /// - Nil (empty list) is falsy
    /// - Everything else is truthy
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Int(n) => *n != 0,
            Value::Str(s) => !s.is_empty(),
            Value::Unit => false,
            Value::Tuple(elements) => !elements.is_empty(),
            Value::Cons { .. } => true,
            Value::Nil => false,
            Value::Array(arr) => !arr.borrow().is_empty(),
        }
    }

    /// Checks if the value is Unit
    pub fn is_unit(&self) -> bool {
        matches!(self, Value::Unit)
    }

    /// Checks if the value is a Tuple
    pub fn is_tuple(&self) -> bool {
        matches!(self, Value::Tuple(_))
    }

    /// Checks if the value is a Cons cell
    pub fn is_cons(&self) -> bool {
        matches!(self, Value::Cons { .. })
    }

    /// Checks if the value is Nil (empty list)
    pub fn is_nil(&self) -> bool {
        matches!(self, Value::Nil)
    }

    /// Checks if the value is an Array
    pub fn is_array(&self) -> bool {
        matches!(self, Value::Array(_))
    }

    /// Attempts to extract an array reference from the value
    /// Returns Some(Rc<RefCell<Vec<Value>>>) if the value is Array, None otherwise
    pub fn as_array(&self) -> Option<Rc<RefCell<Vec<Value>>>> {
        if let Value::Array(arr) = self {
            Some(arr.clone())
        } else {
            None
        }
    }

    /// Get an element from an array by index
    /// Returns Err if not an array or index out of bounds
    pub fn array_get(&self, index: usize) -> Result<Value, String> {
        match self {
            Value::Array(arr) => {
                let arr = arr.borrow();
                arr.get(index)
                    .cloned()
                    .ok_or_else(|| format!("Array index out of bounds: {}", index))
            }
            _ => Err("Not an array".to_string()),
        }
    }

    /// Set an element in an array by index (mutable)
    /// Returns Err if not an array or index out of bounds
    pub fn array_set(&self, index: usize, value: Value) -> Result<(), String> {
        match self {
            Value::Array(arr) => {
                let mut arr = arr.borrow_mut();
                if index < arr.len() {
                    arr[index] = value;
                    Ok(())
                } else {
                    Err(format!("Array index out of bounds: {}", index))
                }
            }
            _ => Err("Not an array".to_string()),
        }
    }

    /// Get the length of an array
    /// Returns Err if not an array
    pub fn array_length(&self) -> Result<i64, String> {
        match self {
            Value::Array(arr) => Ok(arr.borrow().len() as i64),
            _ => Err("Not an array".to_string()),
        }
    }

    /// Convert a list to a vector of values
    /// Returns None if the list is malformed (tail is not Nil or Cons)
    pub fn list_to_vec(&self) -> Option<Vec<Value>> {
        let mut result = Vec::new();
        let mut current = self;

        loop {
            match current {
                Value::Nil => return Some(result),
                Value::Cons { head, tail } => {
                    result.push((**head).clone());
                    current = tail;
                }
                _ => return None, // Malformed list
            }
        }
    }

    /// Convert a vector of values to a cons list
    pub fn vec_to_cons(elements: Vec<Value>) -> Value {
        elements
            .into_iter()
            .rev()
            .fold(Value::Nil, |acc, elem| Value::Cons {
                head: Box::new(elem),
                tail: Box::new(acc),
            })
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{}", n),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Str(s) => write!(f, "{}", s),
            Value::Unit => write!(f, "()"),
            Value::Tuple(elements) => {
                write!(f, "(")?;
                for (i, element) in elements.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", element)?;
                }
                write!(f, ")")
            }
            Value::Nil => write!(f, "[]"),
            Value::Cons { .. } => {
                // Pretty-print as [e1; e2; e3]
                match self.list_to_vec() {
                    Some(elements) => {
                        write!(f, "[")?;
                        for (i, element) in elements.iter().enumerate() {
                            if i > 0 {
                                write!(f, "; ")?;
                            }
                            write!(f, "{}", element)?;
                        }
                        write!(f, "]")
                    }
                    None => {
                        // Fallback for malformed lists
                        write!(
                            f,
                            "Cons({}, {})",
                            self.as_cons().unwrap().0,
                            self.as_cons().unwrap().1
                        )
                    }
                }
            }
            Value::Array(arr) => {
                // Pretty-print as [|e1; e2; e3|]
                write!(f, "[|")?;
                let arr = arr.borrow();
                for (i, element) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, "; ")?;
                    }
                    write!(f, "{}", element)?;
                }
                write!(f, "|]")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== Construction Tests ==========

    #[test]
    fn test_value_int_construction() {
        let val = Value::Int(42);
        assert_eq!(val, Value::Int(42));
    }

    #[test]
    fn test_value_bool_construction() {
        let val_true = Value::Bool(true);
        let val_false = Value::Bool(false);
        assert_eq!(val_true, Value::Bool(true));
        assert_eq!(val_false, Value::Bool(false));
    }

    #[test]
    fn test_value_str_construction() {
        let val = Value::Str("hello".to_string());
        assert_eq!(val, Value::Str("hello".to_string()));
    }

    #[test]
    fn test_value_unit_construction() {
        let val = Value::Unit;
        assert_eq!(val, Value::Unit);
    }

    #[test]
    fn test_value_tuple_construction() {
        let val = Value::Tuple(vec![Value::Int(1), Value::Int(2)]);
        assert_eq!(val, Value::Tuple(vec![Value::Int(1), Value::Int(2)]));
    }

    // ========== Type Name Tests ==========

    #[test]
    fn test_type_name_int() {
        let val = Value::Int(100);
        assert_eq!(val.type_name(), "int");
    }

    #[test]
    fn test_type_name_bool() {
        let val = Value::Bool(true);
        assert_eq!(val.type_name(), "bool");
    }

    #[test]
    fn test_type_name_str() {
        let val = Value::Str("test".to_string());
        assert_eq!(val.type_name(), "string");
    }

    #[test]
    fn test_type_name_unit() {
        let val = Value::Unit;
        assert_eq!(val.type_name(), "unit");
    }

    #[test]
    fn test_type_name_tuple() {
        let val = Value::Tuple(vec![Value::Int(1), Value::Int(2)]);
        assert_eq!(val.type_name(), "tuple");
    }

    // ========== Extraction Tests (as_*) ==========

    #[test]
    fn test_as_int_success() {
        let val = Value::Int(42);
        assert_eq!(val.as_int(), Some(42));
    }

    #[test]
    fn test_as_int_failure() {
        assert_eq!(Value::Bool(true).as_int(), None);
        assert_eq!(Value::Str("42".to_string()).as_int(), None);
        assert_eq!(Value::Unit.as_int(), None);
        assert_eq!(Value::Tuple(vec![]).as_int(), None);
    }

    #[test]
    fn test_as_bool_success() {
        let val = Value::Bool(true);
        assert_eq!(val.as_bool(), Some(true));
    }

    #[test]
    fn test_as_bool_failure() {
        assert_eq!(Value::Int(1).as_bool(), None);
        assert_eq!(Value::Str("true".to_string()).as_bool(), None);
        assert_eq!(Value::Unit.as_bool(), None);
        assert_eq!(Value::Tuple(vec![]).as_bool(), None);
    }

    #[test]
    fn test_as_str_success() {
        let val = Value::Str("hello".to_string());
        assert_eq!(val.as_str(), Some("hello"));
    }

    #[test]
    fn test_as_str_failure() {
        assert_eq!(Value::Int(42).as_str(), None);
        assert_eq!(Value::Bool(false).as_str(), None);
        assert_eq!(Value::Unit.as_str(), None);
        assert_eq!(Value::Tuple(vec![]).as_str(), None);
    }

    #[test]
    fn test_as_tuple_success() {
        let val = Value::Tuple(vec![Value::Int(1), Value::Int(2)]);
        let tuple = val.as_tuple();
        assert!(tuple.is_some());
        assert_eq!(tuple.unwrap().len(), 2);
        assert_eq!(tuple.unwrap()[0], Value::Int(1));
        assert_eq!(tuple.unwrap()[1], Value::Int(2));
    }

    #[test]
    fn test_as_tuple_failure() {
        assert_eq!(Value::Int(42).as_tuple(), None);
        assert_eq!(Value::Bool(true).as_tuple(), None);
        assert_eq!(Value::Str("test".to_string()).as_tuple(), None);
        assert_eq!(Value::Unit.as_tuple(), None);
    }

    #[test]
    fn test_as_tuple_empty() {
        let val = Value::Tuple(vec![]);
        let tuple = val.as_tuple();
        assert!(tuple.is_some());
        assert_eq!(tuple.unwrap().len(), 0);
    }

    #[test]
    fn test_as_tuple_nested() {
        let val = Value::Tuple(vec![
            Value::Int(1),
            Value::Tuple(vec![Value::Int(2), Value::Int(3)]),
        ]);
        let tuple = val.as_tuple();
        assert!(tuple.is_some());
        assert_eq!(tuple.unwrap().len(), 2);
        assert!(tuple.unwrap()[1].is_tuple());
    }

    // ========== Truthiness Tests ==========

    #[test]
    fn test_is_truthy_bool() {
        assert!(Value::Bool(true).is_truthy());
        assert!(!Value::Bool(false).is_truthy());
    }

    #[test]
    fn test_is_truthy_int() {
        assert!(Value::Int(1).is_truthy());
        assert!(Value::Int(-1).is_truthy());
        assert!(!Value::Int(0).is_truthy());
        assert!(Value::Int(999).is_truthy());
    }

    #[test]
    fn test_is_truthy_str() {
        assert!(Value::Str("hello".to_string()).is_truthy());
        assert!(!Value::Str("".to_string()).is_truthy());
    }

    #[test]
    fn test_is_truthy_unit() {
        assert!(!Value::Unit.is_truthy());
    }

    #[test]
    fn test_is_truthy_tuple() {
        assert!(Value::Tuple(vec![Value::Int(1)]).is_truthy());
        assert!(!Value::Tuple(vec![]).is_truthy());
        assert!(Value::Tuple(vec![Value::Int(1), Value::Int(2)]).is_truthy());
    }

    // ========== Unit Check Tests ==========

    #[test]
    fn test_is_unit() {
        assert!(Value::Unit.is_unit());
        assert!(!Value::Int(0).is_unit());
        assert!(!Value::Bool(false).is_unit());
        assert!(!Value::Str("".to_string()).is_unit());
        assert!(!Value::Tuple(vec![]).is_unit());
    }

    // ========== Tuple Check Tests ==========

    #[test]
    fn test_is_tuple() {
        assert!(Value::Tuple(vec![]).is_tuple());
        assert!(Value::Tuple(vec![Value::Int(1)]).is_tuple());
        assert!(!Value::Int(42).is_tuple());
        assert!(!Value::Bool(true).is_tuple());
        assert!(!Value::Unit.is_tuple());
    }

    // ========== Clone Tests ==========

    #[test]
    fn test_clone_int() {
        let val1 = Value::Int(42);
        let val2 = val1.clone();
        assert_eq!(val1, val2);
    }

    #[test]
    fn test_clone_str() {
        let val1 = Value::Str("hello".to_string());
        let val2 = val1.clone();
        assert_eq!(val1, val2);
    }

    #[test]
    fn test_clone_tuple() {
        let val1 = Value::Tuple(vec![Value::Int(1), Value::Int(2)]);
        let val2 = val1.clone();
        assert_eq!(val1, val2);
    }

    // ========== Display Tests ==========

    #[test]
    fn test_display_int() {
        let val = Value::Int(42);
        assert_eq!(format!("{}", val), "42");
    }

    #[test]
    fn test_display_int_negative() {
        let val = Value::Int(-100);
        assert_eq!(format!("{}", val), "-100");
    }

    #[test]
    fn test_display_bool_true() {
        let val = Value::Bool(true);
        assert_eq!(format!("{}", val), "true");
    }

    #[test]
    fn test_display_bool_false() {
        let val = Value::Bool(false);
        assert_eq!(format!("{}", val), "false");
    }

    #[test]
    fn test_display_str() {
        let val = Value::Str("hello world".to_string());
        assert_eq!(format!("{}", val), "hello world");
    }

    #[test]
    fn test_display_unit() {
        let val = Value::Unit;
        assert_eq!(format!("{}", val), "()");
    }

    #[test]
    fn test_display_tuple_empty() {
        let val = Value::Tuple(vec![]);
        assert_eq!(format!("{}", val), "()");
    }

    #[test]
    fn test_display_tuple_pair() {
        let val = Value::Tuple(vec![Value::Int(1), Value::Int(2)]);
        assert_eq!(format!("{}", val), "(1, 2)");
    }

    #[test]
    fn test_display_tuple_triple() {
        let val = Value::Tuple(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        assert_eq!(format!("{}", val), "(1, 2, 3)");
    }

    #[test]
    fn test_display_tuple_nested() {
        let val = Value::Tuple(vec![
            Value::Int(1),
            Value::Tuple(vec![Value::Int(2), Value::Int(3)]),
        ]);
        assert_eq!(format!("{}", val), "(1, (2, 3))");
    }

    #[test]
    fn test_display_tuple_mixed_types() {
        let val = Value::Tuple(vec![
            Value::Int(42),
            Value::Str("hello".to_string()),
            Value::Bool(true),
        ]);
        assert_eq!(format!("{}", val), "(42, hello, true)");
    }

    // ========== Debug Tests ==========

    #[test]
    fn test_debug_int() {
        let val = Value::Int(42);
        assert_eq!(format!("{:?}", val), "Int(42)");
    }

    #[test]
    fn test_debug_bool() {
        let val = Value::Bool(true);
        assert_eq!(format!("{:?}", val), "Bool(true)");
    }

    #[test]
    fn test_debug_str() {
        let val = Value::Str("test".to_string());
        assert_eq!(format!("{:?}", val), "Str(\"test\")");
    }

    #[test]
    fn test_debug_unit() {
        let val = Value::Unit;
        assert_eq!(format!("{:?}", val), "Unit");
    }

    #[test]
    fn test_debug_tuple() {
        let val = Value::Tuple(vec![Value::Int(1), Value::Int(2)]);
        assert_eq!(format!("{:?}", val), "Tuple([Int(1), Int(2)])");
    }

    // ========== Equality Tests ==========

    #[test]
    fn test_equality_int() {
        assert_eq!(Value::Int(42), Value::Int(42));
        assert_ne!(Value::Int(42), Value::Int(43));
    }

    #[test]
    fn test_equality_bool() {
        assert_eq!(Value::Bool(true), Value::Bool(true));
        assert_ne!(Value::Bool(true), Value::Bool(false));
    }

    #[test]
    fn test_equality_str() {
        assert_eq!(
            Value::Str("hello".to_string()),
            Value::Str("hello".to_string())
        );
        assert_ne!(
            Value::Str("hello".to_string()),
            Value::Str("world".to_string())
        );
    }

    #[test]
    fn test_equality_unit() {
        assert_eq!(Value::Unit, Value::Unit);
    }

    #[test]
    fn test_equality_tuple() {
        assert_eq!(
            Value::Tuple(vec![Value::Int(1), Value::Int(2)]),
            Value::Tuple(vec![Value::Int(1), Value::Int(2)])
        );
        assert_ne!(
            Value::Tuple(vec![Value::Int(1), Value::Int(2)]),
            Value::Tuple(vec![Value::Int(2), Value::Int(1)])
        );
    }

    #[test]
    fn test_equality_tuple_nested() {
        let val1 = Value::Tuple(vec![
            Value::Int(1),
            Value::Tuple(vec![Value::Int(2), Value::Int(3)]),
        ]);
        let val2 = Value::Tuple(vec![
            Value::Int(1),
            Value::Tuple(vec![Value::Int(2), Value::Int(3)]),
        ]);
        assert_eq!(val1, val2);
    }

    #[test]
    fn test_inequality_different_types() {
        assert_ne!(Value::Int(42), Value::Bool(true));
        assert_ne!(Value::Bool(false), Value::Unit);
        assert_ne!(Value::Str("42".to_string()), Value::Int(42));
        assert_ne!(Value::Tuple(vec![]), Value::Unit);
    }

    // ========== Edge Case Tests ==========

    #[test]
    fn test_int_boundary_values() {
        let max = Value::Int(i64::MAX);
        let min = Value::Int(i64::MIN);
        assert_eq!(max.as_int(), Some(i64::MAX));
        assert_eq!(min.as_int(), Some(i64::MIN));
    }

    #[test]
    fn test_empty_string() {
        let val = Value::Str("".to_string());
        assert_eq!(val.as_str(), Some(""));
        assert!(!val.is_truthy());
    }

    #[test]
    fn test_unicode_string() {
        let val = Value::Str("Hello 世界 🦀".to_string());
        assert_eq!(val.as_str(), Some("Hello 世界 🦀"));
        assert_eq!(format!("{}", val), "Hello 世界 🦀");
    }

    #[test]
    fn test_tuple_large() {
        let val = Value::Tuple(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
            Value::Int(4),
            Value::Int(5),
        ]);
        assert_eq!(format!("{}", val), "(1, 2, 3, 4, 5)");
    }

    // ========== List/Cons Tests (Layer 3) ==========

    #[test]
    fn test_value_nil_construction() {
        let val = Value::Nil;
        assert_eq!(val, Value::Nil);
        assert!(val.is_nil());
        assert!(!val.is_cons());
    }

    #[test]
    fn test_value_cons_construction() {
        let val = Value::Cons {
            head: Box::new(Value::Int(1)),
            tail: Box::new(Value::Nil),
        };
        assert!(val.is_cons());
        assert!(!val.is_nil());
    }

    #[test]
    fn test_type_name_nil() {
        assert_eq!(Value::Nil.type_name(), "list");
    }

    #[test]
    fn test_type_name_cons() {
        let val = Value::Cons {
            head: Box::new(Value::Int(1)),
            tail: Box::new(Value::Nil),
        };
        assert_eq!(val.type_name(), "list");
    }

    #[test]
    fn test_is_nil() {
        assert!(Value::Nil.is_nil());
        assert!(!Value::Int(0).is_nil());
        assert!(!Value::Cons {
            head: Box::new(Value::Int(1)),
            tail: Box::new(Value::Nil),
        }
        .is_nil());
    }

    #[test]
    fn test_is_cons() {
        let val = Value::Cons {
            head: Box::new(Value::Int(1)),
            tail: Box::new(Value::Nil),
        };
        assert!(val.is_cons());
        assert!(!Value::Nil.is_cons());
        assert!(!Value::Int(42).is_cons());
    }

    #[test]
    fn test_as_cons_success() {
        let val = Value::Cons {
            head: Box::new(Value::Int(42)),
            tail: Box::new(Value::Nil),
        };
        let cons = val.as_cons();
        assert!(cons.is_some());
        let (head, tail) = cons.unwrap();
        assert_eq!(head, &Value::Int(42));
        assert_eq!(tail, &Value::Nil);
    }

    #[test]
    fn test_as_cons_failure() {
        assert_eq!(Value::Nil.as_cons(), None);
        assert_eq!(Value::Int(42).as_cons(), None);
        assert_eq!(Value::Bool(true).as_cons(), None);
    }

    #[test]
    fn test_display_nil() {
        assert_eq!(format!("{}", Value::Nil), "[]");
    }

    #[test]
    fn test_display_cons_single() {
        let val = Value::Cons {
            head: Box::new(Value::Int(42)),
            tail: Box::new(Value::Nil),
        };
        assert_eq!(format!("{}", val), "[42]");
    }

    #[test]
    fn test_display_cons_multiple() {
        let val = Value::Cons {
            head: Box::new(Value::Int(1)),
            tail: Box::new(Value::Cons {
                head: Box::new(Value::Int(2)),
                tail: Box::new(Value::Cons {
                    head: Box::new(Value::Int(3)),
                    tail: Box::new(Value::Nil),
                }),
            }),
        };
        assert_eq!(format!("{}", val), "[1; 2; 3]");
    }

    #[test]
    fn test_list_to_vec_empty() {
        let val = Value::Nil;
        let vec = val.list_to_vec();
        assert_eq!(vec, Some(vec![]));
    }

    #[test]
    fn test_list_to_vec_single() {
        let val = Value::Cons {
            head: Box::new(Value::Int(42)),
            tail: Box::new(Value::Nil),
        };
        let vec = val.list_to_vec();
        assert_eq!(vec, Some(vec![Value::Int(42)]));
    }

    #[test]
    fn test_list_to_vec_multiple() {
        let val = Value::Cons {
            head: Box::new(Value::Int(1)),
            tail: Box::new(Value::Cons {
                head: Box::new(Value::Int(2)),
                tail: Box::new(Value::Cons {
                    head: Box::new(Value::Int(3)),
                    tail: Box::new(Value::Nil),
                }),
            }),
        };
        let vec = val.list_to_vec();
        assert_eq!(vec, Some(vec![Value::Int(1), Value::Int(2), Value::Int(3)]));
    }

    #[test]
    fn test_list_to_vec_malformed() {
        // Malformed list: tail is not Nil or Cons
        let val = Value::Cons {
            head: Box::new(Value::Int(1)),
            tail: Box::new(Value::Int(2)),
        };
        let vec = val.list_to_vec();
        assert_eq!(vec, None);
    }

    #[test]
    fn test_vec_to_cons_empty() {
        let val = Value::vec_to_cons(vec![]);
        assert_eq!(val, Value::Nil);
    }

    #[test]
    fn test_vec_to_cons_single() {
        let val = Value::vec_to_cons(vec![Value::Int(42)]);
        assert_eq!(
            val,
            Value::Cons {
                head: Box::new(Value::Int(42)),
                tail: Box::new(Value::Nil),
            }
        );
    }

    #[test]
    fn test_vec_to_cons_multiple() {
        let val = Value::vec_to_cons(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        assert_eq!(
            val,
            Value::Cons {
                head: Box::new(Value::Int(1)),
                tail: Box::new(Value::Cons {
                    head: Box::new(Value::Int(2)),
                    tail: Box::new(Value::Cons {
                        head: Box::new(Value::Int(3)),
                        tail: Box::new(Value::Nil),
                    }),
                }),
            }
        );
    }

    #[test]
    fn test_cons_structural_equality() {
        let list1 = Value::Cons {
            head: Box::new(Value::Int(1)),
            tail: Box::new(Value::Cons {
                head: Box::new(Value::Int(2)),
                tail: Box::new(Value::Nil),
            }),
        };
        let list2 = Value::Cons {
            head: Box::new(Value::Int(1)),
            tail: Box::new(Value::Cons {
                head: Box::new(Value::Int(2)),
                tail: Box::new(Value::Nil),
            }),
        };
        assert_eq!(list1, list2);
    }

    #[test]
    fn test_cons_inequality() {
        let list1 = Value::Cons {
            head: Box::new(Value::Int(1)),
            tail: Box::new(Value::Nil),
        };
        let list2 = Value::Cons {
            head: Box::new(Value::Int(2)),
            tail: Box::new(Value::Nil),
        };
        assert_ne!(list1, list2);
    }

    #[test]
    fn test_cons_nested_lists() {
        // [[1; 2]; [3; 4]]
        let inner1 = Value::vec_to_cons(vec![Value::Int(1), Value::Int(2)]);
        let inner2 = Value::vec_to_cons(vec![Value::Int(3), Value::Int(4)]);
        let outer = Value::vec_to_cons(vec![inner1, inner2]);
        assert_eq!(format!("{}", outer), "[[1; 2]; [3; 4]]");
    }

    #[test]
    fn test_is_truthy_nil() {
        assert!(!Value::Nil.is_truthy());
    }

    #[test]
    fn test_is_truthy_cons() {
        let val = Value::Cons {
            head: Box::new(Value::Int(1)),
            tail: Box::new(Value::Nil),
        };
        assert!(val.is_truthy());
    }

    #[test]
    fn test_clone_nil() {
        let val1 = Value::Nil;
        let val2 = val1.clone();
        assert_eq!(val1, val2);
    }

    #[test]
    fn test_clone_cons() {
        let val1 = Value::Cons {
            head: Box::new(Value::Int(42)),
            tail: Box::new(Value::Nil),
        };
        let val2 = val1.clone();
        assert_eq!(val1, val2);
    }

    #[test]
    fn test_cons_roundtrip() {
        // Test vec -> cons -> vec roundtrip
        let original = vec![Value::Int(1), Value::Int(2), Value::Int(3)];
        let cons = Value::vec_to_cons(original.clone());
        let result = cons.list_to_vec().unwrap();
        assert_eq!(original, result);
    }

    #[test]
    fn test_debug_nil() {
        assert_eq!(format!("{:?}", Value::Nil), "Nil");
    }

    #[test]
    fn test_debug_cons() {
        let val = Value::Cons {
            head: Box::new(Value::Int(42)),
            tail: Box::new(Value::Nil),
        };
        let debug_str = format!("{:?}", val);
        assert!(debug_str.contains("Cons"));
        assert!(debug_str.contains("Int(42)"));
    }

    // ========== Array Tests (Layer 3 - Runtime) ==========

    #[test]
    fn test_array_empty_construction() {
        let arr = Value::Array(Rc::new(RefCell::new(vec![])));
        assert!(arr.is_array());
        assert_eq!(format!("{}", arr), "[||]");
    }

    #[test]
    fn test_array_single_element() {
        let arr = Value::Array(Rc::new(RefCell::new(vec![Value::Int(42)])));
        assert!(arr.is_array());
        assert_eq!(format!("{}", arr), "[|42|]");
    }

    #[test]
    fn test_array_multiple_elements() {
        let arr = Value::Array(Rc::new(RefCell::new(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
        ])));
        assert_eq!(format!("{}", arr), "[|1; 2; 3|]");
    }

    #[test]
    fn test_array_type_name() {
        let arr = Value::Array(Rc::new(RefCell::new(vec![Value::Int(1)])));
        assert_eq!(arr.type_name(), "array");
    }

    #[test]
    fn test_array_is_array() {
        let arr = Value::Array(Rc::new(RefCell::new(vec![])));
        assert!(arr.is_array());
        assert!(!Value::Int(42).is_array());
        assert!(!Value::Nil.is_array());
    }

    #[test]
    fn test_array_as_array_success() {
        let arr = Value::Array(Rc::new(RefCell::new(vec![Value::Int(1), Value::Int(2)])));
        let arr_ref = arr.as_array();
        assert!(arr_ref.is_some());
        let arr_ref = arr_ref.unwrap();
        assert_eq!(arr_ref.borrow().len(), 2);
        assert_eq!(arr_ref.borrow()[0], Value::Int(1));
    }

    #[test]
    fn test_array_as_array_failure() {
        assert!(Value::Int(42).as_array().is_none());
        assert!(Value::Nil.as_array().is_none());
        assert!(Value::Tuple(vec![]).as_array().is_none());
    }

    #[test]
    fn test_array_get_success() {
        let arr = Value::Array(Rc::new(RefCell::new(vec![
            Value::Int(10),
            Value::Int(20),
            Value::Int(30),
        ])));
        assert_eq!(arr.array_get(0), Ok(Value::Int(10)));
        assert_eq!(arr.array_get(1), Ok(Value::Int(20)));
        assert_eq!(arr.array_get(2), Ok(Value::Int(30)));
    }

    #[test]
    fn test_array_get_out_of_bounds() {
        let arr = Value::Array(Rc::new(RefCell::new(vec![Value::Int(1)])));
        assert!(arr.array_get(1).is_err());
        assert!(arr.array_get(10).is_err());
    }

    #[test]
    fn test_array_get_not_array() {
        let val = Value::Int(42);
        assert_eq!(val.array_get(0), Err("Not an array".to_string()));
    }

    #[test]
    fn test_array_set_success() {
        let arr = Value::Array(Rc::new(RefCell::new(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
        ])));
        assert!(arr.array_set(1, Value::Int(99)).is_ok());
        assert_eq!(arr.array_get(1), Ok(Value::Int(99)));
    }

    #[test]
    fn test_array_set_out_of_bounds() {
        let arr = Value::Array(Rc::new(RefCell::new(vec![Value::Int(1)])));
        assert!(arr.array_set(1, Value::Int(2)).is_err());
    }

    #[test]
    fn test_array_set_not_array() {
        let val = Value::Int(42);
        assert_eq!(
            val.array_set(0, Value::Int(1)),
            Err("Not an array".to_string())
        );
    }

    #[test]
    fn test_array_length_success() {
        let arr1 = Value::Array(Rc::new(RefCell::new(vec![])));
        assert_eq!(arr1.array_length(), Ok(0));

        let arr2 = Value::Array(Rc::new(RefCell::new(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
        ])));
        assert_eq!(arr2.array_length(), Ok(3));
    }

    #[test]
    fn test_array_length_not_array() {
        let val = Value::Int(42);
        assert_eq!(val.array_length(), Err("Not an array".to_string()));
    }

    #[test]
    fn test_array_equality_structural() {
        let arr1 = Value::Array(Rc::new(RefCell::new(vec![Value::Int(1), Value::Int(2)])));
        let arr2 = Value::Array(Rc::new(RefCell::new(vec![Value::Int(1), Value::Int(2)])));
        assert_eq!(arr1, arr2);
    }

    #[test]
    fn test_array_equality_reference() {
        let arr_rc = Rc::new(RefCell::new(vec![Value::Int(1), Value::Int(2)]));
        let arr1 = Value::Array(arr_rc.clone());
        let arr2 = Value::Array(arr_rc);
        assert_eq!(arr1, arr2);
    }

    #[test]
    fn test_array_inequality() {
        let arr1 = Value::Array(Rc::new(RefCell::new(vec![Value::Int(1), Value::Int(2)])));
        let arr2 = Value::Array(Rc::new(RefCell::new(vec![Value::Int(1), Value::Int(3)])));
        assert_ne!(arr1, arr2);
    }

    #[test]
    fn test_array_nested() {
        let inner = Value::Array(Rc::new(RefCell::new(vec![Value::Int(1), Value::Int(2)])));
        let outer = Value::Array(Rc::new(RefCell::new(vec![inner])));
        assert_eq!(format!("{}", outer), "[|[|1; 2|]|]");
    }

    #[test]
    fn test_array_mixed_types() {
        let arr = Value::Array(Rc::new(RefCell::new(vec![
            Value::Int(42),
            Value::Str("hello".to_string()),
            Value::Bool(true),
        ])));
        assert_eq!(format!("{}", arr), "[|42; hello; true|]");
    }

    #[test]
    fn test_array_is_truthy() {
        let arr1 = Value::Array(Rc::new(RefCell::new(vec![])));
        assert!(!arr1.is_truthy());

        let arr2 = Value::Array(Rc::new(RefCell::new(vec![Value::Int(1)])));
        assert!(arr2.is_truthy());
    }

    #[test]
    fn test_array_clone() {
        let arr1 = Value::Array(Rc::new(RefCell::new(vec![Value::Int(1), Value::Int(2)])));
        let arr2 = arr1.clone();
        assert_eq!(arr1, arr2);

        // Verify they share the same Rc (mutation affects both)
        arr1.array_set(0, Value::Int(99)).unwrap();
        assert_eq!(arr2.array_get(0), Ok(Value::Int(99)));
    }

    #[test]
    fn test_array_mutation() {
        let arr = Value::Array(Rc::new(RefCell::new(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
        ])));

        // Mutate array
        arr.array_set(0, Value::Int(10)).unwrap();
        arr.array_set(2, Value::Int(30)).unwrap();

        // Verify mutations
        assert_eq!(arr.array_get(0), Ok(Value::Int(10)));
        assert_eq!(arr.array_get(1), Ok(Value::Int(2)));
        assert_eq!(arr.array_get(2), Ok(Value::Int(30)));
    }
}
