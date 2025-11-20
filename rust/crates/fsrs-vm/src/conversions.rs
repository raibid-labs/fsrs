// Type Conversions between Rust and FSRS Values
// Provides automatic marshalling for host interop

use crate::value::Value;
use std::convert::{TryFrom, TryInto};

// ========== From Rust to FSRS ==========

impl From<i64> for Value {
    fn from(n: i64) -> Self {
        Value::Int(n)
    }
}

impl From<i32> for Value {
    fn from(n: i32) -> Self {
        Value::Int(n as i64)
    }
}

impl From<usize> for Value {
    fn from(n: usize) -> Self {
        Value::Int(n as i64)
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::Str(s)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::Str(s.to_string())
    }
}

impl From<()> for Value {
    fn from(_: ()) -> Self {
        Value::Unit
    }
}

impl<T: Into<Value>> From<Vec<T>> for Value {
    fn from(vec: Vec<T>) -> Self {
        let elements: Vec<Value> = vec.into_iter().map(|v| v.into()).collect();
        Value::vec_to_cons(elements)
    }
}

// ========== From FSRS to Rust ==========

impl TryFrom<Value> for i64 {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        value
            .as_int()
            .ok_or_else(|| format!("Expected Int, got {}", value.type_name()))
    }
}

impl TryFrom<Value> for i32 {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let n = value
            .as_int()
            .ok_or_else(|| format!("Expected Int, got {}", value.type_name()))?;
        i32::try_from(n).map_err(|_| format!("Int value {} out of i32 range", n))
    }
}

impl TryFrom<Value> for usize {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let n = value
            .as_int()
            .ok_or_else(|| format!("Expected Int, got {}", value.type_name()))?;
        if n < 0 {
            return Err(format!("Cannot convert negative int {} to usize", n));
        }
        Ok(n as usize)
    }
}

impl TryFrom<Value> for bool {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        value
            .as_bool()
            .ok_or_else(|| format!("Expected Bool, got {}", value.type_name()))
    }
}

impl TryFrom<Value> for String {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        value
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| format!("Expected String, got {}", value.type_name()))
    }
}

impl TryFrom<Value> for () {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if value.is_unit() {
            Ok(())
        } else {
            Err(format!("Expected Unit, got {}", value.type_name()))
        }
    }
}

impl<T: TryFrom<Value>> TryFrom<Value> for Vec<T>
where
    T::Error: std::fmt::Debug,
{
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let mut result = Vec::new();
        let mut current = value;

        loop {
            match current {
                Value::Nil => break,
                Value::Cons { head, tail } => {
                    let item = (*head)
                        .try_into()
                        .map_err(|e| format!("Failed to convert list element: {:?}", e))?;
                    result.push(item);
                    current = *tail;
                }
                _ => {
                    return Err(format!(
                        "Expected List (Cons or Nil), got {}",
                        current.type_name()
                    ))
                }
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== From Rust to FSRS Tests ==========

    #[test]
    fn test_from_i64() {
        let v: Value = 42i64.into();
        assert_eq!(v, Value::Int(42));
    }

    #[test]
    fn test_from_i32() {
        let v: Value = 42i32.into();
        assert_eq!(v, Value::Int(42));
    }

    #[test]
    fn test_from_usize() {
        let v: Value = 42usize.into();
        assert_eq!(v, Value::Int(42));
    }

    #[test]
    fn test_from_bool() {
        let v_true: Value = true.into();
        let v_false: Value = false.into();
        assert_eq!(v_true, Value::Bool(true));
        assert_eq!(v_false, Value::Bool(false));
    }

    #[test]
    fn test_from_string() {
        let v: Value = "hello".to_string().into();
        assert_eq!(v, Value::Str("hello".to_string()));
    }

    #[test]
    fn test_from_str() {
        let v: Value = "hello".into();
        assert_eq!(v, Value::Str("hello".to_string()));
    }

    #[test]
    fn test_from_unit() {
        let v: Value = ().into();
        assert_eq!(v, Value::Unit);
    }

    #[test]
    fn test_from_vec() {
        let v: Value = vec![1i64, 2, 3].into();
        let expected = Value::vec_to_cons(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        assert_eq!(v, expected);
    }

    // ========== From FSRS to Rust Tests ==========

    #[test]
    fn test_try_from_i64() {
        let v = Value::Int(42);
        let n: i64 = v.try_into().unwrap();
        assert_eq!(n, 42);
    }

    #[test]
    fn test_try_from_i32() {
        let v = Value::Int(42);
        let n: i32 = v.try_into().unwrap();
        assert_eq!(n, 42);
    }

    #[test]
    fn test_try_from_usize() {
        let v = Value::Int(42);
        let n: usize = v.try_into().unwrap();
        assert_eq!(n, 42);
    }

    #[test]
    fn test_try_from_bool() {
        let v = Value::Bool(true);
        let b: bool = v.try_into().unwrap();
        assert!(b);
    }

    #[test]
    fn test_try_from_string() {
        let v = Value::Str("hello".to_string());
        let s: String = v.try_into().unwrap();
        assert_eq!(s, "hello");
    }

    #[test]
    fn test_try_from_unit() {
        let v = Value::Unit;
        let u: () = v.try_into().unwrap();
        assert_eq!(u, ());
    }

    #[test]
    fn test_try_from_vec() {
        let v = Value::vec_to_cons(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        let vec: Vec<i64> = v.try_into().unwrap();
        assert_eq!(vec, vec![1, 2, 3]);
    }

    #[test]
    fn test_try_from_type_mismatch() {
        let v = Value::Int(42);
        let result: Result<bool, String> = v.try_into();
        assert!(result.is_err());
    }

    #[test]
    fn test_roundtrip_i64() {
        let original = 42i64;
        let v: Value = original.into();
        let result: i64 = v.try_into().unwrap();
        assert_eq!(original, result);
    }

    #[test]
    fn test_roundtrip_vec() {
        let original = vec![1i64, 2, 3];
        let v: Value = original.clone().into();
        let result: Vec<i64> = v.try_into().unwrap();
        assert_eq!(original, result);
    }
}
