//! Source location tracking for error reporting.
//!
//! This module provides types for tracking source locations (spans and positions)
//! throughout the parsing and type checking process. These are used to provide
//! accurate error messages with source context.
//!
//! # Example
//!
//! ```rust
//! use fsrs_frontend::span::{Position, Span};
//!
//! let start = Position::new(1, 5, 4);
//! let end = Position::new(1, 10, 9);
//! let span = Span::new(start, end);
//!
//! assert_eq!(span.format_location(), "line 1, column 5");
//! ```

use std::fmt;

/// A position in source code.
///
/// Represents a single point in the source file with line, column, and byte offset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
    /// Byte offset in source (0-indexed)
    pub offset: usize,
}

impl Position {
    /// Create a new position.
    ///
    /// # Arguments
    ///
    /// * `line` - Line number (1-indexed)
    /// * `column` - Column number (1-indexed)
    /// * `offset` - Byte offset (0-indexed)
    pub fn new(line: usize, column: usize, offset: usize) -> Self {
        Position {
            line,
            column,
            offset,
        }
    }

    /// Create a position at the start of the file.
    pub fn start() -> Self {
        Position::new(1, 1, 0)
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

/// A span of source code between two positions.
///
/// Represents a range in the source file, typically corresponding to a token or expression.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    /// Start position (inclusive)
    pub start: Position,
    /// End position (exclusive)
    pub end: Position,
}

impl Span {
    /// Create a new span from start and end positions.
    pub fn new(start: Position, end: Position) -> Self {
        Span { start, end }
    }

    /// Create a span covering a single position.
    pub fn point(pos: Position) -> Self {
        Span {
            start: pos,
            end: pos,
        }
    }

    /// Merge two spans into a span covering both.
    ///
    /// The resulting span starts at the earlier start position
    /// and ends at the later end position.
    pub fn merge(&self, other: &Span) -> Span {
        let start = if self.start.offset < other.start.offset {
            self.start
        } else {
            other.start
        };
        let end = if self.end.offset > other.end.offset {
            self.end
        } else {
            other.end
        };
        Span { start, end }
    }

    /// Format the location for error messages.
    ///
    /// Returns a string like "line 5, column 10" for the start position.
    pub fn format_location(&self) -> String {
        format!("line {}, column {}", self.start.line, self.start.column)
    }

    /// Get the length of this span in bytes.
    pub fn len(&self) -> usize {
        self.end.offset.saturating_sub(self.start.offset)
    }

    /// Check if this span is empty (start == end).
    pub fn is_empty(&self) -> bool {
        self.start.offset == self.end.offset
    }

    /// Check if this span is on a single line.
    pub fn is_single_line(&self) -> bool {
        self.start.line == self.end.line
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.start.line == self.end.line {
            write!(
                f,
                "{}:{}-{}",
                self.start.line, self.start.column, self.end.column
            )
        } else {
            write!(f, "{}-{}", self.start, self.end)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Position Tests
    // ========================================================================

    #[test]
    fn test_position_new() {
        let pos = Position::new(5, 10, 42);
        assert_eq!(pos.line, 5);
        assert_eq!(pos.column, 10);
        assert_eq!(pos.offset, 42);
    }

    #[test]
    fn test_position_start() {
        let pos = Position::start();
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 1);
        assert_eq!(pos.offset, 0);
    }

    #[test]
    fn test_position_display() {
        let pos = Position::new(10, 25, 100);
        assert_eq!(format!("{}", pos), "10:25");
    }

    #[test]
    fn test_position_equality() {
        let pos1 = Position::new(1, 1, 0);
        let pos2 = Position::new(1, 1, 0);
        let pos3 = Position::new(1, 2, 1);
        assert_eq!(pos1, pos2);
        assert_ne!(pos1, pos3);
    }

    // ========================================================================
    // Span Tests
    // ========================================================================

    #[test]
    fn test_span_new() {
        let start = Position::new(1, 1, 0);
        let end = Position::new(1, 5, 4);
        let span = Span::new(start, end);
        assert_eq!(span.start, start);
        assert_eq!(span.end, end);
    }

    #[test]
    fn test_span_point() {
        let pos = Position::new(5, 10, 42);
        let span = Span::point(pos);
        assert_eq!(span.start, pos);
        assert_eq!(span.end, pos);
        assert!(span.is_empty());
    }

    #[test]
    fn test_span_merge_sequential() {
        let span1 = Span::new(Position::new(1, 1, 0), Position::new(1, 5, 4));
        let span2 = Span::new(Position::new(1, 5, 4), Position::new(1, 10, 9));
        let merged = span1.merge(&span2);
        assert_eq!(merged.start, Position::new(1, 1, 0));
        assert_eq!(merged.end, Position::new(1, 10, 9));
    }

    #[test]
    fn test_span_merge_overlapping() {
        let span1 = Span::new(Position::new(1, 1, 0), Position::new(1, 7, 6));
        let span2 = Span::new(Position::new(1, 5, 4), Position::new(1, 10, 9));
        let merged = span1.merge(&span2);
        assert_eq!(merged.start, Position::new(1, 1, 0));
        assert_eq!(merged.end, Position::new(1, 10, 9));
    }

    #[test]
    fn test_span_merge_reverse_order() {
        let span1 = Span::new(Position::new(1, 5, 4), Position::new(1, 10, 9));
        let span2 = Span::new(Position::new(1, 1, 0), Position::new(1, 5, 4));
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
    fn test_span_len() {
        let span = Span::new(Position::new(1, 1, 0), Position::new(1, 5, 4));
        assert_eq!(span.len(), 4);
    }

    #[test]
    fn test_span_is_empty() {
        let pos = Position::new(1, 1, 0);
        let empty = Span::point(pos);
        let non_empty = Span::new(pos, Position::new(1, 5, 4));

        assert!(empty.is_empty());
        assert!(!non_empty.is_empty());
    }

    #[test]
    fn test_span_is_single_line() {
        let single = Span::new(Position::new(5, 1, 10), Position::new(5, 10, 19));
        let multi = Span::new(Position::new(5, 1, 10), Position::new(6, 5, 25));

        assert!(single.is_single_line());
        assert!(!multi.is_single_line());
    }

    #[test]
    fn test_span_display_single_line() {
        let span = Span::new(Position::new(5, 10, 42), Position::new(5, 15, 47));
        assert_eq!(format!("{}", span), "5:10-15");
    }

    #[test]
    fn test_span_display_multi_line() {
        let span = Span::new(Position::new(5, 10, 42), Position::new(7, 5, 67));
        assert_eq!(format!("{}", span), "5:10-7:5");
    }

    #[test]
    fn test_span_equality() {
        let span1 = Span::new(Position::new(1, 1, 0), Position::new(1, 5, 4));
        let span2 = Span::new(Position::new(1, 1, 0), Position::new(1, 5, 4));
        let span3 = Span::new(Position::new(1, 1, 0), Position::new(1, 6, 5));

        assert_eq!(span1, span2);
        assert_ne!(span1, span3);
    }

    #[test]
    fn test_span_merge_multiline() {
        let span1 = Span::new(Position::new(1, 1, 0), Position::new(2, 5, 15));
        let span2 = Span::new(Position::new(3, 1, 20), Position::new(4, 10, 35));
        let merged = span1.merge(&span2);
        assert_eq!(merged.start, Position::new(1, 1, 0));
        assert_eq!(merged.end, Position::new(4, 10, 35));
    }
}
