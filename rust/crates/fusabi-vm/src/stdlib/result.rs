// Fusabi Result Standard Library
// Provides helper functions for working with Result variants

use crate::value::Value;
use crate::vm::{Vm, VmError};

/// Result.isOk : Result<'a, 'b> -> bool
/// Returns true if the result is Ok, false if Error
pub fn result_is_ok(result: &Value) -> Result<Value, VmError> {
    match result {
        Value::Variant { variant_name, .. } => Ok(Value::Bool(variant_name == "Ok")),
        _ => Err(VmError::TypeMismatch {
            expected: "Result variant",
            got: result.type_name(),
        }),
    }
}

/// Result.isError : Result<'a, 'b> -> bool
/// Returns true if the result is Error, false if Ok
pub fn result_is_error(result: &Value) -> Result<Value, VmError> {
    match result {
        Value::Variant { variant_name, .. } => Ok(Value::Bool(variant_name == "Error")),
        _ => Err(VmError::TypeMismatch {
            expected: "Result variant",
            got: result.type_name(),
        }),
    }
}

/// Result.defaultValue : 'a -> Result<'a, 'b> -> 'a
/// Returns the value inside Ok, or the default value if Error
pub fn result_default_value(default: &Value, result: &Value) -> Result<Value, VmError> {
    match result {
        Value::Variant {
            variant_name,
            fields,
            ..
        } => {
            if variant_name == "Ok" && !fields.is_empty() {
                Ok(fields[0].clone())
            } else {
                Ok(default.clone())
            }
        }
        _ => Err(VmError::TypeMismatch {
            expected: "Result variant",
            got: result.type_name(),
        }),
    }
}

/// Result.defaultWith : ('b -> 'a) -> Result<'a, 'b> -> 'a
/// Returns the value inside Ok, or calls the default function with the error if Error
pub fn result_default_with(vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::Runtime(format!(
            "Result.defaultWith expects 2 arguments, got {}",
            args.len()
        )));
    }

    let default_fn = &args[0];
    let result = &args[1];

    match result {
        Value::Variant {
            variant_name,
            fields,
            ..
        } => {
            if variant_name == "Ok" && !fields.is_empty() {
                Ok(fields[0].clone())
            } else if variant_name == "Error" && !fields.is_empty() {
                vm.call_value(default_fn.clone(), &[fields[0].clone()])
            } else {
                // Error with no fields - call with Unit
                vm.call_value(default_fn.clone(), &[Value::Unit])
            }
        }
        _ => Err(VmError::TypeMismatch {
            expected: "Result variant",
            got: result.type_name(),
        }),
    }
}

/// Result.map : ('a -> 'c) -> Result<'a, 'b> -> Result<'c, 'b>
/// Transforms the value inside Ok with the given function, passes through Error
pub fn result_map(vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::Runtime(format!(
            "Result.map expects 2 arguments, got {}",
            args.len()
        )));
    }

    let f = &args[0];
    let result = &args[1];

    match result {
        Value::Variant {
            type_name,
            variant_name,
            fields,
        } => {
            if variant_name == "Ok" && !fields.is_empty() {
                let mapped_value = vm.call_value(f.clone(), &[fields[0].clone()])?;
                Ok(Value::Variant {
                    type_name: type_name.clone(),
                    variant_name: "Ok".to_string(),
                    fields: vec![mapped_value],
                })
            } else {
                Ok(result.clone())
            }
        }
        _ => Err(VmError::TypeMismatch {
            expected: "Result variant",
            got: result.type_name(),
        }),
    }
}

/// Result.mapError : ('b -> 'c) -> Result<'a, 'b> -> Result<'a, 'c>
/// Transforms the error inside Error with the given function, passes through Ok
pub fn result_map_error(vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::Runtime(format!(
            "Result.mapError expects 2 arguments, got {}",
            args.len()
        )));
    }

    let f = &args[0];
    let result = &args[1];

    match result {
        Value::Variant {
            type_name,
            variant_name,
            fields,
        } => {
            if variant_name == "Error" && !fields.is_empty() {
                let mapped_error = vm.call_value(f.clone(), &[fields[0].clone()])?;
                Ok(Value::Variant {
                    type_name: type_name.clone(),
                    variant_name: "Error".to_string(),
                    fields: vec![mapped_error],
                })
            } else {
                Ok(result.clone())
            }
        }
        _ => Err(VmError::TypeMismatch {
            expected: "Result variant",
            got: result.type_name(),
        }),
    }
}

/// Result.bind : ('a -> Result<'c, 'b>) -> Result<'a, 'b> -> Result<'c, 'b>
/// Monadic bind for Result (also known as flatMap or andThen)
pub fn result_bind(vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::Runtime(format!(
            "Result.bind expects 2 arguments, got {}",
            args.len()
        )));
    }

    let f = &args[0];
    let result = &args[1];

    match result {
        Value::Variant {
            variant_name,
            fields,
            ..
        } => {
            if variant_name == "Ok" && !fields.is_empty() {
                vm.call_value(f.clone(), &[fields[0].clone()])
            } else {
                Ok(result.clone())
            }
        }
        _ => Err(VmError::TypeMismatch {
            expected: "Result variant",
            got: result.type_name(),
        }),
    }
}

/// Result.iter : ('a -> unit) -> Result<'a, 'b> -> unit
/// Calls the function with the Ok value if Ok, does nothing if Error
pub fn result_iter(vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::Runtime(format!(
            "Result.iter expects 2 arguments, got {}",
            args.len()
        )));
    }

    let f = &args[0];
    let result = &args[1];

    match result {
        Value::Variant {
            variant_name,
            fields,
            ..
        } => {
            if variant_name == "Ok" && !fields.is_empty() {
                vm.call_value(f.clone(), &[fields[0].clone()])?;
            }
            Ok(Value::Unit)
        }
        _ => Err(VmError::TypeMismatch {
            expected: "Result variant",
            got: result.type_name(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ok(value: Value) -> Value {
        Value::Variant {
            type_name: "Result".to_string(),
            variant_name: "Ok".to_string(),
            fields: vec![value],
        }
    }

    fn make_error(error: Value) -> Value {
        Value::Variant {
            type_name: "Result".to_string(),
            variant_name: "Error".to_string(),
            fields: vec![error],
        }
    }

    #[test]
    fn test_result_is_ok_true() {
        let result = make_ok(Value::Int(42));
        let output = result_is_ok(&result).unwrap();
        assert_eq!(output, Value::Bool(true));
    }

    #[test]
    fn test_result_is_ok_false() {
        let result = make_error(Value::Str("error".to_string()));
        let output = result_is_ok(&result).unwrap();
        assert_eq!(output, Value::Bool(false));
    }

    #[test]
    fn test_result_is_error_true() {
        let result = make_error(Value::Str("error".to_string()));
        let output = result_is_error(&result).unwrap();
        assert_eq!(output, Value::Bool(true));
    }

    #[test]
    fn test_result_is_error_false() {
        let result = make_ok(Value::Int(42));
        let output = result_is_error(&result).unwrap();
        assert_eq!(output, Value::Bool(false));
    }

    #[test]
    fn test_result_default_value_ok() {
        let result = make_ok(Value::Int(42));
        let default = Value::Int(0);
        let output = result_default_value(&default, &result).unwrap();
        assert_eq!(output, Value::Int(42));
    }

    #[test]
    fn test_result_default_value_error() {
        let result = make_error(Value::Str("error".to_string()));
        let default = Value::Int(99);
        let output = result_default_value(&default, &result).unwrap();
        assert_eq!(output, Value::Int(99));
    }

    #[test]
    fn test_result_default_value_different_types() {
        let result = make_ok(Value::Str("success".to_string()));
        let default = Value::Str("default".to_string());
        let output = result_default_value(&default, &result).unwrap();
        assert_eq!(output, Value::Str("success".to_string()));
    }

    #[test]
    fn test_result_type_error() {
        let not_result = Value::Int(42);
        assert!(result_is_ok(&not_result).is_err());
        assert!(result_is_error(&not_result).is_err());
        assert!(result_default_value(&Value::Int(0), &not_result).is_err());
    }

    #[test]
    fn test_result_different_variant_type() {
        // Test with a non-Result variant
        let other_variant = Value::Variant {
            type_name: "Option".to_string(),
            variant_name: "Some".to_string(),
            fields: vec![Value::Int(42)],
        };

        // Should still work - we only check variant_name, not type_name
        let is_ok = result_is_ok(&other_variant).unwrap();
        assert_eq!(is_ok, Value::Bool(false));

        let is_error = result_is_error(&other_variant).unwrap();
        assert_eq!(is_error, Value::Bool(false));
    }
}
