//! Crossterm-based renderer implementation.
//!
//! This module provides a renderer that uses crossterm to draw to the terminal.
//! It implements differential rendering to minimize terminal updates.

use std::io::Write;

use crossterm::ExecutableCommand;
use fusabi_tui_core::buffer::{Buffer, Cell};
use fusabi_tui_core::layout::Rect;
use fusabi_tui_core::style::{Color, Modifier};

use crate::error::{RenderError, Result};
use crate::renderer::Renderer;

/// A renderer that uses crossterm to draw to the terminal.
///
/// This renderer implements differential rendering by comparing the current buffer
/// with the last rendered buffer and only updating cells that have changed.
pub struct CrosstermRenderer<W: Write + Send> {
    /// The output writer (typically stdout)
    writer: W,
    /// The current buffer being rendered
    buffer: Buffer,
    /// The last buffer that was rendered (for differential rendering)
    last_buffer: Option<Buffer>,
}

impl<W: Write + Send> CrosstermRenderer<W> {
    /// Creates a new crossterm renderer with the given writer.
    ///
    /// The renderer will query the terminal size on initialization.
    ///
    /// # Errors
    ///
    /// Returns an error if the terminal size cannot be determined.
    pub fn new(writer: W) -> Result<Self> {
        let (width, height) = crossterm::terminal::size()
            .map_err(|e| RenderError::Backend(format!("Failed to get terminal size: {}", e)))?;

        Ok(Self::with_size(writer, width, height))
    }

    /// Creates a new crossterm renderer with a specific size.
    ///
    /// This is useful for testing or when the terminal size is already known.
    pub fn with_size(writer: W, width: u16, height: u16) -> Self {
        let area = Rect::new(0, 0, width, height);
        Self {
            writer,
            buffer: Buffer::new(area),
            last_buffer: None,
        }
    }

    /// Converts a fusabi-tui Color to a crossterm Color.
    fn convert_color(color: Color) -> crossterm::style::Color {
        use crossterm::style::Color as CColor;

        match color {
            Color::Black => CColor::Black,
            Color::Red => CColor::DarkRed,
            Color::Green => CColor::DarkGreen,
            Color::Yellow => CColor::DarkYellow,
            Color::Blue => CColor::DarkBlue,
            Color::Magenta => CColor::DarkMagenta,
            Color::Cyan => CColor::DarkCyan,
            Color::White => CColor::Grey,
            Color::DarkGray => CColor::DarkGrey,
            Color::LightRed => CColor::Red,
            Color::LightGreen => CColor::Green,
            Color::LightYellow => CColor::Yellow,
            Color::LightBlue => CColor::Blue,
            Color::LightMagenta => CColor::Magenta,
            Color::LightCyan => CColor::Cyan,
            Color::LightWhite => CColor::White,
            Color::Rgb(r, g, b) => CColor::Rgb { r, g, b },
            Color::Indexed(i) => CColor::AnsiValue(i),
            Color::Reset => CColor::Reset,
        }
    }

    /// Applies text modifiers to the output.
    fn apply_modifiers(&mut self, modifier: Modifier) -> Result<()> {
        use crossterm::style::Attribute;

        if modifier.contains(Modifier::BOLD) {
            crossterm::execute!(self.writer, crossterm::style::SetAttribute(Attribute::Bold))?;
        }
        if modifier.contains(Modifier::DIM) {
            crossterm::execute!(self.writer, crossterm::style::SetAttribute(Attribute::Dim))?;
        }
        if modifier.contains(Modifier::ITALIC) {
            crossterm::execute!(self.writer, crossterm::style::SetAttribute(Attribute::Italic))?;
        }
        if modifier.contains(Modifier::UNDERLINED) {
            crossterm::execute!(
                self.writer,
                crossterm::style::SetAttribute(Attribute::Underlined)
            )?;
        }
        if modifier.contains(Modifier::SLOW_BLINK) {
            crossterm::execute!(
                self.writer,
                crossterm::style::SetAttribute(Attribute::SlowBlink)
            )?;
        }
        if modifier.contains(Modifier::RAPID_BLINK) {
            crossterm::execute!(
                self.writer,
                crossterm::style::SetAttribute(Attribute::RapidBlink)
            )?;
        }
        if modifier.contains(Modifier::REVERSED) {
            crossterm::execute!(
                self.writer,
                crossterm::style::SetAttribute(Attribute::Reverse)
            )?;
        }
        if modifier.contains(Modifier::HIDDEN) {
            crossterm::execute!(self.writer, crossterm::style::SetAttribute(Attribute::Hidden))?;
        }
        if modifier.contains(Modifier::CROSSED_OUT) {
            crossterm::execute!(
                self.writer,
                crossterm::style::SetAttribute(Attribute::CrossedOut)
            )?;
        }

        Ok(())
    }

    /// Renders a single cell at the specified position.
    fn render_cell(&mut self, x: u16, y: u16, cell: &Cell) -> Result<()> {
        use crossterm::{cursor::MoveTo, style::*, Command};

        // Move cursor to position
        MoveTo(x, y).execute(&mut self.writer)?;

        // Reset attributes
        crossterm::execute!(self.writer, SetAttribute(Attribute::Reset))?;

        // Set foreground color
        SetForegroundColor(Self::convert_color(cell.fg)).execute(&mut self.writer)?;

        // Set background color
        SetBackgroundColor(Self::convert_color(cell.bg)).execute(&mut self.writer)?;

        // Apply modifiers
        self.apply_modifiers(cell.modifier)?;

        // Write the symbol
        write!(self.writer, "{}", cell.symbol)?;

        Ok(())
    }
}

impl<W: Write + Send> Renderer for CrosstermRenderer<W> {
    fn draw(&mut self, buffer: &Buffer) -> Result<()> {
        // Check if buffer size matches our expected size
        if buffer.area != self.buffer.area {
            return Err(RenderError::SizeMismatch {
                expected: self.buffer.area,
                actual: buffer.area,
            });
        }

        // Compute the diff between the last buffer and the new buffer
        let updates = if let Some(ref last) = self.last_buffer {
            last.diff(buffer)
        } else {
            // First render - draw everything
            let mut all_cells = Vec::new();
            for y in 0..buffer.area.height {
                for x in 0..buffer.area.width {
                    if let Some(cell) = buffer.get(x, y) {
                        all_cells.push((x, y, cell));
                    }
                }
            }
            all_cells
        };

        // Render only the changed cells
        for (x, y, cell) in updates {
            self.render_cell(x, y, cell)?;
        }

        // Update our internal buffer and last_buffer
        self.buffer = buffer.clone();
        self.last_buffer = Some(buffer.clone());

        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        self.writer.flush()?;
        Ok(())
    }

    fn size(&self) -> Result<Rect> {
        let (width, height) = crossterm::terminal::size()
            .map_err(|e| RenderError::Backend(format!("Failed to get terminal size: {}", e)))?;
        Ok(Rect::new(0, 0, width, height))
    }

    fn clear(&mut self) -> Result<()> {
        use crossterm::{terminal::Clear, terminal::ClearType, Command};
        Clear(ClearType::All).execute(&mut self.writer)?;
        self.last_buffer = None;
        Ok(())
    }

    fn show_cursor(&mut self, show: bool) -> Result<()> {
        use crossterm::{cursor::Hide, cursor::Show, Command};
        if show {
            Show.execute(&mut self.writer)?;
        } else {
            Hide.execute(&mut self.writer)?;
        }
        Ok(())
    }

    fn set_cursor(&mut self, x: u16, y: u16) -> Result<()> {
        use crossterm::{cursor::MoveTo, Command};
        MoveTo(x, y).execute(&mut self.writer)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crossterm_renderer_new() {
        let mut buffer = Vec::new();
        let result = CrosstermRenderer::with_size(&mut buffer, 80, 24);
        assert_eq!(result.buffer.area.width, 80);
        assert_eq!(result.buffer.area.height, 24);
    }

    #[test]
    fn test_color_conversion() {
        use crossterm::style::Color as CColor;

        assert_eq!(
            CrosstermRenderer::<Vec<u8>>::convert_color(Color::Black),
            CColor::Black
        );
        assert_eq!(
            CrosstermRenderer::<Vec<u8>>::convert_color(Color::Red),
            CColor::DarkRed
        );
        assert_eq!(
            CrosstermRenderer::<Vec<u8>>::convert_color(Color::LightRed),
            CColor::Red
        );
        assert_eq!(
            CrosstermRenderer::<Vec<u8>>::convert_color(Color::Rgb(255, 128, 0)),
            CColor::Rgb {
                r: 255,
                g: 128,
                b: 0
            }
        );
    }

    #[test]
    fn test_clear() {
        let mut output = Vec::new();
        let mut renderer = CrosstermRenderer::with_size(&mut output, 10, 5);
        renderer.clear().unwrap();
        // Clear should invalidate last_buffer
        assert!(renderer.last_buffer.is_none());
    }

    #[test]
    fn test_size_mismatch() {
        let mut output = Vec::new();
        let mut renderer = CrosstermRenderer::with_size(&mut output, 10, 5);
        let buffer = Buffer::new(Rect::new(0, 0, 20, 10));

        let result = renderer.draw(&buffer);
        assert!(result.is_err());
        if let Err(RenderError::SizeMismatch { expected, actual }) = result {
            assert_eq!(expected, Rect::new(0, 0, 10, 5));
            assert_eq!(actual, Rect::new(0, 0, 20, 10));
        }
    }
}
