// Fusabi Standard Library - Print Functions
// Provides output functions similar to F#'s print/printfn

use crate::value::Value;
use crate::vm::VmError;

/// print : 'a -> unit
/// Prints a value to stdout without a trailing newline
pub fn print_value(value: &Value) -> Result<Value, VmError> {
    print!("{}", value);
    Ok(Value::Unit)
}

/// printfn : 'a -> unit
/// Prints a value to stdout with a trailing newline
pub fn printfn_value(value: &Value) -> Result<Value, VmError> {
    println!("{}", value);
    Ok(Value::Unit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_basic_types() {
        // Test with various types - these would print to stdout
        assert_eq!(print_value(&Value::Int(42)), Ok(Value::Unit));
        assert_eq!(print_value(&Value::Float(3.14)), Ok(Value::Unit));
        assert_eq!(print_value(&Value::Bool(true)), Ok(Value::Unit));
        assert_eq!(
            print_value(&Value::Str("hello".to_string())),
            Ok(Value::Unit)
        );
        assert_eq!(print_value(&Value::Unit), Ok(Value::Unit));
    }

    #[test]
    fn test_printfn_basic_types() {
        // Test with various types - these would print to stdout with newline
        assert_eq!(printfn_value(&Value::Int(42)), Ok(Value::Unit));
        assert_eq!(printfn_value(&Value::Float(3.14)), Ok(Value::Unit));
        assert_eq!(printfn_value(&Value::Bool(true)), Ok(Value::Unit));
        assert_eq!(
            printfn_value(&Value::Str("hello".to_string())),
            Ok(Value::Unit)
        );
        assert_eq!(printfn_value(&Value::Unit), Ok(Value::Unit));
    }

    #[test]
    fn test_print_list() {
        // Test with list type
        let list = Value::Cons {
            head: Box::new(Value::Int(1)),
            tail: Box::new(Value::Cons {
                head: Box::new(Value::Int(2)),
                tail: Box::new(Value::Nil),
            }),
        };
        assert_eq!(print_value(&list), Ok(Value::Unit));
        assert_eq!(printfn_value(&list), Ok(Value::Unit));
    }

    #[test]
    fn test_print_tuple() {
        // Test with tuple
        let tuple = Value::Tuple(vec![Value::Int(1), Value::Str("test".to_string())]);
        assert_eq!(print_value(&tuple), Ok(Value::Unit));
        assert_eq!(printfn_value(&tuple), Ok(Value::Unit));
    }
}
