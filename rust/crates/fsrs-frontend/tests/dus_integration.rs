//! Integration tests for Discriminated Unions Layer 4: Compiler Integration
//!
//! This test suite ensures that the compiler correctly emits bytecode for
//! discriminated union construction and pattern matching.

use fsrs_frontend::ast::{Expr, Literal, MatchArm, Pattern};
use fsrs_frontend::compiler::Compiler;
use fsrs_vm::instruction::Instruction;

// ========================================================================
// Simple Enum Tests
// ========================================================================

#[test]
fn test_compile_simple_variant_no_fields() {
    // None
    let expr = Expr::VariantConstruct {
        type_name: "Option".to_string(),
        variant: "None".to_string(),
        fields: vec![],
    };

    let chunk = Compiler::compile(&expr).unwrap();

    // Should have: LoadConst("Option"), LoadConst("None"), MakeVariant(0), Return
    assert!(chunk.constants.iter().any(|v| v.as_str() == Some("Option")));
    assert!(chunk.constants.iter().any(|v| v.as_str() == Some("None")));
    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::MakeVariant(0))));
    assert_eq!(
        chunk.instructions[chunk.instructions.len() - 1],
        Instruction::Return
    );
}

#[test]
fn test_compile_variant_with_single_field() {
    // Some(42)
    let expr = Expr::VariantConstruct {
        type_name: "Option".to_string(),
        variant: "Some".to_string(),
        fields: vec![Box::new(Expr::Lit(Literal::Int(42)))],
    };

    let chunk = Compiler::compile(&expr).unwrap();

    // Should compile: type_name, variant_name, field, MakeVariant(1)
    assert!(chunk.constants.iter().any(|v| v.as_str() == Some("Option")));
    assert!(chunk.constants.iter().any(|v| v.as_str() == Some("Some")));
    assert!(chunk.constants.iter().any(|v| v.as_int() == Some(42)));
    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::MakeVariant(1))));
}

#[test]
fn test_compile_variant_with_multiple_fields() {
    // Rectangle(10, 20)
    let expr = Expr::VariantConstruct {
        type_name: "Shape".to_string(),
        variant: "Rectangle".to_string(),
        fields: vec![
            Box::new(Expr::Lit(Literal::Int(10))),
            Box::new(Expr::Lit(Literal::Int(20))),
        ],
    };

    let chunk = Compiler::compile(&expr).unwrap();

    assert!(chunk.constants.iter().any(|v| v.as_str() == Some("Shape")));
    assert!(chunk
        .constants
        .iter()
        .any(|v| v.as_str() == Some("Rectangle")));
    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::MakeVariant(2))));
}

// ========================================================================
// Pattern Matching Tests
// ========================================================================

#[test]
fn test_compile_match_simple_variant() {
    // match x with | None -> 0 | Some(y) -> y
    let expr = Expr::Match {
        scrutinee: Box::new(Expr::Var("x".to_string())),
        arms: vec![
            MatchArm {
                pattern: Pattern::Variant {
                    variant: "None".to_string(),
                    patterns: vec![],
                },
                body: Box::new(Expr::Lit(Literal::Int(0))),
            },
            MatchArm {
                pattern: Pattern::Variant {
                    variant: "Some".to_string(),
                    patterns: vec![Pattern::Var("y".to_string())],
                },
                body: Box::new(Expr::Var("y".to_string())),
            },
        ],
    };

    // This test requires x to be defined
    let expr_with_let = Expr::Let {
        name: "x".to_string(),
        value: Box::new(Expr::VariantConstruct {
            type_name: "Option".to_string(),
            variant: "Some".to_string(),
            fields: vec![Box::new(Expr::Lit(Literal::Int(42)))],
        }),
        body: Box::new(expr),
    };

    let chunk = Compiler::compile(&expr_with_let).unwrap();

    // Check for CheckVariantTag instructions
    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::CheckVariantTag(s) if s == "None")));
    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::CheckVariantTag(s) if s == "Some")));
}

#[test]
fn test_compile_match_variant_with_field_binding() {
    // match Some(42) with | Some(x) -> x | None -> 0
    let expr = Expr::Match {
        scrutinee: Box::new(Expr::VariantConstruct {
            type_name: "Option".to_string(),
            variant: "Some".to_string(),
            fields: vec![Box::new(Expr::Lit(Literal::Int(42)))],
        }),
        arms: vec![
            MatchArm {
                pattern: Pattern::Variant {
                    variant: "Some".to_string(),
                    patterns: vec![Pattern::Var("x".to_string())],
                },
                body: Box::new(Expr::Var("x".to_string())),
            },
            MatchArm {
                pattern: Pattern::Variant {
                    variant: "None".to_string(),
                    patterns: vec![],
                },
                body: Box::new(Expr::Lit(Literal::Int(0))),
            },
        ],
    };

    let chunk = Compiler::compile(&expr).unwrap();

    // Check for GetVariantField instruction
    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::GetVariantField(0))));
}

#[test]
fn test_compile_match_variant_multiple_fields() {
    // match Rectangle(10, 20) with | Rectangle(w, h) -> w + h | _ -> 0
    let expr = Expr::Match {
        scrutinee: Box::new(Expr::VariantConstruct {
            type_name: "Shape".to_string(),
            variant: "Rectangle".to_string(),
            fields: vec![
                Box::new(Expr::Lit(Literal::Int(10))),
                Box::new(Expr::Lit(Literal::Int(20))),
            ],
        }),
        arms: vec![
            MatchArm {
                pattern: Pattern::Variant {
                    variant: "Rectangle".to_string(),
                    patterns: vec![Pattern::Var("w".to_string()), Pattern::Var("h".to_string())],
                },
                body: Box::new(Expr::BinOp {
                    op: fsrs_frontend::ast::BinOp::Add,
                    left: Box::new(Expr::Var("w".to_string())),
                    right: Box::new(Expr::Var("h".to_string())),
                }),
            },
            MatchArm {
                pattern: Pattern::Wildcard,
                body: Box::new(Expr::Lit(Literal::Int(0))),
            },
        ],
    };

    let chunk = Compiler::compile(&expr).unwrap();

    // Check for GetVariantField instructions for both fields
    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::GetVariantField(0))));
    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::GetVariantField(1))));
}

// ========================================================================
// Nested Variant Tests
// ========================================================================

#[test]
fn test_compile_nested_variants() {
    // Ok(Some(42))
    let expr = Expr::VariantConstruct {
        type_name: "Result".to_string(),
        variant: "Ok".to_string(),
        fields: vec![Box::new(Expr::VariantConstruct {
            type_name: "Option".to_string(),
            variant: "Some".to_string(),
            fields: vec![Box::new(Expr::Lit(Literal::Int(42)))],
        })],
    };

    let chunk = Compiler::compile(&expr).unwrap();

    // Should have two MakeVariant instructions
    let make_variant_count = chunk
        .instructions
        .iter()
        .filter(|i| matches!(i, Instruction::MakeVariant(_)))
        .count();
    assert_eq!(make_variant_count, 2);
}

#[test]
fn test_compile_match_nested_variant_patterns() {
    // match Ok(Some(42)) with | Ok(Some(x)) -> x | _ -> 0
    let expr = Expr::Match {
        scrutinee: Box::new(Expr::VariantConstruct {
            type_name: "Result".to_string(),
            variant: "Ok".to_string(),
            fields: vec![Box::new(Expr::VariantConstruct {
                type_name: "Option".to_string(),
                variant: "Some".to_string(),
                fields: vec![Box::new(Expr::Lit(Literal::Int(42)))],
            })],
        }),
        arms: vec![
            MatchArm {
                pattern: Pattern::Variant {
                    variant: "Ok".to_string(),
                    patterns: vec![Pattern::Variant {
                        variant: "Some".to_string(),
                        patterns: vec![Pattern::Var("x".to_string())],
                    }],
                },
                body: Box::new(Expr::Var("x".to_string())),
            },
            MatchArm {
                pattern: Pattern::Wildcard,
                body: Box::new(Expr::Lit(Literal::Int(0))),
            },
        ],
    };

    let chunk = Compiler::compile(&expr).unwrap();

    // Check for CheckVariantTag for Ok (outer pattern)
    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::CheckVariantTag(s) if s == "Ok")));
    // GetVariantField should be present to extract the inner variant
    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::GetVariantField(0))));

    // Note: Nested variant tag checking ("Some") happens during pattern bindings
    // The current implementation handles this through recursive compile_pattern_bindings
}

// ========================================================================
// Direction Enum Tests (Classic Example)
// ========================================================================

#[test]
fn test_compile_direction_enum() {
    // match Left with | Left -> 1 | Right -> 2 | Up -> 3 | Down -> 4
    let expr = Expr::Match {
        scrutinee: Box::new(Expr::VariantConstruct {
            type_name: "Direction".to_string(),
            variant: "Left".to_string(),
            fields: vec![],
        }),
        arms: vec![
            MatchArm {
                pattern: Pattern::Variant {
                    variant: "Left".to_string(),
                    patterns: vec![],
                },
                body: Box::new(Expr::Lit(Literal::Int(1))),
            },
            MatchArm {
                pattern: Pattern::Variant {
                    variant: "Right".to_string(),
                    patterns: vec![],
                },
                body: Box::new(Expr::Lit(Literal::Int(2))),
            },
            MatchArm {
                pattern: Pattern::Variant {
                    variant: "Up".to_string(),
                    patterns: vec![],
                },
                body: Box::new(Expr::Lit(Literal::Int(3))),
            },
            MatchArm {
                pattern: Pattern::Variant {
                    variant: "Down".to_string(),
                    patterns: vec![],
                },
                body: Box::new(Expr::Lit(Literal::Int(4))),
            },
        ],
    };

    let chunk = Compiler::compile(&expr).unwrap();

    // All variants should be checked
    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::CheckVariantTag(s) if s == "Left")));
    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::CheckVariantTag(s) if s == "Right")));
    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::CheckVariantTag(s) if s == "Up")));
}

// ========================================================================
// Variant in Let Bindings
// ========================================================================

#[test]
fn test_compile_variant_in_let() {
    // let opt = Some(42) in opt
    let expr = Expr::Let {
        name: "opt".to_string(),
        value: Box::new(Expr::VariantConstruct {
            type_name: "Option".to_string(),
            variant: "Some".to_string(),
            fields: vec![Box::new(Expr::Lit(Literal::Int(42)))],
        }),
        body: Box::new(Expr::Var("opt".to_string())),
    };

    let chunk = Compiler::compile(&expr).unwrap();

    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::MakeVariant(1))));
    assert!(chunk.instructions.contains(&Instruction::StoreLocal(0)));
    assert!(chunk.instructions.contains(&Instruction::LoadLocal(0)));
}

// ========================================================================
// Variant with Different Field Types
// ========================================================================

#[test]
fn test_compile_variant_mixed_field_types() {
    // Person("Alice", 30, true)
    let expr = Expr::VariantConstruct {
        type_name: "Entity".to_string(),
        variant: "Person".to_string(),
        fields: vec![
            Box::new(Expr::Lit(Literal::Str("Alice".to_string()))),
            Box::new(Expr::Lit(Literal::Int(30))),
            Box::new(Expr::Lit(Literal::Bool(true))),
        ],
    };

    let chunk = Compiler::compile(&expr).unwrap();

    assert!(chunk.constants.iter().any(|v| v.as_str() == Some("Alice")));
    assert!(chunk.constants.iter().any(|v| v.as_int() == Some(30)));
    assert!(chunk.constants.iter().any(|v| v.as_bool() == Some(true)));
    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::MakeVariant(3))));
}

// ========================================================================
// Variant in Collections
// ========================================================================

#[test]
fn test_compile_list_of_variants() {
    // [Some(1); None; Some(2)]
    let expr = Expr::List(vec![
        Expr::VariantConstruct {
            type_name: "Option".to_string(),
            variant: "Some".to_string(),
            fields: vec![Box::new(Expr::Lit(Literal::Int(1)))],
        },
        Expr::VariantConstruct {
            type_name: "Option".to_string(),
            variant: "None".to_string(),
            fields: vec![],
        },
        Expr::VariantConstruct {
            type_name: "Option".to_string(),
            variant: "Some".to_string(),
            fields: vec![Box::new(Expr::Lit(Literal::Int(2)))],
        },
    ]);

    let chunk = Compiler::compile(&expr).unwrap();

    let make_variant_count = chunk
        .instructions
        .iter()
        .filter(|i| matches!(i, Instruction::MakeVariant(_)))
        .count();
    assert_eq!(make_variant_count, 3);
    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::MakeList(3))));
}

#[test]
fn test_compile_tuple_with_variants() {
    // (Some(1), None)
    let expr = Expr::Tuple(vec![
        Expr::VariantConstruct {
            type_name: "Option".to_string(),
            variant: "Some".to_string(),
            fields: vec![Box::new(Expr::Lit(Literal::Int(1)))],
        },
        Expr::VariantConstruct {
            type_name: "Option".to_string(),
            variant: "None".to_string(),
            fields: vec![],
        },
    ]);

    let chunk = Compiler::compile(&expr).unwrap();

    let make_variant_count = chunk
        .instructions
        .iter()
        .filter(|i| matches!(i, Instruction::MakeVariant(_)))
        .count();
    assert_eq!(make_variant_count, 2);
    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::MakeTuple(2))));
}

// ========================================================================
// Complex Expression Tests
// ========================================================================

#[test]
fn test_compile_variant_field_with_expression() {
    // Some(1 + 2)
    let expr = Expr::VariantConstruct {
        type_name: "Option".to_string(),
        variant: "Some".to_string(),
        fields: vec![Box::new(Expr::BinOp {
            op: fsrs_frontend::ast::BinOp::Add,
            left: Box::new(Expr::Lit(Literal::Int(1))),
            right: Box::new(Expr::Lit(Literal::Int(2))),
        })],
    };

    let chunk = Compiler::compile(&expr).unwrap();

    assert!(chunk.instructions.contains(&Instruction::Add));
    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::MakeVariant(1))));
}

#[test]
fn test_compile_match_with_wildcard_and_variant() {
    // match x with | Some(42) -> 1 | _ -> 0
    let expr = Expr::Let {
        name: "x".to_string(),
        value: Box::new(Expr::VariantConstruct {
            type_name: "Option".to_string(),
            variant: "Some".to_string(),
            fields: vec![Box::new(Expr::Lit(Literal::Int(42)))],
        }),
        body: Box::new(Expr::Match {
            scrutinee: Box::new(Expr::Var("x".to_string())),
            arms: vec![
                MatchArm {
                    pattern: Pattern::Variant {
                        variant: "Some".to_string(),
                        patterns: vec![Pattern::Literal(Literal::Int(42))],
                    },
                    body: Box::new(Expr::Lit(Literal::Int(1))),
                },
                MatchArm {
                    pattern: Pattern::Wildcard,
                    body: Box::new(Expr::Lit(Literal::Int(0))),
                },
            ],
        }),
    };

    let chunk = Compiler::compile(&expr).unwrap();

    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::CheckVariantTag(s) if s == "Some")));
    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::GetVariantField(0))));
}

// ========================================================================
// Edge Cases
// ========================================================================

#[test]
fn test_compile_variant_with_zero_fields() {
    // Unit
    let expr = Expr::VariantConstruct {
        type_name: "Value".to_string(),
        variant: "Unit".to_string(),
        fields: vec![],
    };

    let chunk = Compiler::compile(&expr).unwrap();

    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::MakeVariant(0))));
}

#[test]
fn test_compile_empty_type_name() {
    // Allow empty type_name (typechecker hasn't run yet)
    let expr = Expr::VariantConstruct {
        type_name: String::new(),
        variant: "Some".to_string(),
        fields: vec![Box::new(Expr::Lit(Literal::Int(42)))],
    };

    let chunk = Compiler::compile(&expr).unwrap();

    // Should still compile successfully
    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::MakeVariant(1))));
}
