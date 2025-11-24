//! Integration tests for type checking in the Fusabi compiler
//!
//! These tests validate that the compiler integration with type checking works correctly
//! and maintains backward compatibility with existing code.

use fusabi::{run_source, run_source_checked, run_source_with_options, RunOptions};
use fusabi_vm::Value;

// ============================================================================
// Type Checking Enabled Tests
// ============================================================================

#[test]
fn test_type_checking_simple_literal() {
    let source = "42";
    let result = run_source_checked(source);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Int(42));
}

#[test]
fn test_type_checking_boolean() {
    let source = "true";
    let result = run_source_checked(source);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Bool(true));
}

#[test]
fn test_type_checking_string() {
    let source = r#""hello""#;
    let result = run_source_checked(source);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Str("hello".to_string()));
}

#[test]
fn test_type_checking_arithmetic() {
    let source = "5 + 10";
    let result = run_source_checked(source);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Int(15));
}

#[test]
fn test_type_checking_comparison() {
    let source = "10 > 5";
    let result = run_source_checked(source);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Bool(true));
}

#[test]
fn test_type_checking_let_binding() {
    let source = "let x = 5 in x + 10";
    let result = run_source_checked(source);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Int(15));
}

#[test]
fn test_type_checking_nested_let() {
    let source = r#"
        let x = 10 in
        let y = 20 in
        let z = x + y in
        z * 2
    "#;
    let result = run_source_checked(source);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Int(60));
}

#[test]
fn test_type_checking_if_then_else() {
    let source = "if true then 42 else 0";
    let result = run_source_checked(source);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Int(42));
}

#[test]
fn test_type_checking_if_with_condition() {
    let source = "if 10 > 5 then 1 else 0";
    let result = run_source_checked(source);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Int(1));
}

#[test]
fn test_type_checking_complex_expression() {
    let source = r#"
        let a = 5 in
        let b = 10 in
        if a < b then a + b else a - b
    "#;
    let result = run_source_checked(source);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Int(15));
}

// ============================================================================
// Backward Compatibility Tests (Type Checking Disabled)
// ============================================================================

#[test]
fn test_backward_compat_simple() {
    let source = "42";
    let result = run_source(source);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Int(42));
}

#[test]
fn test_backward_compat_arithmetic() {
    let source = "5 + 10";
    let result = run_source(source);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Int(15));
}

#[test]
fn test_backward_compat_let_binding() {
    let source = "let x = 20 in x * 2";
    let result = run_source(source);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Int(40));
}

// ============================================================================
// RunOptions Tests
// ============================================================================

#[test]
fn test_run_options_default_values() {
    let options = RunOptions::default();
    assert!(!options.enable_type_checking);
    assert!(!options.verbose);
    assert!(!options.strict_mode);
}

#[test]
fn test_run_with_options_type_checking_disabled() {
    let options = RunOptions {
        enable_type_checking: false,
        verbose: false,
        strict_mode: false,
    };
    let result = run_source_with_options("5 + 10", options);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Int(15));
}

#[test]
fn test_run_with_options_type_checking_enabled() {
    let options = RunOptions {
        enable_type_checking: true,
        verbose: false,
        strict_mode: false,
    };
    let result = run_source_with_options("5 + 10", options);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Int(15));
}

#[test]
fn test_run_with_options_verbose() {
    let options = RunOptions {
        enable_type_checking: true,
        verbose: true, // This will print to stdout during test
        strict_mode: false,
    };
    let result = run_source_with_options("42", options);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Int(42));
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_error_undefined_variable_no_type_checking() {
    let result = run_source("undefined_var");
    assert!(result.is_err());
    // Should get compile error (undefined variable)
}

#[test]
fn test_error_undefined_variable_with_type_checking() {
    let result = run_source_checked("undefined_var");
    assert!(result.is_err());
    // Should get compile error (undefined variable)
}

#[test]
fn test_error_lex_error() {
    let result = run_source_checked("42 @ 10");
    assert!(result.is_err());
}

#[test]
fn test_error_parse_error() {
    let result = run_source_checked("let x =");
    assert!(result.is_err());
}

// ============================================================================
// Advanced Expression Tests
// ============================================================================

#[test]
fn test_multiple_bindings() {
    let source = r#"
        let x = 1 in
        let y = 2 in
        let z = 3 in
        x + y + z
    "#;
    let result = run_source_checked(source);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Int(6));
}

#[test]
fn test_nested_arithmetic() {
    let source = "(10 + 5) * (20 - 10)";
    let result = run_source_checked(source);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Int(150));
}

#[test]
fn test_boolean_operators() {
    let source = "true && false || true";
    let result = run_source_checked(source);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Bool(true));
}

#[test]
fn test_comparison_chain() {
    let source = "if 5 < 10 && 10 < 20 then 1 else 0";
    let result = run_source_checked(source);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Int(1));
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_empty_string() {
    let source = r#""""#;
    let result = run_source_checked(source);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Str("".to_string()));
}

#[test]
fn test_zero() {
    let source = "0";
    let result = run_source_checked(source);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Int(0));
}

#[test]
fn test_negative_number() {
    let source = "0 - 42";
    let result = run_source_checked(source);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Int(-42));
}

#[test]
fn test_shadowing() {
    let source = r#"
        let x = 10 in
        let x = 20 in
        x
    "#;
    let result = run_source_checked(source);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Int(20));
}

// ============================================================================
// Consistency Tests (Type Checking vs No Type Checking)
// ============================================================================

#[test]
fn test_consistency_simple() {
    let source = "42";
    let result_no_tc = run_source(source).unwrap();
    let result_with_tc = run_source_checked(source).unwrap();
    assert_eq!(result_no_tc, result_with_tc);
}

#[test]
fn test_consistency_arithmetic() {
    let source = "10 + 20 * 3";
    let result_no_tc = run_source(source).unwrap();
    let result_with_tc = run_source_checked(source).unwrap();
    assert_eq!(result_no_tc, result_with_tc);
}

#[test]
fn test_consistency_let_binding() {
    let source = "let x = 5 in let y = 10 in x + y";
    let result_no_tc = run_source(source).unwrap();
    let result_with_tc = run_source_checked(source).unwrap();
    assert_eq!(result_no_tc, result_with_tc);
}

#[test]
fn test_consistency_if_expression() {
    let source = "if true then 1 else 0";
    let result_no_tc = run_source(source).unwrap();
    let result_with_tc = run_source_checked(source).unwrap();
    assert_eq!(result_no_tc, result_with_tc);
}

#[test]
fn test_consistency_comparison() {
    let source = "10 > 5";
    let result_no_tc = run_source(source).unwrap();
    let result_with_tc = run_source_checked(source).unwrap();
    assert_eq!(result_no_tc, result_with_tc);
}

// ============================================================================
// Performance Baseline Tests
// ============================================================================

#[test]
fn test_large_expression() {
    let source = r#"
        let a = 1 in
        let b = 2 in
        let c = 3 in
        let d = 4 in
        let e = 5 in
        let f = a + b in
        let g = c + d in
        let h = e + f in
        g + h
    "#;
    let result = run_source_checked(source);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Int(15));
}
