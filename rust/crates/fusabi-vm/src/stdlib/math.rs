// Fusabi Math Standard Library
// Provides mathematical constants and functions

use crate::value::Value;
use crate::vm::VmError;

// ============================================================================
// Mathematical Constants (as functions taking unit)
// ============================================================================

/// Math.pi : unit -> float
/// Returns the mathematical constant π (pi)
pub fn math_pi(_unit: &Value) -> Result<Value, VmError> {
    Ok(Value::Float(std::f64::consts::PI))
}

/// Math.e : unit -> float
/// Returns the mathematical constant e (Euler's number)
pub fn math_e(_unit: &Value) -> Result<Value, VmError> {
    Ok(Value::Float(std::f64::consts::E))
}

// ============================================================================
// Basic Math Functions
// ============================================================================

/// Math.abs : int -> int
/// Math.abs : float -> float
/// Returns the absolute value of a number
pub fn math_abs(n: &Value) -> Result<Value, VmError> {
    match n {
        Value::Int(i) => Ok(Value::Int(i.abs())),
        Value::Float(f) => Ok(Value::Float(f.abs())),
        _ => Err(VmError::TypeMismatch {
            expected: "int or float",
            got: n.type_name(),
        }),
    }
}

/// Math.sqrt : float -> float
/// Returns the square root of a number
pub fn math_sqrt(n: &Value) -> Result<Value, VmError> {
    match n {
        Value::Float(f) => {
            if *f < 0.0 {
                Err(VmError::Runtime(
                    "Cannot compute square root of negative number".to_string(),
                ))
            } else {
                Ok(Value::Float(f.sqrt()))
            }
        }
        Value::Int(i) => {
            let f = *i as f64;
            if f < 0.0 {
                Err(VmError::Runtime(
                    "Cannot compute square root of negative number".to_string(),
                ))
            } else {
                Ok(Value::Float(f.sqrt()))
            }
        }
        _ => Err(VmError::TypeMismatch {
            expected: "int or float",
            got: n.type_name(),
        }),
    }
}

/// Math.pow : float -> float -> float
/// Returns base raised to the power of exponent
pub fn math_pow(base: &Value, exponent: &Value) -> Result<Value, VmError> {
    let base_f = match base {
        Value::Float(f) => *f,
        Value::Int(i) => *i as f64,
        _ => {
            return Err(VmError::TypeMismatch {
                expected: "int or float",
                got: base.type_name(),
            })
        }
    };

    let exp_f = match exponent {
        Value::Float(f) => *f,
        Value::Int(i) => *i as f64,
        _ => {
            return Err(VmError::TypeMismatch {
                expected: "int or float",
                got: exponent.type_name(),
            })
        }
    };

    Ok(Value::Float(base_f.powf(exp_f)))
}

/// Math.max : int -> int -> int
/// Math.max : float -> float -> float
/// Returns the maximum of two values
pub fn math_max(a: &Value, b: &Value) -> Result<Value, VmError> {
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => Ok(Value::Int((*x).max(*y))),
        (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x.max(*y))),
        (Value::Int(x), Value::Float(y)) => Ok(Value::Float((*x as f64).max(*y))),
        (Value::Float(x), Value::Int(y)) => Ok(Value::Float(x.max(*y as f64))),
        _ => Err(VmError::Runtime(format!(
            "Math.max expects int or float, got {} and {}",
            a.type_name(),
            b.type_name()
        ))),
    }
}

/// Math.min : int -> int -> int
/// Math.min : float -> float -> float
/// Returns the minimum of two values
pub fn math_min(a: &Value, b: &Value) -> Result<Value, VmError> {
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => Ok(Value::Int((*x).min(*y))),
        (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x.min(*y))),
        (Value::Int(x), Value::Float(y)) => Ok(Value::Float((*x as f64).min(*y))),
        (Value::Float(x), Value::Int(y)) => Ok(Value::Float(x.min(*y as f64))),
        _ => Err(VmError::Runtime(format!(
            "Math.min expects int or float, got {} and {}",
            a.type_name(),
            b.type_name()
        ))),
    }
}

// ============================================================================
// Trigonometric Functions (radians)
// ============================================================================

/// Math.sin : float -> float
/// Returns the sine of an angle in radians
pub fn math_sin(angle: &Value) -> Result<Value, VmError> {
    match angle {
        Value::Float(f) => Ok(Value::Float(f.sin())),
        Value::Int(i) => Ok(Value::Float((*i as f64).sin())),
        _ => Err(VmError::TypeMismatch {
            expected: "int or float",
            got: angle.type_name(),
        }),
    }
}

/// Math.cos : float -> float
/// Returns the cosine of an angle in radians
pub fn math_cos(angle: &Value) -> Result<Value, VmError> {
    match angle {
        Value::Float(f) => Ok(Value::Float(f.cos())),
        Value::Int(i) => Ok(Value::Float((*i as f64).cos())),
        _ => Err(VmError::TypeMismatch {
            expected: "int or float",
            got: angle.type_name(),
        }),
    }
}

/// Math.tan : float -> float
/// Returns the tangent of an angle in radians
pub fn math_tan(angle: &Value) -> Result<Value, VmError> {
    match angle {
        Value::Float(f) => Ok(Value::Float(f.tan())),
        Value::Int(i) => Ok(Value::Float((*i as f64).tan())),
        _ => Err(VmError::TypeMismatch {
            expected: "int or float",
            got: angle.type_name(),
        }),
    }
}

/// Math.asin : float -> float
/// Returns the arcsine (inverse sine) of a value, result in radians
pub fn math_asin(x: &Value) -> Result<Value, VmError> {
    let f = match x {
        Value::Float(f) => *f,
        Value::Int(i) => *i as f64,
        _ => {
            return Err(VmError::TypeMismatch {
                expected: "int or float",
                got: x.type_name(),
            })
        }
    };

    if f < -1.0 || f > 1.0 {
        return Err(VmError::Runtime(
            "asin argument must be in range [-1, 1]".to_string(),
        ));
    }

    Ok(Value::Float(f.asin()))
}

/// Math.acos : float -> float
/// Returns the arccosine (inverse cosine) of a value, result in radians
pub fn math_acos(x: &Value) -> Result<Value, VmError> {
    let f = match x {
        Value::Float(f) => *f,
        Value::Int(i) => *i as f64,
        _ => {
            return Err(VmError::TypeMismatch {
                expected: "int or float",
                got: x.type_name(),
            })
        }
    };

    if f < -1.0 || f > 1.0 {
        return Err(VmError::Runtime(
            "acos argument must be in range [-1, 1]".to_string(),
        ));
    }

    Ok(Value::Float(f.acos()))
}

/// Math.atan : float -> float
/// Returns the arctangent (inverse tangent) of a value, result in radians
pub fn math_atan(x: &Value) -> Result<Value, VmError> {
    match x {
        Value::Float(f) => Ok(Value::Float(f.atan())),
        Value::Int(i) => Ok(Value::Float((*i as f64).atan())),
        _ => Err(VmError::TypeMismatch {
            expected: "int or float",
            got: x.type_name(),
        }),
    }
}

/// Math.atan2 : float -> float -> float
/// Returns the arctangent of y/x in radians, using the signs to determine the quadrant
pub fn math_atan2(y: &Value, x: &Value) -> Result<Value, VmError> {
    let y_f = match y {
        Value::Float(f) => *f,
        Value::Int(i) => *i as f64,
        _ => {
            return Err(VmError::TypeMismatch {
                expected: "int or float",
                got: y.type_name(),
            })
        }
    };

    let x_f = match x {
        Value::Float(f) => *f,
        Value::Int(i) => *i as f64,
        _ => {
            return Err(VmError::TypeMismatch {
                expected: "int or float",
                got: x.type_name(),
            })
        }
    };

    Ok(Value::Float(y_f.atan2(x_f)))
}

// ============================================================================
// Logarithmic and Exponential Functions
// ============================================================================

/// Math.log : float -> float
/// Returns the natural logarithm (base e) of a number
pub fn math_log(x: &Value) -> Result<Value, VmError> {
    let f = match x {
        Value::Float(f) => *f,
        Value::Int(i) => *i as f64,
        _ => {
            return Err(VmError::TypeMismatch {
                expected: "int or float",
                got: x.type_name(),
            })
        }
    };

    if f <= 0.0 {
        return Err(VmError::Runtime(
            "log argument must be positive".to_string(),
        ));
    }

    Ok(Value::Float(f.ln()))
}

/// Math.log10 : float -> float
/// Returns the base-10 logarithm of a number
pub fn math_log10(x: &Value) -> Result<Value, VmError> {
    let f = match x {
        Value::Float(f) => *f,
        Value::Int(i) => *i as f64,
        _ => {
            return Err(VmError::TypeMismatch {
                expected: "int or float",
                got: x.type_name(),
            })
        }
    };

    if f <= 0.0 {
        return Err(VmError::Runtime(
            "log10 argument must be positive".to_string(),
        ));
    }

    Ok(Value::Float(f.log10()))
}

/// Math.exp : float -> float
/// Returns e raised to the power of x
pub fn math_exp(x: &Value) -> Result<Value, VmError> {
    match x {
        Value::Float(f) => Ok(Value::Float(f.exp())),
        Value::Int(i) => Ok(Value::Float((*i as f64).exp())),
        _ => Err(VmError::TypeMismatch {
            expected: "int or float",
            got: x.type_name(),
        }),
    }
}

// ============================================================================
// Rounding Functions
// ============================================================================

/// Math.floor : float -> float
/// Returns the largest integer less than or equal to the number
pub fn math_floor(x: &Value) -> Result<Value, VmError> {
    match x {
        Value::Float(f) => Ok(Value::Float(f.floor())),
        Value::Int(i) => Ok(Value::Float(*i as f64)),
        _ => Err(VmError::TypeMismatch {
            expected: "int or float",
            got: x.type_name(),
        }),
    }
}

/// Math.ceil : float -> float
/// Returns the smallest integer greater than or equal to the number
pub fn math_ceil(x: &Value) -> Result<Value, VmError> {
    match x {
        Value::Float(f) => Ok(Value::Float(f.ceil())),
        Value::Int(i) => Ok(Value::Float(*i as f64)),
        _ => Err(VmError::TypeMismatch {
            expected: "int or float",
            got: x.type_name(),
        }),
    }
}

/// Math.round : float -> float
/// Returns the nearest integer, rounding half-way cases away from 0.0
pub fn math_round(x: &Value) -> Result<Value, VmError> {
    match x {
        Value::Float(f) => Ok(Value::Float(f.round())),
        Value::Int(i) => Ok(Value::Float(*i as f64)),
        _ => Err(VmError::TypeMismatch {
            expected: "int or float",
            got: x.type_name(),
        }),
    }
}

/// Math.truncate : float -> float
/// Returns the integer part of a number, removing any fractional digits
pub fn math_truncate(x: &Value) -> Result<Value, VmError> {
    match x {
        Value::Float(f) => Ok(Value::Float(f.trunc())),
        Value::Int(i) => Ok(Value::Float(*i as f64)),
        _ => Err(VmError::TypeMismatch {
            expected: "int or float",
            got: x.type_name(),
        }),
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Helper for float comparison with tolerance
    fn assert_float_eq(a: f64, b: f64, epsilon: f64) {
        assert!((a - b).abs() < epsilon, "{} != {}", a, b);
    }

    #[test]
    fn test_math_pi() {
        let result = math_pi(&Value::Unit).unwrap();
        match result {
            Value::Float(f) => assert_float_eq(f, std::f64::consts::PI, 1e-10),
            _ => panic!("Expected Float"),
        }
    }

    #[test]
    fn test_math_e() {
        let result = math_e(&Value::Unit).unwrap();
        match result {
            Value::Float(f) => assert_float_eq(f, std::f64::consts::E, 1e-10),
            _ => panic!("Expected Float"),
        }
    }

    #[test]
    fn test_math_abs_int() {
        assert_eq!(math_abs(&Value::Int(-5)).unwrap(), Value::Int(5));
        assert_eq!(math_abs(&Value::Int(5)).unwrap(), Value::Int(5));
        assert_eq!(math_abs(&Value::Int(0)).unwrap(), Value::Int(0));
    }

    #[test]
    fn test_math_abs_float() {
        assert_eq!(math_abs(&Value::Float(-3.14)).unwrap(), Value::Float(3.14));
        assert_eq!(math_abs(&Value::Float(3.14)).unwrap(), Value::Float(3.14));
    }

    #[test]
    fn test_math_sqrt() {
        match math_sqrt(&Value::Float(9.0)).unwrap() {
            Value::Float(f) => assert_float_eq(f, 3.0, 1e-10),
            _ => panic!("Expected Float"),
        }
        match math_sqrt(&Value::Int(16)).unwrap() {
            Value::Float(f) => assert_float_eq(f, 4.0, 1e-10),
            _ => panic!("Expected Float"),
        }
        assert!(math_sqrt(&Value::Float(-1.0)).is_err());
    }

    #[test]
    fn test_math_pow() {
        match math_pow(&Value::Float(2.0), &Value::Float(3.0)).unwrap() {
            Value::Float(f) => assert_float_eq(f, 8.0, 1e-10),
            _ => panic!("Expected Float"),
        }
        match math_pow(&Value::Int(2), &Value::Int(10)).unwrap() {
            Value::Float(f) => assert_float_eq(f, 1024.0, 1e-10),
            _ => panic!("Expected Float"),
        }
    }

    #[test]
    fn test_math_max() {
        assert_eq!(math_max(&Value::Int(5), &Value::Int(3)).unwrap(), Value::Int(5));
        assert_eq!(
            math_max(&Value::Float(3.14), &Value::Float(2.71)).unwrap(),
            Value::Float(3.14)
        );
        assert_eq!(
            math_max(&Value::Int(5), &Value::Float(3.14)).unwrap(),
            Value::Float(5.0)
        );
    }

    #[test]
    fn test_math_min() {
        assert_eq!(math_min(&Value::Int(5), &Value::Int(3)).unwrap(), Value::Int(3));
        assert_eq!(
            math_min(&Value::Float(3.14), &Value::Float(2.71)).unwrap(),
            Value::Float(2.71)
        );
        assert_eq!(
            math_min(&Value::Int(5), &Value::Float(3.14)).unwrap(),
            Value::Float(3.14)
        );
    }

    #[test]
    fn test_math_sin() {
        match math_sin(&Value::Float(0.0)).unwrap() {
            Value::Float(f) => assert_float_eq(f, 0.0, 1e-10),
            _ => panic!("Expected Float"),
        }
        match math_sin(&Value::Float(std::f64::consts::PI / 2.0)).unwrap() {
            Value::Float(f) => assert_float_eq(f, 1.0, 1e-10),
            _ => panic!("Expected Float"),
        }
    }

    #[test]
    fn test_math_cos() {
        match math_cos(&Value::Float(0.0)).unwrap() {
            Value::Float(f) => assert_float_eq(f, 1.0, 1e-10),
            _ => panic!("Expected Float"),
        }
        match math_cos(&Value::Float(std::f64::consts::PI)).unwrap() {
            Value::Float(f) => assert_float_eq(f, -1.0, 1e-10),
            _ => panic!("Expected Float"),
        }
    }

    #[test]
    fn test_math_tan() {
        match math_tan(&Value::Float(0.0)).unwrap() {
            Value::Float(f) => assert_float_eq(f, 0.0, 1e-10),
            _ => panic!("Expected Float"),
        }
        match math_tan(&Value::Float(std::f64::consts::PI / 4.0)).unwrap() {
            Value::Float(f) => assert_float_eq(f, 1.0, 1e-10),
            _ => panic!("Expected Float"),
        }
    }

    #[test]
    fn test_math_asin() {
        match math_asin(&Value::Float(0.0)).unwrap() {
            Value::Float(f) => assert_float_eq(f, 0.0, 1e-10),
            _ => panic!("Expected Float"),
        }
        match math_asin(&Value::Float(1.0)).unwrap() {
            Value::Float(f) => assert_float_eq(f, std::f64::consts::PI / 2.0, 1e-10),
            _ => panic!("Expected Float"),
        }
        assert!(math_asin(&Value::Float(1.5)).is_err());
    }

    #[test]
    fn test_math_acos() {
        match math_acos(&Value::Float(1.0)).unwrap() {
            Value::Float(f) => assert_float_eq(f, 0.0, 1e-10),
            _ => panic!("Expected Float"),
        }
        match math_acos(&Value::Float(0.0)).unwrap() {
            Value::Float(f) => assert_float_eq(f, std::f64::consts::PI / 2.0, 1e-10),
            _ => panic!("Expected Float"),
        }
        assert!(math_acos(&Value::Float(1.5)).is_err());
    }

    #[test]
    fn test_math_atan() {
        match math_atan(&Value::Float(0.0)).unwrap() {
            Value::Float(f) => assert_float_eq(f, 0.0, 1e-10),
            _ => panic!("Expected Float"),
        }
        match math_atan(&Value::Float(1.0)).unwrap() {
            Value::Float(f) => assert_float_eq(f, std::f64::consts::PI / 4.0, 1e-10),
            _ => panic!("Expected Float"),
        }
    }

    #[test]
    fn test_math_atan2() {
        match math_atan2(&Value::Float(0.0), &Value::Float(1.0)).unwrap() {
            Value::Float(f) => assert_float_eq(f, 0.0, 1e-10),
            _ => panic!("Expected Float"),
        }
        match math_atan2(&Value::Float(1.0), &Value::Float(0.0)).unwrap() {
            Value::Float(f) => assert_float_eq(f, std::f64::consts::PI / 2.0, 1e-10),
            _ => panic!("Expected Float"),
        }
    }

    #[test]
    fn test_math_log() {
        match math_log(&Value::Float(std::f64::consts::E)).unwrap() {
            Value::Float(f) => assert_float_eq(f, 1.0, 1e-10),
            _ => panic!("Expected Float"),
        }
        assert!(math_log(&Value::Float(0.0)).is_err());
        assert!(math_log(&Value::Float(-1.0)).is_err());
    }

    #[test]
    fn test_math_log10() {
        match math_log10(&Value::Float(100.0)).unwrap() {
            Value::Float(f) => assert_float_eq(f, 2.0, 1e-10),
            _ => panic!("Expected Float"),
        }
        match math_log10(&Value::Int(1000)).unwrap() {
            Value::Float(f) => assert_float_eq(f, 3.0, 1e-10),
            _ => panic!("Expected Float"),
        }
        assert!(math_log10(&Value::Float(0.0)).is_err());
    }

    #[test]
    fn test_math_exp() {
        match math_exp(&Value::Float(0.0)).unwrap() {
            Value::Float(f) => assert_float_eq(f, 1.0, 1e-10),
            _ => panic!("Expected Float"),
        }
        match math_exp(&Value::Float(1.0)).unwrap() {
            Value::Float(f) => assert_float_eq(f, std::f64::consts::E, 1e-10),
            _ => panic!("Expected Float"),
        }
    }

    #[test]
    fn test_math_floor() {
        assert_eq!(math_floor(&Value::Float(3.7)).unwrap(), Value::Float(3.0));
        assert_eq!(math_floor(&Value::Float(-3.7)).unwrap(), Value::Float(-4.0));
        assert_eq!(math_floor(&Value::Int(5)).unwrap(), Value::Float(5.0));
    }

    #[test]
    fn test_math_ceil() {
        assert_eq!(math_ceil(&Value::Float(3.2)).unwrap(), Value::Float(4.0));
        assert_eq!(math_ceil(&Value::Float(-3.2)).unwrap(), Value::Float(-3.0));
        assert_eq!(math_ceil(&Value::Int(5)).unwrap(), Value::Float(5.0));
    }

    #[test]
    fn test_math_round() {
        assert_eq!(math_round(&Value::Float(3.4)).unwrap(), Value::Float(3.0));
        assert_eq!(math_round(&Value::Float(3.5)).unwrap(), Value::Float(4.0));
        assert_eq!(math_round(&Value::Float(-3.5)).unwrap(), Value::Float(-4.0));
    }

    #[test]
    fn test_math_truncate() {
        assert_eq!(math_truncate(&Value::Float(3.7)).unwrap(), Value::Float(3.0));
        assert_eq!(math_truncate(&Value::Float(-3.7)).unwrap(), Value::Float(-3.0));
        assert_eq!(math_truncate(&Value::Int(5)).unwrap(), Value::Float(5.0));
    }

    #[test]
    fn test_type_errors() {
        let not_number = Value::Str("hello".to_string());
        assert!(math_abs(&not_number).is_err());
        assert!(math_sqrt(&not_number).is_err());
        assert!(math_sin(&not_number).is_err());
        assert!(math_log(&not_number).is_err());
        assert!(math_floor(&not_number).is_err());
    }
}
