//! Comprehensive tests for list literal syntax with comma separators (Issue #125)
//!
//! Tests cover:
//! - Comma-separated list syntax: [1, 2, 3]
//! - Backward compatibility with semicolon syntax: [1; 2; 3]
//! - Trailing commas: [1, 2, 3,]
//! - Empty lists: []
//! - Nested lists: [[1, 2], [3, 4]]
//! - Lists with different element types
//! - Mixed separators (should work with either)

use fusabi_frontend::ast::{Expr, Literal};
use fusabi_frontend::lexer::Lexer;
use fusabi_frontend::parser::Parser;

// Helper to parse source code
fn parse(input: &str) -> Result<Expr, String> {
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().map_err(|e| format!("{}", e))?;
    let mut parser = Parser::new(tokens);
    parser.parse().map_err(|e| format!("{}", e))
}

// Helper to verify list structure
fn assert_list_with_elements(expr: Expr, expected_count: usize) {
    match expr {
        Expr::List(elements) => {
            assert_eq!(
                elements.len(),
                expected_count,
                "Expected {} elements, got {}",
                expected_count,
                elements.len()
            );
        }
        _ => panic!("Expected List, got {:?}", expr),
    }
}

// ============================================================================
// Comma-separated syntax tests
// ============================================================================

#[test]
fn test_parse_empty_list() {
    let expr = parse("[]").unwrap();
    assert_list_with_elements(expr, 0);
}

#[test]
fn test_parse_single_element_comma_list() {
    let expr = parse("[1]").unwrap();
    assert_list_with_elements(expr, 1);
}

#[test]
fn test_parse_two_element_comma_list() {
    let expr = parse("[1, 2]").unwrap();
    assert_list_with_elements(expr, 2);
}

#[test]
fn test_parse_three_element_comma_list() {
    let expr = parse("[1, 2, 3]").unwrap();
    assert_list_with_elements(expr, 3);
}

#[test]
fn test_parse_multiple_element_comma_list() {
    let expr = parse("[1, 2, 3, 4, 5]").unwrap();
    assert_list_with_elements(expr, 5);
}

#[test]
fn test_parse_comma_list_with_trailing_comma() {
    let expr = parse("[1, 2, 3,]").unwrap();
    assert_list_with_elements(expr, 3);
}

#[test]
fn test_parse_single_element_with_trailing_comma() {
    let expr = parse("[42,]").unwrap();
    assert_list_with_elements(expr, 1);
}

// ============================================================================
// Semicolon syntax tests (backward compatibility)
// ============================================================================

#[test]
fn test_parse_semicolon_list() {
    let expr = parse("[1; 2; 3]").unwrap();
    assert_list_with_elements(expr, 3);
}

#[test]
fn test_parse_semicolon_list_with_trailing() {
    let expr = parse("[1; 2; 3;]").unwrap();
    assert_list_with_elements(expr, 3);
}

// ============================================================================
// Different element types
// ============================================================================

#[test]
fn test_parse_string_comma_list() {
    let expr = parse(r#"["hello", "world"]"#).unwrap();
    assert_list_with_elements(expr, 2);
}

#[test]
fn test_parse_float_comma_list() {
    let expr = parse("[1.5, 2.5, 3.5]").unwrap();
    assert_list_with_elements(expr, 3);
}

#[test]
fn test_parse_bool_comma_list() {
    let expr = parse("[true, false, true]").unwrap();
    assert_list_with_elements(expr, 3);
}

#[test]
fn test_parse_string_list_with_trailing_comma() {
    let expr = parse(r#"["a", "b", "c",]"#).unwrap();
    assert_list_with_elements(expr, 3);
}

// ============================================================================
// Lists with expressions
// ============================================================================

#[test]
fn test_parse_list_with_arithmetic_expressions() {
    let expr = parse("[1 + 2, 3 * 4, 5 - 1]").unwrap();
    match expr {
        Expr::List(elements) => {
            assert_eq!(elements.len(), 3);
            assert!(elements[0].is_binop());
            assert!(elements[1].is_binop());
            assert!(elements[2].is_binop());
        }
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_parse_list_with_variables() {
    let expr = parse("let x = 1 in let y = 2 in [x, y]").unwrap();
    match expr {
        Expr::Let { body, .. } => match *body {
            Expr::Let { body, .. } => match *body {
                Expr::List(elements) => {
                    assert_eq!(elements.len(), 2);
                }
                _ => panic!("Expected List in nested let"),
            },
            _ => panic!("Expected nested Let"),
        },
        _ => panic!("Expected Let expression"),
    }
}

#[test]
fn test_parse_list_with_function_calls() {
    let expr = parse("[f 1, g 2, h 3]").unwrap();
    match expr {
        Expr::List(elements) => {
            assert_eq!(elements.len(), 3);
            assert!(elements[0].is_app());
            assert!(elements[1].is_app());
            assert!(elements[2].is_app());
        }
        _ => panic!("Expected List"),
    }
}

// ============================================================================
// Nested lists
// ============================================================================

#[test]
fn test_parse_nested_empty_lists() {
    let expr = parse("[[], []]").unwrap();
    match expr {
        Expr::List(elements) => {
            assert_eq!(elements.len(), 2);
            assert!(matches!(elements[0], Expr::List(_)));
            assert!(matches!(elements[1], Expr::List(_)));
        }
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_parse_nested_lists_comma() {
    let expr = parse("[[1, 2], [3, 4]]").unwrap();
    match expr {
        Expr::List(outer) => {
            assert_eq!(outer.len(), 2);
            match &outer[0] {
                Expr::List(inner) => assert_eq!(inner.len(), 2),
                _ => panic!("Expected nested List"),
            }
            match &outer[1] {
                Expr::List(inner) => assert_eq!(inner.len(), 2),
                _ => panic!("Expected nested List"),
            }
        }
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_parse_nested_lists_mixed_separators() {
    // Outer uses comma, inner uses semicolon
    let expr = parse("[[1; 2], [3; 4]]").unwrap();
    match expr {
        Expr::List(outer) => {
            assert_eq!(outer.len(), 2);
            match &outer[0] {
                Expr::List(inner) => assert_eq!(inner.len(), 2),
                _ => panic!("Expected nested List"),
            }
        }
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_parse_deeply_nested_lists() {
    let expr = parse("[[[1, 2], [3, 4]], [[5, 6], [7, 8]]]").unwrap();
    match expr {
        Expr::List(outer) => {
            assert_eq!(outer.len(), 2);
        }
        _ => panic!("Expected List"),
    }
}

// ============================================================================
// Lists in different contexts
// ============================================================================

#[test]
fn test_parse_list_in_let_binding() {
    let expr = parse("let nums = [1, 2, 3] in nums").unwrap();
    match expr {
        Expr::Let { value, .. } => match *value {
            Expr::List(elements) => assert_eq!(elements.len(), 3),
            _ => panic!("Expected List in let value"),
        },
        _ => panic!("Expected Let"),
    }
}

#[test]
fn test_parse_list_in_if_condition() {
    let expr = parse("if [1, 2] = [1, 2] then true else false").unwrap();
    match expr {
        Expr::If { cond, .. } => match *cond {
            Expr::BinOp { left, right, .. } => {
                assert!(matches!(*left, Expr::List(_)));
                assert!(matches!(*right, Expr::List(_)));
            }
            _ => panic!("Expected BinOp in if condition"),
        },
        _ => panic!("Expected If"),
    }
}

#[test]
fn test_parse_list_as_function_argument() {
    let expr = parse("f [1, 2, 3]").unwrap();
    match expr {
        Expr::App { arg, .. } => match *arg {
            Expr::List(elements) => assert_eq!(elements.len(), 3),
            _ => panic!("Expected List as argument"),
        },
        _ => panic!("Expected App"),
    }
}

// ============================================================================
// Cons operator with lists
// ============================================================================

#[test]
fn test_parse_cons_with_comma_list() {
    let expr = parse("1 :: [2, 3]").unwrap();
    match expr {
        Expr::Cons { head, tail } => {
            assert!(matches!(*head, Expr::Lit(Literal::Int(1))));
            match *tail {
                Expr::List(elements) => assert_eq!(elements.len(), 2),
                _ => panic!("Expected List in tail"),
            }
        }
        _ => panic!("Expected Cons"),
    }
}

#[test]
fn test_parse_cons_with_empty_list() {
    let expr = parse("42 :: []").unwrap();
    match expr {
        Expr::Cons { tail, .. } => match *tail {
            Expr::List(elements) => assert_eq!(elements.len(), 0),
            _ => panic!("Expected empty List"),
        },
        _ => panic!("Expected Cons"),
    }
}

// ============================================================================
// Edge cases
// ============================================================================

#[test]
fn test_parse_list_with_whitespace() {
    let expr = parse("[  1  ,  2  ,  3  ]").unwrap();
    assert_list_with_elements(expr, 3);
}

#[test]
fn test_parse_list_with_newlines() {
    let expr = parse("[\n  1,\n  2,\n  3\n]").unwrap();
    assert_list_with_elements(expr, 3);
}

#[test]
fn test_parse_list_single_line() {
    let expr = parse("[1,2,3,4,5,6,7,8,9,10]").unwrap();
    assert_list_with_elements(expr, 10);
}

// ============================================================================
// Complex expressions in lists
// ============================================================================

#[test]
fn test_parse_list_with_tuples() {
    let expr = parse("[(1, 2), (3, 4)]").unwrap();
    match expr {
        Expr::List(elements) => {
            assert_eq!(elements.len(), 2);
            assert!(matches!(elements[0], Expr::Tuple(_)));
            assert!(matches!(elements[1], Expr::Tuple(_)));
        }
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_parse_list_with_lambdas() {
    let expr = parse("[fun x -> x, fun y -> y + 1]").unwrap();
    match expr {
        Expr::List(elements) => {
            assert_eq!(elements.len(), 2);
            assert!(elements[0].is_lambda());
            assert!(elements[1].is_lambda());
        }
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_parse_list_with_nested_expressions() {
    let expr = parse("[1 + (2 * 3), (4 - 5) * 6]").unwrap();
    match expr {
        Expr::List(elements) => {
            assert_eq!(elements.len(), 2);
            assert!(elements[0].is_binop());
            assert!(elements[1].is_binop());
        }
        _ => panic!("Expected List"),
    }
}

// ============================================================================
// Real-world usage examples
// ============================================================================

#[test]
fn test_bytecode_api_example() {
    // This is the exact syntax from the failing test
    let expr = parse("let list = [1, 2, 3] in list").unwrap();
    match expr {
        Expr::Let { value, body, .. } => {
            match *value {
                Expr::List(elements) => assert_eq!(elements.len(), 3),
                _ => panic!("Expected List"),
            }
            match *body {
                Expr::Var(name) => assert_eq!(name, "list"),
                _ => panic!("Expected Var"),
            }
        }
        _ => panic!("Expected Let"),
    }
}

#[test]
fn test_list_with_map_operation() {
    // Simulating List.map usage
    let expr = parse("let nums = [1, 2, 3] in List.map (fun x -> x * 2) nums").unwrap();
    match expr {
        Expr::Let { value, .. } => match *value {
            Expr::List(elements) => assert_eq!(elements.len(), 3),
            _ => panic!("Expected List"),
        },
        _ => panic!("Expected Let"),
    }
}

#[test]
fn test_multiple_lists() {
    let expr = parse("let a = [1, 2] in let b = [3, 4] in [a, b]").unwrap();
    // Just verify it parses without error
    assert!(expr.is_let());
}
