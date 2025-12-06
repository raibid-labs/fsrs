// Pretty error reporting with source code snippets

use crate::chunk::{Chunk, SourceSpan};
use crate::vm::VmError;
use std::fmt::Write;

/// Format a runtime error with source context
pub fn format_error(error: &VmError, chunk: &Chunk, instruction_offset: usize) -> String {
    let mut output = String::new();

    // Get span for the faulting instruction
    let span = chunk.span_at(instruction_offset);

    // Error header
    let _ = writeln!(output, "Error: {}", error);

    // Location info if available
    if let Some(span) = span {
        let file = chunk.source_file.as_deref().unwrap_or("<input>");
        let _ = writeln!(output, "  --> {}:{}:{}", file, span.line, span.column);

        // Source snippet if available
        if let Some(source) = &chunk.source {
            if let Some(snippet) = format_source_snippet(source, span) {
                let _ = write!(output, "{}", snippet);
            }
        }
    }

    output
}

/// Format a source code snippet with a caret pointing to the error location
fn format_source_snippet(source: &str, span: SourceSpan) -> Option<String> {
    let lines: Vec<&str> = source.lines().collect();
    let line_idx = span.line.saturating_sub(1) as usize;

    if line_idx >= lines.len() {
        return None;
    }

    let line = lines[line_idx];
    let line_num = span.line;
    let line_num_width = line_num.to_string().len().max(1);

    let mut output = String::new();

    // Blank line with gutter
    let _ = writeln!(output, "{:width$} |", "", width = line_num_width);

    // Source line
    let _ = writeln!(
        output,
        "{:width$} | {}",
        line_num,
        line,
        width = line_num_width
    );

    // Caret line
    let col = span.column.saturating_sub(1) as usize;
    let caret_count = (span.length as usize).max(1);
    let carets = "^".repeat(caret_count);
    let _ = writeln!(
        output,
        "{:width$} | {:>col$}{}",
        "",
        "",
        carets,
        width = line_num_width,
        col = col
    );

    Some(output)
}

/// A structured runtime error with location information
#[derive(Debug, Clone)]
pub struct RuntimeError {
    pub error: VmError,
    pub span: Option<SourceSpan>,
    pub source_file: Option<String>,
}

impl RuntimeError {
    pub fn new(error: VmError) -> Self {
        RuntimeError {
            error,
            span: None,
            source_file: None,
        }
    }

    pub fn with_span(mut self, span: SourceSpan) -> Self {
        if span.is_known() {
            self.span = Some(span);
        }
        self
    }

    pub fn with_file(mut self, file: impl Into<String>) -> Self {
        self.source_file = Some(file.into());
        self
    }

    /// Format the error with source context
    pub fn format(&self, source: Option<&str>) -> String {
        let mut output = String::new();

        // Error header
        let _ = writeln!(output, "Error: {}", self.error);

        // Location info if available
        if let Some(span) = self.span {
            let file = self.source_file.as_deref().unwrap_or("<input>");
            let _ = writeln!(output, "  --> {}:{}:{}", file, span.line, span.column);

            // Source snippet if available
            if let Some(source) = source {
                if let Some(snippet) = format_source_snippet(source, span) {
                    let _ = write!(output, "{}", snippet);
                }
            }
        }

        output
    }
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error)?;
        if let Some(span) = self.span {
            let file = self.source_file.as_deref().unwrap_or("<input>");
            write!(f, " at {}:{}:{}", file, span.line, span.column)?;
        }
        Ok(())
    }
}

impl std::error::Error for RuntimeError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_source_snippet() {
        let source = "let x = foo + 1\nlet y = 2";
        let span = SourceSpan::new(1, 9, 8, 3); // "foo"

        let snippet = format_source_snippet(source, span).unwrap();
        assert!(snippet.contains("let x = foo + 1"));
        assert!(snippet.contains("^^^"));
    }

    #[test]
    fn test_format_error_with_span() {
        let mut chunk = Chunk::new();
        chunk.set_source("let x = foo + 1");
        chunk.set_source_file("main.fsx");
        chunk.emit_with_span(
            crate::instruction::Instruction::Return,
            SourceSpan::new(1, 9, 8, 3),
        );

        let error = VmError::Runtime("undefined variable 'foo'".to_string());
        let formatted = format_error(&error, &chunk, 0);

        assert!(formatted.contains("undefined variable 'foo'"));
        assert!(formatted.contains("main.fsx:1:9"));
        assert!(formatted.contains("let x = foo + 1"));
        assert!(formatted.contains("^^^"));
    }

    #[test]
    fn test_format_error_without_span() {
        let chunk = Chunk::new();
        let error = VmError::StackUnderflow;
        let formatted = format_error(&error, &chunk, 0);

        assert!(formatted.contains("Stack underflow"));
        assert!(!formatted.contains("-->"));
    }

    #[test]
    fn test_runtime_error_display() {
        let err = RuntimeError::new(VmError::DivisionByZero)
            .with_span(SourceSpan::new(5, 10, 42, 1))
            .with_file("test.fsx");

        let s = format!("{}", err);
        assert!(s.contains("Division by zero"));
        assert!(s.contains("test.fsx:5:10"));
    }
}
