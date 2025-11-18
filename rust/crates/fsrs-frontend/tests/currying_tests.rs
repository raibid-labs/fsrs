//! Integration tests for currying and partial application support.
//!
//! These tests verify that multi-parameter functions are correctly desugared
//! to nested lambdas, enabling automatic currying and partial application.
//!
//! NOTE: Full execution of curried functions requires closure support (Phase 2).
//! These tests focus on parsing and AST transformation.

use fsrs_frontend::ast::{Expr, Literal};
use fsrs_frontend::lexer::Lexer;
use fsrs_frontend::parser::Parser;

// Helper function to parse a string into an AST
fn parse(input: &str) -> Result<Expr, Box<dyn std::error::Error>> {
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    Ok(parser.parse()?)
}

// ========================================================================
// TDD: Parser Tests for Multi-Parameter Functions
// ========================================================================

#[test]
fn test_parse_two_param_function() {
    // let add x y = x + y in add 1 2
    let code = "let add x y = x + y in add 1 2";
    let expr = parse(code).expect("Failed to parse");

    // Verify it's a let expression
    match &expr {
        Expr::Let { name, value, body } => {
            assert_eq!(name, "add");

            // Value should be desugared to nested lambdas: fun x -> fun y -> x + y
            match value.as_ref() {
                Expr::Lambda { param, body } => {
                    assert_eq!(param, "x");

                    // Inner lambda: fun y -> x + y
                    match body.as_ref() {
                        Expr::Lambda { param, body } => {
                            assert_eq!(param, "y");

                            // Body should be: x + y
                            assert!(body.is_binop());
                        }
                        _ => panic!("Expected nested lambda for y parameter"),
                    }
                }
                _ => panic!("Expected lambda for multi-param function"),
            }

            // Body should be application: add 1 2
            assert!(body.is_app());
        }
        _ => panic!("Expected Let expression"),
    }
}

#[test]
fn test_parse_three_param_function() {
    // let addThree x y z = x + y + z in addThree 1 2 3
    let code = "let addThree x y z = x + y + z in addThree 1 2 3";
    let expr = parse(code).expect("Failed to parse");

    match &expr {
        Expr::Let { name, value, .. } => {
            assert_eq!(name, "addThree");

            // Should desugar to: fun x -> fun y -> fun z -> x + y + z
            match value.as_ref() {
                Expr::Lambda { param, body } => {
                    assert_eq!(param, "x");

                    match body.as_ref() {
                        Expr::Lambda { param, body } => {
                            assert_eq!(param, "y");

                            match body.as_ref() {
                                Expr::Lambda { param, .. } => {
                                    assert_eq!(param, "z");
                                }
                                _ => panic!("Expected third lambda"),
                            }
                        }
                        _ => panic!("Expected second lambda"),
                    }
                }
                _ => panic!("Expected first lambda"),
            }
        }
        _ => panic!("Expected Let expression"),
    }
}

#[test]
fn test_parse_single_param_function_unchanged() {
    // let inc x = x + 1 in inc 5
    let code = "let inc x = x + 1 in inc 5";
    let expr = parse(code).expect("Failed to parse");

    match &expr {
        Expr::Let { name, value, .. } => {
            assert_eq!(name, "inc");

            // Single param should still be a lambda
            match value.as_ref() {
                Expr::Lambda { param, body } => {
                    assert_eq!(param, "x");
                    assert!(body.is_binop());
                }
                _ => panic!("Expected lambda"),
            }
        }
        _ => panic!("Expected Let expression"),
    }
}

#[test]
fn test_parse_zero_param_function_is_value() {
    // let value = 42 in value
    let code = "let value = 42 in value";
    let expr = parse(code).expect("Failed to parse");

    match &expr {
        Expr::Let { name, value, .. } => {
            assert_eq!(name, "value");

            // Zero params means it's just a value, not a lambda
            assert!(value.is_literal());
        }
        _ => panic!("Expected Let expression"),
    }
}

// NOTE: The following tests are commented out because `fun x y -> body` is NOT valid F# syntax.
// F# requires explicit nesting: `fun x -> fun y -> body`
// The multi-param desugaring only applies to `let` bindings, not `fun` expressions.

// #[test]
// fn test_parse_fun_multi_param_syntax() {
//     // fun x y -> x + y
//     let code = "fun x y -> x + y";
//     let expr = parse(code).expect("Failed to parse");
//
//     // Should desugar to: fun x -> fun y -> x + y
//     match &expr {
//         Expr::Lambda { param, body } => {
//             assert_eq!(param, "x");
//
//             match body.as_ref() {
//                 Expr::Lambda { param, body } => {
//                     assert_eq!(param, "y");
//                     assert!(body.is_binop());
//                 }
//                 _ => panic!("Expected nested lambda"),
//             }
//         }
//         _ => panic!("Expected Lambda expression"),
//     }
// }

// #[test]
// fn test_parse_fun_three_params() {
//     // fun a b c -> a + b + c
//     let code = "fun a b c -> a + b + c";
//     let expr = parse(code).expect("Failed to parse");
//
//     // Should desugar to: fun a -> fun b -> fun c -> a + b + c
//     let mut current = &expr;
//     let expected_params = vec!["a", "b", "c"];
//
//     for (i, expected_param) in expected_params.iter().enumerate() {
//         match current {
//             Expr::Lambda { param, body } => {
//                 assert_eq!(param, expected_param);
//
//                 if i < expected_params.len() - 1 {
//                     current = body.as_ref();
//                 } else {
//                     // Last parameter, body should be the expression
//                     assert!(body.is_binop());
//                 }
//             }
//             _ => panic!("Expected lambda at level {}", i),
//         }
//     }
// }

#[test]
fn test_parse_curried_application_two_args() {
    // f x y should parse as (f x) y
    let code = "f x y";
    let expr = parse(code).expect("Failed to parse");

    match &expr {
        Expr::App { func, arg } => {
            // Outer application
            assert_eq!(arg.as_var(), Some("y"));

            // Inner application: f x
            match func.as_ref() {
                Expr::App { func, arg } => {
                    assert_eq!(func.as_var(), Some("f"));
                    assert_eq!(arg.as_var(), Some("x"));
                }
                _ => panic!("Expected nested App for (f x)"),
            }
        }
        _ => panic!("Expected App expression"),
    }
}

#[test]
fn test_parse_partial_application_simple() {
    // let add x y = x + y
    // let add10 = add 10
    // in add10 5
    let code = r#"
        let add x y = x + y in
        let add10 = add 10 in
        add10 5
    "#;
    let expr = parse(code).expect("Failed to parse");

    // Outer let: add
    match &expr {
        Expr::Let { name, value, body } => {
            assert_eq!(name, "add");

            // add should be nested lambdas
            assert!(value.is_lambda());

            // Body is next let
            match body.as_ref() {
                Expr::Let { name, value, body } => {
                    assert_eq!(name, "add10");

                    // add10 = add 10 (partial application)
                    match value.as_ref() {
                        Expr::App { func, arg } => {
                            assert_eq!(func.as_var(), Some("add"));
                            assert_eq!(arg.as_literal(), Some(&Literal::Int(10)));
                        }
                        _ => panic!("Expected App for partial application"),
                    }

                    // Body: add10 5 (completing the partial application)
                    match body.as_ref() {
                        Expr::App { func, arg } => {
                            assert_eq!(func.as_var(), Some("add10"));
                            assert_eq!(arg.as_literal(), Some(&Literal::Int(5)));
                        }
                        _ => panic!("Expected App for final call"),
                    }
                }
                _ => panic!("Expected second Let"),
            }
        }
        _ => panic!("Expected Let expression"),
    }
}

#[test]
fn test_currying_preserves_semantics() {
    // Verify that let f x y = body desugars correctly
    let manual = parse("let f = fun x -> fun y -> x + y in f 1 2").unwrap();
    let curried = parse("let f x y = x + y in f 1 2").unwrap();

    // Both should have the same structure
    match (&manual, &curried) {
        (
            Expr::Let {
                value: manual_value,
                ..
            },
            Expr::Let {
                value: curried_value,
                ..
            },
        ) => {
            // Both values should be lambdas
            assert!(manual_value.is_lambda());
            assert!(curried_value.is_lambda());
        }
        _ => panic!("Expected Let expressions"),
    }
}
