//! Gauge widget for displaying progress or percentage bars.
//!
//! This module provides a `Gauge` widget that visualizes progress using a horizontal bar
//! with customizable filled and unfilled styles.

use fusabi_tui_core::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    symbols::block,
};
use unicode_width::UnicodeWidthStr;

use crate::widget::Widget;

/// Set of characters used to render a gauge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GaugeCharSet {
    /// Full block character for filled portion
    Full,
    /// Vertical bar characters with varying widths
    VerticalBars,
}

impl GaugeCharSet {
    /// Returns the character to use for the filled portion.
    fn filled_char(self) -> &'static str {
        match self {
            Self::Full => block::FULL,
            Self::VerticalBars => block::FULL,
        }
    }

    /// Returns the character to use for the empty portion.
    fn empty_char(self) -> &'static str {
        " "
    }
}

/// A gauge widget for displaying progress as a horizontal bar.
///
/// The gauge can display either a ratio (0.0 to 1.0) or a percentage (0 to 100),
/// with an optional label displayed in the center.
///
/// # Examples
///
/// ```
/// use fusabi_tui_core::{buffer::Buffer, layout::Rect, style::{Color, Style}};
/// use fusabi_tui_widgets::{Gauge, Widget};
///
/// let gauge = Gauge::default()
///     .percent(75)
///     .label("75%")
///     .style(Style::default().fg(Color::White))
///     .gauge_style(Style::default().fg(Color::Green));
///
/// let area = Rect::new(0, 0, 40, 1);
/// let mut buffer = Buffer::new(area);
/// gauge.render(area, &mut buffer);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Gauge {
    ratio: f64,
    label: Option<String>,
    style: Style,
    gauge_style: Style,
    char_set: GaugeCharSet,
}

impl Default for Gauge {
    fn default() -> Self {
        Self {
            ratio: 0.0,
            label: None,
            style: Style::default(),
            gauge_style: Style::default(),
            char_set: GaugeCharSet::Full,
        }
    }
}

impl Gauge {
    /// Creates a new gauge with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the ratio of the gauge (0.0 to 1.0).
    ///
    /// Values outside this range will be clamped.
    pub fn ratio(mut self, ratio: f64) -> Self {
        self.ratio = ratio.clamp(0.0, 1.0);
        self
    }

    /// Sets the percentage of the gauge (0 to 100).
    ///
    /// Values outside this range will be clamped.
    pub fn percent(mut self, percent: u16) -> Self {
        let percent = percent.min(100);
        self.ratio = f64::from(percent) / 100.0;
        self
    }

    /// Sets the label to display in the center of the gauge.
    pub fn label<T>(mut self, label: T) -> Self
    where
        T: Into<String>,
    {
        self.label = Some(label.into());
        self
    }

    /// Sets the default style for the gauge (unfilled portion).
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Sets the style for the filled portion of the gauge.
    pub fn gauge_style(mut self, style: Style) -> Self {
        self.gauge_style = style;
        self
    }

    /// Sets the character set to use for rendering.
    pub fn char_set(mut self, char_set: GaugeCharSet) -> Self {
        self.char_set = char_set;
        self
    }

    /// Calculates the width of the filled portion.
    fn filled_width(&self, total_width: u16) -> u16 {
        (f64::from(total_width) * self.ratio).round() as u16
    }
}

impl Widget for Gauge {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        if area.area() == 0 {
            return;
        }

        // Only use the first row
        let gauge_area = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: 1,
        };

        // Clear the area with the base style
        for x in gauge_area.left()..gauge_area.right() {
            if let Some(cell) = buf.get_mut(x, gauge_area.y) {
                cell.symbol = " ".to_string();
                cell.set_style(self.style);
            }
        }

        // Calculate filled width
        let filled_width = self.filled_width(gauge_area.width);

        // Render filled portion
        let filled_char = self.char_set.filled_char();
        for i in 0..filled_width {
            let x = gauge_area.x.saturating_add(i);
            if x >= gauge_area.right() {
                break;
            }
            if let Some(cell) = buf.get_mut(x, gauge_area.y) {
                cell.symbol = filled_char.to_string();
                cell.set_style(self.gauge_style);
            }
        }

        // Render unfilled portion
        let empty_char = self.char_set.empty_char();
        for i in filled_width..gauge_area.width {
            let x = gauge_area.x.saturating_add(i);
            if x >= gauge_area.right() {
                break;
            }
            if let Some(cell) = buf.get_mut(x, gauge_area.y) {
                cell.symbol = empty_char.to_string();
                cell.set_style(self.style);
            }
        }

        // Render label in the center if present
        if let Some(ref label) = self.label {
            let label_width = label.width();
            if label_width <= gauge_area.width as usize {
                let label_x = gauge_area.x.saturating_add(
                    (gauge_area.width.saturating_sub(label_width as u16)) / 2,
                );

                for (i, ch) in label.chars().enumerate() {
                    let x = label_x.saturating_add(i as u16);
                    if x >= gauge_area.right() {
                        break;
                    }
                    if let Some(cell) = buf.get_mut(x, gauge_area.y) {
                        cell.symbol = ch.to_string();
                        // Keep the background style but use label foreground
                        // Determine if this position is in the filled or unfilled area
                        if x < gauge_area.x.saturating_add(filled_width) {
                            // In filled area - use gauge style background with inverted foreground
                            let mut label_style = self.gauge_style;
                            if let Some(fg) = self.style.fg {
                                label_style.fg = Some(fg);
                            }
                            cell.set_style(label_style);
                        } else {
                            // In unfilled area - use normal style
                            let mut label_style = self.style;
                            if let Some(fg) = self.gauge_style.fg {
                                label_style.fg = Some(fg);
                            }
                            cell.set_style(label_style);
                        }
                    }
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
    fn test_gauge_new() {
        let gauge = Gauge::new();
        assert_eq!(gauge.ratio, 0.0);
        assert_eq!(gauge.label, None);
    }

    #[test]
    fn test_gauge_ratio() {
        let gauge = Gauge::new().ratio(0.5);
        assert_eq!(gauge.ratio, 0.5);
    }

    #[test]
    fn test_gauge_ratio_clamping() {
        let gauge1 = Gauge::new().ratio(-0.1);
        assert_eq!(gauge1.ratio, 0.0);

        let gauge2 = Gauge::new().ratio(1.5);
        assert_eq!(gauge2.ratio, 1.0);
    }

    #[test]
    fn test_gauge_percent() {
        let gauge = Gauge::new().percent(75);
        assert!((gauge.ratio - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    fn test_gauge_percent_clamping() {
        let gauge = Gauge::new().percent(150);
        assert_eq!(gauge.ratio, 1.0);
    }

    #[test]
    fn test_gauge_label() {
        let gauge = Gauge::new().label("50%");
        assert_eq!(gauge.label, Some("50%".to_string()));
    }

    #[test]
    fn test_gauge_filled_width() {
        let gauge = Gauge::new().ratio(0.5);
        assert_eq!(gauge.filled_width(100), 50);

        let gauge = Gauge::new().ratio(0.75);
        assert_eq!(gauge.filled_width(100), 75);
    }

    #[test]
    fn test_gauge_render_empty() {
        let gauge = Gauge::new().ratio(0.0);
        let area = Rect::new(0, 0, 10, 1);
        let mut buffer = Buffer::new(area);
        gauge.render(area, &mut buffer);

        // All cells should be empty
        for x in 0..10 {
            let cell = buffer.get(x, 0).unwrap();
            assert_eq!(cell.symbol, " ");
        }
    }

    #[test]
    fn test_gauge_render_full() {
        let gauge = Gauge::new().ratio(1.0).gauge_style(Style::default().fg(Color::Green));
        let area = Rect::new(0, 0, 10, 1);
        let mut buffer = Buffer::new(area);
        gauge.render(area, &mut buffer);

        // All cells should be filled
        for x in 0..10 {
            let cell = buffer.get(x, 0).unwrap();
            assert_eq!(cell.symbol, block::FULL);
            assert_eq!(cell.fg, Color::Green);
        }
    }

    #[test]
    fn test_gauge_render_half() {
        let gauge = Gauge::new().ratio(0.5).gauge_style(Style::default().fg(Color::Blue));
        let area = Rect::new(0, 0, 10, 1);
        let mut buffer = Buffer::new(area);
        gauge.render(area, &mut buffer);

        // First half filled, second half empty
        for x in 0..5 {
            let cell = buffer.get(x, 0).unwrap();
            assert_eq!(cell.symbol, block::FULL);
            assert_eq!(cell.fg, Color::Blue);
        }
        for x in 5..10 {
            let cell = buffer.get(x, 0).unwrap();
            assert_eq!(cell.symbol, " ");
        }
    }

    #[test]
    fn test_gauge_render_with_label() {
        let gauge = Gauge::new()
            .ratio(0.5)
            .label("50%")
            .style(Style::default().fg(Color::White))
            .gauge_style(Style::default().fg(Color::Green));

        let area = Rect::new(0, 0, 20, 1);
        let mut buffer = Buffer::new(area);
        gauge.render(area, &mut buffer);

        // Label should be centered (position 8-11 for "50%" in width 20)
        // Characters before and after should be gauge fill
        let label_start = (20 - 3) / 2; // 8
        assert_eq!(buffer.get(label_start, 0).unwrap().symbol, "5");
        assert_eq!(buffer.get(label_start + 1, 0).unwrap().symbol, "0");
        assert_eq!(buffer.get(label_start + 2, 0).unwrap().symbol, "%");
    }

    #[test]
    fn test_gauge_char_set() {
        let gauge1 = Gauge::new().char_set(GaugeCharSet::Full);
        assert_eq!(gauge1.char_set, GaugeCharSet::Full);

        let gauge2 = Gauge::new().char_set(GaugeCharSet::VerticalBars);
        assert_eq!(gauge2.char_set, GaugeCharSet::VerticalBars);
    }
}
