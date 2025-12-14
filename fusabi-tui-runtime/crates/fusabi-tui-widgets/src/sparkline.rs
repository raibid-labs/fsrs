//! Sparkline widget for inline mini-charts.
//!
//! This module provides a `Sparkline` widget that displays a small inline chart
//! using vertical bar characters to visualize data trends.

use fusabi_tui_core::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    symbols::bar,
};

use crate::widget::Widget;

/// Set of bar characters to use for rendering sparklines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SparklineBarSet {
    /// Unicode vertical bars (default): ▁▂▃▄▅▆▇█
    Unicode,
    /// ASCII characters: _.,:|
    Ascii,
}

impl SparklineBarSet {
    /// Returns the bar character for a given value (0-8).
    fn get_bar(&self, value: usize) -> &'static str {
        match self {
            Self::Unicode => {
                let index = value.min(8);
                bar::VERTICAL_BARS[index]
            }
            Self::Ascii => match value {
                0 => " ",
                1..=2 => "_",
                3..=4 => ".",
                5..=6 => ":",
                7 => "|",
                _ => "|",
            },
        }
    }
}

/// A sparkline widget for displaying inline mini-charts.
///
/// Sparklines are small, word-sized graphics that can be embedded in text to
/// show trends and variations in data.
///
/// # Examples
///
/// ```
/// use fusabi_tui_core::{buffer::Buffer, layout::Rect, style::{Color, Style}};
/// use fusabi_tui_widgets::{Sparkline, Widget};
///
/// let data = vec![1, 5, 3, 8, 2, 4, 7, 6];
/// let sparkline = Sparkline::default()
///     .data(&data)
///     .style(Style::default().fg(Color::Cyan))
///     .max(10);
///
/// let area = Rect::new(0, 0, 20, 1);
/// let mut buffer = Buffer::new(area);
/// sparkline.render(area, &mut buffer);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sparkline {
    data: Vec<u64>,
    max: Option<u64>,
    style: Style,
    bar_set: SparklineBarSet,
}

impl Default for Sparkline {
    fn default() -> Self {
        Self {
            data: Vec::new(),
            max: None,
            style: Style::default(),
            bar_set: SparklineBarSet::Unicode,
        }
    }
}

impl Sparkline {
    /// Creates a new sparkline with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the data points for the sparkline.
    pub fn data(mut self, data: &[u64]) -> Self {
        self.data = data.to_vec();
        self
    }

    /// Sets the maximum value for scaling.
    ///
    /// If not set, the maximum value in the data will be used.
    pub fn max(mut self, max: u64) -> Self {
        self.max = Some(max);
        self
    }

    /// Sets the style for the sparkline bars.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Sets the bar character set to use.
    pub fn bar_set(mut self, bar_set: SparklineBarSet) -> Self {
        self.bar_set = bar_set;
        self
    }

    /// Calculates the maximum value for scaling.
    fn calculate_max(&self) -> u64 {
        if let Some(max) = self.max {
            max
        } else {
            self.data.iter().copied().max().unwrap_or(1)
        }
    }

    /// Scales a data value to a bar index (0-8).
    fn scale_value(&self, value: u64, max: u64) -> usize {
        if max == 0 {
            return 0;
        }
        let ratio = value as f64 / max as f64;
        (ratio * 8.0).round() as usize
    }
}

impl Widget for Sparkline {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        if area.area() == 0 || self.data.is_empty() {
            return;
        }

        // Only use the first row
        let sparkline_area = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: 1,
        };

        let max_value = self.calculate_max();
        let mut x = sparkline_area.x;

        // Render each data point
        for &value in &self.data {
            if x >= sparkline_area.right() {
                break;
            }

            let bar_index = self.scale_value(value, max_value);
            let bar_char = self.bar_set.get_bar(bar_index);

            if let Some(cell) = buf.get_mut(x, sparkline_area.y) {
                cell.symbol = bar_char.to_string();
                cell.set_style(self.style);
            }

            x = x.saturating_add(1);
        }

        // Fill remaining space with empty bars
        while x < sparkline_area.right() {
            if let Some(cell) = buf.get_mut(x, sparkline_area.y) {
                cell.symbol = self.bar_set.get_bar(0).to_string();
                cell.set_style(self.style);
            }
            x = x.saturating_add(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fusabi_tui_core::style::Color;

    #[test]
    fn test_sparkline_new() {
        let sparkline = Sparkline::new();
        assert!(sparkline.data.is_empty());
        assert_eq!(sparkline.max, None);
        assert_eq!(sparkline.bar_set, SparklineBarSet::Unicode);
    }

    #[test]
    fn test_sparkline_data() {
        let data = vec![1, 2, 3, 4, 5];
        let sparkline = Sparkline::new().data(&data);
        assert_eq!(sparkline.data, data);
    }

    #[test]
    fn test_sparkline_max() {
        let sparkline = Sparkline::new().max(100);
        assert_eq!(sparkline.max, Some(100));
    }

    #[test]
    fn test_sparkline_calculate_max_from_data() {
        let sparkline = Sparkline::new().data(&[1, 5, 3, 8, 2]);
        assert_eq!(sparkline.calculate_max(), 8);
    }

    #[test]
    fn test_sparkline_calculate_max_explicit() {
        let sparkline = Sparkline::new().data(&[1, 5, 3, 8, 2]).max(10);
        assert_eq!(sparkline.calculate_max(), 10);
    }

    #[test]
    fn test_sparkline_calculate_max_empty() {
        let sparkline = Sparkline::new();
        assert_eq!(sparkline.calculate_max(), 1);
    }

    #[test]
    fn test_sparkline_scale_value() {
        let sparkline = Sparkline::new();
        assert_eq!(sparkline.scale_value(0, 10), 0);
        assert_eq!(sparkline.scale_value(5, 10), 4);
        assert_eq!(sparkline.scale_value(10, 10), 8);
    }

    #[test]
    fn test_sparkline_scale_value_zero_max() {
        let sparkline = Sparkline::new();
        assert_eq!(sparkline.scale_value(5, 0), 0);
    }

    #[test]
    fn test_sparkline_bar_set_unicode() {
        let bar_set = SparklineBarSet::Unicode;
        assert_eq!(bar_set.get_bar(0), bar::EMPTY);
        assert_eq!(bar_set.get_bar(4), bar::HALF);
        assert_eq!(bar_set.get_bar(8), bar::FULL);
    }

    #[test]
    fn test_sparkline_bar_set_ascii() {
        let bar_set = SparklineBarSet::Ascii;
        assert_eq!(bar_set.get_bar(0), " ");
        assert_eq!(bar_set.get_bar(1), "_");
        assert_eq!(bar_set.get_bar(3), ".");
        assert_eq!(bar_set.get_bar(5), ":");
        assert_eq!(bar_set.get_bar(7), "|");
        assert_eq!(bar_set.get_bar(8), "|");
    }

    #[test]
    fn test_sparkline_render_empty() {
        let sparkline = Sparkline::new();
        let area = Rect::new(0, 0, 10, 1);
        let mut buffer = Buffer::new(area);
        sparkline.render(area, &mut buffer);

        // All cells should be empty bars
        for x in 0..10 {
            let cell = buffer.get(x, 0).unwrap();
            assert_eq!(cell.symbol, bar::EMPTY);
        }
    }

    #[test]
    fn test_sparkline_render_with_data() {
        let data = vec![0, 4, 8];
        let sparkline = Sparkline::new()
            .data(&data)
            .max(8)
            .style(Style::default().fg(Color::Green));

        let area = Rect::new(0, 0, 10, 1);
        let mut buffer = Buffer::new(area);
        sparkline.render(area, &mut buffer);

        // Check first three bars
        assert_eq!(buffer.get(0, 0).unwrap().symbol, bar::EMPTY);
        assert_eq!(buffer.get(1, 0).unwrap().symbol, bar::HALF);
        assert_eq!(buffer.get(2, 0).unwrap().symbol, bar::FULL);

        // Check style
        assert_eq!(buffer.get(0, 0).unwrap().fg, Color::Green);
        assert_eq!(buffer.get(1, 0).unwrap().fg, Color::Green);
        assert_eq!(buffer.get(2, 0).unwrap().fg, Color::Green);

        // Remaining should be empty
        for x in 3..10 {
            assert_eq!(buffer.get(x, 0).unwrap().symbol, bar::EMPTY);
        }
    }

    #[test]
    fn test_sparkline_render_ascii() {
        let data = vec![0, 2, 5, 8];
        let sparkline = Sparkline::new()
            .data(&data)
            .max(8)
            .bar_set(SparklineBarSet::Ascii);

        let area = Rect::new(0, 0, 10, 1);
        let mut buffer = Buffer::new(area);
        sparkline.render(area, &mut buffer);

        assert_eq!(buffer.get(0, 0).unwrap().symbol, " ");
        assert_eq!(buffer.get(1, 0).unwrap().symbol, "_");
        assert_eq!(buffer.get(2, 0).unwrap().symbol, ":");
        assert_eq!(buffer.get(3, 0).unwrap().symbol, "|");
    }

    #[test]
    fn test_sparkline_render_truncated() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let sparkline = Sparkline::new().data(&data);

        let area = Rect::new(0, 0, 5, 1); // Only 5 cells wide
        let mut buffer = Buffer::new(area);
        sparkline.render(area, &mut buffer);

        // Should only render first 5 values
        for x in 0..5 {
            let cell = buffer.get(x, 0).unwrap();
            // All should have some bar (not error)
            assert!(!cell.symbol.is_empty());
        }
    }

    #[test]
    fn test_sparkline_auto_scale() {
        let data = vec![10, 20, 30];
        let sparkline = Sparkline::new().data(&data);

        // Should auto-scale to max value of 30
        assert_eq!(sparkline.calculate_max(), 30);

        let area = Rect::new(0, 0, 10, 1);
        let mut buffer = Buffer::new(area);
        sparkline.render(area, &mut buffer);

        // Values should be scaled relative to 30
        // 10/30 ≈ 0.33 -> bar index ~3
        // 20/30 ≈ 0.67 -> bar index ~5
        // 30/30 = 1.0 -> bar index 8 (full)
        let cell2 = buffer.get(2, 0).unwrap();
        assert_eq!(cell2.symbol, bar::FULL);
    }
}
