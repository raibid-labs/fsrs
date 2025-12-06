// Integration tests for fusabi host application
// These tests verify the end-to-end functionality of the Fusabi pipeline
// Only testing Phase 1 MVP features

use fusabi::run_source;
use fusabi_vm::Value;

#[cfg(test)]
mod pipeline_integration_tests {
    use super::*;

    #[test]
    fn test_simple_literal() {
        let source = "42";
        let result = run_source(source).expect("Failed to execute literal");
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_simple_arithmetic() {
        let source = "1 + 2 * 3";
        let result = run_source(source).expect("Failed to execute arithmetic");
        assert_eq!(result, Value::Int(7));
    }

    #[test]
    fn test_arithmetic_complex() {
        let source = "(10 - 3) * 2 + 5";
        let result = run_source(source).expect("Failed to execute complex arithmetic");
        assert_eq!(result, Value::Int(19));
    }

    #[test]
    fn test_boolean_literal() {
        let source = "true";
        let result = run_source(source).expect("Failed to execute boolean");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_string_literal() {
        let source = r#""hello""#;
        let result = run_source(source).expect("Failed to execute string");
        assert_eq!(result, Value::Str("hello".to_string()));
    }

    #[test]
    fn test_string_concatenation() {
        let source = r#""hello" ++ "world""#;
        let result = run_source(source).expect("Failed to execute string concatenation");
        assert_eq!(result, Value::Str("helloworld".to_string()));
    }

    #[test]
    fn test_string_concat_with_spaces() {
        let source = r#""hello" ++ " " ++ "world""#;
        let result = run_source(source).expect("Failed to execute string concat chain");
        assert_eq!(result, Value::Str("hello world".to_string()));
    }

    #[test]
    fn test_string_concat_example() {
        let source =
            r#"let host = "localhost" in let port = "8080" in "http://" ++ host ++ ":" ++ port"#;
        let result = run_source(source).expect("Failed to execute URL building");
        assert_eq!(result, Value::Str("http://localhost:8080".to_string()));
    }

    #[test]
    fn test_unit_literal() {
        let source = "()";
        let result = run_source(source).expect("Failed to execute unit");
        assert_eq!(result, Value::Unit);
    }
}

#[cfg(test)]
mod let_binding_tests {
    use super::*;

    #[test]
    fn test_simple_let_binding() {
        let source = "let x = 42 in x";
        let result = run_source(source).expect("Failed to execute let binding");
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_let_with_arithmetic() {
        let source = "let x = 10 in x + 5";
        let result = run_source(source).expect("Failed to execute let with arithmetic");
        assert_eq!(result, Value::Int(15));
    }

    #[test]
    fn test_nested_let_bindings() {
        let source = "let x = 5 in let y = 10 in x + y";
        let result = run_source(source).expect("Failed to execute nested let");
        assert_eq!(result, Value::Int(15));
    }

    #[test]
    fn test_let_shadowing() {
        let source = "let x = 10 in let x = 20 in x";
        let result = run_source(source).expect("Failed to execute let shadowing");
        assert_eq!(result, Value::Int(20));
    }

    #[test]
    fn test_let_with_computation() {
        let source = "let x = 2 + 3 in x * 4";
        let result = run_source(source).expect("Failed to execute let with computation");
        assert_eq!(result, Value::Int(20));
    }
}

#[cfg(test)]
mod conditional_tests {
    use super::*;

    #[test]
    fn test_if_true_branch() {
        let source = "if true then 42 else 0";
        let result = run_source(source).expect("Failed to execute if-true");
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_if_false_branch() {
        let source = "if false then 42 else 0";
        let result = run_source(source).expect("Failed to execute if-false");
        assert_eq!(result, Value::Int(0));
    }

    #[test]
    fn test_if_with_comparison() {
        let source = "if 10 > 5 then 1 else 2";
        let result = run_source(source).expect("Failed to execute if with comparison");
        assert_eq!(result, Value::Int(1));
    }

    #[test]
    fn test_if_with_equality() {
        let source = "if 5 = 5 then 100 else 200";
        let result = run_source(source).expect("Failed to execute if with equality");
        assert_eq!(result, Value::Int(100));
    }

    #[test]
    fn test_nested_if() {
        let source = "if true then (if false then 1 else 2) else 3";
        let result = run_source(source).expect("Failed to execute nested if");
        assert_eq!(result, Value::Int(2));
    }

    #[test]
    fn test_if_with_let_binding() {
        let source = "let x = 10 in if x > 5 then x else 0";
        let result = run_source(source).expect("Failed to execute if with let");
        assert_eq!(result, Value::Int(10));
    }
}

#[cfg(test)]
mod comparison_tests {
    use super::*;

    #[test]
    fn test_less_than_true() {
        let source = "5 < 10";
        let result = run_source(source).expect("Failed to execute <");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_less_than_false() {
        let source = "10 < 5";
        let result = run_source(source).expect("Failed to execute <");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_greater_than_true() {
        let source = "10 > 5";
        let result = run_source(source).expect("Failed to execute >");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_less_than_or_equal() {
        let source = "5 <= 5";
        let result = run_source(source).expect("Failed to execute <=");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_greater_than_or_equal() {
        let source = "10 >= 10";
        let result = run_source(source).expect("Failed to execute >=");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_equality() {
        let source = "42 = 42";
        let result = run_source(source).expect("Failed to execute =");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_inequality() {
        let source = "42 <> 0";
        let result = run_source(source).expect("Failed to execute <>");
        assert_eq!(result, Value::Bool(true));
    }
}

#[cfg(test)]
mod logical_operations_tests {
    use super::*;

    #[test]
    fn test_and_true() {
        let source = "true && true";
        let result = run_source(source).expect("Failed to execute &&");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_and_false() {
        let source = "true && false";
        let result = run_source(source).expect("Failed to execute &&");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_or_true() {
        let source = "false || true";
        let result = run_source(source).expect("Failed to execute ||");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_or_false() {
        let source = "false || false";
        let result = run_source(source).expect("Failed to execute ||");
        assert_eq!(result, Value::Bool(false));
    }
}

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[test]
    fn test_undefined_variable() {
        let source = "x + 1";
        let result = run_source(source);
        assert!(result.is_err(), "Should fail on undefined variable");
    }

    #[test]
    fn test_division_by_zero() {
        let source = "10 / 0";
        let result = run_source(source);
        assert!(result.is_err(), "Should fail on division by zero");
    }

    #[test]
    fn test_type_mismatch_add() {
        // This will fail during VM execution (type mismatch)
        let source = "true + 5";
        let result = run_source(source);
        assert!(result.is_err(), "Should fail on type mismatch");
    }

    #[test]
    fn test_lexer_error() {
        // Invalid token
        let source = "42 @ 10";
        let result = run_source(source);
        assert!(result.is_err(), "Should fail on invalid token");
    }

    #[test]
    fn test_parser_error() {
        // Unclosed parenthesis
        let source = "(42 + 10";
        let result = run_source(source);
        assert!(result.is_err(), "Should fail on parse error");
    }
}

#[cfg(test)]
mod complex_integration_tests {
    use super::*;

    #[test]
    fn test_fibonacci_like() {
        let source = "let a = 0 in let b = 1 in let c = a + b in let d = b + c in d";
        let result = run_source(source).expect("Failed to execute fibonacci-like");
        assert_eq!(result, Value::Int(2));
    }

    #[test]
    fn test_factorial_partial() {
        let source = "let n = 5 in let f1 = n in let f2 = f1 * (n - 1) in f2";
        let result = run_source(source).expect("Failed to execute factorial partial");
        assert_eq!(result, Value::Int(20));
    }

    #[test]
    fn test_max_function() {
        let source = "let x = 10 in let y = 20 in if x > y then x else y";
        let result = run_source(source).expect("Failed to execute max");
        assert_eq!(result, Value::Int(20));
    }

    #[test]
    fn test_absolute_value() {
        // Use 0 - x instead of -x since unary minus might not be implemented
        let source = "let x = 0 - 5 in if x < 0 then 0 - x else x";
        let result = run_source(source).expect("Failed to execute abs");
        assert_eq!(result, Value::Int(5));
    }

    #[test]
    fn test_boolean_logic_complex() {
        let source = "let a = true in let b = false in if a && (b || a) then 1 else 0";
        let result = run_source(source).expect("Failed to execute boolean logic");
        assert_eq!(result, Value::Int(1));
    }
}
