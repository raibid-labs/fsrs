//! Text primitives for building styled text content.
//!
//! This module provides the building blocks for composing styled text:
//! - [`Span`] - A single styled text segment
//! - [`Line`] - A sequence of spans forming a single line
//! - [`Text`] - A sequence of lines forming multi-line text

use fusabi_tui_core::style::Style;
use unicode_width::UnicodeWidthStr;

/// A styled text segment.
///
/// A span represents a contiguous piece of text with a single style applied.
///
/// # Examples
///
/// ```rust
/// use fusabi_tui_core::style::{Color, Style};
/// use fusabi_tui_widgets::Span;
///
/// let span = Span::styled("Hello", Style::default().fg(Color::Green));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Span<'a> {
    /// The text content
    pub content: std::borrow::Cow<'a, str>,
    /// The style to apply
    pub style: Style,
}

impl<'a> Span<'a> {
    /// Creates a new unstyled span.
    pub fn raw<T>(content: T) -> Self
    where
        T: Into<std::borrow::Cow<'a, str>>,
    {
        Self {
            content: content.into(),
            style: Style::default(),
        }
    }

    /// Creates a new styled span.
    pub fn styled<T>(content: T, style: Style) -> Self
    where
        T: Into<std::borrow::Cow<'a, str>>,
    {
        Self {
            content: content.into(),
            style,
        }
    }

    /// Returns the width of the span in terminal cells.
    pub fn width(&self) -> usize {
        self.content.width()
    }

    /// Sets the style of the span.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl<'a> From<&'a str> for Span<'a> {
    fn from(s: &'a str) -> Self {
        Self::raw(s)
    }
}

impl From<String> for Span<'static> {
    fn from(s: String) -> Self {
        Self::raw(s)
    }
}

impl<'a> From<&'a String> for Span<'a> {
    fn from(s: &'a String) -> Self {
        Self::raw(s.as_str())
    }
}

/// A line of text composed of styled spans.
///
/// A line is a horizontal sequence of spans. Lines are separated by newlines
/// when rendered as part of [`Text`].
///
/// # Examples
///
/// ```rust
/// use fusabi_tui_core::style::{Color, Style};
/// use fusabi_tui_widgets::{Line, Span};
///
/// let line = Line::from(vec![
///     Span::raw("Hello "),
///     Span::styled("World", Style::default().fg(Color::Green)),
/// ]);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Line<'a> {
    /// The spans that make up this line
    pub spans: Vec<Span<'a>>,
}

impl<'a> Line<'a> {
    /// Creates a new line from a vector of spans.
    pub fn from_spans(spans: Vec<Span<'a>>) -> Self {
        Self { spans }
    }

    /// Creates a new empty line.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Returns the width of the line in terminal cells.
    pub fn width(&self) -> usize {
        self.spans.iter().map(Span::width).sum()
    }

    /// Returns the number of spans in the line.
    pub fn span_count(&self) -> usize {
        self.spans.len()
    }

    /// Adds a span to the line.
    pub fn push_span(&mut self, span: Span<'a>) {
        self.spans.push(span);
    }

    /// Applies a style to all spans in the line that don't have a style set.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        for span in &mut self.spans {
            if span.style == Style::default() {
                span.style = style;
            }
        }
        self
    }
}

impl<'a> From<&'a str> for Line<'a> {
    fn from(s: &'a str) -> Self {
        Self {
            spans: vec![Span::raw(s)],
        }
    }
}

impl From<String> for Line<'static> {
    fn from(s: String) -> Self {
        Self {
            spans: vec![Span::raw(s)],
        }
    }
}

impl<'a> From<Span<'a>> for Line<'a> {
    fn from(span: Span<'a>) -> Self {
        Self { spans: vec![span] }
    }
}

impl<'a> From<Vec<Span<'a>>> for Line<'a> {
    fn from(spans: Vec<Span<'a>>) -> Self {
        Self { spans }
    }
}

/// Multi-line text composed of lines.
///
/// Text is the top-level structure for representing styled text content.
/// It can be constructed from strings, lines, or vectors of lines.
///
/// # Examples
///
/// ```rust
/// use fusabi_tui_core::style::{Color, Style};
/// use fusabi_tui_widgets::{Text, Line, Span};
///
/// // From a simple string
/// let text = Text::from("Hello\nWorld");
///
/// // From styled lines
/// let text = Text::from(vec![
///     Line::from("Header"),
///     Line::from(vec![
///         Span::raw("Styled "),
///         Span::styled("text", Style::default().fg(Color::Green)),
///     ]),
/// ]);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Text<'a> {
    /// The lines that make up this text
    pub lines: Vec<Line<'a>>,
}

impl<'a> Text<'a> {
    /// Creates a new text from a vector of lines.
    pub fn from_lines(lines: Vec<Line<'a>>) -> Self {
        Self { lines }
    }

    /// Creates a new empty text.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Returns the number of lines in the text.
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Returns the width of the longest line in terminal cells.
    pub fn width(&self) -> usize {
        self.lines.iter().map(Line::width).max().unwrap_or(0)
    }

    /// Returns the height of the text in lines.
    pub fn height(&self) -> usize {
        self.lines.len()
    }

    /// Adds a line to the text.
    pub fn push_line(&mut self, line: Line<'a>) {
        self.lines.push(line);
    }

    /// Applies a style to all lines that don't have a style set.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        for line in &mut self.lines {
            *line = std::mem::take(line).style(style);
        }
        self
    }
}

impl<'a> From<&'a str> for Text<'a> {
    fn from(s: &'a str) -> Self {
        let lines = s.lines().map(Line::from).collect();
        Self { lines }
    }
}

impl From<String> for Text<'static> {
    fn from(s: String) -> Self {
        let lines = s.lines().map(|line| Line::from(line.to_string())).collect();
        Self { lines }
    }
}

impl<'a> From<Line<'a>> for Text<'a> {
    fn from(line: Line<'a>) -> Self {
        Self { lines: vec![line] }
    }
}

impl<'a> From<Vec<Line<'a>>> for Text<'a> {
    fn from(lines: Vec<Line<'a>>) -> Self {
        Self { lines }
    }
}

impl<'a> From<Span<'a>> for Text<'a> {
    fn from(span: Span<'a>) -> Self {
        Self {
            lines: vec![Line::from(span)],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fusabi_tui_core::style::Color;

    #[test]
    fn test_span_creation() {
        let span = Span::raw("test");
        assert_eq!(span.content, "test");
        assert_eq!(span.style, Style::default());

        let styled = Span::styled("test", Style::default().fg(Color::Green));
        assert_eq!(styled.content, "test");
        assert_eq!(styled.style.fg, Some(Color::Green));
    }

    #[test]
    fn test_span_width() {
        let span = Span::raw("hello");
        assert_eq!(span.width(), 5);

        let wide = Span::raw("你好");
        assert_eq!(wide.width(), 4); // Chinese characters are 2 cells wide
    }

    #[test]
    fn test_span_from_str() {
        let span: Span = "test".into();
        assert_eq!(span.content, "test");
    }

    #[test]
    fn test_line_creation() {
        let line = Line::from("test");
        assert_eq!(line.spans.len(), 1);
        assert_eq!(line.spans[0].content, "test");

        let line = Line::from(vec![Span::raw("a"), Span::raw("b")]);
        assert_eq!(line.spans.len(), 2);
    }

    #[test]
    fn test_line_width() {
        let line = Line::from(vec![Span::raw("hello "), Span::raw("world")]);
        assert_eq!(line.width(), 11);
    }

    #[test]
    fn test_line_style() {
        let style = Style::default().fg(Color::Green);
        let line = Line::from(vec![Span::raw("test")]).style(style);
        assert_eq!(line.spans[0].style.fg, Some(Color::Green));
    }

    #[test]
    fn test_text_from_str() {
        let text = Text::from("line1\nline2\nline3");
        assert_eq!(text.lines.len(), 3);
        assert_eq!(text.lines[0].spans[0].content, "line1");
        assert_eq!(text.lines[1].spans[0].content, "line2");
        assert_eq!(text.lines[2].spans[0].content, "line3");
    }

    #[test]
    fn test_text_dimensions() {
        let text = Text::from("hello\nworld!\nhi");
        assert_eq!(text.height(), 3);
        assert_eq!(text.width(), 6); // "world!" is the longest
    }

    #[test]
    fn test_text_from_lines() {
        let text = Text::from(vec![
            Line::from("line1"),
            Line::from("line2"),
        ]);
        assert_eq!(text.lines.len(), 2);
    }

    #[test]
    fn test_text_style() {
        let style = Style::default().fg(Color::Blue);
        let text = Text::from(vec![
            Line::from(Span::raw("test")),
        ]).style(style);
        assert_eq!(text.lines[0].spans[0].style.fg, Some(Color::Blue));
    }
}
