//! Execution tests for Discriminated Unions (DUs)
//! Tests the complete pipeline: AST -> Compiler -> VM execution
//! Verifies that DU bytecode instructions actually execute correctly

use fusabi_frontend::ast::{Expr, Literal, MatchArm, Pattern};
use fusabi_frontend::compiler::Compiler;
use fusabi_vm::value::Value;
use fusabi_vm::vm::Vm;

#[test]
fn test_execute_variant_simple_enum() {
    let expr = Expr::VariantConstruct {
        type_name: "Option".to_string(),
        variant: "None".to_string(),
        fields: vec![],
    };
    let chunk = Compiler::compile(&expr).unwrap();
    let mut vm = Vm::new();
    let result = vm.execute(chunk).unwrap();

    match result {
        Value::Variant {
            type_name,
            variant_name,
            fields,
        } => {
            assert_eq!(type_name, "Option");
            assert_eq!(variant_name, "None");
            assert_eq!(fields.len(), 0);
        }
        _ => panic!("Expected Variant value, got {:?}", result),
    }
}

#[test]
fn test_execute_variant_with_single_field() {
    let expr = Expr::VariantConstruct {
        type_name: "Option".to_string(),
        variant: "Some".to_string(),
        fields: vec![Box::new(Expr::Lit(Literal::Int(42)))],
    };
    let chunk = Compiler::compile(&expr).unwrap();
    let mut vm = Vm::new();
    let result = vm.execute(chunk).unwrap();

    match result {
        Value::Variant {
            type_name,
            variant_name,
            fields,
        } => {
            assert_eq!(type_name, "Option");
            assert_eq!(variant_name, "Some");
            assert_eq!(fields.len(), 1);
            assert_eq!(fields[0], Value::Int(42));
        }
        _ => panic!("Expected Variant value, got {:?}", result),
    }
}

#[test]
fn test_execute_match_simple_variant_literal() {
    let expr = Expr::Match {
        scrutinee: Box::new(Expr::VariantConstruct {
            type_name: "Option".to_string(),
            variant: "None".to_string(),
            fields: vec![],
        }),
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
                    patterns: vec![Pattern::Var("x".to_string())],
                },
                body: Box::new(Expr::Var("x".to_string())),
            },
        ],
    };
    let chunk = Compiler::compile(&expr).unwrap();
    let mut vm = Vm::new();
    let result = vm.execute(chunk).unwrap();

    assert_eq!(result, Value::Int(0));
}
