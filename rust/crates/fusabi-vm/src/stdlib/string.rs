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
    let mut result = String::new();
    let mut current = list.clone();

    loop {
        match current {
            Value::Nil => {
                return Ok(Value::Str(result));
            }
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

/// String.format : string -> any list -> string
/// Formats a string using printf-style formatting
/// Supported specifiers: %s (string), %d (int), %f (float), %.Nf (float with precision), %% (literal %)
/// Example: String.format "%s version %d.%d" ["MyApp"; 1; 0] returns "MyApp version 1.0"
pub fn string_format(format_str: &Value, args: &Value) -> Result<Value, VmError> {
    // Extract the format string
    let fmt = match format_str {
        Value::Str(s) => s,
        _ => {
            return Err(VmError::TypeMismatch {
                expected: "string",
                got: format_str.type_name(),
            })
        }
    };

    // Convert the list to a Vec for easier indexing
    let mut arg_vec = Vec::new();
    let mut current = args.clone();
    loop {
        match current {
            Value::Nil => break,
            Value::Cons { head, tail } => {
                arg_vec.push((*head).clone());
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

    // Process the format string
    let mut result = String::new();
    let mut chars = fmt.chars().peekable();
    let mut arg_index = 0;

    while let Some(ch) = chars.next() {
        if ch == '%' {
            if let Some(&next_ch) = chars.peek() {
                match next_ch {
                    '%' => {
                        // Literal %
                        result.push('%');
                        chars.next();
                    }
                    's' => {
                        // String specifier
                        chars.next();
                        if arg_index >= arg_vec.len() {
                            return Err(VmError::Runtime(
                                "Not enough arguments for format string".to_string(),
                            ));
                        }
                        let arg_to_str = arg_vec[arg_index].to_string(); // Convert any value to string using Display
                        result.push_str(&arg_to_str);
                        arg_index += 1;
                    }
                    'd' => {
                        // Integer specifier
                        chars.next();
                        if arg_index >= arg_vec.len() {
                            return Err(VmError::Runtime(
                                "Not enough arguments for format string".to_string(),
                            ));
                        }
                        match &arg_vec[arg_index] {
                            Value::Int(n) => result.push_str(&n.to_string()),
                            _ => {
                                return Err(VmError::Runtime(format!(
                                    "Expected int for %d, got {}",
                                    arg_vec[arg_index].type_name()
                                )))
                            }
                        }
                        arg_index += 1;
                    }
                    'f' => {
                        // Float specifier
                        chars.next();
                        if arg_index >= arg_vec.len() {
                            return Err(VmError::Runtime(
                                "Not enough arguments for format string".to_string(),
                            ));
                        }
                        match &arg_vec[arg_index] {
                            Value::Float(f) => result.push_str(&f.to_string()),
                            Value::Int(n) => result.push_str(&format!("{}.0", n)),
                            _ => {
                                return Err(VmError::Runtime(format!(
                                    "Expected float for %f, got {}",
                                    arg_vec[arg_index].type_name()
                                )))
                            }
                        }
                        arg_index += 1;
                    }
                    '.' => {
                        // Precision specifier (e.g., %.2f)
                        chars.next(); // consume '.'
                        let mut precision_str = String::new();
                        while let Some(&digit) = chars.peek() {
                            if digit.is_ascii_digit() {
                                precision_str.push(digit);
                                chars.next();
                            } else {
                                break;
                            }
                        }

                        if let Some(&'f') = chars.peek() {
                            chars.next(); // consume 'f'
                            let precision: usize = precision_str.parse().map_err(|_| {
                                VmError::Runtime("Invalid precision specifier".to_string())
                            })?;

                            if arg_index >= arg_vec.len() {
                                return Err(VmError::Runtime(
                                    "Not enough arguments for format string".to_string(),
                                ));
                            }

                            match &arg_vec[arg_index] {
                                Value::Float(f) => {
                                    result.push_str(&format!("{:.prec$}", f, prec = precision))
                                }
                                Value::Int(n) => result.push_str(&format!(
                                    "{:.prec$}",
                                    *n as f64,
                                    prec = precision
                                )),
                                _ => {
                                    return Err(VmError::Runtime(format!(
                                        "Expected float for %.{}f, got {}",
                                        precision,
                                        arg_vec[arg_index].type_name()
                                    )))
                                }
                            }
                            arg_index += 1;
                        } else {
                            return Err(VmError::Runtime(format!(
                                "Invalid format specifier: %.{}",
                                precision_str
                            )));
                        }
                    }
                    _ => {
                        return Err(VmError::Runtime(format!(
                            "Unknown format specifier: %{}",
                            next_ch
                        )))
                    }
                }
            } else {
                // Trailing % at end of string
                return Err(VmError::Runtime(
                    "Incomplete format specifier at end of string".to_string(),
                ));
            }
        } else {
            result.push(ch);
        }
    }

    // Check if all arguments were used
    if arg_index < arg_vec.len() {
        return Err(VmError::Runtime(format!(
            "Too many arguments for format string: expected {}, got {}",
            arg_index,
            arg_vec.len()
        )));
    }
    Ok(Value::Str(result))
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

    #[test]
    fn test_string_format_basic() {
        let fmt = Value::Str("Hello, %s!".to_string());
        let args = Value::vec_to_cons(vec![Value::Str("World".to_string())]);
        let result = string_format(&fmt, &args).unwrap();
        assert_eq!(result, Value::Str("Hello, World!".to_string()));
    }

    #[test]
    fn test_string_format_integer() {
        let fmt = Value::Str("Count: %d".to_string());
        let args = Value::vec_to_cons(vec![Value::Int(42)]);
        let result = string_format(&fmt, &args).unwrap();
        assert_eq!(result, Value::Str("Count: 42".to_string()));
    }

    #[test]
    fn test_string_format_float() {
        let fmt = Value::Str("Pi: %f".to_string());
        let args = Value::vec_to_cons(vec![Value::Float(3.14159)]);
        let result = string_format(&fmt, &args).unwrap();
        assert_eq!(result, Value::Str("Pi: 3.14159".to_string()));
    }

    #[test]
    fn test_string_format_precision() {
        let fmt = Value::Str("Value: %.2f".to_string());
        let args = Value::vec_to_cons(vec![Value::Float(3.14159)]);
        let result = string_format(&fmt, &args).unwrap();
        assert_eq!(result, Value::Str("Value: 3.14".to_string()));
    }

    #[test]
    fn test_string_format_precision_int() {
        let fmt = Value::Str("Value: %.2f".to_string());
        let args = Value::vec_to_cons(vec![Value::Int(42)]);
        let result = string_format(&fmt, &args).unwrap();
        assert_eq!(result, Value::Str("Value: 42.00".to_string()));
    }

    #[test]
    fn test_string_format_multiple_args() {
        let fmt = Value::Str("%s version %d.%d".to_string());
        let args = Value::vec_to_cons(vec![
            Value::Str("MyApp".to_string()),
            Value::Int(1),
            Value::Int(0),
        ]);
        let result = string_format(&fmt, &args).unwrap();
        assert_eq!(result, Value::Str("MyApp version 1.0".to_string()));
    }

    #[test]
    fn test_string_format_literal_percent() {
        let fmt = Value::Str("Progress: %d%%".to_string());
        let args = Value::vec_to_cons(vec![Value::Int(75)]);
        let result = string_format(&fmt, &args).unwrap();
        assert_eq!(result, Value::Str("Progress: 75%".to_string()));
    }

    #[test]
    fn test_string_format_mixed_types() {
        let fmt = Value::Str("%s: %d items at $%.2f each".to_string());
        let args = Value::vec_to_cons(vec![
            Value::Str("Product".to_string()),
            Value::Int(5),
            Value::Float(12.99),
        ]);
        let result = string_format(&fmt, &args).unwrap();
        assert_eq!(
            result,
            Value::Str("Product: 5 items at $12.99 each".to_string())
        );
    }

    #[test]
    fn test_string_format_empty_args() {
        let fmt = Value::Str("No args here".to_string());
        let args = Value::Nil;
        let result = string_format(&fmt, &args).unwrap();
        assert_eq!(result, Value::Str("No args here".to_string()));
    }

    #[test]
    fn test_string_format_string_with_int() {
        let fmt = Value::Str("Number as string: %s".to_string());
        let args = Value::vec_to_cons(vec![Value::Int(42)]);
        let result = string_format(&fmt, &args).unwrap();
        assert_eq!(result, Value::Str("Number as string: 42".to_string()));
    }

    #[test]
    fn test_string_format_not_enough_args() {
        let fmt = Value::Str("Hello, %s %s!".to_string());
        let args = Value::vec_to_cons(vec![Value::Str("World".to_string())]);
        let result = string_format(&fmt, &args);
        assert!(result.is_err());
    }

    #[test]
    fn test_string_format_too_many_args() {
        let fmt = Value::Str("Hello, %s!".to_string());
        let args = Value::vec_to_cons(vec![
            Value::Str("World".to_string()),
            Value::Str("Extra".to_string()),
        ]);
        let result = string_format(&fmt, &args);
        assert!(result.is_err());
    }

    #[test]
    fn test_string_format_wrong_type_for_d() {
        let fmt = Value::Str("Count: %d".to_string());
        let args = Value::vec_to_cons(vec![Value::Str("not a number".to_string())]);
        let result = string_format(&fmt, &args);
        assert!(result.is_err());
    }

    #[test]
    fn test_string_format_wrong_type_for_f() {
        let fmt = Value::Str("Value: %f".to_string());
        let args = Value::vec_to_cons(vec![Value::Str("not a number".to_string())]);
        let result = string_format(&fmt, &args);
        assert!(result.is_err());
    }

    #[test]
    fn test_string_format_invalid_specifier() {
        let fmt = Value::Str("Invalid: %x".to_string());
        let args = Value::vec_to_cons(vec![Value::Int(42)]);
        let result = string_format(&fmt, &args);
        assert!(result.is_err());
    }

    #[test]
    fn test_string_format_trailing_percent() {
        let fmt = Value::Str("Trailing %".to_string());
        let args = Value::Nil;
        let result = string_format(&fmt, &args);
        assert!(result.is_err());
    }

    #[test]
    fn test_string_format_non_string_format() {
        let fmt = Value::Int(42);
        let args = Value::Nil;
        let result = string_format(&fmt, &args);
        assert!(result.is_err());
    }

    #[test]
    fn test_string_format_non_list_args() {
        let fmt = Value::Str("Hello, %s!".to_string());
        let args = Value::Str("not a list".to_string());
        let result = string_format(&fmt, &args);
        assert!(result.is_err());
    }
}
