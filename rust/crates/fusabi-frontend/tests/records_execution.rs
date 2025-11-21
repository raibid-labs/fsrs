//! Execution tests for Records
//! Tests the complete pipeline: AST -> Compiler -> VM execution
//! Verifies that record bytecode instructions actually execute correctly

use fusabi_frontend::ast::{BinOp, Expr, Literal};
use fusabi_frontend::compiler::Compiler;
use fusabi_vm::value::Value;
use fusabi_vm::vm::Vm;

// ========================================================================
// Record Creation Tests
// ========================================================================

#[test]
fn test_execute_record_empty() {
    // Test: {}
    let expr = Expr::RecordLiteral {
        type_name: "EmptyRecord".to_string(),
        fields: vec![],
    };
    let chunk = Compiler::compile(&expr).unwrap();
    let mut vm = Vm::new();
    let result = vm.execute(chunk).unwrap();

    match result {
        Value::Record(fields) => {
            assert_eq!(fields.borrow().len(), 0);
        }
        _ => panic!("Expected Record value, got {:?}", result),
    }
}

#[test]
fn test_execute_record_single_field() {
    // Test: { x = 42 }
    let expr = Expr::RecordLiteral {
        type_name: "Point1D".to_string(),
        fields: vec![("x".to_string(), Box::new(Expr::Lit(Literal::Int(42))))],
    };
    let chunk = Compiler::compile(&expr).unwrap();
    let mut vm = Vm::new();
    let result = vm.execute(chunk).unwrap();

    match result {
        Value::Record(fields) => {
            assert_eq!(fields.borrow().len(), 1);
            assert_eq!(fields.borrow().get("x"), Some(&Value::Int(42)));
        }
        _ => panic!("Expected Record value, got {:?}", result),
    }
}

#[test]
fn test_execute_record_multiple_fields() {
    // Test: { name = "Alice"; age = 30 }
    let expr = Expr::RecordLiteral {
        type_name: "Person".to_string(),
        fields: vec![
            (
                "name".to_string(),
                Box::new(Expr::Lit(Literal::Str("Alice".to_string()))),
            ),
            ("age".to_string(), Box::new(Expr::Lit(Literal::Int(30)))),
        ],
    };
    let chunk = Compiler::compile(&expr).unwrap();
    let mut vm = Vm::new();
    let result = vm.execute(chunk).unwrap();

    match result {
        Value::Record(fields) => {
            assert_eq!(fields.borrow().len(), 2);
            assert_eq!(
                fields.borrow().get("name"),
                Some(&Value::Str("Alice".to_string()))
            );
            assert_eq!(fields.borrow().get("age"), Some(&Value::Int(30)));
        }
        _ => panic!("Expected Record value, got {:?}", result),
    }
}

#[test]
fn test_execute_record_with_expressions() {
    // Test: { x = 10 + 5; y = 20 * 2 }
    let expr = Expr::RecordLiteral {
        type_name: "Point".to_string(),
        fields: vec![
            (
                "x".to_string(),
                Box::new(Expr::BinOp {
                    op: BinOp::Add,
                    left: Box::new(Expr::Lit(Literal::Int(10))),
                    right: Box::new(Expr::Lit(Literal::Int(5))),
                }),
            ),
            (
                "y".to_string(),
                Box::new(Expr::BinOp {
                    op: BinOp::Mul,
                    left: Box::new(Expr::Lit(Literal::Int(20))),
                    right: Box::new(Expr::Lit(Literal::Int(2))),
                }),
            ),
        ],
    };
    let chunk = Compiler::compile(&expr).unwrap();
    let mut vm = Vm::new();
    let result = vm.execute(chunk).unwrap();

    match result {
        Value::Record(fields) => {
            assert_eq!(fields.borrow().len(), 2);
            assert_eq!(fields.borrow().get("x"), Some(&Value::Int(15)));
            assert_eq!(fields.borrow().get("y"), Some(&Value::Int(40)));
        }
        _ => panic!("Expected Record value, got {:?}", result),
    }
}

// ========================================================================
// Record Field Access Tests
// ========================================================================

#[test]
fn test_execute_record_field_access_simple() {
    // Test: let person = { name = "Bob"; age = 25 } in person.name
    let expr = Expr::Let {
        name: "person".to_string(),
        value: Box::new(Expr::RecordLiteral {
            type_name: "Person".to_string(),
            fields: vec![
                (
                    "name".to_string(),
                    Box::new(Expr::Lit(Literal::Str("Bob".to_string()))),
                ),
                ("age".to_string(), Box::new(Expr::Lit(Literal::Int(25)))),
            ],
        }),
        body: Box::new(Expr::RecordAccess {
            record: Box::new(Expr::Var("person".to_string())),
            field: "name".to_string(),
        }),
    };
    let chunk = Compiler::compile(&expr).unwrap();
    let mut vm = Vm::new();
    let result = vm.execute(chunk).unwrap();

    assert_eq!(result, Value::Str("Bob".to_string()));
}

#[test]
fn test_execute_record_field_access_number() {
    // Test: let point = { x = 100; y = 200 } in point.x
    let expr = Expr::Let {
        name: "point".to_string(),
        value: Box::new(Expr::RecordLiteral {
            type_name: "Point".to_string(),
            fields: vec![
                ("x".to_string(), Box::new(Expr::Lit(Literal::Int(100)))),
                ("y".to_string(), Box::new(Expr::Lit(Literal::Int(200)))),
            ],
        }),
        body: Box::new(Expr::RecordAccess {
            record: Box::new(Expr::Var("point".to_string())),
            field: "x".to_string(),
        }),
    };
    let chunk = Compiler::compile(&expr).unwrap();
    let mut vm = Vm::new();
    let result = vm.execute(chunk).unwrap();

    assert_eq!(result, Value::Int(100));
}

#[test]
fn test_execute_record_field_in_expression() {
    // Test: let point = { x = 10; y = 20 } in point.x + point.y
    let expr = Expr::Let {
        name: "point".to_string(),
        value: Box::new(Expr::RecordLiteral {
            type_name: "Point".to_string(),
            fields: vec![
                ("x".to_string(), Box::new(Expr::Lit(Literal::Int(10)))),
                ("y".to_string(), Box::new(Expr::Lit(Literal::Int(20)))),
            ],
        }),
        body: Box::new(Expr::BinOp {
            op: BinOp::Add,
            left: Box::new(Expr::RecordAccess {
                record: Box::new(Expr::Var("point".to_string())),
                field: "x".to_string(),
            }),
            right: Box::new(Expr::RecordAccess {
                record: Box::new(Expr::Var("point".to_string())),
                field: "y".to_string(),
            }),
        }),
    };
    let chunk = Compiler::compile(&expr).unwrap();
    let mut vm = Vm::new();
    let result = vm.execute(chunk).unwrap();

    assert_eq!(result, Value::Int(30));
}

// ========================================================================
// Nested Record Tests
// ========================================================================

#[test]
fn test_execute_nested_record_creation() {
    // Test: { inner = { value = 42 }; outer = "test" }
    let expr = Expr::RecordLiteral {
        type_name: "Outer".to_string(),
        fields: vec![
            (
                "inner".to_string(),
                Box::new(Expr::RecordLiteral {
                    type_name: "Inner".to_string(),
                    fields: vec![("value".to_string(), Box::new(Expr::Lit(Literal::Int(42))))],
                }),
            ),
            (
                "outer".to_string(),
                Box::new(Expr::Lit(Literal::Str("test".to_string()))),
            ),
        ],
    };
    let chunk = Compiler::compile(&expr).unwrap();
    let mut vm = Vm::new();
    let result = vm.execute(chunk).unwrap();

    match result {
        Value::Record(fields) => {
            assert_eq!(fields.borrow().len(), 2);
            assert_eq!(
                fields.borrow().get("outer"),
                Some(&Value::Str("test".to_string()))
            );

            // Check nested record
            if let Some(Value::Record(inner)) = fields.borrow().get("inner") {
                assert_eq!(inner.borrow().get("value"), Some(&Value::Int(42)));
            } else {
                panic!("Expected nested record");
            }
        }
        _ => panic!("Expected Record value, got {:?}", result),
    }
}

#[test]
fn test_execute_nested_record_field_access() {
    // Test: let outer = { inner = { value = 99 } } in outer.inner
    let expr = Expr::Let {
        name: "outer".to_string(),
        value: Box::new(Expr::RecordLiteral {
            type_name: "Outer".to_string(),
            fields: vec![(
                "inner".to_string(),
                Box::new(Expr::RecordLiteral {
                    type_name: "Inner".to_string(),
                    fields: vec![("value".to_string(), Box::new(Expr::Lit(Literal::Int(99))))],
                }),
            )],
        }),
        body: Box::new(Expr::RecordAccess {
            record: Box::new(Expr::Var("outer".to_string())),
            field: "inner".to_string(),
        }),
    };
    let chunk = Compiler::compile(&expr).unwrap();
    let mut vm = Vm::new();
    let result = vm.execute(chunk).unwrap();

    match result {
        Value::Record(fields) => {
            assert_eq!(fields.borrow().get("value"), Some(&Value::Int(99)));
        }
        _ => panic!("Expected Record value, got {:?}", result),
    }
}

// ========================================================================
// Record Update Tests (Immutable)
// ========================================================================

#[test]
fn test_execute_record_update_single_field() {
    // Test: let p = { x = 10; y = 20 } in { p with x = 100 }
    let expr = Expr::Let {
        name: "p".to_string(),
        value: Box::new(Expr::RecordLiteral {
            type_name: "Point".to_string(),
            fields: vec![
                ("x".to_string(), Box::new(Expr::Lit(Literal::Int(10)))),
                ("y".to_string(), Box::new(Expr::Lit(Literal::Int(20)))),
            ],
        }),
        body: Box::new(Expr::RecordUpdate {
            record: Box::new(Expr::Var("p".to_string())),
            fields: vec![("x".to_string(), Box::new(Expr::Lit(Literal::Int(100))))],
        }),
    };
    let chunk = Compiler::compile(&expr).unwrap();
    let mut vm = Vm::new();
    let result = vm.execute(chunk).unwrap();

    match result {
        Value::Record(fields) => {
            assert_eq!(fields.borrow().len(), 2);
            assert_eq!(fields.borrow().get("x"), Some(&Value::Int(100)));
            assert_eq!(fields.borrow().get("y"), Some(&Value::Int(20)));
        }
        _ => panic!("Expected Record value, got {:?}", result),
    }
}

#[test]
fn test_execute_record_update_multiple_fields() {
    // Test: let p = { a = 1; b = 2; c = 3 } in { p with a = 10; c = 30 }
    let expr = Expr::Let {
        name: "p".to_string(),
        value: Box::new(Expr::RecordLiteral {
            type_name: "Record".to_string(),
            fields: vec![
                ("a".to_string(), Box::new(Expr::Lit(Literal::Int(1)))),
                ("b".to_string(), Box::new(Expr::Lit(Literal::Int(2)))),
                ("c".to_string(), Box::new(Expr::Lit(Literal::Int(3)))),
            ],
        }),
        body: Box::new(Expr::RecordUpdate {
            record: Box::new(Expr::Var("p".to_string())),
            fields: vec![
                ("a".to_string(), Box::new(Expr::Lit(Literal::Int(10)))),
                ("c".to_string(), Box::new(Expr::Lit(Literal::Int(30)))),
            ],
        }),
    };
    let chunk = Compiler::compile(&expr).unwrap();
    let mut vm = Vm::new();
    let result = vm.execute(chunk).unwrap();

    match result {
        Value::Record(fields) => {
            assert_eq!(fields.borrow().len(), 3);
            assert_eq!(fields.borrow().get("a"), Some(&Value::Int(10)));
            assert_eq!(fields.borrow().get("b"), Some(&Value::Int(2)));
            assert_eq!(fields.borrow().get("c"), Some(&Value::Int(30)));
        }
        _ => panic!("Expected Record value, got {:?}", result),
    }
}

// ========================================================================
// Records in Complex Expressions
// ========================================================================

#[test]
fn test_execute_record_in_tuple() {
    // Test: ({ x = 1 }, { y = 2 })
    let expr = Expr::Tuple(vec![
        Expr::RecordLiteral {
            type_name: "R1".to_string(),
            fields: vec![("x".to_string(), Box::new(Expr::Lit(Literal::Int(1))))],
        },
        Expr::RecordLiteral {
            type_name: "R2".to_string(),
            fields: vec![("y".to_string(), Box::new(Expr::Lit(Literal::Int(2))))],
        },
    ]);
    let chunk = Compiler::compile(&expr).unwrap();
    let mut vm = Vm::new();
    let result = vm.execute(chunk).unwrap();

    match result {
        Value::Tuple(elements) => {
            assert_eq!(elements.len(), 2);

            // Check first record
            if let Value::Record(r1) = &elements[0] {
                assert_eq!(r1.borrow().get("x"), Some(&Value::Int(1)));
            } else {
                panic!("Expected first tuple element to be a record");
            }

            // Check second record
            if let Value::Record(r2) = &elements[1] {
                assert_eq!(r2.borrow().get("y"), Some(&Value::Int(2)));
            } else {
                panic!("Expected second tuple element to be a record");
            }
        }
        _ => panic!("Expected Tuple value, got {:?}", result),
    }
}

#[test]
fn test_execute_record_in_if_expression() {
    // Test: if true then { x = 1 } else { x = 2 }
    let expr = Expr::If {
        cond: Box::new(Expr::Lit(Literal::Bool(true))),
        then_branch: Box::new(Expr::RecordLiteral {
            type_name: "R".to_string(),
            fields: vec![("x".to_string(), Box::new(Expr::Lit(Literal::Int(1))))],
        }),
        else_branch: Box::new(Expr::RecordLiteral {
            type_name: "R".to_string(),
            fields: vec![("x".to_string(), Box::new(Expr::Lit(Literal::Int(2))))],
        }),
    };
    let chunk = Compiler::compile(&expr).unwrap();
    let mut vm = Vm::new();
    let result = vm.execute(chunk).unwrap();

    match result {
        Value::Record(fields) => {
            assert_eq!(fields.borrow().get("x"), Some(&Value::Int(1)));
        }
        _ => panic!("Expected Record value, got {:?}", result),
    }
}

#[test]
fn test_execute_record_with_bool_fields() {
    // Test: { active = true; disabled = false }
    let expr = Expr::RecordLiteral {
        type_name: "Flags".to_string(),
        fields: vec![
            (
                "active".to_string(),
                Box::new(Expr::Lit(Literal::Bool(true))),
            ),
            (
                "disabled".to_string(),
                Box::new(Expr::Lit(Literal::Bool(false))),
            ),
        ],
    };
    let chunk = Compiler::compile(&expr).unwrap();
    let mut vm = Vm::new();
    let result = vm.execute(chunk).unwrap();

    match result {
        Value::Record(fields) => {
            assert_eq!(fields.borrow().get("active"), Some(&Value::Bool(true)));
            assert_eq!(fields.borrow().get("disabled"), Some(&Value::Bool(false)));
        }
        _ => panic!("Expected Record value, got {:?}", result),
    }
}

#[test]
fn test_execute_record_with_mixed_types() {
    // Test: { count = 42; name = "test"; active = true }
    let expr = Expr::RecordLiteral {
        type_name: "MixedRecord".to_string(),
        fields: vec![
            ("count".to_string(), Box::new(Expr::Lit(Literal::Int(42)))),
            (
                "name".to_string(),
                Box::new(Expr::Lit(Literal::Str("test".to_string()))),
            ),
            (
                "active".to_string(),
                Box::new(Expr::Lit(Literal::Bool(true))),
            ),
        ],
    };
    let chunk = Compiler::compile(&expr).unwrap();
    let mut vm = Vm::new();
    let result = vm.execute(chunk).unwrap();

    match result {
        Value::Record(fields) => {
            assert_eq!(fields.borrow().len(), 3);
            assert_eq!(fields.borrow().get("count"), Some(&Value::Int(42)));
            assert_eq!(
                fields.borrow().get("name"),
                Some(&Value::Str("test".to_string()))
            );
            assert_eq!(fields.borrow().get("active"), Some(&Value::Bool(true)));
        }
        _ => panic!("Expected Record value, got {:?}", result),
    }
}

#[test]
fn test_execute_record_field_access_in_tuple() {
    // Test: let p = { x = 5; y = 10 } in (p.x, p.y)
    let expr = Expr::Let {
        name: "p".to_string(),
        value: Box::new(Expr::RecordLiteral {
            type_name: "Point".to_string(),
            fields: vec![
                ("x".to_string(), Box::new(Expr::Lit(Literal::Int(5)))),
                ("y".to_string(), Box::new(Expr::Lit(Literal::Int(10)))),
            ],
        }),
        body: Box::new(Expr::Tuple(vec![
            Expr::RecordAccess {
                record: Box::new(Expr::Var("p".to_string())),
                field: "x".to_string(),
            },
            Expr::RecordAccess {
                record: Box::new(Expr::Var("p".to_string())),
                field: "y".to_string(),
            },
        ])),
    };
    let chunk = Compiler::compile(&expr).unwrap();
    let mut vm = Vm::new();
    let result = vm.execute(chunk).unwrap();

    assert_eq!(result, Value::Tuple(vec![Value::Int(5), Value::Int(10)]));
}
