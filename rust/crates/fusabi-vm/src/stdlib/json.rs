// JSON parsing and serialization module for Fusabi
// Provides functions to convert between JSON strings and Fusabi values

#[cfg(feature = "json")]
use crate::value::Value;
#[cfg(feature = "json")]
use crate::vm::VmError;
#[cfg(feature = "json")]
use std::cell::RefCell;
#[cfg(feature = "json")]
use std::collections::HashMap;
#[cfg(feature = "json")]
use std::rc::Rc;

#[cfg(feature = "json")]
/// Parse a JSON string into a Fusabi Value
/// Mapping:
/// - JSON null -> Value::Unit
/// - JSON bool -> Value::Bool
/// - JSON number (integer) -> Value::Int
/// - JSON number (float) -> Value::Float
/// - JSON string -> Value::Str
/// - JSON array -> Value::Array
/// - JSON object -> Value::Record
pub fn json_parse(arg: &Value) -> Result<Value, VmError> {
    match arg {
        Value::Str(json_str) => {
            let parsed: serde_json::Value = serde_json::from_str(json_str).map_err(|e| {
                VmError::Runtime(format!("JSON parse error: {}", e))
            })?;

            json_value_to_fusabi(&parsed)
        }
        _ => Err(VmError::TypeMismatch {
            expected: "string",
            got: arg.type_name(),
        }),
    }
}

#[cfg(feature = "json")]
/// Convert a serde_json::Value to a Fusabi Value
fn json_value_to_fusabi(json: &serde_json::Value) -> Result<Value, VmError> {
    match json {
        serde_json::Value::Null => Ok(Value::Unit),
        serde_json::Value::Bool(b) => Ok(Value::Bool(*b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Value::Int(i))
            } else if let Some(f) = n.as_f64() {
                Ok(Value::Float(f))
            } else {
                Err(VmError::Runtime("Invalid JSON number".to_string()))
            }
        }
        serde_json::Value::String(s) => Ok(Value::Str(s.clone())),
        serde_json::Value::Array(arr) => {
            let mut values = Vec::new();
            for item in arr {
                values.push(json_value_to_fusabi(item)?);
            }
            Ok(Value::Array(Rc::new(RefCell::new(values))))
        }
        serde_json::Value::Object(obj) => {
            let mut fields = HashMap::new();
            for (key, value) in obj {
                fields.insert(key.clone(), json_value_to_fusabi(value)?);
            }
            Ok(Value::Record(Rc::new(RefCell::new(fields))))
        }
    }
}

#[cfg(feature = "json")]
/// Convert a Fusabi Value to a JSON string
/// Mapping:
/// - Value::Unit -> "null"
/// - Value::Bool -> "true" or "false"
/// - Value::Int -> number
/// - Value::Float -> number
/// - Value::Str -> string
/// - Value::Array -> array
/// - Value::Record -> object
/// - Value::Map -> object
/// - Value::Tuple -> array
/// - Value::Cons/Nil -> array (converted from list)
pub fn json_stringify(arg: &Value) -> Result<Value, VmError> {
    let json_value = fusabi_value_to_json(arg)?;
    let json_str = serde_json::to_string(&json_value).map_err(|e| {
        VmError::Runtime(format!("JSON stringify error: {}", e))
    })?;
    Ok(Value::Str(json_str))
}

#[cfg(feature = "json")]
/// Convert a Fusabi Value to a serde_json::Value
fn fusabi_value_to_json(value: &Value) -> Result<serde_json::Value, VmError> {
    match value {
        Value::Unit => Ok(serde_json::Value::Null),
        Value::Bool(b) => Ok(serde_json::Value::Bool(*b)),
        Value::Int(i) => Ok(serde_json::Value::Number((*i).into())),
        Value::Float(f) => {
            serde_json::Number::from_f64(*f)
                .map(serde_json::Value::Number)
                .ok_or_else(|| VmError::Runtime("Invalid float value for JSON".to_string()))
        }
        Value::Str(s) => Ok(serde_json::Value::String(s.clone())),
        Value::Array(arr) => {
            let borrowed = arr.borrow();
            let mut json_arr = Vec::new();
            for item in borrowed.iter() {
                json_arr.push(fusabi_value_to_json(item)?);
            }
            Ok(serde_json::Value::Array(json_arr))
        }
        Value::Tuple(items) => {
            let mut json_arr = Vec::new();
            for item in items {
                json_arr.push(fusabi_value_to_json(item)?);
            }
            Ok(serde_json::Value::Array(json_arr))
        }
        Value::Record(rec) | Value::Map(rec) => {
            let borrowed = rec.borrow();
            let mut json_obj = serde_json::Map::new();
            for (key, val) in borrowed.iter() {
                json_obj.insert(key.clone(), fusabi_value_to_json(val)?);
            }
            Ok(serde_json::Value::Object(json_obj))
        }
        Value::Cons { .. } | Value::Nil => {
            // Convert cons list to array
            let vec = cons_to_vec(value)?;
            let mut json_arr = Vec::new();
            for item in vec {
                json_arr.push(fusabi_value_to_json(&item)?);
            }
            Ok(serde_json::Value::Array(json_arr))
        }
        _ => Err(VmError::Runtime(format!(
            "Cannot convert {} to JSON",
            value.type_name()
        ))),
    }
}

#[cfg(feature = "json")]
/// Helper function to convert a cons list to a vector
fn cons_to_vec(value: &Value) -> Result<Vec<Value>, VmError> {
    let mut result = Vec::new();
    let mut current = value.clone();

    loop {
        match current {
            Value::Nil => return Ok(result),
            Value::Cons { head, tail } => {
                result.push((*head).clone());
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

#[cfg(feature = "json")]
/// Pretty-print a Fusabi Value as formatted JSON
pub fn json_stringify_pretty(arg: &Value) -> Result<Value, VmError> {
    let json_value = fusabi_value_to_json(arg)?;
    let json_str = serde_json::to_string_pretty(&json_value).map_err(|e| {
        VmError::Runtime(format!("JSON stringify error: {}", e))
    })?;
    Ok(Value::Str(json_str))
}

#[cfg(test)]
#[cfg(feature = "json")]
mod tests {
    use super::*;

    #[test]
    fn test_parse_null() {
        let result = json_parse(&Value::Str("null".to_string())).unwrap();
        assert_eq!(result, Value::Unit);
    }

    #[test]
    fn test_parse_bool() {
        let result = json_parse(&Value::Str("true".to_string())).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_parse_int() {
        let result = json_parse(&Value::Str("42".to_string())).unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_parse_float() {
        let result = json_parse(&Value::Str("3.14".to_string())).unwrap();
        assert_eq!(result, Value::Float(3.14));
    }

    #[test]
    fn test_parse_string() {
        let result = json_parse(&Value::Str("\"hello\"".to_string())).unwrap();
        assert_eq!(result, Value::Str("hello".to_string()));
    }

    #[test]
    fn test_parse_array() {
        let result = json_parse(&Value::Str("[1, 2, 3]".to_string())).unwrap();
        match result {
            Value::Array(arr) => {
                let borrowed = arr.borrow();
                assert_eq!(borrowed.len(), 3);
                assert_eq!(borrowed[0], Value::Int(1));
                assert_eq!(borrowed[1], Value::Int(2));
                assert_eq!(borrowed[2], Value::Int(3));
            }
            _ => panic!("Expected Array"),
        }
    }

    #[test]
    fn test_parse_object() {
        let result = json_parse(&Value::Str(r#"{"name": "Alice", "age": 30}"#.to_string())).unwrap();
        match result {
            Value::Record(rec) => {
                let borrowed = rec.borrow();
                assert_eq!(borrowed.get("name"), Some(&Value::Str("Alice".to_string())));
                assert_eq!(borrowed.get("age"), Some(&Value::Int(30)));
            }
            _ => panic!("Expected Record"),
        }
    }

    #[test]
    fn test_stringify_null() {
        let result = json_stringify(&Value::Unit).unwrap();
        assert_eq!(result, Value::Str("null".to_string()));
    }

    #[test]
    fn test_stringify_bool() {
        let result = json_stringify(&Value::Bool(true)).unwrap();
        assert_eq!(result, Value::Str("true".to_string()));
    }

    #[test]
    fn test_stringify_int() {
        let result = json_stringify(&Value::Int(42)).unwrap();
        assert_eq!(result, Value::Str("42".to_string()));
    }

    #[test]
    fn test_stringify_string() {
        let result = json_stringify(&Value::Str("hello".to_string())).unwrap();
        assert_eq!(result, Value::Str("\"hello\"".to_string()));
    }

    #[test]
    fn test_stringify_array() {
        let arr = Value::Array(Rc::new(RefCell::new(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
        ])));
        let result = json_stringify(&arr).unwrap();
        assert_eq!(result, Value::Str("[1,2,3]".to_string()));
    }

    #[test]
    fn test_round_trip() {
        let json_str = r#"{"name":"Bob","items":[1,2,3],"active":true}"#;
        let parsed = json_parse(&Value::Str(json_str.to_string())).unwrap();
        let stringified = json_stringify(&parsed).unwrap();

        // Parse both to compare (order might differ)
        let original: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let round_trip: serde_json::Value = match stringified {
            Value::Str(s) => serde_json::from_str(&s).unwrap(),
            _ => panic!("Expected string"),
        };

        assert_eq!(original, round_trip);
    }
}
