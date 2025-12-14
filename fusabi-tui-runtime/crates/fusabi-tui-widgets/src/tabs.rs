//! Tabs widget for tab navigation.
//!
//! This module provides a `Tabs` widget that displays a list of tab titles with
//! highlighting for the selected tab and customizable dividers.

use fusabi_tui_core::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
};

use crate::widget::Widget;

/// A tabs widget for displaying tab navigation.
///
/// Shows a horizontal list of tab titles with the selected tab highlighted.
/// Tabs are separated by a divider character.
///
/// # Examples
///
/// ```
/// use fusabi_tui_core::{buffer::Buffer, layout::Rect, style::{Color, Style}};
/// use fusabi_tui_widgets::{Tabs, Widget};
///
/// let tabs = Tabs::new(vec!["Tab 1", "Tab 2", "Tab 3"])
///     .select(1)
///     .style(Style::default().fg(Color::White))
///     .highlight_style(Style::default().fg(Color::Yellow))
///     .divider("|");
///
/// let area = Rect::new(0, 0, 40, 1);
/// let mut buffer = Buffer::new(area);
/// tabs.render(area, &mut buffer);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tabs {
    titles: Vec<String>,
    selected: usize,
    style: Style,
    highlight_style: Style,
    divider: String,
}

impl Tabs {
    /// Creates a new tabs widget with the given titles.
    pub fn new<T>(titles: Vec<T>) -> Self
    where
        T: Into<String>,
    {
        Self {
            titles: titles.into_iter().map(Into::into).collect(),
            selected: 0,
            style: Style::default(),
            highlight_style: Style::default(),
            divider: " ".to_string(),
        }
    }

    /// Sets the index of the selected tab.
    pub fn select(mut self, index: usize) -> Self {
        self.selected = index;
        self
    }

    /// Sets the default style for unselected tabs.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Sets the style for the selected tab.
    pub fn highlight_style(mut self, style: Style) -> Self {
        self.highlight_style = style;
        self
    }

    /// Sets the divider string between tabs.
    pub fn divider<T>(mut self, divider: T) -> Self
    where
        T: Into<String>,
    {
        self.divider = divider.into();
        self
    }

    /// Returns the titles of all tabs.
    pub fn titles(&self) -> &[String] {
        &self.titles
    }

    /// Returns the index of the selected tab.
    pub fn selected(&self) -> usize {
        self.selected
    }
}

impl Widget for Tabs {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        if area.area() == 0 || self.titles.is_empty() {
            return;
        }

        // Only use the first row
        let tabs_area = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: 1,
        };

        // Clear the area with the base style
        for x in tabs_area.left()..tabs_area.right() {
            if let Some(cell) = buf.get_mut(x, tabs_area.y) {
                cell.symbol = " ".to_string();
                cell.set_style(self.style);
            }
        }

        let mut x = tabs_area.x;

        // Render each tab
        for (i, title) in self.titles.iter().enumerate() {
            // Check if we have space
            if x >= tabs_area.right() {
                break;
            }

            // Determine style for this tab
            let tab_style = if i == self.selected {
                self.highlight_style
            } else {
                self.style
            };

            // Calculate how much space we have left
            let remaining_width = tabs_area.right().saturating_sub(x) as usize;

            // Render the tab title (truncate if necessary)
            let mut current_width = 0;
            for ch in title.chars() {
                if x >= tabs_area.right() {
                    break;
                }
                let ch_width = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
                if current_width + ch_width > remaining_width {
                    break;
                }

                if let Some(cell) = buf.get_mut(x, tabs_area.y) {
                    cell.symbol = ch.to_string();
                    cell.set_style(tab_style);
                }

                x = x.saturating_add(1);
                current_width += ch_width;
            }

            // Add divider after tab (except for the last tab)
            if i < self.titles.len() - 1 {
                let divider_remaining = tabs_area.right().saturating_sub(x) as usize;
                let mut divider_current = 0;

                for ch in self.divider.chars() {
                    if x >= tabs_area.right() {
                        break;
                    }
                    let ch_width = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
                    if divider_current + ch_width > divider_remaining {
                        break;
                    }

                    if let Some(cell) = buf.get_mut(x, tabs_area.y) {
                        cell.symbol = ch.to_string();
                        cell.set_style(self.style);
                    }

                    x = x.saturating_add(1);
                    divider_current += ch_width;
                }
            }
        }

        // Fill remaining space with default style
        while x < tabs_area.right() {
            if let Some(cell) = buf.get_mut(x, tabs_area.y) {
                cell.symbol = " ".to_string();
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
    fn test_tabs_new() {
        let tabs = Tabs::new(vec!["Tab 1", "Tab 2", "Tab 3"]);
        assert_eq!(tabs.titles.len(), 3);
        assert_eq!(tabs.selected, 0);
        assert_eq!(tabs.divider, " ");
    }

    #[test]
    fn test_tabs_select() {
        let tabs = Tabs::new(vec!["A", "B", "C"]).select(2);
        assert_eq!(tabs.selected, 2);
    }

    #[test]
    fn test_tabs_style() {
        let style = Style::default().fg(Color::White);
        let tabs = Tabs::new(vec!["A", "B"]).style(style);
        assert_eq!(tabs.style, style);
    }

    #[test]
    fn test_tabs_highlight_style() {
        let style = Style::default().fg(Color::Yellow);
        let tabs = Tabs::new(vec!["A", "B"]).highlight_style(style);
        assert_eq!(tabs.highlight_style, style);
    }

    #[test]
    fn test_tabs_divider() {
        let tabs = Tabs::new(vec!["A", "B"]).divider("|");
        assert_eq!(tabs.divider, "|");
    }

    #[test]
    fn test_tabs_titles() {
        let tabs = Tabs::new(vec!["Tab 1", "Tab 2"]);
        assert_eq!(tabs.titles(), &["Tab 1", "Tab 2"]);
    }

    #[test]
    fn test_tabs_selected() {
        let tabs = Tabs::new(vec!["A", "B", "C"]).select(1);
        assert_eq!(tabs.selected(), 1);
    }

    #[test]
    fn test_tabs_render_empty() {
        let tabs = Tabs::new::<String>(vec![]);
        let area = Rect::new(0, 0, 20, 1);
        let mut buffer = Buffer::new(area);
        tabs.render(area, &mut buffer);

        // Should just fill with spaces
        for x in 0..20 {
            assert_eq!(buffer.get(x, 0).unwrap().symbol, " ");
        }
    }

    #[test]
    fn test_tabs_render_single() {
        let tabs = Tabs::new(vec!["Single"])
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().fg(Color::Yellow));

        let area = Rect::new(0, 0, 20, 1);
        let mut buffer = Buffer::new(area);
        tabs.render(area, &mut buffer);

        // Check the tab title
        assert_eq!(buffer.get(0, 0).unwrap().symbol, "S");
        assert_eq!(buffer.get(1, 0).unwrap().symbol, "i");
        assert_eq!(buffer.get(5, 0).unwrap().symbol, "e");

        // Should be highlighted (selected by default)
        assert_eq!(buffer.get(0, 0).unwrap().fg, Color::Yellow);
    }

    #[test]
    fn test_tabs_render_multiple() {
        let tabs = Tabs::new(vec!["Tab1", "Tab2", "Tab3"])
            .select(1)
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().fg(Color::Green))
            .divider("|");

        let area = Rect::new(0, 0, 30, 1);
        let mut buffer = Buffer::new(area);
        tabs.render(area, &mut buffer);

        // First tab should not be highlighted
        assert_eq!(buffer.get(0, 0).unwrap().symbol, "T");
        assert_eq!(buffer.get(0, 0).unwrap().fg, Color::White);

        // Divider after first tab
        assert_eq!(buffer.get(4, 0).unwrap().symbol, "|");

        // Second tab should be highlighted
        assert_eq!(buffer.get(5, 0).unwrap().symbol, "T");
        assert_eq!(buffer.get(5, 0).unwrap().fg, Color::Green);

        // Divider after second tab
        assert_eq!(buffer.get(9, 0).unwrap().symbol, "|");

        // Third tab should not be highlighted
        assert_eq!(buffer.get(10, 0).unwrap().symbol, "T");
        assert_eq!(buffer.get(10, 0).unwrap().fg, Color::White);
    }

    #[test]
    fn test_tabs_render_with_spaces_divider() {
        let tabs = Tabs::new(vec!["A", "B", "C"])
            .divider(" | ");

        let area = Rect::new(0, 0, 20, 1);
        let mut buffer = Buffer::new(area);
        tabs.render(area, &mut buffer);

        // "A | B | C"
        assert_eq!(buffer.get(0, 0).unwrap().symbol, "A");
        assert_eq!(buffer.get(1, 0).unwrap().symbol, " ");
        assert_eq!(buffer.get(2, 0).unwrap().symbol, "|");
        assert_eq!(buffer.get(3, 0).unwrap().symbol, " ");
        assert_eq!(buffer.get(4, 0).unwrap().symbol, "B");
    }

    #[test]
    fn test_tabs_render_truncated() {
        let tabs = Tabs::new(vec!["VeryLongTabName1", "VeryLongTabName2"])
            .divider("|");

        let area = Rect::new(0, 0, 10, 1); // Only 10 cells wide
        let mut buffer = Buffer::new(area);
        tabs.render(area, &mut buffer);

        // Should truncate the first tab
        assert_eq!(buffer.get(0, 0).unwrap().symbol, "V");
        // Should not overflow
        for x in 0..10 {
            let cell = buffer.get(x, 0).unwrap();
            assert!(!cell.symbol.is_empty() || cell.symbol == " ");
        }
    }

    #[test]
    fn test_tabs_render_out_of_bounds_selection() {
        let tabs = Tabs::new(vec!["A", "B"]).select(5); // Out of bounds

        let area = Rect::new(0, 0, 10, 1);
        let mut buffer = Buffer::new(area);
        tabs.render(area, &mut buffer);

        // Should render without panic, no tab will be highlighted
        assert_eq!(buffer.get(0, 0).unwrap().symbol, "A");
        assert_eq!(buffer.get(0, 0).unwrap().fg, Color::Reset); // Default style
    }

    #[test]
    fn test_tabs_render_unicode_divider() {
        let tabs = Tabs::new(vec!["Tab1", "Tab2"])
            .divider(" │ ");

        let area = Rect::new(0, 0, 20, 1);
        let mut buffer = Buffer::new(area);
        tabs.render(area, &mut buffer);

        // Should handle unicode divider correctly
        assert_eq!(buffer.get(0, 0).unwrap().symbol, "T");
        assert_eq!(buffer.get(5, 0).unwrap().symbol, "│");
    }
}
