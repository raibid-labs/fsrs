// Fusabi Option Standard Library
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
/// Option.defaultWith : (unit -> 'a) -> 'a option -> 'a
/// Returns the value inside Some, or calls the default function if None
pub fn option_default_with(vm: &mut crate::vm::Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::Runtime(format!(
            "Option.defaultWith expects 2 arguments, got {}",
            args.len()
        )));
    }

    let default_fn = &args[0];
    let opt = &args[1];

    match opt {
        Value::Variant {
            variant_name,
            fields,
            ..
        } => {
            if variant_name == "Some" && !fields.is_empty() {
                Ok(fields[0].clone())
            } else {
                vm.call_value(default_fn.clone(), &[Value::Unit])
            }
        }
        _ => Err(VmError::TypeMismatch {
            expected: "Option variant",
            got: opt.type_name(),
        }),
    }
}

/// Option.map : ('a -> 'b) -> 'a option -> 'b option
/// Transforms the value inside Some with the given function
pub fn option_map(vm: &mut crate::vm::Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::Runtime(format!(
            "Option.map expects 2 arguments, got {}",
            args.len()
        )));
    }

    let f = &args[0];
    let opt = &args[1];

    match opt {
        Value::Variant {
            type_name,
            variant_name,
            fields,
        } => {
            if variant_name == "Some" && !fields.is_empty() {
                let result = vm.call_value(f.clone(), &[fields[0].clone()])?;
                Ok(Value::Variant {
                    type_name: type_name.clone(),
                    variant_name: "Some".to_string(),
                    fields: vec![result],
                })
            } else {
                Ok(opt.clone())
            }
        }
        _ => Err(VmError::TypeMismatch {
            expected: "Option variant",
            got: opt.type_name(),
        }),
    }
}

/// Option.bind : ('a -> 'b option) -> 'a option -> 'b option
/// Monadic bind for Option (also known as flatMap or andThen)
pub fn option_bind(vm: &mut crate::vm::Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::Runtime(format!(
            "Option.bind expects 2 arguments, got {}",
            args.len()
        )));
    }

    let f = &args[0];
    let opt = &args[1];

    match opt {
        Value::Variant {
            variant_name,
            fields,
            ..
        } => {
            if variant_name == "Some" && !fields.is_empty() {
                vm.call_value(f.clone(), &[fields[0].clone()])
            } else {
                Ok(opt.clone())
            }
        }
        _ => Err(VmError::TypeMismatch {
            expected: "Option variant",
            got: opt.type_name(),
        }),
    }
}

/// Option.iter : ('a -> unit) -> 'a option -> unit
/// Calls the function with the value if Some, does nothing if None
pub fn option_iter(vm: &mut crate::vm::Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::Runtime(format!(
            "Option.iter expects 2 arguments, got {}",
            args.len()
        )));
    }

    let f = &args[0];
    let opt = &args[1];

    match opt {
        Value::Variant {
            variant_name,
            fields,
            ..
        } => {
            if variant_name == "Some" && !fields.is_empty() {
                vm.call_value(f.clone(), &[fields[0].clone()])?;
            }
            Ok(Value::Unit)
        }
        _ => Err(VmError::TypeMismatch {
            expected: "Option variant",
            got: opt.type_name(),
        }),
    }
}

/// Option.map2 : ('a -> 'b -> 'c) -> 'a option -> 'b option -> 'c option
/// Combines two options with a function
pub fn option_map2(vm: &mut crate::vm::Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 3 {
        return Err(VmError::Runtime(format!(
            "Option.map2 expects 3 arguments, got {}",
            args.len()
        )));
    }

    let f = &args[0];
    let opt1 = &args[1];
    let opt2 = &args[2];

    match (opt1, opt2) {
        (
            Value::Variant {
                variant_name: vn1,
                fields: fields1,
                ..
            },
            Value::Variant {
                type_name: tn2,
                variant_name: vn2,
                fields: fields2,
            },
        ) => {
            if vn1 == "Some" && vn2 == "Some" && !fields1.is_empty() && !fields2.is_empty() {
                let result = vm.call_value(f.clone(), &[fields1[0].clone(), fields2[0].clone()])?;
                Ok(Value::Variant {
                    type_name: tn2.clone(),
                    variant_name: "Some".to_string(),
                    fields: vec![result],
                })
            } else {
                Ok(Value::Variant {
                    type_name: tn2.clone(),
                    variant_name: "None".to_string(),
                    fields: vec![],
                })
            }
        }
        _ => Err(VmError::TypeMismatch {
            expected: "Option variant",
            got: "non-variant",
        }),
    }
}

/// Option.orElse : 'a option -> 'a option -> 'a option
/// Returns the first option if Some, otherwise returns the second option
pub fn option_or_else(opt1: &Value, opt2: &Value) -> Result<Value, VmError> {
    match opt1 {
        Value::Variant {
            variant_name,
            fields,
            ..
        } => {
            if variant_name == "Some" && !fields.is_empty() {
                Ok(opt1.clone())
            } else {
                Ok(opt2.clone())
            }
        }
        _ => Err(VmError::TypeMismatch {
            expected: "Option variant",
            got: opt1.type_name(),
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
