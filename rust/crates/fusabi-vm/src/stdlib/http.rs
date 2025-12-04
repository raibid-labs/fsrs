// HTTP client module for Fusabi
// Provides HTTP request functionality via reqwest

#[cfg(feature = "http")]
use crate::value::Value;
#[cfg(feature = "http")]
use crate::vm::VmError;
#[cfg(feature = "http")]
use std::collections::HashMap;
#[cfg(feature = "http")]
use std::sync::{Arc, Mutex};

#[cfg(feature = "http")]
/// Http.get : string -> string
/// Performs a GET request and returns the response body as a string
pub fn http_get(url: &Value) -> Result<Value, VmError> {
    let url_str = match url {
        Value::Str(s) => s,
        _ => {
            return Err(VmError::TypeMismatch {
                expected: "string",
                got: url.type_name(),
            })
        }
    };

    let response = reqwest::blocking::get(url_str).map_err(|e| {
        VmError::Runtime(format!("HTTP GET error: {}", e))
    })?;

    let body = response.text().map_err(|e| {
        VmError::Runtime(format!("Failed to read response body: {}", e))
    })?;

    Ok(Value::Str(body))
}

#[cfg(feature = "http")]
/// Http.post : string -> string -> string
/// Performs a POST request with the given body and returns the response body as a string
pub fn http_post(url: &Value, body: &Value) -> Result<Value, VmError> {
    let url_str = match url {
        Value::Str(s) => s,
        _ => {
            return Err(VmError::TypeMismatch {
                expected: "string",
                got: url.type_name(),
            })
        }
    };

    let body_str = match body {
        Value::Str(s) => s.clone(),
        _ => {
            return Err(VmError::TypeMismatch {
                expected: "string",
                got: body.type_name(),
            })
        }
    };

    let client = reqwest::blocking::Client::new();
    let response = client
        .post(url_str)
        .body(body_str)
        .send()
        .map_err(|e| VmError::Runtime(format!("HTTP POST error: {}", e)))?;

    let response_body = response.text().map_err(|e| {
        VmError::Runtime(format!("Failed to read response body: {}", e))
    })?;

    Ok(Value::Str(response_body))
}

#[cfg(feature = "http")]
/// Http.getJson : string -> 'a
/// Performs a GET request and parses the response as JSON
pub fn http_get_json(url: &Value) -> Result<Value, VmError> {
    let url_str = match url {
        Value::Str(s) => s,
        _ => {
            return Err(VmError::TypeMismatch {
                expected: "string",
                got: url.type_name(),
            })
        }
    };

    let response = reqwest::blocking::get(url_str).map_err(|e| {
        VmError::Runtime(format!("HTTP GET error: {}", e))
    })?;

    let json: serde_json::Value = response.json().map_err(|e| {
        VmError::Runtime(format!("Failed to parse JSON response: {}", e))
    })?;

    json_value_to_fusabi(&json)
}

#[cfg(feature = "http")]
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
            Ok(Value::Array(Arc::new(Mutex::new(values))))
        }
        serde_json::Value::Object(obj) => {
            let mut fields = HashMap::new();
            for (key, value) in obj {
                fields.insert(key.clone(), json_value_to_fusabi(value)?);
            }
            Ok(Value::Record(Arc::new(Mutex::new(fields))))
        }
    }
}

#[cfg(test)]
#[cfg(feature = "http")]
mod tests {
    use super::*;

    #[test]
    fn test_http_get_type_error() {
        let result = http_get(&Value::Int(42));
        assert!(result.is_err());
    }

    #[test]
    fn test_http_post_type_error_url() {
        let result = http_post(&Value::Int(42), &Value::Str("body".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_http_post_type_error_body() {
        let result = http_post(&Value::Str("http://example.com".to_string()), &Value::Int(42));
        assert!(result.is_err());
    }

    #[test]
    fn test_http_get_json_type_error() {
        let result = http_get_json(&Value::Int(42));
        assert!(result.is_err());
    }
}
