// FSRS Option Standard Library
// Provides helper functions for working with Option variants

use crate::value::Value;
use crate::vm::VmError;

/// Option.isSome : 'a option -> bool
/// Returns true if the option is Some, false if None
pub fn option_is_some(opt: &Value) -> Result<Value, VmError> {
    match opt {
        Value::Variant { variant_name, .. } => Ok(Value::Bool(variant_name == "Some")),
        _ => Err(VmError::TypeMismatch {
            expected: "Option variant",
            got: opt.type_name(),
        }),
    }
}

/// Option.isNone : 'a option -> bool
/// Returns true if the option is None, false if Some
pub fn option_is_none(opt: &Value) -> Result<Value, VmError> {
    match opt {
        Value::Variant { variant_name, .. } => Ok(Value::Bool(variant_name == "None")),
        _ => Err(VmError::TypeMismatch {
            expected: "Option variant",
            got: opt.type_name(),
        }),
    }
}

/// Option.defaultValue : 'a -> 'a option -> 'a
/// Returns the value inside Some, or the default value if None
pub fn option_default_value(default: &Value, opt: &Value) -> Result<Value, VmError> {
    match opt {
        Value::Variant {
            variant_name,
            fields,
            ..
        } => {
            if variant_name == "Some" && !fields.is_empty() {
                Ok(fields[0].clone())
            } else {
                Ok(default.clone())
            }
        }
        _ => Err(VmError::TypeMismatch {
            expected: "Option variant",
            got: opt.type_name(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_some(value: Value) -> Value {
        Value::Variant {
            type_name: "Option".to_string(),
            variant_name: "Some".to_string(),
            fields: vec![value],
        }
    }

    fn make_none() -> Value {
        Value::Variant {
            type_name: "Option".to_string(),
            variant_name: "None".to_string(),
            fields: vec![],
        }
    }

    #[test]
    fn test_option_is_some_true() {
        let opt = make_some(Value::Int(42));
        let result = option_is_some(&opt).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_option_is_some_false() {
        let opt = make_none();
        let result = option_is_some(&opt).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_option_is_none_true() {
        let opt = make_none();
        let result = option_is_none(&opt).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_option_is_none_false() {
        let opt = make_some(Value::Int(42));
        let result = option_is_none(&opt).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_option_default_value_some() {
        let opt = make_some(Value::Int(42));
        let default = Value::Int(0);
        let result = option_default_value(&default, &opt).unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_option_default_value_none() {
        let opt = make_none();
        let default = Value::Int(99);
        let result = option_default_value(&default, &opt).unwrap();
        assert_eq!(result, Value::Int(99));
    }

    #[test]
    fn test_option_default_value_different_types() {
        let opt = make_some(Value::Str("hello".to_string()));
        let default = Value::Str("default".to_string());
        let result = option_default_value(&default, &opt).unwrap();
        assert_eq!(result, Value::Str("hello".to_string()));
    }

    #[test]
    fn test_option_type_error() {
        let not_option = Value::Int(42);
        assert!(option_is_some(&not_option).is_err());
        assert!(option_is_none(&not_option).is_err());
        assert!(option_default_value(&Value::Int(0), &not_option).is_err());
    }

    #[test]
    fn test_option_different_variant_type() {
        // Test with a non-Option variant
        let other_variant = Value::Variant {
            type_name: "Result".to_string(),
            variant_name: "Ok".to_string(),
            fields: vec![Value::Int(42)],
        };

        // Should still work - we only check variant_name, not type_name
        let is_some = option_is_some(&other_variant).unwrap();
        assert_eq!(is_some, Value::Bool(false));

        let is_none = option_is_none(&other_variant).unwrap();
        assert_eq!(is_none, Value::Bool(false));
    }
}
