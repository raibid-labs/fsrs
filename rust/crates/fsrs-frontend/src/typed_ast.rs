//! Typed AST - Optional type-annotated AST representation
//!
//! This module provides typed versions of AST nodes that carry type information
//! alongside the expression structure. This is useful for:
//! - Type-directed code generation
//! - Type-aware optimizations
//! - Debugging and diagnostics
//!
//! The typed AST is optional - the compiler can work with or without type information.

use crate::ast::{Expr, Pattern};
use crate::types::Type;

/// Span information for source code locations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// Start offset in source
    pub start: usize,
    /// End offset in source
    pub end: usize,
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
}

impl Span {
    /// Create a new span
    pub fn new(start: usize, end: usize, line: usize, column: usize) -> Self {
        Span {
            start,
            end,
            line,
            column,
        }
    }

    /// Create a span from just offsets
    pub fn from_offsets(start: usize, end: usize) -> Self {
        Span {
            start,
            end,
            line: 1,
            column: 1,
        }
    }

    /// Combine two spans into one spanning both
    pub fn combine(&self, other: &Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
            line: self.line.min(other.line),
            column: self.column.min(other.column),
        }
    }
}

/// Expression with type annotation
///
/// Wraps an expression with its inferred/checked type and optional source location.
#[derive(Debug, Clone)]
pub struct TypedExpr {
    /// The underlying expression
    pub expr: Expr,
    /// The type of this expression
    pub ty: Type,
    /// Optional source span
    pub span: Option<Span>,
}

impl TypedExpr {
    /// Create a new typed expression without span information
    pub fn new(expr: Expr, ty: Type) -> Self {
        TypedExpr {
            expr,
            ty,
            span: None,
        }
    }

    /// Create a new typed expression with span information
    pub fn with_span(expr: Expr, ty: Type, span: Span) -> Self {
        TypedExpr {
            expr,
            ty,
            span: Some(span),
        }
    }

    /// Get the type of this expression
    pub fn get_type(&self) -> &Type {
        &self.ty
    }

    /// Get the underlying expression
    pub fn get_expr(&self) -> &Expr {
        &self.expr
    }

    /// Get the span if available
    pub fn get_span(&self) -> Option<&Span> {
        self.span.as_ref()
    }

    /// Convert to untyped expression
    pub fn into_expr(self) -> Expr {
        self.expr
    }
}

/// Pattern with type annotation
///
/// Wraps a pattern with its inferred/checked type.
#[derive(Debug, Clone)]
pub struct TypedPattern {
    /// The underlying pattern
    pub pattern: Pattern,
    /// The type of this pattern
    pub ty: Type,
}

impl TypedPattern {
    /// Create a new typed pattern
    pub fn new(pattern: Pattern, ty: Type) -> Self {
        TypedPattern { pattern, ty }
    }

    /// Get the type of this pattern
    pub fn get_type(&self) -> &Type {
        &self.ty
    }

    /// Get the underlying pattern
    pub fn get_pattern(&self) -> &Pattern {
        &self.pattern
    }

    /// Convert to untyped pattern
    pub fn into_pattern(self) -> Pattern {
        self.pattern
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Literal;

    #[test]
    fn test_span_new() {
        let span = Span::new(0, 10, 1, 1);
        assert_eq!(span.start, 0);
        assert_eq!(span.end, 10);
        assert_eq!(span.line, 1);
        assert_eq!(span.column, 1);
    }

    #[test]
    fn test_span_from_offsets() {
        let span = Span::from_offsets(5, 15);
        assert_eq!(span.start, 5);
        assert_eq!(span.end, 15);
    }

    #[test]
    fn test_span_combine() {
        let span1 = Span::new(0, 5, 1, 1);
        let span2 = Span::new(10, 15, 1, 11);
        let combined = span1.combine(&span2);
        assert_eq!(combined.start, 0);
        assert_eq!(combined.end, 15);
    }

    #[test]
    fn test_typed_expr_new() {
        let expr = Expr::Lit(Literal::Int(42));
        let ty = Type::Int;
        let typed = TypedExpr::new(expr.clone(), ty.clone());
        assert_eq!(typed.get_type(), &ty);
        assert_eq!(typed.get_expr(), &expr);
        assert!(typed.get_span().is_none());
    }

    #[test]
    fn test_typed_expr_with_span() {
        let expr = Expr::Lit(Literal::Bool(true));
        let ty = Type::Bool;
        let span = Span::new(0, 4, 1, 1);
        let typed = TypedExpr::with_span(expr, ty.clone(), span);
        assert_eq!(typed.get_type(), &ty);
        assert!(typed.get_span().is_some());
    }

    #[test]
    fn test_typed_expr_into_expr() {
        let expr = Expr::Lit(Literal::Int(42));
        let ty = Type::Int;
        let typed = TypedExpr::new(expr.clone(), ty);
        let untyped = typed.into_expr();
        assert_eq!(untyped, expr);
    }

    #[test]
    fn test_typed_pattern_new() {
        let pattern = Pattern::Wildcard;
        let ty = Type::Int;
        let typed = TypedPattern::new(pattern.clone(), ty.clone());
        assert_eq!(typed.get_type(), &ty);
        assert_eq!(typed.get_pattern(), &pattern);
    }

    #[test]
    fn test_typed_pattern_into_pattern() {
        let pattern = Pattern::Var("x".to_string());
        let ty = Type::String;
        let typed = TypedPattern::new(pattern.clone(), ty);
        let untyped = typed.into_pattern();
        assert_eq!(untyped, pattern);
    }
}
