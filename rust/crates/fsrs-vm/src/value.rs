// FSRS VM Value Representation
// Defines runtime values for the bytecode VM

use std::fmt;

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
}

impl Value {
    /// Returns the type name of the value as a string
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Int(_) => "int",
            Value::Bool(_) => "bool",
            Value::Str(_) => "string",
            Value::Unit => "unit",
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

    /// Checks if the value is "truthy" for conditional logic
    /// - Bool(false) and Unit are falsy
    /// - Int(0) is falsy
    /// - Everything else is truthy
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Int(n) => *n != 0,
            Value::Str(s) => !s.is_empty(),
            Value::Unit => false,
        }
    }

    /// Checks if the value is Unit
    pub fn is_unit(&self) -> bool {
        matches!(self, Value::Unit)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{}", n),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Str(s) => write!(f, "{}", s),
            Value::Unit => write!(f, "()"),
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
    }

    // ========== Truthiness Tests ==========

    #[test]
    fn test_is_truthy_bool() {
        assert_eq!(Value::Bool(true).is_truthy(), true);
        assert_eq!(Value::Bool(false).is_truthy(), false);
    }

    #[test]
    fn test_is_truthy_int() {
        assert_eq!(Value::Int(1).is_truthy(), true);
        assert_eq!(Value::Int(-1).is_truthy(), true);
        assert_eq!(Value::Int(0).is_truthy(), false);
        assert_eq!(Value::Int(999).is_truthy(), true);
    }

    #[test]
    fn test_is_truthy_str() {
        assert_eq!(Value::Str("hello".to_string()).is_truthy(), true);
        assert_eq!(Value::Str("".to_string()).is_truthy(), false);
    }

    #[test]
    fn test_is_truthy_unit() {
        assert_eq!(Value::Unit.is_truthy(), false);
    }

    // ========== Unit Check Tests ==========

    #[test]
    fn test_is_unit() {
        assert_eq!(Value::Unit.is_unit(), true);
        assert_eq!(Value::Int(0).is_unit(), false);
        assert_eq!(Value::Bool(false).is_unit(), false);
        assert_eq!(Value::Str("".to_string()).is_unit(), false);
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
    fn test_inequality_different_types() {
        assert_ne!(Value::Int(42), Value::Bool(true));
        assert_ne!(Value::Bool(false), Value::Unit);
        assert_ne!(Value::Str("42".to_string()), Value::Int(42));
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
        assert_eq!(val.is_truthy(), false);
    }

    #[test]
    fn test_unicode_string() {
        let val = Value::Str("Hello 世界 🦀".to_string());
        assert_eq!(val.as_str(), Some("Hello 世界 🦀"));
        assert_eq!(format!("{}", val), "Hello 世界 🦀");
    }
}
