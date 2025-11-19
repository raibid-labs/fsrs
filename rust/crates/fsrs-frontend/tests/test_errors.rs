//! Comprehensive tests for error reporting infrastructure.
//!
//! This test suite validates:
//! - Span creation and manipulation
//! - Error formatting for all error kinds
//! - Source highlighting
//! - Suggestion generation
//! - Context tracking

use fsrs_frontend::error::{TypeError, TypeErrorKind};
use fsrs_frontend::span::{Position, Span};
use fsrs_frontend::types::{Type, TypeVar};
use std::collections::HashMap;

// ========================================================================
// Span Tests
// ========================================================================

#[test]
fn test_span_creation() {
    let start = Position::new(1, 1, 0);
    let end = Position::new(1, 5, 4);
    let span = Span::new(start, end);
    assert_eq!(span.start, start);
    assert_eq!(span.end, end);
}

#[test]
fn test_span_merge_adjacent() {
    let span1 = Span::new(Position::new(1, 1, 0), Position::new(1, 5, 4));
    let span2 = Span::new(Position::new(1, 5, 4), Position::new(1, 10, 9));
    let merged = span1.merge(&span2);

    assert_eq!(merged.start, Position::new(1, 1, 0));
    assert_eq!(merged.end, Position::new(1, 10, 9));
}

#[test]
fn test_span_merge_overlapping() {
    let span1 = Span::new(Position::new(1, 1, 0), Position::new(1, 7, 6));
    let span2 = Span::new(Position::new(1, 3, 2), Position::new(1, 10, 9));
    let merged = span1.merge(&span2);

    assert_eq!(merged.start, Position::new(1, 1, 0));
    assert_eq!(merged.end, Position::new(1, 10, 9));
}

#[test]
fn test_span_format_location() {
    let span = Span::new(Position::new(5, 10, 42), Position::new(5, 15, 47));
    assert_eq!(span.format_location(), "line 5, column 10");
}

#[test]
fn test_span_single_vs_multi_line() {
    let single = Span::new(Position::new(1, 1, 0), Position::new(1, 10, 9));
    let multi = Span::new(Position::new(1, 1, 0), Position::new(3, 5, 25));

    assert!(single.is_single_line());
    assert!(!multi.is_single_line());
}

// ========================================================================
// Type Mismatch Error Tests
// ========================================================================

#[test]
fn test_error_mismatch_int_string() {
    let err = TypeError::new(TypeErrorKind::Mismatch {
        expected: Type::Int,
        got: Type::String,
    });

    let display = format!("{}", err);
    assert!(display.contains("Type mismatch"));
    assert!(display.contains("Expected: int"));
    assert!(display.contains("Got:      string"));
}

#[test]
fn test_error_mismatch_function_types() {
    let err = TypeError::new(TypeErrorKind::Mismatch {
        expected: Type::Function(Box::new(Type::Int), Box::new(Type::Bool)),
        got: Type::Function(Box::new(Type::String), Box::new(Type::Bool)),
    });

    let display = format!("{}", err);
    assert!(display.contains("Type mismatch"));
    assert!(display.contains("int -> bool"));
    assert!(display.contains("string -> bool"));
}

#[test]
fn test_error_mismatch_list_types() {
    let err = TypeError::new(TypeErrorKind::Mismatch {
        expected: Type::List(Box::new(Type::Int)),
        got: Type::List(Box::new(Type::String)),
    });

    let display = format!("{}", err);
    assert!(display.contains("int list"));
    assert!(display.contains("string list"));
}

// ========================================================================
// Occurs Check Error Tests
// ========================================================================

#[test]
fn test_error_occurs_check_simple() {
    let var = TypeVar::new(0, "a");
    let err = TypeError::new(TypeErrorKind::OccursCheck {
        var: var.clone(),
        in_type: Type::List(Box::new(Type::Var(var))),
    });

    let display = format!("{}", err);
    assert!(display.contains("Occurs check failed"));
    assert!(display.contains("infinite type"));
}

#[test]
fn test_error_occurs_check_function() {
    let var = TypeVar::new(1, "b");
    let err = TypeError::new(TypeErrorKind::OccursCheck {
        var: var.clone(),
        in_type: Type::Function(Box::new(Type::Var(var.clone())), Box::new(Type::Var(var))),
    });

    let display = format!("{}", err);
    assert!(display.contains("'b"));
}

// ========================================================================
// Unbound Variable Error Tests
// ========================================================================

#[test]
fn test_error_unbound_variable() {
    let err = TypeError::new(TypeErrorKind::UnboundVariable {
        name: "x".to_string(),
    });

    let display = format!("{}", err);
    assert!(display.contains("Unbound variable: x"));
}

#[test]
fn test_error_unbound_variable_suggestion() {
    let err = TypeError::new(TypeErrorKind::UnboundVariable {
        name: "foo".to_string(),
    });

    let suggestion = err.suggest_fix();
    assert!(suggestion.is_some());
    assert!(suggestion
        .unwrap()
        .contains("Did you forget to define 'foo'?"));
}

// ========================================================================
// Field Error Tests
// ========================================================================

#[test]
fn test_error_field_not_found() {
    let mut fields = HashMap::new();
    fields.insert("x".to_string(), Type::Int);
    fields.insert("y".to_string(), Type::Int);
    let record = Type::Record(fields);

    let err = TypeError::new(TypeErrorKind::FieldNotFound {
        record_type: record,
        field: "z".to_string(),
    });

    let display = format!("{}", err);
    assert!(display.contains("Field 'z' not found"));
}

#[test]
fn test_error_field_not_found_suggestion() {
    let mut fields = HashMap::new();
    fields.insert("name".to_string(), Type::String);
    fields.insert("age".to_string(), Type::Int);
    let record = Type::Record(fields);

    let err = TypeError::new(TypeErrorKind::FieldNotFound {
        record_type: record,
        field: "address".to_string(),
    });

    let suggestion = err.suggest_fix();
    assert!(suggestion.is_some());
    let text = suggestion.unwrap();
    assert!(text.contains("Available fields"));
}

#[test]
fn test_error_duplicate_field() {
    let err = TypeError::new(TypeErrorKind::DuplicateField {
        field: "name".to_string(),
    });

    let display = format!("{}", err);
    assert!(display.contains("Duplicate field"));
    assert!(display.contains("name"));
}

#[test]
fn test_error_missing_field() {
    let mut fields = HashMap::new();
    fields.insert("name".to_string(), Type::String);
    let record = Type::Record(fields);

    let err = TypeError::new(TypeErrorKind::MissingField {
        record_type: record,
        field: "age".to_string(),
    });

    let display = format!("{}", err);
    assert!(display.contains("Missing field 'age'"));
}

// ========================================================================
// Function Error Tests
// ========================================================================

#[test]
fn test_error_not_a_function() {
    let err = TypeError::new(TypeErrorKind::NotAFunction { got: Type::Int });

    let display = format!("{}", err);
    assert!(display.contains("Not a function"));
    assert!(display.contains("int"));
}

#[test]
fn test_error_arity_mismatch_too_few() {
    let err = TypeError::new(TypeErrorKind::ArityMismatch {
        expected: 3,
        got: 2,
    });

    let display = format!("{}", err);
    assert!(display.contains("Arity mismatch"));
    assert!(display.contains("expected 3"));
    assert!(display.contains("got 2"));
}

#[test]
fn test_error_arity_mismatch_too_many() {
    let err = TypeError::new(TypeErrorKind::ArityMismatch {
        expected: 1,
        got: 3,
    });

    let display = format!("{}", err);
    assert!(display.contains("expected 1"));
    assert!(display.contains("got 3"));
}

// ========================================================================
// Collection Error Tests
// ========================================================================

#[test]
fn test_error_not_a_tuple() {
    let err = TypeError::new(TypeErrorKind::NotATuple { got: Type::Int });

    let display = format!("{}", err);
    assert!(display.contains("Not a tuple"));
}

#[test]
fn test_error_tuple_index_out_of_bounds() {
    let tuple = Type::Tuple(vec![Type::Int, Type::Bool, Type::String]);
    let err = TypeError::new(TypeErrorKind::TupleIndexOutOfBounds {
        tuple_type: tuple,
        index: 5,
        size: 3,
    });

    let display = format!("{}", err);
    assert!(display.contains("Tuple index out of bounds"));
    assert!(display.contains("index 5"));
    assert!(display.contains("3 elements"));
}

#[test]
fn test_error_not_a_list() {
    let err = TypeError::new(TypeErrorKind::NotAList { got: Type::Int });

    let display = format!("{}", err);
    assert!(display.contains("Not a list"));
}

#[test]
fn test_error_not_an_array() {
    let err = TypeError::new(TypeErrorKind::NotAnArray { got: Type::String });

    let display = format!("{}", err);
    assert!(display.contains("Not an array"));
}

#[test]
fn test_error_not_a_record() {
    let err = TypeError::new(TypeErrorKind::NotARecord { got: Type::Bool });

    let display = format!("{}", err);
    assert!(display.contains("Not a record"));
}

// ========================================================================
// Pattern Matching Error Tests
// ========================================================================

#[test]
fn test_error_pattern_mismatch() {
    let err = TypeError::new(TypeErrorKind::PatternMismatch {
        pattern_type: Type::Int,
        scrutinee_type: Type::String,
    });

    let display = format!("{}", err);
    assert!(display.contains("Pattern match type mismatch"));
    assert!(display.contains("Pattern type:   int"));
    assert!(display.contains("Scrutinee type: string"));
}

// ========================================================================
// Source Highlighting Tests
// ========================================================================

#[test]
fn test_format_with_source_simple() {
    let source = "let x = 42";
    let span = Span::new(Position::new(1, 9, 8), Position::new(1, 11, 10));
    let err = TypeError::with_span(
        TypeErrorKind::Mismatch {
            expected: Type::String,
            got: Type::Int,
        },
        span,
    );

    let formatted = err.format(source);
    assert!(formatted.contains("Error:"));
    assert!(formatted.contains("line 1, column 9"));
    assert!(formatted.contains("let x = 42"));
    assert!(formatted.contains("^^"));
}

#[test]
fn test_format_with_source_multiline() {
    let source = "let x = 42\nlet y = x + 1\nlet z = y * 2";
    let span = Span::new(Position::new(2, 9, 19), Position::new(2, 10, 20));
    let err = TypeError::with_span(
        TypeErrorKind::UnboundVariable {
            name: "x".to_string(),
        },
        span,
    );

    let formatted = err.format(source);
    assert!(formatted.contains("line 2, column 9"));
    assert!(formatted.contains("let y = x + 1"));
}

#[test]
fn test_format_with_context_stack() {
    let source = "f(g(h(x)))";
    let span = Span::new(Position::new(1, 7, 6), Position::new(1, 8, 7));
    let err = TypeError::with_span(
        TypeErrorKind::UnboundVariable {
            name: "x".to_string(),
        },
        span,
    )
    .with_context("function h".to_string())
    .with_context("function g".to_string())
    .with_context("function f".to_string());

    let formatted = err.format(source);
    assert!(formatted.contains("in function h"));
    assert!(formatted.contains("in function g"));
    assert!(formatted.contains("in function f"));
}

// ========================================================================
// Suggestion Tests
// ========================================================================

#[test]
fn test_suggestion_int_to_string() {
    let err = TypeError::new(TypeErrorKind::Mismatch {
        expected: Type::Int,
        got: Type::String,
    });

    let suggestion = err.suggest_fix();
    assert!(suggestion.is_some());
    assert!(suggestion.unwrap().contains("conversion"));
}

#[test]
fn test_suggestion_list_to_array() {
    let err = TypeError::new(TypeErrorKind::Mismatch {
        expected: Type::List(Box::new(Type::Int)),
        got: Type::Array(Box::new(Type::Int)),
    });

    let suggestion = err.suggest_fix();
    assert!(suggestion.is_some());
    assert!(suggestion.unwrap().contains("conversion"));
}

#[test]
fn test_suggestion_function_arity() {
    let err = TypeError::new(TypeErrorKind::ArityMismatch {
        expected: 2,
        got: 1,
    });

    let suggestion = err.suggest_fix();
    assert!(suggestion.is_some());
    let text = suggestion.unwrap();
    assert!(text.contains("You provided 1 arguments but 2 are required"));
}

#[test]
fn test_suggestion_tuple_index() {
    let tuple = Type::Tuple(vec![Type::Int, Type::Bool]);
    let err = TypeError::new(TypeErrorKind::TupleIndexOutOfBounds {
        tuple_type: tuple,
        index: 3,
        size: 2,
    });

    let suggestion = err.suggest_fix();
    assert!(suggestion.is_some());
    let text = suggestion.unwrap();
    assert!(text.contains("valid indices: 0-1"));
}

// ========================================================================
// Complete Error Formatting Tests
// ========================================================================

#[test]
fn test_complete_error_message_with_everything() {
    let source = "let add x y = x + y\nlet result = add 42";
    let span = Span::new(Position::new(2, 14, 33), Position::new(2, 20, 39));
    let err = TypeError::with_span(
        TypeErrorKind::ArityMismatch {
            expected: 2,
            got: 1,
        },
        span,
    )
    .with_context("function application".to_string());

    let formatted = err.format(source);

    // Check all components are present
    assert!(formatted.contains("Error: Arity mismatch"));
    assert!(formatted.contains("line 2, column 14"));
    assert!(formatted.contains("let result = add 42"));
    assert!(formatted.contains("in function application"));
    assert!(formatted.contains("Help:"));
}

#[test]
fn test_error_without_span() {
    let err = TypeError::new(TypeErrorKind::Custom {
        message: "Internal compiler error".to_string(),
    });

    let formatted = err.format("");
    assert!(formatted.contains("Internal compiler error"));
    assert!(!formatted.contains("-->"));
}

#[test]
fn test_error_builder_pattern() {
    let err = TypeError::new(TypeErrorKind::UnboundVariable {
        name: "x".to_string(),
    })
    .with_context("let binding".to_string())
    .with_context("function body".to_string());

    assert_eq!(err.context.len(), 2);
    assert_eq!(err.context[0], "let binding");
    assert_eq!(err.context[1], "function body");
}

// ========================================================================
// Edge Cases
// ========================================================================

#[test]
fn test_span_at_end_of_line() {
    let source = "let x = 42";
    let span = Span::new(Position::new(1, 10, 9), Position::new(1, 11, 10));
    let err = TypeError::with_span(
        TypeErrorKind::Mismatch {
            expected: Type::String,
            got: Type::Int,
        },
        span,
    );

    let formatted = err.format(source);
    assert!(formatted.contains("let x = 42"));
}

#[test]
fn test_empty_source() {
    let err = TypeError::new(TypeErrorKind::Custom {
        message: "Error".to_string(),
    });

    let formatted = err.format("");
    assert!(formatted.contains("Error:"));
}

#[test]
fn test_span_beyond_source_length() {
    let source = "let x = 42";
    let span = Span::new(Position::new(5, 1, 100), Position::new(5, 10, 109));
    let err = TypeError::with_span(
        TypeErrorKind::Custom {
            message: "Error".to_string(),
        },
        span,
    );

    let formatted = err.format(source);
    // Should not crash, even with invalid span
    assert!(formatted.contains("Error:"));
}

#[test]
fn test_all_error_kinds_display() {
    // Ensure all error kinds can be displayed without panicking
    let errors = vec![
        TypeErrorKind::Mismatch {
            expected: Type::Int,
            got: Type::Bool,
        },
        TypeErrorKind::OccursCheck {
            var: TypeVar::new(0, "a"),
            in_type: Type::Int,
        },
        TypeErrorKind::UnboundVariable {
            name: "x".to_string(),
        },
        TypeErrorKind::FieldNotFound {
            record_type: Type::Record(HashMap::new()),
            field: "x".to_string(),
        },
        TypeErrorKind::ArityMismatch {
            expected: 1,
            got: 2,
        },
        TypeErrorKind::NotAFunction { got: Type::Int },
        TypeErrorKind::PatternMismatch {
            pattern_type: Type::Int,
            scrutinee_type: Type::Bool,
        },
        TypeErrorKind::NotATuple { got: Type::Int },
        TypeErrorKind::TupleIndexOutOfBounds {
            tuple_type: Type::Tuple(vec![]),
            index: 0,
            size: 0,
        },
        TypeErrorKind::NotAList { got: Type::Int },
        TypeErrorKind::NotAnArray { got: Type::Int },
        TypeErrorKind::NotARecord { got: Type::Int },
        TypeErrorKind::DuplicateField {
            field: "x".to_string(),
        },
        TypeErrorKind::MissingField {
            record_type: Type::Record(HashMap::new()),
            field: "x".to_string(),
        },
        TypeErrorKind::Custom {
            message: "test".to_string(),
        },
    ];

    for kind in errors {
        let err = TypeError::new(kind);
        let _ = format!("{}", err);
    }
}
