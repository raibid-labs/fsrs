//! Integration tests for Option type syntax support
//!
//! Tests to ensure that Option constructors (Some and None) work naturally
//! without requiring parentheses for single arguments.

use fusabi_frontend::ast::{Expr, Literal};
use fusabi_frontend::compiler::Compiler;
use fusabi_frontend::lexer::Lexer;
use fusabi_frontend::parser::Parser;

// Helper function to parse a string
fn parse_expr(input: &str) -> Result<Expr, String> {
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().map_err(|e| format!("Lex error: {}", e))?;
    let mut parser = Parser::new(tokens);
    parser.parse().map_err(|e| format!("Parse error: {}", e))
}

#[test]
fn test_none_no_parens() {
    // None should work without parentheses
    let expr = parse_expr("None").unwrap();
    assert!(expr.is_variant_construct());

    if let Expr::VariantConstruct {
        variant, fields, ..
    } = expr
    {
        assert_eq!(variant, "None");
        assert_eq!(fields.len(), 0);
    } else {
        panic!("Expected VariantConstruct");
    }
}

#[test]
fn test_some_with_parens() {
    // Some(42) should work (current implementation)
    let expr = parse_expr("Some(42)").unwrap();
    assert!(expr.is_variant_construct());

    if let Expr::VariantConstruct {
        variant, fields, ..
    } = expr
    {
        assert_eq!(variant, "Some");
        assert_eq!(fields.len(), 1);
    } else {
        panic!("Expected VariantConstruct");
    }
}

#[test]
fn test_some_without_parens() {
    // Some 42 should work (F# style)
    let expr = parse_expr("Some 42").unwrap();
    assert!(
        expr.is_variant_construct(),
        "Some 42 should be parsed as variant, got: {:?}",
        expr
    );

    if let Expr::VariantConstruct {
        variant, fields, ..
    } = expr
    {
        assert_eq!(variant, "Some");
        assert_eq!(fields.len(), 1);
        // Check the field is Int(42)
        if let Expr::Lit(Literal::Int(n)) = *fields[0].as_ref() {
            assert_eq!(n, 42);
        } else {
            panic!("Expected Int(42) in field");
        }
    } else {
        panic!("Expected VariantConstruct, got: {:?}", expr);
    }
}

#[test]
fn test_some_with_variable() {
    // Some x should work
    let expr = parse_expr("let x = 42 in Some x").unwrap();

    if let Expr::Let { body, .. } = expr {
        assert!(body.is_variant_construct());
        if let Expr::VariantConstruct {
            variant, fields, ..
        } = *body
        {
            assert_eq!(variant, "Some");
            assert_eq!(fields.len(), 1);
            assert!(fields[0].is_var());
        }
    } else {
        panic!("Expected Let expression");
    }
}

#[test]
fn test_some_with_expression() {
    // Some (x + 1) should work
    let expr = parse_expr("let x = 41 in Some (x + 1)").unwrap();

    if let Expr::Let { body, .. } = expr {
        assert!(body.is_variant_construct());
    } else {
        panic!("Expected Let expression");
    }
}

#[test]
fn test_nested_option() {
    // Some (Some 42) should work
    let expr = parse_expr("Some (Some 42)").unwrap();
    assert!(expr.is_variant_construct());

    if let Expr::VariantConstruct {
        variant, fields, ..
    } = expr
    {
        assert_eq!(variant, "Some");
        assert_eq!(fields.len(), 1);
        // Inner should also be a variant construct
        assert!(fields[0].is_variant_construct());
    }
}

#[test]
fn test_option_in_let_binding() {
    // let opt = Some 42 in opt
    let expr = parse_expr("let opt = Some 42 in opt").unwrap();

    if let Expr::Let { value, .. } = expr {
        assert!(value.is_variant_construct());
    } else {
        panic!("Expected Let expression");
    }
}

#[test]
fn test_option_with_string() {
    // Some "hello" should work
    let expr = parse_expr("Some \"hello\"").unwrap();
    assert!(expr.is_variant_construct());

    if let Expr::VariantConstruct {
        variant, fields, ..
    } = expr
    {
        assert_eq!(variant, "Some");
        assert_eq!(fields.len(), 1);
        match fields[0].as_ref() {
            Expr::Lit(Literal::Str(s)) => {
                assert_eq!(s, "hello");
            }
            _ => panic!("Expected string literal"),
        }
    }
}

#[test]
fn test_multiple_variants_in_list() {
    // [Some 1; None; Some 2] should work
    let expr = parse_expr("[Some 1; None; Some 2]").unwrap();

    if let Expr::List(elements) = expr {
        assert_eq!(elements.len(), 3);
        assert!(elements[0].is_variant_construct());
        assert!(elements[1].is_variant_construct());
        assert!(elements[2].is_variant_construct());
    } else {
        panic!("Expected List");
    }
}

#[test]
fn test_compile_some_without_parens() {
    // Ensure Some 42 compiles correctly
    let expr = parse_expr("Some 42").unwrap();
    let chunk = Compiler::compile(&expr).unwrap();

    // Should have MakeVariant instruction
    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, fusabi_vm::instruction::Instruction::MakeVariant(1))));
}
