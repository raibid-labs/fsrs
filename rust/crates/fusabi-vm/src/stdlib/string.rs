// Fusabi String Standard Library
// Provides common string operations

use crate::value::Value;
use crate::vm::VmError;

/// String.length : string -> int
/// Returns the length of a string in characters (not bytes)
pub fn string_length(s: &Value) -> Result<Value, VmError> {
    match s {
        Value::Str(string) => Ok(Value::Int(string.chars().count() as i64)),
        _ => Err(VmError::TypeMismatch {
            expected: "string",
            got: s.type_name(),
        }),
    }
}

/// String.trim : string -> string
/// Removes leading and trailing whitespace
pub fn string_trim(s: &Value) -> Result<Value, VmError> {
    match s {
        Value::Str(string) => Ok(Value::Str(string.trim().to_string())),
        _ => Err(VmError::TypeMismatch {
            expected: "string",
            got: s.type_name(),
        }),
    }
}

/// String.toLower : string -> string
/// Converts string to lowercase
pub fn string_to_lower(s: &Value) -> Result<Value, VmError> {
    match s {
        Value::Str(string) => Ok(Value::Str(string.to_lowercase())),
        _ => Err(VmError::TypeMismatch {
            expected: "string",
            got: s.type_name(),
        }),
    }
}

/// String.toUpper : string -> string
/// Converts string to uppercase
pub fn string_to_upper(s: &Value) -> Result<Value, VmError> {
    match s {
        Value::Str(string) => Ok(Value::Str(string.to_uppercase())),
        _ => Err(VmError::TypeMismatch {
            expected: "string",
            got: s.type_name(),
        }),
    }
}

/// String.split : string -> string -> string list
/// Splits a string by a delimiter into a list of strings
pub fn string_split(delimiter: &Value, s: &Value) -> Result<Value, VmError> {
    match (delimiter, s) {
        (Value::Str(delim), Value::Str(string)) => {
            let parts: Vec<&str> = string.split(delim.as_str()).collect();
            let mut result = Value::Nil;

            // Build list in reverse order
            for part in parts.iter().rev() {
                result = Value::Cons {
                    head: Box::new(Value::Str(part.to_string())),
                    tail: Box::new(result),
                };
            }

            Ok(result)
        }
        (Value::Str(_), _) => Err(VmError::TypeMismatch {
            expected: "string",
            got: s.type_name(),
        }),
        _ => Err(VmError::TypeMismatch {
            expected: "string",
            got: delimiter.type_name(),
        }),
    }
}

/// String.concat : string list -> string
/// Concatenates a list of strings into a single string
pub fn string_concat(list: &Value) -> Result<Value, VmError> {
    println!("DEBUG: string_concat called with list={:?}", list);
    let mut result = String::new();
    let mut current = list.clone();

    loop {
        match current {
            Value::Nil => {
                println!("DEBUG: string_concat returning Str({:?})", result);
                return Ok(Value::Str(result));
            },
            Value::Cons { head, tail } => {
                if let Value::Str(s) = &*head {
                    result.push_str(s);
                } else {
                    return Err(VmError::TypeMismatch {
                        expected: "string list",
                        got: "list with non-string elements",
                    });
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
}

/// String.contains : string -> string -> bool
/// Returns true if haystack contains needle
pub fn string_contains(needle: &Value, haystack: &Value) -> Result<Value, VmError> {
    match (needle, haystack) {
        (Value::Str(n), Value::Str(h)) => Ok(Value::Bool(h.contains(n.as_str()))),
        (Value::Str(_), _) => Err(VmError::TypeMismatch {
            expected: "string",
            got: haystack.type_name(),
        }),
        _ => Err(VmError::TypeMismatch {
            expected: "string",
            got: needle.type_name(),
        }),
    }
}

/// String.startsWith : string -> string -> bool
/// Returns true if string starts with the given prefix
pub fn string_starts_with(prefix: &Value, s: &Value) -> Result<Value, VmError> {
    match (prefix, s) {
        (Value::Str(pre), Value::Str(string)) => Ok(Value::Bool(string.starts_with(pre.as_str()))),
        (Value::Str(_), _) => Err(VmError::TypeMismatch {
            expected: "string",
            got: s.type_name(),
        }),
        _ => Err(VmError::TypeMismatch {
            expected: "string",
            got: prefix.type_name(),
        }),
    }
}

/// String.endsWith : string -> string -> bool
/// Returns true if string ends with the given suffix
pub fn string_ends_with(suffix: &Value, s: &Value) -> Result<Value, VmError> {
    match (suffix, s) {
        (Value::Str(suf), Value::Str(string)) => Ok(Value::Bool(string.ends_with(suf.as_str()))),
        (Value::Str(_), _) => Err(VmError::TypeMismatch {
            expected: "string",
            got: s.type_name(),
        }),
        _ => Err(VmError::TypeMismatch {
            expected: "string",
            got: suffix.type_name(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_length() {
        let s = Value::Str("hello".to_string());
        let result = string_length(&s).unwrap();
        assert_eq!(result, Value::Int(5));
    }

    #[test]
    fn test_string_length_empty() {
        let s = Value::Str("".to_string());
        let result = string_length(&s).unwrap();
        assert_eq!(result, Value::Int(0));
    }

    #[test]
    fn test_string_length_unicode() {
        let s = Value::Str("Hello 世界".to_string());
        let result = string_length(&s).unwrap();
        assert_eq!(result, Value::Int(8)); // 6 ASCII + space + 2 Chinese chars
    }

    #[test]
    fn test_string_trim() {
        let s = Value::Str("  hello  ".to_string());
        let result = string_trim(&s).unwrap();
        assert_eq!(result, Value::Str("hello".to_string()));
    }

    #[test]
    fn test_string_trim_no_whitespace() {
        let s = Value::Str("hello".to_string());
        let result = string_trim(&s).unwrap();
        assert_eq!(result, Value::Str("hello".to_string()));
    }

    #[test]
    fn test_string_to_lower() {
        let s = Value::Str("HELLO World".to_string());
        let result = string_to_lower(&s).unwrap();
        assert_eq!(result, Value::Str("hello world".to_string()));
    }

    #[test]
    fn test_string_to_upper() {
        let s = Value::Str("hello WORLD".to_string());
        let result = string_to_upper(&s).unwrap();
        assert_eq!(result, Value::Str("HELLO WORLD".to_string()));
    }

    #[test]
    fn test_string_split() {
        let delim = Value::Str(" ".to_string());
        let s = Value::Str("hello world foo".to_string());
        let result = string_split(&delim, &s).unwrap();
        let expected = Value::vec_to_cons(vec![
            Value::Str("hello".to_string()),
            Value::Str("world".to_string()),
            Value::Str("foo".to_string()),
        ]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_string_split_no_delimiter() {
        let delim = Value::Str(",".to_string());
        let s = Value::Str("hello".to_string());
        let result = string_split(&delim, &s).unwrap();
        let expected = Value::vec_to_cons(vec![Value::Str("hello".to_string())]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_string_split_empty_parts() {
        let delim = Value::Str(",".to_string());
        let s = Value::Str("a,,b".to_string());
        let result = string_split(&delim, &s).unwrap();
        let expected = Value::vec_to_cons(vec![
            Value::Str("a".to_string()),
            Value::Str("".to_string()),
            Value::Str("b".to_string()),
        ]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_string_concat() {
        let list = Value::vec_to_cons(vec![
            Value::Str("hello".to_string()),
            Value::Str(" ".to_string()),
            Value::Str("world".to_string()),
        ]);
        let result = string_concat(&list).unwrap();
        assert_eq!(result, Value::Str("hello world".to_string()));
    }

    #[test]
    fn test_string_concat_empty() {
        let list = Value::Nil;
        let result = string_concat(&list).unwrap();
        assert_eq!(result, Value::Str("".to_string()));
    }

    #[test]
    fn test_string_concat_type_error() {
        let list = Value::vec_to_cons(vec![Value::Str("hello".to_string()), Value::Int(42)]);
        let result = string_concat(&list);
        assert!(result.is_err());
    }

    #[test]
    fn test_string_contains_true() {
        let needle = Value::Str("world".to_string());
        let haystack = Value::Str("hello world".to_string());
        let result = string_contains(&needle, &haystack).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_string_contains_false() {
        let needle = Value::Str("foo".to_string());
        let haystack = Value::Str("hello world".to_string());
        let result = string_contains(&needle, &haystack).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_string_starts_with_true() {
        let prefix = Value::Str("hello".to_string());
        let s = Value::Str("hello world".to_string());
        let result = string_starts_with(&prefix, &s).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_string_starts_with_false() {
        let prefix = Value::Str("world".to_string());
        let s = Value::Str("hello world".to_string());
        let result = string_starts_with(&prefix, &s).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_string_ends_with_true() {
        let suffix = Value::Str("world".to_string());
        let s = Value::Str("hello world".to_string());
        let result = string_ends_with(&suffix, &s).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_string_ends_with_false() {
        let suffix = Value::Str("hello".to_string());
        let s = Value::Str("hello world".to_string());
        let result = string_ends_with(&suffix, &s).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_string_type_errors() {
        let not_string = Value::Int(42);
        assert!(string_length(&not_string).is_err());
        assert!(string_trim(&not_string).is_err());
        assert!(string_to_lower(&not_string).is_err());
        assert!(string_to_upper(&not_string).is_err());
    }
}
