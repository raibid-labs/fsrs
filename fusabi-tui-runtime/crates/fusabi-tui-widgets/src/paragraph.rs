//! Paragraph widget for displaying multi-line text with wrapping and alignment.

use fusabi_tui_core::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
};
use unicode_width::UnicodeWidthStr;

use crate::{
    block::Block,
    text::{Line, Span, Text},
    widget::Widget,
};

/// Text alignment options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Alignment {
    /// Align text to the left
    Left,
    /// Center text horizontally
    Center,
    /// Align text to the right
    Right,
}

impl Default for Alignment {
    fn default() -> Self {
        Self::Left
    }
}

/// Text wrapping modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Wrap {
    /// Don't wrap text, truncate if necessary
    NoWrap,
    /// Wrap at any character
    Wrap,
    /// Wrap at word boundaries
    WordWrap,
}

impl Default for Wrap {
    fn default() -> Self {
        Self::NoWrap
    }
}

/// A paragraph widget for displaying multi-line text.
///
/// Supports text wrapping, alignment, scrolling, and can be wrapped in a [`Block`].
///
/// # Examples
///
/// ```rust
/// use fusabi_tui_core::{buffer::Buffer, layout::Rect, style::{Color, Style}};
/// use fusabi_tui_widgets::{Paragraph, Wrap, Alignment, Widget};
///
/// let paragraph = Paragraph::new("Hello, World!")
///     .style(Style::default().fg(Color::White))
///     .alignment(Alignment::Center)
///     .wrap(Wrap::WordWrap);
///
/// let area = Rect::new(0, 0, 20, 5);
/// let mut buffer = Buffer::new(area);
/// paragraph.render(area, &mut buffer);
/// ```
#[derive(Debug, Clone)]
pub struct Paragraph<'a> {
    /// The text content to display
    text: Text<'a>,
    /// Optional block to wrap the paragraph
    block: Option<Block>,
    /// Style for the paragraph
    style: Style,
    /// Text alignment
    alignment: Alignment,
    /// Text wrapping mode
    wrap: Wrap,
    /// Horizontal scroll offset
    scroll_x: u16,
    /// Vertical scroll offset
    scroll_y: u16,
}

impl<'a> Paragraph<'a> {
    /// Creates a new paragraph with the given text.
    pub fn new<T>(text: T) -> Self
    where
        T: Into<Text<'a>>,
    {
        Self {
            text: text.into(),
            block: None,
            style: Style::default(),
            alignment: Alignment::default(),
            wrap: Wrap::default(),
            scroll_x: 0,
            scroll_y: 0,
        }
    }

    /// Wraps the paragraph in a block.
    #[must_use]
    pub fn block(mut self, block: Block) -> Self {
        self.block = Some(block);
        self
    }

    /// Sets the style for the paragraph.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Sets the text alignment.
    #[must_use]
    pub fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Sets the text wrapping mode.
    #[must_use]
    pub fn wrap(mut self, wrap: Wrap) -> Self {
        self.wrap = wrap;
        self
    }

    /// Sets the scroll offset.
    #[must_use]
    pub fn scroll(mut self, x: u16, y: u16) -> Self {
        self.scroll_x = x;
        self.scroll_y = y;
        self
    }

    /// Wraps a line of text to fit within the given width.
    fn wrap_line(&self, line: &'a Line<'a>, width: usize) -> Vec<Line<'a>> {
        match self.wrap {
            Wrap::NoWrap => vec![line.clone()],
            Wrap::Wrap => self.wrap_line_char(line, width),
            Wrap::WordWrap => self.wrap_line_word(line, width),
        }
    }

    /// Wraps a line at any character boundary.
    fn wrap_line_char(&self, line: &Line<'a>, width: usize) -> Vec<Line<'a>> {
        let mut wrapped = Vec::new();
        let mut current_line = Vec::new();
        let mut current_width = 0;

        for span in &line.spans {
            let content = span.content.as_ref();
            let mut chars = content.chars().peekable();

            while chars.peek().is_some() {
                let mut segment = String::new();
                let mut segment_width = 0;

                while let Some(&ch) = chars.peek() {
                    let ch_width = ch.to_string().width();
                    if current_width + segment_width + ch_width > width {
                        break;
                    }
                    segment.push(ch);
                    segment_width += ch_width;
                    chars.next();
                }

                if !segment.is_empty() {
                    current_line.push(Span::styled(segment, span.style));
                    current_width += segment_width;
                }

                if current_width >= width || chars.peek().is_some() {
                    if !current_line.is_empty() {
                        wrapped.push(Line::from_spans(current_line));
                        current_line = Vec::new();
                        current_width = 0;
                    }
                }
            }
        }

        if !current_line.is_empty() {
            wrapped.push(Line::from_spans(current_line));
        }

        if wrapped.is_empty() {
            wrapped.push(Line::empty());
        }

        wrapped
    }

    /// Wraps a line at word boundaries.
    fn wrap_line_word(&self, line: &'a Line<'a>, width: usize) -> Vec<Line<'a>> {
        let mut wrapped = Vec::new();
        let mut current_line = Vec::new();
        let mut current_width = 0;

        for span in &line.spans {
            let content = span.content.as_ref();
            let words: Vec<&str> = content.split_inclusive(char::is_whitespace).collect();

            for word in words {
                let word_width = word.width();

                if current_width + word_width > width && current_width > 0 {
                    // Start a new line
                    wrapped.push(Line::from_spans(current_line));
                    current_line = Vec::new();
                    current_width = 0;
                }

                if word_width > width {
                    // Word is too long, wrap at character boundary
                    let char_wrapped = self.wrap_line_char(
                        &Line::from(Span::styled(word, span.style)),
                        width,
                    );
                    for (i, char_line) in char_wrapped.into_iter().enumerate() {
                        if i > 0 && !current_line.is_empty() {
                            wrapped.push(Line::from_spans(current_line));
                            current_line = Vec::new();
                            current_width = 0;
                        }
                        current_line.extend(char_line.spans);
                        current_width = current_line.iter().map(Span::width).sum();
                    }
                } else {
                    current_line.push(Span::styled(word, span.style));
                    current_width += word_width;
                }
            }
        }

        if !current_line.is_empty() {
            wrapped.push(Line::from_spans(current_line));
        }

        if wrapped.is_empty() {
            wrapped.push(Line::empty());
        }

        wrapped
    }

    /// Aligns a line of text within the given width.
    fn align_line<'b>(&self, line: &'b Line<'a>, width: usize) -> (usize, &'b Line<'a>) {
        let line_width = line.width();
        let offset = match self.alignment {
            Alignment::Left => 0,
            Alignment::Center => width.saturating_sub(line_width) / 2,
            Alignment::Right => width.saturating_sub(line_width),
        };
        (offset, line)
    }
}

impl Widget for Paragraph<'_> {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        // Render block and get inner area
        let text_area = if let Some(ref block) = self.block {
            block.render(area, buf);
            block.inner(area)
        } else {
            buf.set_style(area, self.style);
            area
        };

        if text_area.width == 0 || text_area.height == 0 {
            return;
        }

        // Wrap all lines
        let mut wrapped_lines = Vec::new();
        for line in &self.text.lines {
            wrapped_lines.extend(self.wrap_line(line, text_area.width as usize));
        }

        // Apply vertical scroll
        let start_line = self.scroll_y as usize;
        let end_line = (start_line + text_area.height as usize).min(wrapped_lines.len());

        // Render visible lines
        for (y, line) in wrapped_lines[start_line..end_line].iter().enumerate() {
            let (x_offset, line) = self.align_line(line, text_area.width as usize);
            let mut x = text_area.x + x_offset as u16;
            let y = text_area.y + y as u16;

            // Apply horizontal scroll to spans
            let mut skip_width = self.scroll_x as usize;
            
            for span in &line.spans {
                let span_width = span.width();
                
                if skip_width >= span_width {
                    skip_width -= span_width;
                    continue;
                }

                let content = if skip_width > 0 {
                    // Skip some characters from this span
                    let mut skipped = 0;
                    let mut chars_to_skip = 0;
                    for ch in span.content.chars() {
                        let ch_width = ch.to_string().width();
                        if skipped + ch_width > skip_width {
                            break;
                        }
                        skipped += ch_width;
                        chars_to_skip += 1;
                    }
                    span.content.chars().skip(chars_to_skip).collect::<String>()
                } else {
                    span.content.to_string()
                };

                skip_width = 0;

                // Render the span
                for ch in content.chars() {
                    if x >= text_area.x + text_area.width {
                        break;
                    }
                    buf.set_string(x, y, &ch.to_string(), span.style);
                    x += ch.to_string().width() as u16;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fusabi_tui_core::style::Color;

    #[test]
    fn test_paragraph_creation() {
        let p = Paragraph::new("test");
        assert_eq!(p.text.lines.len(), 1);
    }

    #[test]
    fn test_paragraph_with_block() {
        use crate::{Block, Borders};
        
        let block = Block::default().borders(Borders::ALL);
        let p = Paragraph::new("test").block(block);
        assert!(p.block.is_some());
    }

    #[test]
    fn test_paragraph_alignment() {
        let p = Paragraph::new("test")
            .alignment(Alignment::Center);
        assert_eq!(p.alignment, Alignment::Center);
    }

    #[test]
    fn test_paragraph_wrap() {
        let p = Paragraph::new("test")
            .wrap(Wrap::WordWrap);
        assert_eq!(p.wrap, Wrap::WordWrap);
    }

    #[test]
    fn test_paragraph_render() {
        let p = Paragraph::new("Hello\nWorld")
            .style(Style::default().fg(Color::Green));

        let area = Rect::new(0, 0, 10, 5);
        let mut buffer = Buffer::new(area);
        p.render(area, &mut buffer);

        assert_eq!(buffer.get(0, 0).unwrap().symbol, "H");
        assert_eq!(buffer.get(0, 1).unwrap().symbol, "W");
    }

    #[test]
    fn test_wrap_line_char() {
        let p = Paragraph::new("").wrap(Wrap::Wrap);
        let line = Line::from("Hello World");
        let wrapped = p.wrap_line(&line, 5);
        
        assert!(wrapped.len() >= 2);
    }

    #[test]
    fn test_wrap_line_word() {
        let p = Paragraph::new("").wrap(Wrap::WordWrap);
        let line = Line::from("Hello World");
        let wrapped = p.wrap_line(&line, 7);
        
        assert!(wrapped.len() >= 2);
    }

    #[test]
    fn test_align_line_center() {
        let p = Paragraph::new("").alignment(Alignment::Center);
        let line = Line::from("Hi");
        let (offset, _) = p.align_line(&line, 10);
        
        assert_eq!(offset, 4); // (10 - 2) / 2 = 4
    }

    #[test]
    fn test_align_line_right() {
        let p = Paragraph::new("").alignment(Alignment::Right);
        let line = Line::from("Hi");
        let (offset, _) = p.align_line(&line, 10);
        
        assert_eq!(offset, 8); // 10 - 2 = 8
    }

    #[test]
    fn test_paragraph_scroll() {
        let p = Paragraph::new("Line1\nLine2\nLine3\nLine4")
            .scroll(0, 1);

        let area = Rect::new(0, 0, 10, 2);
        let mut buffer = Buffer::new(area);
        p.render(area, &mut buffer);

        // Should render Line2 and Line3 (skipping Line1 due to scroll)
        assert_eq!(buffer.get(0, 0).unwrap().symbol, "L");
        assert_eq!(buffer.get(4, 0).unwrap().symbol, "2");
    }
}
