// Pattern Matching Integration Tests - Layer 4 (Issue #27)
// Comprehensive tests for pattern matching compilation and execution

use fsrs_frontend::ast::*;
use fsrs_frontend::compiler::Compiler;
use fsrs_vm::{Value, Vm};

// Helper to compile and run an expression
fn run_expr(expr: &Expr) -> Result<Value, Box<dyn std::error::Error>> {
    let chunk = Compiler::compile(expr)?;
    let mut vm = Vm::new();
    let result = vm.execute(chunk)?;
    Ok(result)
}

// ============================================================================
// SECTION 1: Literal Pattern Matching (10 tests)
// ============================================================================

#[test]
fn test_match_literal_int_zero() {
    // match 0 with | 0 -> "zero" | _ -> "other"
    let expr = Expr::Match {
        scrutinee: Box::new(Expr::Lit(Literal::Int(0))),
        arms: vec![
            MatchArm {
                pattern: Pattern::Literal(Literal::Int(0)),
                body: Box::new(Expr::Lit(Literal::Str("zero".into()))),
            },
            MatchArm {
                pattern: Pattern::Wildcard,
                body: Box::new(Expr::Lit(Literal::Str("other".into()))),
            },
        ],
    };

    let result = run_expr(&expr).unwrap();
    assert_eq!(result, Value::Str("zero".into()));
}

#[test]
fn test_match_literal_int_nonzero() {
    // match 42 with | 0 -> "zero" | _ -> "other"
    let expr = Expr::Match {
        scrutinee: Box::new(Expr::Lit(Literal::Int(42))),
        arms: vec![
            MatchArm {
                pattern: Pattern::Literal(Literal::Int(0)),
                body: Box::new(Expr::Lit(Literal::Str("zero".into()))),
            },
            MatchArm {
                pattern: Pattern::Wildcard,
                body: Box::new(Expr::Lit(Literal::Str("other".into()))),
            },
        ],
    };

    let result = run_expr(&expr).unwrap();
    assert_eq!(result, Value::Str("other".into()));
}

#[test]
fn test_match_literal_bool_true() {
    // match true with | true -> 1 | false -> 0
    let expr = Expr::Match {
        scrutinee: Box::new(Expr::Lit(Literal::Bool(true))),
        arms: vec![
            MatchArm {
                pattern: Pattern::Literal(Literal::Bool(true)),
                body: Box::new(Expr::Lit(Literal::Int(1))),
            },
            MatchArm {
                pattern: Pattern::Literal(Literal::Bool(false)),
                body: Box::new(Expr::Lit(Literal::Int(0))),
            },
        ],
    };

    let result = run_expr(&expr).unwrap();
    assert_eq!(result, Value::Int(1));
}

#[test]
fn test_match_literal_bool_false() {
    // match false with | true -> 1 | false -> 0
    let expr = Expr::Match {
        scrutinee: Box::new(Expr::Lit(Literal::Bool(false))),
        arms: vec![
            MatchArm {
                pattern: Pattern::Literal(Literal::Bool(true)),
                body: Box::new(Expr::Lit(Literal::Int(1))),
            },
            MatchArm {
                pattern: Pattern::Literal(Literal::Bool(false)),
                body: Box::new(Expr::Lit(Literal::Int(0))),
            },
        ],
    };

    let result = run_expr(&expr).unwrap();
    assert_eq!(result, Value::Int(0));
}

#[test]
fn test_match_literal_string_match() {
    // match "hello" with | "hello" -> 1 | _ -> 0
    let expr = Expr::Match {
        scrutinee: Box::new(Expr::Lit(Literal::Str("hello".into()))),
        arms: vec![
            MatchArm {
                pattern: Pattern::Literal(Literal::Str("hello".into())),
                body: Box::new(Expr::Lit(Literal::Int(1))),
            },
            MatchArm {
                pattern: Pattern::Wildcard,
                body: Box::new(Expr::Lit(Literal::Int(0))),
            },
        ],
    };

    let result = run_expr(&expr).unwrap();
    assert_eq!(result, Value::Int(1));
}

#[test]
fn test_match_literal_string_nomatch() {
    // match "world" with | "hello" -> 1 | _ -> 0
    let expr = Expr::Match {
        scrutinee: Box::new(Expr::Lit(Literal::Str("world".into()))),
        arms: vec![
            MatchArm {
                pattern: Pattern::Literal(Literal::Str("hello".into())),
                body: Box::new(Expr::Lit(Literal::Int(1))),
            },
            MatchArm {
                pattern: Pattern::Wildcard,
                body: Box::new(Expr::Lit(Literal::Int(0))),
            },
        ],
    };

    let result = run_expr(&expr).unwrap();
    assert_eq!(result, Value::Int(0));
}

#[test]
fn test_match_multiple_literal_arms() {
    // match 2 with | 0 -> "zero" | 1 -> "one" | 2 -> "two" | _ -> "many"
    let expr = Expr::Match {
        scrutinee: Box::new(Expr::Lit(Literal::Int(2))),
        arms: vec![
            MatchArm {
                pattern: Pattern::Literal(Literal::Int(0)),
                body: Box::new(Expr::Lit(Literal::Str("zero".into()))),
            },
            MatchArm {
                pattern: Pattern::Literal(Literal::Int(1)),
                body: Box::new(Expr::Lit(Literal::Str("one".into()))),
            },
            MatchArm {
                pattern: Pattern::Literal(Literal::Int(2)),
                body: Box::new(Expr::Lit(Literal::Str("two".into()))),
            },
            MatchArm {
                pattern: Pattern::Wildcard,
                body: Box::new(Expr::Lit(Literal::Str("many".into()))),
            },
        ],
    };

    let result = run_expr(&expr).unwrap();
    assert_eq!(result, Value::Str("two".into()));
}

#[test]
fn test_match_literal_first_match_wins() {
    // match 0 with | 0 -> "first" | 0 -> "second" | _ -> "other"
    let expr = Expr::Match {
        scrutinee: Box::new(Expr::Lit(Literal::Int(0))),
        arms: vec![
            MatchArm {
                pattern: Pattern::Literal(Literal::Int(0)),
                body: Box::new(Expr::Lit(Literal::Str("first".into()))),
            },
            MatchArm {
                pattern: Pattern::Literal(Literal::Int(0)),
                body: Box::new(Expr::Lit(Literal::Str("second".into()))),
            },
            MatchArm {
                pattern: Pattern::Wildcard,
                body: Box::new(Expr::Lit(Literal::Str("other".into()))),
            },
        ],
    };

    let result = run_expr(&expr).unwrap();
    assert_eq!(result, Value::Str("first".into()));
}

#[test]
fn test_match_literal_with_expression_body() {
    // match 1 with | 1 -> 10 + 5 | _ -> 0
    let expr = Expr::Match {
        scrutinee: Box::new(Expr::Lit(Literal::Int(1))),
        arms: vec![
            MatchArm {
                pattern: Pattern::Literal(Literal::Int(1)),
                body: Box::new(Expr::BinOp {
                    op: BinOp::Add,
                    left: Box::new(Expr::Lit(Literal::Int(10))),
                    right: Box::new(Expr::Lit(Literal::Int(5))),
                }),
            },
            MatchArm {
                pattern: Pattern::Wildcard,
                body: Box::new(Expr::Lit(Literal::Int(0))),
            },
        ],
    };

    let result = run_expr(&expr).unwrap();
    assert_eq!(result, Value::Int(15));
}

#[test]
fn test_match_literal_negative_int() {
    // match -5 with | -5 -> "negative five" | _ -> "other"
    let expr = Expr::Match {
        scrutinee: Box::new(Expr::Lit(Literal::Int(-5))),
        arms: vec![
            MatchArm {
                pattern: Pattern::Literal(Literal::Int(-5)),
                body: Box::new(Expr::Lit(Literal::Str("negative five".into()))),
            },
            MatchArm {
                pattern: Pattern::Wildcard,
                body: Box::new(Expr::Lit(Literal::Str("other".into()))),
            },
        ],
    };

    let result = run_expr(&expr).unwrap();
    assert_eq!(result, Value::Str("negative five".into()));
}

// ============================================================================
// SECTION 2: Variable Pattern Binding (5 tests)
// ============================================================================

#[test]
fn test_match_variable_binding_simple() {
    // match 42 with | x -> x
    let expr = Expr::Match {
        scrutinee: Box::new(Expr::Lit(Literal::Int(42))),
        arms: vec![MatchArm {
            pattern: Pattern::Var("x".into()),
            body: Box::new(Expr::Var("x".into())),
        }],
    };

    let result = run_expr(&expr).unwrap();
    assert_eq!(result, Value::Int(42));
}

#[test]
fn test_match_variable_binding_with_operation() {
    // match 21 with | x -> x * 2
    let expr = Expr::Match {
        scrutinee: Box::new(Expr::Lit(Literal::Int(21))),
        arms: vec![MatchArm {
            pattern: Pattern::Var("x".into()),
            body: Box::new(Expr::BinOp {
                op: BinOp::Mul,
                left: Box::new(Expr::Var("x".into())),
                right: Box::new(Expr::Lit(Literal::Int(2))),
            }),
        }],
    };

    let result = run_expr(&expr).unwrap();
    assert_eq!(result, Value::Int(42));
}

#[test]
fn test_match_variable_binding_second_arm() {
    // match 100 with | 0 -> 0 | n -> n + 1
    let expr = Expr::Match {
        scrutinee: Box::new(Expr::Lit(Literal::Int(100))),
        arms: vec![
            MatchArm {
                pattern: Pattern::Literal(Literal::Int(0)),
                body: Box::new(Expr::Lit(Literal::Int(0))),
            },
            MatchArm {
                pattern: Pattern::Var("n".into()),
                body: Box::new(Expr::BinOp {
                    op: BinOp::Add,
                    left: Box::new(Expr::Var("n".into())),
                    right: Box::new(Expr::Lit(Literal::Int(1))),
                }),
            },
        ],
    };

    let result = run_expr(&expr).unwrap();
    assert_eq!(result, Value::Int(101));
}

#[test]
fn test_match_variable_binding_string() {
    // match "hello" with | s -> s
    let expr = Expr::Match {
        scrutinee: Box::new(Expr::Lit(Literal::Str("hello".into()))),
        arms: vec![MatchArm {
            pattern: Pattern::Var("s".into()),
            body: Box::new(Expr::Var("s".into())),
        }],
    };

    let result = run_expr(&expr).unwrap();
    assert_eq!(result, Value::Str("hello".into()));
}

#[test]
fn test_match_variable_binding_bool() {
    // match true with | b -> b
    let expr = Expr::Match {
        scrutinee: Box::new(Expr::Lit(Literal::Bool(true))),
        arms: vec![MatchArm {
            pattern: Pattern::Var("b".into()),
            body: Box::new(Expr::Var("b".into())),
        }],
    };

    let result = run_expr(&expr).unwrap();
    assert_eq!(result, Value::Bool(true));
}

// ============================================================================
// SECTION 3: Wildcard Pattern (3 tests)
// ============================================================================

#[test]
fn test_match_wildcard_simple() {
    // match 42 with | _ -> "anything"
    let expr = Expr::Match {
        scrutinee: Box::new(Expr::Lit(Literal::Int(42))),
        arms: vec![MatchArm {
            pattern: Pattern::Wildcard,
            body: Box::new(Expr::Lit(Literal::Str("anything".into()))),
        }],
    };

    let result = run_expr(&expr).unwrap();
    assert_eq!(result, Value::Str("anything".into()));
}

#[test]
fn test_match_wildcard_as_default() {
    // match 999 with | 0 -> "zero" | 1 -> "one" | _ -> "other"
    let expr = Expr::Match {
        scrutinee: Box::new(Expr::Lit(Literal::Int(999))),
        arms: vec![
            MatchArm {
                pattern: Pattern::Literal(Literal::Int(0)),
                body: Box::new(Expr::Lit(Literal::Str("zero".into()))),
            },
            MatchArm {
                pattern: Pattern::Literal(Literal::Int(1)),
                body: Box::new(Expr::Lit(Literal::Str("one".into()))),
            },
            MatchArm {
                pattern: Pattern::Wildcard,
                body: Box::new(Expr::Lit(Literal::Str("other".into()))),
            },
        ],
    };

    let result = run_expr(&expr).unwrap();
    assert_eq!(result, Value::Str("other".into()));
}

#[test]
fn test_match_wildcard_catches_all() {
    // match "anything" with | _ -> 42
    let expr = Expr::Match {
        scrutinee: Box::new(Expr::Lit(Literal::Str("anything".into()))),
        arms: vec![MatchArm {
            pattern: Pattern::Wildcard,
            body: Box::new(Expr::Lit(Literal::Int(42))),
        }],
    };

    let result = run_expr(&expr).unwrap();
    assert_eq!(result, Value::Int(42));
}

// ============================================================================
// SECTION 4: Tuple Pattern Matching (10 tests)
// ============================================================================

#[test]
fn test_match_tuple_literal_pair() {
    // match (0, 0) with | (0, 0) -> "origin" | _ -> "other"
    let expr = Expr::Match {
        scrutinee: Box::new(Expr::Tuple(vec![
            Expr::Lit(Literal::Int(0)),
            Expr::Lit(Literal::Int(0)),
        ])),
        arms: vec![
            MatchArm {
                pattern: Pattern::Tuple(vec![
                    Pattern::Literal(Literal::Int(0)),
                    Pattern::Literal(Literal::Int(0)),
                ]),
                body: Box::new(Expr::Lit(Literal::Str("origin".into()))),
            },
            MatchArm {
                pattern: Pattern::Wildcard,
                body: Box::new(Expr::Lit(Literal::Str("other".into()))),
            },
        ],
    };

    let result = run_expr(&expr).unwrap();
    assert_eq!(result, Value::Str("origin".into()));
}

#[test]
fn test_match_tuple_literal_nomatch() {
    // match (1, 2) with | (0, 0) -> "origin" | _ -> "other"
    let expr = Expr::Match {
        scrutinee: Box::new(Expr::Tuple(vec![
            Expr::Lit(Literal::Int(1)),
            Expr::Lit(Literal::Int(2)),
        ])),
        arms: vec![
            MatchArm {
                pattern: Pattern::Tuple(vec![
                    Pattern::Literal(Literal::Int(0)),
                    Pattern::Literal(Literal::Int(0)),
                ]),
                body: Box::new(Expr::Lit(Literal::Str("origin".into()))),
            },
            MatchArm {
                pattern: Pattern::Wildcard,
                body: Box::new(Expr::Lit(Literal::Str("other".into()))),
            },
        ],
    };

    let result = run_expr(&expr).unwrap();
    assert_eq!(result, Value::Str("other".into()));
}

#[test]
fn test_match_tuple_with_wildcard() {
    // match (0, 5) with | (0, _) -> "y-axis" | _ -> "other"
    let expr = Expr::Match {
        scrutinee: Box::new(Expr::Tuple(vec![
            Expr::Lit(Literal::Int(0)),
            Expr::Lit(Literal::Int(5)),
        ])),
        arms: vec![
            MatchArm {
                pattern: Pattern::Tuple(vec![Pattern::Literal(Literal::Int(0)), Pattern::Wildcard]),
                body: Box::new(Expr::Lit(Literal::Str("y-axis".into()))),
            },
            MatchArm {
                pattern: Pattern::Wildcard,
                body: Box::new(Expr::Lit(Literal::Str("other".into()))),
            },
        ],
    };

    let result = run_expr(&expr).unwrap();
    assert_eq!(result, Value::Str("y-axis".into()));
}

#[test]
fn test_match_tuple_with_variable_binding() {
    // match (3, 4) with | (x, y) -> x + y
    let expr = Expr::Match {
        scrutinee: Box::new(Expr::Tuple(vec![
            Expr::Lit(Literal::Int(3)),
            Expr::Lit(Literal::Int(4)),
        ])),
        arms: vec![MatchArm {
            pattern: Pattern::Tuple(vec![Pattern::Var("x".into()), Pattern::Var("y".into())]),
            body: Box::new(Expr::BinOp {
                op: BinOp::Add,
                left: Box::new(Expr::Var("x".into())),
                right: Box::new(Expr::Var("y".into())),
            }),
        }],
    };

    let result = run_expr(&expr).unwrap();
    assert_eq!(result, Value::Int(7));
}

#[test]
fn test_match_tuple_mixed_patterns() {
    // match (0, 10) with | (0, y) -> y | (x, 0) -> x | (x, y) -> x + y
    let expr = Expr::Match {
        scrutinee: Box::new(Expr::Tuple(vec![
            Expr::Lit(Literal::Int(0)),
            Expr::Lit(Literal::Int(10)),
        ])),
        arms: vec![
            MatchArm {
                pattern: Pattern::Tuple(vec![
                    Pattern::Literal(Literal::Int(0)),
                    Pattern::Var("y".into()),
                ]),
                body: Box::new(Expr::Var("y".into())),
            },
            MatchArm {
                pattern: Pattern::Tuple(vec![
                    Pattern::Var("x".into()),
                    Pattern::Literal(Literal::Int(0)),
                ]),
                body: Box::new(Expr::Var("x".into())),
            },
            MatchArm {
                pattern: Pattern::Tuple(vec![Pattern::Var("x".into()), Pattern::Var("y".into())]),
                body: Box::new(Expr::BinOp {
                    op: BinOp::Add,
                    left: Box::new(Expr::Var("x".into())),
                    right: Box::new(Expr::Var("y".into())),
                }),
            },
        ],
    };

    let result = run_expr(&expr).unwrap();
    assert_eq!(result, Value::Int(10));
}

#[test]
fn test_match_tuple_triple() {
    // match (1, 2, 3) with | (a, b, c) -> a + b + c
    let expr = Expr::Match {
        scrutinee: Box::new(Expr::Tuple(vec![
            Expr::Lit(Literal::Int(1)),
            Expr::Lit(Literal::Int(2)),
            Expr::Lit(Literal::Int(3)),
        ])),
        arms: vec![MatchArm {
            pattern: Pattern::Tuple(vec![
                Pattern::Var("a".into()),
                Pattern::Var("b".into()),
                Pattern::Var("c".into()),
            ]),
            body: Box::new(Expr::BinOp {
                op: BinOp::Add,
                left: Box::new(Expr::BinOp {
                    op: BinOp::Add,
                    left: Box::new(Expr::Var("a".into())),
                    right: Box::new(Expr::Var("b".into())),
                }),
                right: Box::new(Expr::Var("c".into())),
            }),
        }],
    };

    let result = run_expr(&expr).unwrap();
    assert_eq!(result, Value::Int(6));
}

#[test]
fn test_match_tuple_single_element() {
    // match (42) with | (x) -> x
    let expr = Expr::Match {
        scrutinee: Box::new(Expr::Tuple(vec![Expr::Lit(Literal::Int(42))])),
        arms: vec![MatchArm {
            pattern: Pattern::Tuple(vec![Pattern::Var("x".into())]),
            body: Box::new(Expr::Var("x".into())),
        }],
    };

    let result = run_expr(&expr).unwrap();
    assert_eq!(result, Value::Int(42));
}

#[test]
fn test_match_tuple_empty() {
    // match () with | () -> 42
    let expr = Expr::Match {
        scrutinee: Box::new(Expr::Tuple(vec![])),
        arms: vec![MatchArm {
            pattern: Pattern::Tuple(vec![]),
            body: Box::new(Expr::Lit(Literal::Int(42))),
        }],
    };

    let result = run_expr(&expr).unwrap();
    assert_eq!(result, Value::Int(42));
}

#[test]
fn test_match_tuple_wildcard_entire() {
    // match (1, 2) with | _ -> 99
    let expr = Expr::Match {
        scrutinee: Box::new(Expr::Tuple(vec![
            Expr::Lit(Literal::Int(1)),
            Expr::Lit(Literal::Int(2)),
        ])),
        arms: vec![MatchArm {
            pattern: Pattern::Wildcard,
            body: Box::new(Expr::Lit(Literal::Int(99))),
        }],
    };

    let result = run_expr(&expr).unwrap();
    assert_eq!(result, Value::Int(99));
}

#[test]
fn test_match_tuple_variable_entire() {
    // match (10, 20) with | t -> t
    let expr = Expr::Match {
        scrutinee: Box::new(Expr::Tuple(vec![
            Expr::Lit(Literal::Int(10)),
            Expr::Lit(Literal::Int(20)),
        ])),
        arms: vec![MatchArm {
            pattern: Pattern::Var("t".into()),
            body: Box::new(Expr::Var("t".into())),
        }],
    };

    let result = run_expr(&expr).unwrap();
    assert_eq!(result, Value::Tuple(vec![Value::Int(10), Value::Int(20)]));
}

// ============================================================================
// SECTION 5: Match in Let Bindings (3 tests)
// ============================================================================

#[test]
fn test_match_in_let_value() {
    // let x = (match 1 with | 1 -> 10 | _ -> 0) in x + 5
    let expr = Expr::Let {
        name: "x".into(),
        value: Box::new(Expr::Match {
            scrutinee: Box::new(Expr::Lit(Literal::Int(1))),
            arms: vec![
                MatchArm {
                    pattern: Pattern::Literal(Literal::Int(1)),
                    body: Box::new(Expr::Lit(Literal::Int(10))),
                },
                MatchArm {
                    pattern: Pattern::Wildcard,
                    body: Box::new(Expr::Lit(Literal::Int(0))),
                },
            ],
        }),
        body: Box::new(Expr::BinOp {
            op: BinOp::Add,
            left: Box::new(Expr::Var("x".into())),
            right: Box::new(Expr::Lit(Literal::Int(5))),
        }),
    };

    let result = run_expr(&expr).unwrap();
    assert_eq!(result, Value::Int(15));
}

#[test]
fn test_match_in_let_body() {
    // let n = 42 in (match n with | 42 -> "yes" | _ -> "no")
    let expr = Expr::Let {
        name: "n".into(),
        value: Box::new(Expr::Lit(Literal::Int(42))),
        body: Box::new(Expr::Match {
            scrutinee: Box::new(Expr::Var("n".into())),
            arms: vec![
                MatchArm {
                    pattern: Pattern::Literal(Literal::Int(42)),
                    body: Box::new(Expr::Lit(Literal::Str("yes".into()))),
                },
                MatchArm {
                    pattern: Pattern::Wildcard,
                    body: Box::new(Expr::Lit(Literal::Str("no".into()))),
                },
            ],
        }),
    };

    let result = run_expr(&expr).unwrap();
    assert_eq!(result, Value::Str("yes".into()));
}

#[test]
fn test_match_nested_let() {
    // let x = 10 in (match x with | n -> let y = n * 2 in y + 1)
    let expr = Expr::Let {
        name: "x".into(),
        value: Box::new(Expr::Lit(Literal::Int(10))),
        body: Box::new(Expr::Match {
            scrutinee: Box::new(Expr::Var("x".into())),
            arms: vec![MatchArm {
                pattern: Pattern::Var("n".into()),
                body: Box::new(Expr::Let {
                    name: "y".into(),
                    value: Box::new(Expr::BinOp {
                        op: BinOp::Mul,
                        left: Box::new(Expr::Var("n".into())),
                        right: Box::new(Expr::Lit(Literal::Int(2))),
                    }),
                    body: Box::new(Expr::BinOp {
                        op: BinOp::Add,
                        left: Box::new(Expr::Var("y".into())),
                        right: Box::new(Expr::Lit(Literal::Int(1))),
                    }),
                }),
            }],
        }),
    };

    let result = run_expr(&expr).unwrap();
    assert_eq!(result, Value::Int(21));
}

// ============================================================================
// SECTION 6: Match as Function Body (4 tests)
// ============================================================================

// Note: These tests use match expressions in function-like contexts

#[test]
fn test_match_classify_number() {
    // Simulates: let classify n = match n with | 0 -> "zero" | 1 -> "one" | _ -> "many"
    // For now, just test the match expression directly
    let expr = Expr::Match {
        scrutinee: Box::new(Expr::Lit(Literal::Int(0))),
        arms: vec![
            MatchArm {
                pattern: Pattern::Literal(Literal::Int(0)),
                body: Box::new(Expr::Lit(Literal::Str("zero".into()))),
            },
            MatchArm {
                pattern: Pattern::Literal(Literal::Int(1)),
                body: Box::new(Expr::Lit(Literal::Str("one".into()))),
            },
            MatchArm {
                pattern: Pattern::Wildcard,
                body: Box::new(Expr::Lit(Literal::Str("many".into()))),
            },
        ],
    };

    let result = run_expr(&expr).unwrap();
    assert_eq!(result, Value::Str("zero".into()));
}

#[test]
fn test_match_abs_value() {
    // Simulates: let abs n = match n with | x when x < 0 -> -x | x -> x
    // Without guards, test: match -5 with | n -> if n < 0 then -n else n
    let expr = Expr::Match {
        scrutinee: Box::new(Expr::Lit(Literal::Int(-5))),
        arms: vec![MatchArm {
            pattern: Pattern::Var("n".into()),
            body: Box::new(Expr::If {
                cond: Box::new(Expr::BinOp {
                    op: BinOp::Lt,
                    left: Box::new(Expr::Var("n".into())),
                    right: Box::new(Expr::Lit(Literal::Int(0))),
                }),
                then_branch: Box::new(Expr::BinOp {
                    op: BinOp::Sub,
                    left: Box::new(Expr::Lit(Literal::Int(0))),
                    right: Box::new(Expr::Var("n".into())),
                }),
                else_branch: Box::new(Expr::Var("n".into())),
            }),
        }],
    };

    let result = run_expr(&expr).unwrap();
    assert_eq!(result, Value::Int(5));
}

#[test]
fn test_match_tuple_destructure_function() {
    // Simulates: let first p = match p with | (x, _) -> x
    let expr = Expr::Match {
        scrutinee: Box::new(Expr::Tuple(vec![
            Expr::Lit(Literal::Int(10)),
            Expr::Lit(Literal::Int(20)),
        ])),
        arms: vec![MatchArm {
            pattern: Pattern::Tuple(vec![Pattern::Var("x".into()), Pattern::Wildcard]),
            body: Box::new(Expr::Var("x".into())),
        }],
    };

    let result = run_expr(&expr).unwrap();
    assert_eq!(result, Value::Int(10));
}

#[test]
fn test_match_swap_function() {
    // Simulates: let swap p = match p with | (x, y) -> (y, x)
    let expr = Expr::Match {
        scrutinee: Box::new(Expr::Tuple(vec![
            Expr::Lit(Literal::Int(1)),
            Expr::Lit(Literal::Int(2)),
        ])),
        arms: vec![MatchArm {
            pattern: Pattern::Tuple(vec![Pattern::Var("x".into()), Pattern::Var("y".into())]),
            body: Box::new(Expr::Tuple(vec![
                Expr::Var("y".into()),
                Expr::Var("x".into()),
            ])),
        }],
    };

    let result = run_expr(&expr).unwrap();
    assert_eq!(result, Value::Tuple(vec![Value::Int(2), Value::Int(1)]));
}

// ============================================================================
// SECTION 7: Complex Patterns (5+ tests)
// ============================================================================

#[test]
fn test_match_point_classification() {
    // match (0, 0) with
    // | (0, 0) -> "origin"
    // | (0, y) -> "y-axis"
    // | (x, 0) -> "x-axis"
    // | (x, y) -> "quadrant"
    let expr = Expr::Match {
        scrutinee: Box::new(Expr::Tuple(vec![
            Expr::Lit(Literal::Int(0)),
            Expr::Lit(Literal::Int(0)),
        ])),
        arms: vec![
            MatchArm {
                pattern: Pattern::Tuple(vec![
                    Pattern::Literal(Literal::Int(0)),
                    Pattern::Literal(Literal::Int(0)),
                ]),
                body: Box::new(Expr::Lit(Literal::Str("origin".into()))),
            },
            MatchArm {
                pattern: Pattern::Tuple(vec![
                    Pattern::Literal(Literal::Int(0)),
                    Pattern::Var("y".into()),
                ]),
                body: Box::new(Expr::Lit(Literal::Str("y-axis".into()))),
            },
            MatchArm {
                pattern: Pattern::Tuple(vec![
                    Pattern::Var("x".into()),
                    Pattern::Literal(Literal::Int(0)),
                ]),
                body: Box::new(Expr::Lit(Literal::Str("x-axis".into()))),
            },
            MatchArm {
                pattern: Pattern::Tuple(vec![Pattern::Var("x".into()), Pattern::Var("y".into())]),
                body: Box::new(Expr::Lit(Literal::Str("quadrant".into()))),
            },
        ],
    };

    let result = run_expr(&expr).unwrap();
    assert_eq!(result, Value::Str("origin".into()));
}

#[test]
fn test_match_point_y_axis() {
    let expr = Expr::Match {
        scrutinee: Box::new(Expr::Tuple(vec![
            Expr::Lit(Literal::Int(0)),
            Expr::Lit(Literal::Int(5)),
        ])),
        arms: vec![
            MatchArm {
                pattern: Pattern::Tuple(vec![
                    Pattern::Literal(Literal::Int(0)),
                    Pattern::Literal(Literal::Int(0)),
                ]),
                body: Box::new(Expr::Lit(Literal::Str("origin".into()))),
            },
            MatchArm {
                pattern: Pattern::Tuple(vec![
                    Pattern::Literal(Literal::Int(0)),
                    Pattern::Var("y".into()),
                ]),
                body: Box::new(Expr::Lit(Literal::Str("y-axis".into()))),
            },
            MatchArm {
                pattern: Pattern::Tuple(vec![
                    Pattern::Var("x".into()),
                    Pattern::Literal(Literal::Int(0)),
                ]),
                body: Box::new(Expr::Lit(Literal::Str("x-axis".into()))),
            },
            MatchArm {
                pattern: Pattern::Tuple(vec![Pattern::Var("x".into()), Pattern::Var("y".into())]),
                body: Box::new(Expr::Lit(Literal::Str("quadrant".into()))),
            },
        ],
    };

    let result = run_expr(&expr).unwrap();
    assert_eq!(result, Value::Str("y-axis".into()));
}

#[test]
fn test_match_complex_calculation() {
    // match (2, 3) with | (a, b) -> a * a + b * b
    let expr = Expr::Match {
        scrutinee: Box::new(Expr::Tuple(vec![
            Expr::Lit(Literal::Int(2)),
            Expr::Lit(Literal::Int(3)),
        ])),
        arms: vec![MatchArm {
            pattern: Pattern::Tuple(vec![Pattern::Var("a".into()), Pattern::Var("b".into())]),
            body: Box::new(Expr::BinOp {
                op: BinOp::Add,
                left: Box::new(Expr::BinOp {
                    op: BinOp::Mul,
                    left: Box::new(Expr::Var("a".into())),
                    right: Box::new(Expr::Var("a".into())),
                }),
                right: Box::new(Expr::BinOp {
                    op: BinOp::Mul,
                    left: Box::new(Expr::Var("b".into())),
                    right: Box::new(Expr::Var("b".into())),
                }),
            }),
        }],
    };

    let result = run_expr(&expr).unwrap();
    assert_eq!(result, Value::Int(13)); // 2*2 + 3*3 = 4 + 9 = 13
}

#[test]
fn test_match_mixed_types_in_tuple() {
    // match (42, "hello", true) with | (n, s, b) -> n
    let expr = Expr::Match {
        scrutinee: Box::new(Expr::Tuple(vec![
            Expr::Lit(Literal::Int(42)),
            Expr::Lit(Literal::Str("hello".into())),
            Expr::Lit(Literal::Bool(true)),
        ])),
        arms: vec![MatchArm {
            pattern: Pattern::Tuple(vec![
                Pattern::Var("n".into()),
                Pattern::Var("s".into()),
                Pattern::Var("b".into()),
            ]),
            body: Box::new(Expr::Var("n".into())),
        }],
    };

    let result = run_expr(&expr).unwrap();
    assert_eq!(result, Value::Int(42));
}

#[test]
fn test_match_boolean_with_computation() {
    // match true with | true -> 1 + 1 | false -> 2 * 2
    let expr = Expr::Match {
        scrutinee: Box::new(Expr::Lit(Literal::Bool(true))),
        arms: vec![
            MatchArm {
                pattern: Pattern::Literal(Literal::Bool(true)),
                body: Box::new(Expr::BinOp {
                    op: BinOp::Add,
                    left: Box::new(Expr::Lit(Literal::Int(1))),
                    right: Box::new(Expr::Lit(Literal::Int(1))),
                }),
            },
            MatchArm {
                pattern: Pattern::Literal(Literal::Bool(false)),
                body: Box::new(Expr::BinOp {
                    op: BinOp::Mul,
                    left: Box::new(Expr::Lit(Literal::Int(2))),
                    right: Box::new(Expr::Lit(Literal::Int(2))),
                }),
            },
        ],
    };

    let result = run_expr(&expr).unwrap();
    assert_eq!(result, Value::Int(2));
}

#[test]
fn test_match_string_with_nested_match() {
    // match "test" with
    // | "test" -> (match 1 with | 1 -> "ok" | _ -> "fail")
    // | _ -> "no"
    let expr = Expr::Match {
        scrutinee: Box::new(Expr::Lit(Literal::Str("test".into()))),
        arms: vec![
            MatchArm {
                pattern: Pattern::Literal(Literal::Str("test".into())),
                body: Box::new(Expr::Match {
                    scrutinee: Box::new(Expr::Lit(Literal::Int(1))),
                    arms: vec![
                        MatchArm {
                            pattern: Pattern::Literal(Literal::Int(1)),
                            body: Box::new(Expr::Lit(Literal::Str("ok".into()))),
                        },
                        MatchArm {
                            pattern: Pattern::Wildcard,
                            body: Box::new(Expr::Lit(Literal::Str("fail".into()))),
                        },
                    ],
                }),
            },
            MatchArm {
                pattern: Pattern::Wildcard,
                body: Box::new(Expr::Lit(Literal::Str("no".into()))),
            },
        ],
    };

    let result = run_expr(&expr).unwrap();
    assert_eq!(result, Value::Str("ok".into()));
}
