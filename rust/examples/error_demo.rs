//! Beautiful Error Message Demonstration
//!
//! This example demonstrates the comprehensive error reporting infrastructure
//! built for FSRS type checking with:
//! - Source location tracking
//! - Source code highlighting
//! - Context tracking
//! - Helpful suggestions

use fsrs_frontend::error::{TypeError, TypeErrorKind};
use fsrs_frontend::span::{Position, Span};
use fsrs_frontend::types::{Type, TypeVar};
use std::collections::HashMap;

fn main() {
    println!("=== FSRS Error Reporting Demo ===\n");

    // Example 1: Type Mismatch with Source Highlighting
    println!("Example 1: Type Mismatch\n");
    let source1 = "let x = \"hello\"";
    let span1 = Span::new(Position::new(1, 9, 8), Position::new(1, 16, 15));
    let err1 = TypeError::with_span(
        TypeErrorKind::Mismatch {
            expected: Type::Int,
            got: Type::String,
        },
        span1,
    )
    .with_context("variable binding".to_string());

    println!("{}", err1.format(source1));
    println!("{}", "=".repeat(70));
    println!();

    // Example 2: Unbound Variable with Suggestion
    println!("Example 2: Unbound Variable\n");
    let source2 = "let result = add x y";
    let span2 = Span::new(Position::new(1, 14, 13), Position::new(1, 17, 16));
    let err2 = TypeError::with_span(
        TypeErrorKind::UnboundVariable {
            name: "add".to_string(),
        },
        span2,
    )
    .with_context("function application".to_string());

    println!("{}", err2.format(source2));
    println!("{}", "=".repeat(70));
    println!();

    // Example 3: Field Not Found in Record
    println!("Example 3: Field Not Found\n");
    let source3 = "person.address";
    let mut fields = HashMap::new();
    fields.insert("name".to_string(), Type::String);
    fields.insert("age".to_string(), Type::Int);
    let record = Type::Record(fields);

    let span3 = Span::new(Position::new(1, 8, 7), Position::new(1, 15, 14));
    let err3 = TypeError::with_span(
        TypeErrorKind::FieldNotFound {
            record_type: record,
            field: "address".to_string(),
        },
        span3,
    );

    println!("{}", err3.format(source3));
    println!("{}", "=".repeat(70));
    println!();

    // Example 4: Occurs Check (Infinite Type)
    println!("Example 4: Occurs Check\n");
    let source4 = "let rec infinite = fun x -> infinite x";
    let var = TypeVar::new(0, "a");
    let span4 = Span::new(Position::new(1, 24, 23), Position::new(1, 39, 38));
    let err4 = TypeError::with_span(
        TypeErrorKind::OccursCheck {
            var: var.clone(),
            in_type: Type::Function(
                Box::new(Type::Var(var.clone())),
                Box::new(Type::Var(var)),
            ),
        },
        span4,
    );

    println!("{}", err4.format(source4));
    println!("{}", "=".repeat(70));
    println!();

    // Example 5: Arity Mismatch
    println!("Example 5: Function Arity Mismatch\n");
    let source5 = "let add x y = x + y\nlet result = add 42";
    let span5 = Span::new(Position::new(2, 14, 33), Position::new(2, 20, 39));
    let err5 = TypeError::with_span(
        TypeErrorKind::ArityMismatch {
            expected: 2,
            got: 1,
        },
        span5,
    )
    .with_context("function application".to_string())
    .with_context("in let binding".to_string());

    println!("{}", err5.format(source5));
    println!("{}", "=".repeat(70));
    println!();

    // Example 6: Tuple Index Out of Bounds
    println!("Example 6: Tuple Index Out of Bounds\n");
    let source6 = "let pair = (1, 2)\nlet x = pair.5";
    let tuple = Type::Tuple(vec![Type::Int, Type::Int]);
    let span6 = Span::new(Position::new(2, 13, 30), Position::new(2, 15, 32));
    let err6 = TypeError::with_span(
        TypeErrorKind::TupleIndexOutOfBounds {
            tuple_type: tuple,
            index: 5,
            size: 2,
        },
        span6,
    );

    println!("{}", err6.format(source6));
    println!("{}", "=".repeat(70));
    println!();

    // Example 7: Not a Function
    println!("Example 7: Not a Function\n");
    let source7 = "let x = 42 in x()";
    let span7 = Span::new(Position::new(1, 15, 14), Position::new(1, 18, 17));
    let err7 = TypeError::with_span(
        TypeErrorKind::NotAFunction { got: Type::Int },
        span7,
    );

    println!("{}", err7.format(source7));
    println!("{}", "=".repeat(70));
    println!();

    // Example 8: Pattern Mismatch
    println!("Example 8: Pattern Match Type Mismatch\n");
    let source8 = "match x with\n| 0 -> \"zero\"\n| _ -> \"many\"";
    let span8 = Span::new(Position::new(2, 3, 15), Position::new(2, 4, 16));
    let err8 = TypeError::with_span(
        TypeErrorKind::PatternMismatch {
            pattern_type: Type::Int,
            scrutinee_type: Type::String,
        },
        span8,
    );

    println!("{}", err8.format(source8));
    println!("{}", "=".repeat(70));
    println!();

    println!("All examples demonstrate beautiful, helpful error messages!");
    println!("The error reporting infrastructure includes:");
    println!("  - Source location tracking (Span)");
    println!("  - Source code highlighting");
    println!("  - Context stack for nested expressions");
    println!("  - Intelligent suggestions for common mistakes");
}
