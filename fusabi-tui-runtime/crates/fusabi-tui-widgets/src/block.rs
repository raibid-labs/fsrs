//! Block widget for creating bordered containers.
//!
//! The Block widget is a fundamental building block for TUI layouts, providing
//! a bordered container with optional title and padding.

use crate::borders::{BorderType, Borders};
use crate::widget::Widget;
use fusabi_tui_core::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
};

/// Position of the title within the block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TitlePosition {
    /// Title at the top of the block
    Top,
    /// Title at the bottom of the block
    Bottom,
}

impl Default for TitlePosition {
    fn default() -> Self {
        Self::Top
    }
}

/// Horizontal alignment of the title text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TitleAlignment {
    /// Align title to the left
    Left,
    /// Align title to the center
    Center,
    /// Align title to the right
    Right,
}

impl Default for TitleAlignment {
    fn default() -> Self {
        Self::Left
    }
}

/// A title for a block with position and alignment.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Title {
    /// The text of the title
    pub content: String,
    /// The position of the title (top or bottom)
    pub position: TitlePosition,
    /// The alignment of the title (left, center, or right)
    pub alignment: TitleAlignment,
    /// The style of the title text
    pub style: Style,
}

impl Title {
    /// Creates a new title with the given text.
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            position: TitlePosition::default(),
            alignment: TitleAlignment::default(),
            style: Style::default(),
        }
    }

    /// Sets the position of the title.
    pub fn position(mut self, position: TitlePosition) -> Self {
        self.position = position;
        self
    }

    /// Sets the alignment of the title.
    pub fn alignment(mut self, alignment: TitleAlignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Sets the style of the title.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl<T> From<T> for Title
where
    T: Into<String>,
{
    fn from(content: T) -> Self {
        Self::new(content)
    }
}

/// A block widget that draws borders and an optional title.
///
/// Blocks are fundamental building blocks for creating structured TUI layouts.
/// They can be used to create panels, containers, and organize content with
/// visual boundaries.
///
/// # Examples
///
/// ```
/// use fusabi_tui_widgets::block::{Block, TitleAlignment};
/// use fusabi_tui_widgets::borders::{Borders, BorderType};
/// use fusabi_tui_core::style::{Style, Color, Modifier};
///
/// let block = Block::default()
///     .title("My Panel")
///     .borders(Borders::ALL)
///     .border_type(BorderType::Rounded)
///     .border_style(Style::default().fg(Color::Cyan))
///     .title_alignment(TitleAlignment::Center);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    /// The title of the block
    title: Option<Title>,
    /// Which borders to draw
    borders: Borders,
    /// The type of border characters to use
    border_type: BorderType,
    /// The style to apply to the borders
    border_style: Style,
    /// The style to apply to the interior area
    style: Style,
    /// Padding inside the borders
    padding: Padding,
}

/// Padding specification for a block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Padding {
    /// Left padding
    pub left: u16,
    /// Right padding
    pub right: u16,
    /// Top padding
    pub top: u16,
    /// Bottom padding
    pub bottom: u16,
}

impl Padding {
    /// Creates uniform padding on all sides.
    pub const fn uniform(padding: u16) -> Self {
        Self {
            left: padding,
            right: padding,
            top: padding,
            bottom: padding,
        }
    }

    /// Creates horizontal padding (left and right).
    pub const fn horizontal(padding: u16) -> Self {
        Self {
            left: padding,
            right: padding,
            top: 0,
            bottom: 0,
        }
    }

    /// Creates vertical padding (top and bottom).
    pub const fn vertical(padding: u16) -> Self {
        Self {
            left: 0,
            right: 0,
            top: padding,
            bottom: padding,
        }
    }

    /// Creates zero padding.
    pub const fn zero() -> Self {
        Self {
            left: 0,
            right: 0,
            top: 0,
            bottom: 0,
        }
    }
}

impl Default for Padding {
    fn default() -> Self {
        Self::zero()
    }
}

impl Default for Block {
    fn default() -> Self {
        Self {
            title: None,
            borders: Borders::NONE,
            border_type: BorderType::default(),
            border_style: Style::default(),
            style: Style::default(),
            padding: Padding::default(),
        }
    }
}

impl Block {
    /// Creates a new block with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the title of the block.
    pub fn title<T>(mut self, title: T) -> Self
    where
        T: Into<Title>,
    {
        self.title = Some(title.into());
        self
    }

    /// Sets which borders to draw.
    pub fn borders(mut self, borders: Borders) -> Self {
        self.borders = borders;
        self
    }

    /// Sets the type of border characters to use.
    pub fn border_type(mut self, border_type: BorderType) -> Self {
        self.border_type = border_type;
        self
    }

    /// Sets the style of the borders.
    pub fn border_style(mut self, style: Style) -> Self {
        self.border_style = style;
        self
    }

    /// Sets the style of the block's interior.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Sets the title alignment.
    pub fn title_alignment(mut self, alignment: TitleAlignment) -> Self {
        if let Some(ref mut title) = self.title {
            title.alignment = alignment;
        }
        self
    }

    /// Sets the title position.
    pub fn title_position(mut self, position: TitlePosition) -> Self {
        if let Some(ref mut title) = self.title {
            title.position = position;
        }
        self
    }

    /// Sets uniform padding on all sides.
    pub fn padding(mut self, padding: Padding) -> Self {
        self.padding = padding;
        self
    }

    /// Computes the inner area after accounting for borders and padding.
    ///
    /// This is the area where child widgets should be rendered.
    pub fn inner(&self, area: Rect) -> Rect {
        let mut inner = area;

        // Account for borders
        if self.borders.contains(Borders::LEFT) {
            inner.x = inner.x.saturating_add(1);
            inner.width = inner.width.saturating_sub(1);
        }
        if self.borders.contains(Borders::RIGHT) {
            inner.width = inner.width.saturating_sub(1);
        }
        if self.borders.contains(Borders::TOP) {
            inner.y = inner.y.saturating_add(1);
            inner.height = inner.height.saturating_sub(1);
        }
        if self.borders.contains(Borders::BOTTOM) {
            inner.height = inner.height.saturating_sub(1);
        }

        // Account for padding
        inner.x = inner.x.saturating_add(self.padding.left);
        inner.width = inner.width.saturating_sub(self.padding.left + self.padding.right);
        inner.y = inner.y.saturating_add(self.padding.top);
        inner.height = inner.height.saturating_sub(self.padding.top + self.padding.bottom);

        inner
    }

    /// Renders the border for the given edge.
    fn render_border(&self, area: Rect, buf: &mut Buffer) {
        let (horizontal, vertical, top_left, top_right, bottom_left, bottom_right) =
            self.border_type.line_symbols();

        // Draw corners
        if self.borders.contains(Borders::TOP | Borders::LEFT) {
            buf.set_string(area.x, area.y, top_left, self.border_style);
        }
        if self.borders.contains(Borders::TOP | Borders::RIGHT) && area.width > 0 {
            buf.set_string(area.x + area.width - 1, area.y, top_right, self.border_style);
        }
        if self.borders.contains(Borders::BOTTOM | Borders::LEFT) && area.height > 0 {
            buf.set_string(area.x, area.y + area.height - 1, bottom_left, self.border_style);
        }
        if self.borders.contains(Borders::BOTTOM | Borders::RIGHT) && area.width > 0 && area.height > 0 {
            buf.set_string(
                area.x + area.width - 1,
                area.y + area.height - 1,
                bottom_right,
                self.border_style,
            );
        }

        // Draw horizontal borders
        if self.borders.contains(Borders::TOP) {
            for x in (area.x + 1)..(area.x + area.width - 1) {
                buf.set_string(x, area.y, horizontal, self.border_style);
            }
        }
        if self.borders.contains(Borders::BOTTOM) && area.height > 0 {
            for x in (area.x + 1)..(area.x + area.width - 1) {
                buf.set_string(x, area.y + area.height - 1, horizontal, self.border_style);
            }
        }

        // Draw vertical borders
        if self.borders.contains(Borders::LEFT) {
            for y in (area.y + 1)..(area.y + area.height - 1) {
                buf.set_string(area.x, y, vertical, self.border_style);
            }
        }
        if self.borders.contains(Borders::RIGHT) && area.width > 0 {
            for y in (area.y + 1)..(area.y + area.height - 1) {
                buf.set_string(area.x + area.width - 1, y, vertical, self.border_style);
            }
        }
    }

    /// Renders the title if present.
    fn render_title(&self, area: Rect, buf: &mut Buffer) {
        if let Some(ref title) = self.title {
            let y = match title.position {
                TitlePosition::Top => area.y,
                TitlePosition::Bottom => area.y + area.height.saturating_sub(1),
            };

            // Calculate available width for title
            let available_width = area.width.saturating_sub(2); // Account for border corners
            if available_width == 0 {
                return;
            }

            let title_text = &title.content;
            let title_width = title_text.len().min(available_width as usize) as u16;

            let x = match title.alignment {
                TitleAlignment::Left => area.x + 1,
                TitleAlignment::Center => {
                    area.x + 1 + (available_width.saturating_sub(title_width)) / 2
                }
                TitleAlignment::Right => {
                    area.x + 1 + available_width.saturating_sub(title_width)
                }
            };

            // Render the title with padding spaces
            let displayed_title = if title_text.len() > available_width as usize {
                &title_text[..available_width as usize]
            } else {
                title_text
            };

            buf.set_string(x, y, displayed_title, title.style);
        }
    }
}

impl Widget for Block {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        if area.width < 2 || area.height < 2 {
            return;
        }

        // Fill the interior with the block's style
        buf.set_style(area, self.style);

        // Draw borders
        if !self.borders.is_empty() {
            self.render_border(area, buf);
        }

        // Draw title
        self.render_title(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fusabi_tui_core::style::Color;

    #[test]
    fn test_padding_uniform() {
        let padding = Padding::uniform(2);
        assert_eq!(padding.left, 2);
        assert_eq!(padding.right, 2);
        assert_eq!(padding.top, 2);
        assert_eq!(padding.bottom, 2);
    }

    #[test]
    fn test_padding_horizontal() {
        let padding = Padding::horizontal(3);
        assert_eq!(padding.left, 3);
        assert_eq!(padding.right, 3);
        assert_eq!(padding.top, 0);
        assert_eq!(padding.bottom, 0);
    }

    #[test]
    fn test_padding_vertical() {
        let padding = Padding::vertical(1);
        assert_eq!(padding.left, 0);
        assert_eq!(padding.right, 0);
        assert_eq!(padding.top, 1);
        assert_eq!(padding.bottom, 1);
    }

    #[test]
    fn test_title_creation() {
        let title = Title::new("Test");
        assert_eq!(title.content, "Test");
        assert_eq!(title.position, TitlePosition::Top);
        assert_eq!(title.alignment, TitleAlignment::Left);
    }

    #[test]
    fn test_title_builder() {
        let title = Title::new("Test")
            .position(TitlePosition::Bottom)
            .alignment(TitleAlignment::Center)
            .style(Style::default().fg(Color::Red));

        assert_eq!(title.position, TitlePosition::Bottom);
        assert_eq!(title.alignment, TitleAlignment::Center);
        assert_eq!(title.style.fg, Some(Color::Red));
    }

    #[test]
    fn test_block_default() {
        let block = Block::default();
        assert!(block.title.is_none());
        assert_eq!(block.borders, Borders::NONE);
        assert_eq!(block.border_type, BorderType::Plain);
    }

    #[test]
    fn test_block_builder() {
        let block = Block::default()
            .title("Test")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Cyan))
            .padding(Padding::uniform(1));

        assert!(block.title.is_some());
        assert_eq!(block.borders, Borders::ALL);
        assert_eq!(block.border_type, BorderType::Rounded);
        assert_eq!(block.border_style.fg, Some(Color::Cyan));
        assert_eq!(block.padding, Padding::uniform(1));
    }

    #[test]
    fn test_block_inner_no_borders() {
        let block = Block::default();
        let area = Rect::new(0, 0, 10, 10);
        let inner = block.inner(area);
        assert_eq!(inner, area);
    }

    #[test]
    fn test_block_inner_all_borders() {
        let block = Block::default().borders(Borders::ALL);
        let area = Rect::new(0, 0, 10, 10);
        let inner = block.inner(area);
        assert_eq!(inner, Rect::new(1, 1, 8, 8));
    }

    #[test]
    fn test_block_inner_with_padding() {
        let block = Block::default()
            .borders(Borders::ALL)
            .padding(Padding::uniform(1));
        let area = Rect::new(0, 0, 10, 10);
        let inner = block.inner(area);
        // 1 for border + 1 for padding on each side
        assert_eq!(inner, Rect::new(2, 2, 6, 6));
    }

    #[test]
    fn test_block_inner_top_bottom_only() {
        let block = Block::default().borders(Borders::TOP | Borders::BOTTOM);
        let area = Rect::new(0, 0, 10, 10);
        let inner = block.inner(area);
        assert_eq!(inner, Rect::new(0, 1, 10, 8));
    }

    #[test]
    fn test_block_inner_left_right_only() {
        let block = Block::default().borders(Borders::LEFT | Borders::RIGHT);
        let area = Rect::new(0, 0, 10, 10);
        let inner = block.inner(area);
        assert_eq!(inner, Rect::new(1, 0, 8, 10));
    }

    #[test]
    fn test_block_render_no_borders() {
        let block = Block::default();
        let area = Rect::new(0, 0, 5, 5);
        let mut buffer = Buffer::new(area);

        block.render(area, &mut buffer);

        // All cells should be default
        for y in 0..5 {
            for x in 0..5 {
                let cell = buffer.get(x, y).unwrap();
                assert_eq!(cell.symbol, " ");
            }
        }
    }

    #[test]
    fn test_block_render_all_borders() {
        let block = Block::default().borders(Borders::ALL);
        let area = Rect::new(0, 0, 5, 5);
        let mut buffer = Buffer::new(area);

        block.render(area, &mut buffer);

        // Check corners
        assert_eq!(buffer.get(0, 0).unwrap().symbol, "┌");
        assert_eq!(buffer.get(4, 0).unwrap().symbol, "┐");
        assert_eq!(buffer.get(0, 4).unwrap().symbol, "└");
        assert_eq!(buffer.get(4, 4).unwrap().symbol, "┘");

        // Check horizontal borders
        assert_eq!(buffer.get(1, 0).unwrap().symbol, "─");
        assert_eq!(buffer.get(2, 0).unwrap().symbol, "─");
        assert_eq!(buffer.get(1, 4).unwrap().symbol, "─");

        // Check vertical borders
        assert_eq!(buffer.get(0, 1).unwrap().symbol, "│");
        assert_eq!(buffer.get(0, 2).unwrap().symbol, "│");
        assert_eq!(buffer.get(4, 1).unwrap().symbol, "│");
    }

    #[test]
    fn test_block_render_with_title() {
        let block = Block::default()
            .title("Test")
            .borders(Borders::ALL);
        let area = Rect::new(0, 0, 10, 5);
        let mut buffer = Buffer::new(area);

        block.render(area, &mut buffer);

        // Check title characters
        assert_eq!(buffer.get(1, 0).unwrap().symbol, "T");
        assert_eq!(buffer.get(2, 0).unwrap().symbol, "e");
        assert_eq!(buffer.get(3, 0).unwrap().symbol, "s");
        assert_eq!(buffer.get(4, 0).unwrap().symbol, "t");
    }

    #[test]
    fn test_block_render_rounded_borders() {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);
        let area = Rect::new(0, 0, 5, 5);
        let mut buffer = Buffer::new(area);

        block.render(area, &mut buffer);

        // Check rounded corners
        assert_eq!(buffer.get(0, 0).unwrap().symbol, "╭");
        assert_eq!(buffer.get(4, 0).unwrap().symbol, "╮");
        assert_eq!(buffer.get(0, 4).unwrap().symbol, "╰");
        assert_eq!(buffer.get(4, 4).unwrap().symbol, "╯");
    }

    #[test]
    fn test_block_title_alignment_center() {
        let block = Block::default()
            .title("Hi")
            .borders(Borders::ALL)
            .title_alignment(TitleAlignment::Center);
        let area = Rect::new(0, 0, 10, 5);
        let mut buffer = Buffer::new(area);

        block.render(area, &mut buffer);

        // Title "Hi" should be centered in width 10 (minus 2 for borders = 8)
        // Position should be around x=4
        assert_eq!(buffer.get(4, 0).unwrap().symbol, "H");
        assert_eq!(buffer.get(5, 0).unwrap().symbol, "i");
    }

    #[test]
    fn test_block_title_position_bottom() {
        let block = Block::default()
            .title(Title::new("Bot").position(TitlePosition::Bottom))
            .borders(Borders::ALL);
        let area = Rect::new(0, 0, 10, 5);
        let mut buffer = Buffer::new(area);

        block.render(area, &mut buffer);

        // Title should be at the bottom (y=4)
        assert_eq!(buffer.get(1, 4).unwrap().symbol, "B");
        assert_eq!(buffer.get(2, 4).unwrap().symbol, "o");
        assert_eq!(buffer.get(3, 4).unwrap().symbol, "t");
    }

    #[test]
    fn test_block_border_style() {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red));
        let area = Rect::new(0, 0, 5, 5);
        let mut buffer = Buffer::new(area);

        block.render(area, &mut buffer);

        // Check that border has the correct color
        assert_eq!(buffer.get(0, 0).unwrap().fg, Color::Red);
        assert_eq!(buffer.get(1, 0).unwrap().fg, Color::Red);
        assert_eq!(buffer.get(0, 1).unwrap().fg, Color::Red);
    }
}
