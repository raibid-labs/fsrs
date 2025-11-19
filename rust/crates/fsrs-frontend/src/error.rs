//! Error reporting for type checking and compilation.
//!
//! This module provides comprehensive error types with beautiful formatting,
//! source location tracking, and helpful suggestions for fixing common mistakes.
//!
//! # Example
//!
//! ```rust
//! use fsrs_frontend::error::{TypeError, TypeErrorKind};
//! use fsrs_frontend::types::Type;
//! use fsrs_frontend::span::{Span, Position};
//!
//! let err = TypeError::with_span(
//!     TypeErrorKind::Mismatch {
//!         expected: Type::Int,
//!         got: Type::String,
//!     },
//!     Span::new(Position::new(1, 5, 4), Position::new(1, 10, 9))
//! );
//!
//! let source = "let x = \"hello\"";
//! println!("{}", err.format(source));
//! ```

use crate::span::Span;
use crate::types::{Type, TypeVar};
use std::fmt;

/// Type error with context and location information.
#[derive(Debug, Clone)]
pub struct TypeError {
    /// The kind of type error
    pub kind: TypeErrorKind,
    /// Source location where the error occurred
    pub span: Option<Span>,
    /// Context stack for nested errors
    pub context: Vec<String>,
}

/// Different kinds of type errors.
#[derive(Debug, Clone)]
pub enum TypeErrorKind {
    /// Type mismatch between expected and actual types
    Mismatch {
        /// The expected type
        expected: Type,
        /// The actual type found
        got: Type,
    },

    /// Occurs check failed (infinite type)
    OccursCheck {
        /// The type variable
        var: TypeVar,
        /// The type it appears in
        in_type: Type,
    },

    /// Unbound variable reference
    UnboundVariable {
        /// Name of the unbound variable
        name: String,
    },

    /// Record field not found
    FieldNotFound {
        /// The record type
        record_type: Type,
        /// The field name that wasn't found
        field: String,
    },

    /// Function arity mismatch
    ArityMismatch {
        /// Expected number of arguments
        expected: usize,
        /// Actual number of arguments provided
        got: usize,
    },

    /// Attempted to call a non-function
    NotAFunction {
        /// The type that was called
        got: Type,
    },

    /// Pattern match type mismatch
    PatternMismatch {
        /// Type of the pattern
        pattern_type: Type,
        /// Type of the scrutinee
        scrutinee_type: Type,
    },

    /// Not a tuple (but tuple access was attempted)
    NotATuple {
        /// The type that was accessed
        got: Type,
    },

    /// Tuple index out of bounds
    TupleIndexOutOfBounds {
        /// The tuple type
        tuple_type: Type,
        /// The index that was accessed
        index: usize,
        /// The actual tuple size
        size: usize,
    },

    /// Not a list (but list operation was attempted)
    NotAList {
        /// The type that was used
        got: Type,
    },

    /// Not an array (but array operation was attempted)
    NotAnArray {
        /// The type that was used
        got: Type,
    },

    /// Not a record (but record operation was attempted)
    NotARecord {
        /// The type that was used
        got: Type,
    },

    /// Duplicate field in record
    DuplicateField {
        /// The field name
        field: String,
    },

    /// Missing field in record
    MissingField {
        /// The record type
        record_type: Type,
        /// The missing field name
        field: String,
    },

    /// Generic error with custom message
    Custom {
        /// Error message
        message: String,
    },
}

impl TypeError {
    /// Create a new type error without location information.
    pub fn new(kind: TypeErrorKind) -> Self {
        TypeError {
            kind,
            span: None,
            context: Vec::new(),
        }
    }

    /// Create a new type error with span information.
    pub fn with_span(kind: TypeErrorKind, span: Span) -> Self {
        TypeError {
            kind,
            span: Some(span),
            context: Vec::new(),
        }
    }

    /// Add context to the error (e.g., "in function application", "in let binding").
    pub fn add_context(&mut self, ctx: String) {
        self.context.push(ctx);
    }

    /// Add context and return self (builder pattern).
    pub fn with_context(mut self, ctx: String) -> Self {
        self.add_context(ctx);
        self
    }

    /// Format the error with source code highlighting.
    ///
    /// Produces a beautiful error message with:
    /// - Error description
    /// - Source location
    /// - Source code excerpt with highlighting
    /// - Context stack
    /// - Helpful suggestions
    pub fn format(&self, source: &str) -> String {
        let mut output = String::new();

        // Error header
        output.push_str(&format!("Error: {}\n", self));

        // Location if available
        if let Some(span) = &self.span {
            output.push_str(&format!("  --> {}\n", span.format_location()));

            // Show source line with highlight
            if let Some(highlight) = self.format_source_highlight(source, span) {
                output.push_str(&highlight);
            }
        }

        // Context stack
        if !self.context.is_empty() {
            output.push('\n');
            for ctx in &self.context {
                output.push_str(&format!("  in {}\n", ctx));
            }
        }

        // Suggestions (if applicable)
        if let Some(suggestion) = self.suggest_fix() {
            output.push_str(&format!("\n  Help: {}\n", suggestion));
        }

        output
    }

    /// Format source code with error highlighting.
    fn format_source_highlight(&self, source: &str, span: &Span) -> Option<String> {
        let lines: Vec<&str> = source.lines().collect();
        if span.start.line == 0 || span.start.line > lines.len() {
            return None;
        }

        let line_idx = span.start.line - 1;
        let line = lines[line_idx];

        let mut output = String::new();
        output.push_str("   |\n");
        output.push_str(&format!("{:3} | {}\n", span.start.line, line));
        output.push_str("   | ");

        // Highlight the error span
        let start_col = span.start.column.saturating_sub(1);
        let end_col = if span.is_single_line() {
            span.end.column.saturating_sub(1)
        } else {
            line.len()
        };

        for i in 0..line.len() {
            if i >= start_col && i < end_col {
                output.push('^');
            } else {
                output.push(' ');
            }
        }
        output.push('\n');

        Some(output)
    }

    /// Suggest a fix for common error patterns.
    pub fn suggest_fix(&self) -> Option<String> {
        match &self.kind {
            TypeErrorKind::UnboundVariable { name } => {
                Some(format!("Did you forget to define '{}'?", name))
            }
            TypeErrorKind::Mismatch { expected, got } => {
                // Type-specific suggestions
                match (expected, got) {
                    (Type::Int, Type::String) | (Type::Int, Type::Float) => {
                        Some("Try using a numeric conversion function".to_string())
                    }
                    (Type::String, Type::Int) | (Type::String, Type::Float) => {
                        Some("Try using string conversion or formatting".to_string())
                    }
                    (Type::List(_), Type::Array(_)) | (Type::Array(_), Type::List(_)) => {
                        Some("Lists and arrays are different types - use appropriate conversion functions".to_string())
                    }
                    (Type::Function(_, _), _) => {
                        Some("Did you forget to apply all function arguments?".to_string())
                    }
                    (_, Type::Function(_, _)) => {
                        Some("Did you provide too many arguments?".to_string())
                    }
                    _ => None,
                }
            }
            TypeErrorKind::NotAFunction { got } => match got {
                Type::Int | Type::Bool | Type::String | Type::Unit | Type::Float => {
                    Some("This is a value, not a function - did you mean to call a function instead?".to_string())
                }
                _ => Some("This expression is not a function and cannot be called".to_string()),
            },
            TypeErrorKind::FieldNotFound { record_type, field: _ } => {
                if let Type::Record(fields) = record_type {
                    let available: Vec<_> = fields.keys().map(|s| s.as_str()).collect();
                    Some(format!(
                        "Available fields are: {}. Did you mean one of these?",
                        available.join(", ")
                    ))
                } else {
                    None
                }
            }
            TypeErrorKind::ArityMismatch { expected, got } => {
                if got < expected {
                    Some(format!("You provided {} arguments but {} are required", got, expected))
                } else {
                    Some(format!("You provided {} arguments but only {} are expected", got, expected))
                }
            }
            TypeErrorKind::TupleIndexOutOfBounds { size, index, .. } => {
                Some(format!(
                    "Tuple has {} elements (valid indices: 0-{}), but you tried to access index {}",
                    size,
                    size - 1,
                    index
                ))
            }
            TypeErrorKind::NotATuple { .. } => {
                Some("Use tuple syntax (x, y, z) to create tuples".to_string())
            }
            TypeErrorKind::NotAList { .. } => {
                Some("Use list syntax [x; y; z] to create lists".to_string())
            }
            TypeErrorKind::NotAnArray { .. } => {
                Some("Use array syntax [|x; y; z|] to create arrays".to_string())
            }
            TypeErrorKind::NotARecord { .. } => {
                Some("Use record syntax { field = value } to create records".to_string())
            }
            TypeErrorKind::DuplicateField { field } => {
                Some(format!("Remove the duplicate definition of field '{}'", field))
            }
            TypeErrorKind::MissingField { field, .. } => {
                Some(format!("Add the required field '{}' to the record", field))
            }
            TypeErrorKind::OccursCheck { var, in_type } => {
                Some(format!(
                    "This would create an infinite type {} = {}. Check your recursive type definitions.",
                    var, in_type
                ))
            }
            TypeErrorKind::PatternMismatch { .. } => {
                Some("Pattern type must match the type of the value being matched".to_string())
            }
            TypeErrorKind::Custom { .. } => None,
        }
    }
}

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            TypeErrorKind::Mismatch { expected, got } => {
                writeln!(f, "Type mismatch")?;
                writeln!(f, "  Expected: {}", expected)?;
                write!(f, "  Got:      {}", got)
            }
            TypeErrorKind::OccursCheck { var, in_type } => {
                writeln!(f, "Occurs check failed")?;
                write!(f, "  Cannot construct infinite type {} = {}", var, in_type)
            }
            TypeErrorKind::UnboundVariable { name } => {
                write!(f, "Unbound variable: {}", name)
            }
            TypeErrorKind::FieldNotFound { record_type, field } => {
                write!(
                    f,
                    "Field '{}' not found in record type {}",
                    field, record_type
                )
            }
            TypeErrorKind::ArityMismatch { expected, got } => {
                write!(
                    f,
                    "Arity mismatch: expected {} arguments, got {}",
                    expected, got
                )
            }
            TypeErrorKind::NotAFunction { got } => {
                write!(f, "Not a function: cannot call value of type {}", got)
            }
            TypeErrorKind::PatternMismatch {
                pattern_type,
                scrutinee_type,
            } => {
                writeln!(f, "Pattern match type mismatch")?;
                writeln!(f, "  Pattern type:   {}", pattern_type)?;
                write!(f, "  Scrutinee type: {}", scrutinee_type)
            }
            TypeErrorKind::NotATuple { got } => {
                write!(
                    f,
                    "Not a tuple: cannot access tuple element of type {}",
                    got
                )
            }
            TypeErrorKind::TupleIndexOutOfBounds {
                tuple_type,
                index,
                size,
            } => {
                write!(
                    f,
                    "Tuple index out of bounds: type {} has {} elements, but index {} was accessed",
                    tuple_type, size, index
                )
            }
            TypeErrorKind::NotAList { got } => {
                write!(
                    f,
                    "Not a list: cannot perform list operation on type {}",
                    got
                )
            }
            TypeErrorKind::NotAnArray { got } => {
                write!(
                    f,
                    "Not an array: cannot perform array operation on type {}",
                    got
                )
            }
            TypeErrorKind::NotARecord { got } => {
                write!(f, "Not a record: cannot access field of type {}", got)
            }
            TypeErrorKind::DuplicateField { field } => {
                write!(f, "Duplicate field in record: '{}'", field)
            }
            TypeErrorKind::MissingField { record_type, field } => {
                write!(
                    f,
                    "Missing field '{}' in record type {}",
                    field, record_type
                )
            }
            TypeErrorKind::Custom { message } => {
                write!(f, "{}", message)
            }
        }
    }
}

impl std::error::Error for TypeError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::span::Position;
    use crate::types::TypeVar;

    // ========================================================================
    // TypeError Creation Tests
    // ========================================================================

    #[test]
    fn test_type_error_new() {
        let err = TypeError::new(TypeErrorKind::UnboundVariable {
            name: "x".to_string(),
        });
        assert!(err.span.is_none());
        assert!(err.context.is_empty());
    }

    #[test]
    fn test_type_error_with_span() {
        let span = Span::new(Position::new(1, 1, 0), Position::new(1, 5, 4));
        let err = TypeError::with_span(
            TypeErrorKind::UnboundVariable {
                name: "x".to_string(),
            },
            span,
        );
        assert_eq!(err.span, Some(span));
    }

    #[test]
    fn test_type_error_add_context() {
        let mut err = TypeError::new(TypeErrorKind::UnboundVariable {
            name: "x".to_string(),
        });
        err.add_context("function application".to_string());
        err.add_context("let binding".to_string());
        assert_eq!(err.context.len(), 2);
        assert_eq!(err.context[0], "function application");
        assert_eq!(err.context[1], "let binding");
    }

    #[test]
    fn test_type_error_with_context_builder() {
        let err = TypeError::new(TypeErrorKind::UnboundVariable {
            name: "x".to_string(),
        })
        .with_context("function application".to_string());
        assert_eq!(err.context.len(), 1);
    }

    // ========================================================================
    // TypeErrorKind Display Tests
    // ========================================================================

    #[test]
    fn test_display_type_mismatch() {
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
    fn test_display_occurs_check() {
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
    fn test_display_unbound_variable() {
        let err = TypeError::new(TypeErrorKind::UnboundVariable {
            name: "foo".to_string(),
        });
        let display = format!("{}", err);
        assert!(display.contains("Unbound variable: foo"));
    }

    #[test]
    fn test_display_field_not_found() {
        use std::collections::HashMap;
        let mut fields = HashMap::new();
        fields.insert("x".to_string(), Type::Int);
        let record = Type::Record(fields);

        let err = TypeError::new(TypeErrorKind::FieldNotFound {
            record_type: record,
            field: "y".to_string(),
        });
        let display = format!("{}", err);
        assert!(display.contains("Field 'y' not found"));
    }

    #[test]
    fn test_display_arity_mismatch() {
        let err = TypeError::new(TypeErrorKind::ArityMismatch {
            expected: 2,
            got: 3,
        });
        let display = format!("{}", err);
        assert!(display.contains("Arity mismatch"));
        assert!(display.contains("expected 2"));
        assert!(display.contains("got 3"));
    }

    #[test]
    fn test_display_not_a_function() {
        let err = TypeError::new(TypeErrorKind::NotAFunction { got: Type::Int });
        let display = format!("{}", err);
        assert!(display.contains("Not a function"));
        assert!(display.contains("int"));
    }

    #[test]
    fn test_display_pattern_mismatch() {
        let err = TypeError::new(TypeErrorKind::PatternMismatch {
            pattern_type: Type::Int,
            scrutinee_type: Type::String,
        });
        let display = format!("{}", err);
        assert!(display.contains("Pattern match type mismatch"));
    }

    #[test]
    fn test_display_not_a_tuple() {
        let err = TypeError::new(TypeErrorKind::NotATuple { got: Type::Int });
        let display = format!("{}", err);
        assert!(display.contains("Not a tuple"));
    }

    #[test]
    fn test_display_tuple_index_out_of_bounds() {
        let tuple = Type::Tuple(vec![Type::Int, Type::Bool]);
        let err = TypeError::new(TypeErrorKind::TupleIndexOutOfBounds {
            tuple_type: tuple,
            index: 3,
            size: 2,
        });
        let display = format!("{}", err);
        assert!(display.contains("Tuple index out of bounds"));
        assert!(display.contains("index 3"));
    }

    #[test]
    fn test_display_not_a_list() {
        let err = TypeError::new(TypeErrorKind::NotAList { got: Type::Int });
        let display = format!("{}", err);
        assert!(display.contains("Not a list"));
    }

    #[test]
    fn test_display_not_an_array() {
        let err = TypeError::new(TypeErrorKind::NotAnArray { got: Type::Int });
        let display = format!("{}", err);
        assert!(display.contains("Not an array"));
    }

    #[test]
    fn test_display_duplicate_field() {
        let err = TypeError::new(TypeErrorKind::DuplicateField {
            field: "name".to_string(),
        });
        let display = format!("{}", err);
        assert!(display.contains("Duplicate field"));
        assert!(display.contains("name"));
    }

    #[test]
    fn test_display_custom() {
        let err = TypeError::new(TypeErrorKind::Custom {
            message: "Something went wrong".to_string(),
        });
        let display = format!("{}", err);
        assert_eq!(display, "Something went wrong");
    }

    // ========================================================================
    // Suggestion Tests
    // ========================================================================

    #[test]
    fn test_suggestion_unbound_variable() {
        let err = TypeError::new(TypeErrorKind::UnboundVariable {
            name: "foo".to_string(),
        });
        let suggestion = err.suggest_fix();
        assert!(suggestion.is_some());
        assert!(suggestion.unwrap().contains("Did you forget to define"));
    }

    #[test]
    fn test_suggestion_int_to_string_mismatch() {
        let err = TypeError::new(TypeErrorKind::Mismatch {
            expected: Type::Int,
            got: Type::String,
        });
        let suggestion = err.suggest_fix();
        assert!(suggestion.is_some());
        assert!(suggestion.unwrap().contains("conversion"));
    }

    #[test]
    fn test_suggestion_not_a_function() {
        let err = TypeError::new(TypeErrorKind::NotAFunction { got: Type::Int });
        let suggestion = err.suggest_fix();
        assert!(suggestion.is_some());
    }

    // ========================================================================
    // Source Formatting Tests
    // ========================================================================

    #[test]
    fn test_format_with_source_simple() {
        let source = "let x = 42";
        let span = Span::new(Position::new(1, 5, 4), Position::new(1, 6, 5));
        let err = TypeError::with_span(
            TypeErrorKind::UnboundVariable {
                name: "x".to_string(),
            },
            span,
        );
        let formatted = err.format(source);
        assert!(formatted.contains("Error:"));
        assert!(formatted.contains("line 1, column 5"));
        assert!(formatted.contains("let x = 42"));
    }

    #[test]
    fn test_format_with_context() {
        let source = "let x = 42";
        let span = Span::new(Position::new(1, 1, 0), Position::new(1, 3, 2));
        let err = TypeError::with_span(
            TypeErrorKind::Mismatch {
                expected: Type::Int,
                got: Type::String,
            },
            span,
        )
        .with_context("function application".to_string());

        let formatted = err.format(source);
        assert!(formatted.contains("in function application"));
    }

    #[test]
    fn test_format_with_suggestion() {
        let source = "let x = y";
        let span = Span::new(Position::new(1, 9, 8), Position::new(1, 10, 9));
        let err = TypeError::with_span(
            TypeErrorKind::UnboundVariable {
                name: "y".to_string(),
            },
            span,
        );
        let formatted = err.format(source);
        assert!(formatted.contains("Help:"));
        assert!(formatted.contains("Did you forget to define"));
    }

    #[test]
    fn test_format_source_highlight() {
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
        assert!(formatted.contains("let x = 42"));
        assert!(formatted.contains("^^"));
    }
}
