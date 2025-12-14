//! Table widget for displaying tabular data.
//!
//! This module provides a `Table` widget that can display data in rows and columns
//! with support for headers, column width constraints, and row selection.

use fusabi_tui_core::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::Style,
};
use unicode_width::UnicodeWidthStr;

use crate::widget::{StatefulWidget, Widget};

/// A cell within a table row.
///
/// Contains the content to display and optional styling.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TableCell {
    content: String,
    style: Style,
}

impl TableCell {
    /// Creates a new table cell with the given content.
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            style: Style::default(),
        }
    }

    /// Sets the style for this cell.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Returns the content of this cell.
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Returns the width of the cell content in columns.
    pub fn width(&self) -> usize {
        self.content.width()
    }
}

impl<T> From<T> for TableCell
where
    T: Into<String>,
{
    fn from(content: T) -> Self {
        Self::new(content)
    }
}

/// A row in a table.
///
/// Contains a sequence of cells with optional height and style.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Row {
    cells: Vec<TableCell>,
    height: u16,
    style: Style,
    bottom_margin: u16,
}

impl Row {
    /// Creates a new row with the given cells.
    pub fn new<T>(cells: Vec<T>) -> Self
    where
        T: Into<TableCell>,
    {
        Self {
            cells: cells.into_iter().map(Into::into).collect(),
            height: 1,
            style: Style::default(),
            bottom_margin: 0,
        }
    }

    /// Sets the height of this row.
    pub fn height(mut self, height: u16) -> Self {
        self.height = height;
        self
    }

    /// Sets the style for this row.
    ///
    /// This style will be applied to all cells that don't have their own style.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Sets the bottom margin for this row.
    pub fn bottom_margin(mut self, margin: u16) -> Self {
        self.bottom_margin = margin;
        self
    }

    /// Returns the cells in this row.
    pub fn cells(&self) -> &[TableCell] {
        &self.cells
    }

    /// Returns the total height including margins.
    pub fn total_height(&self) -> u16 {
        self.height.saturating_add(self.bottom_margin)
    }
}

/// State for a stateful table widget.
///
/// Tracks the selected row and scroll offset.
#[derive(Debug, Clone, Default)]
pub struct TableState {
    selected: Option<usize>,
    offset: usize,
}

impl TableState {
    /// Creates a new table state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the index of the selected row.
    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    /// Selects a row by index.
    pub fn select(&mut self, index: Option<usize>) {
        self.selected = index;
    }

    /// Returns the scroll offset.
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Sets the scroll offset.
    pub fn set_offset(&mut self, offset: usize) {
        self.offset = offset;
    }
}

/// A table widget for displaying tabular data.
///
/// # Examples
///
/// ```
/// use fusabi_tui_core::{buffer::Buffer, layout::{Constraint, Rect}, style::{Color, Style}};
/// use fusabi_tui_widgets::{Table, Row, StatefulWidget};
/// use fusabi_tui_widgets::table::TableState;
///
/// let mut state = TableState::default();
/// state.select(Some(0));
///
/// let table = Table::new(vec![
///     Row::new(vec!["Row 1 Col 1", "Row 1 Col 2"]),
///     Row::new(vec!["Row 2 Col 1", "Row 2 Col 2"]),
/// ])
/// .header(Row::new(vec!["Header 1", "Header 2"]).style(Style::default().fg(Color::Yellow)))
/// .widths(&[Constraint::Length(15), Constraint::Length(15)])
/// .column_spacing(1)
/// .highlight_style(Style::default().fg(Color::Green));
///
/// let area = Rect::new(0, 0, 40, 10);
/// let mut buffer = Buffer::new(area);
/// table.render(area, &mut buffer, &mut state);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    rows: Vec<Row>,
    header: Option<Row>,
    widths: Vec<Constraint>,
    column_spacing: u16,
    style: Style,
    highlight_style: Style,
}

impl Table {
    /// Creates a new table with the given rows.
    pub fn new<T>(rows: Vec<T>) -> Self
    where
        T: Into<Row>,
    {
        Self {
            rows: rows.into_iter().map(Into::into).collect(),
            header: None,
            widths: Vec::new(),
            column_spacing: 1,
            style: Style::default(),
            highlight_style: Style::default(),
        }
    }

    /// Sets the header row for the table.
    pub fn header(mut self, header: Row) -> Self {
        self.header = Some(header);
        self
    }

    /// Sets the column width constraints.
    pub fn widths(mut self, widths: &[Constraint]) -> Self {
        self.widths = widths.to_vec();
        self
    }

    /// Sets the spacing between columns.
    pub fn column_spacing(mut self, spacing: u16) -> Self {
        self.column_spacing = spacing;
        self
    }

    /// Sets the default style for the table.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Sets the highlight style for the selected row.
    pub fn highlight_style(mut self, style: Style) -> Self {
        self.highlight_style = style;
        self
    }

    /// Calculates column widths based on constraints and available space.
    fn calculate_widths(&self, max_width: u16) -> Vec<u16> {
        if self.widths.is_empty() {
            return Vec::new();
        }

        // Account for column spacing
        let spacing_width = self.column_spacing.saturating_mul(self.widths.len().saturating_sub(1) as u16);
        let available = max_width.saturating_sub(spacing_width);

        // Calculate widths based on constraints
        let mut widths = vec![0u16; self.widths.len()];
        let mut remaining = available;
        let mut fill_count = 0;

        // First pass: calculate non-Fill constraints
        for (i, constraint) in self.widths.iter().enumerate() {
            match constraint {
                Constraint::Fill(_) => {
                    fill_count += 1;
                }
                Constraint::Percentage(_) | Constraint::Ratio(_, _) => {
                    let width = match constraint {
                        Constraint::Percentage(p) => {
                            let p = (*p).min(100);
                            (available as u32 * p as u32 / 100) as u16
                        }
                        Constraint::Ratio(n, d) => {
                            if *d == 0 {
                                0
                            } else {
                                (available as u32 * n / d) as u16
                            }
                        }
                        _ => 0,
                    };
                    widths[i] = width;
                    remaining = remaining.saturating_sub(width);
                }
                Constraint::Length(l) => {
                    let width = (*l).min(remaining);
                    widths[i] = width;
                    remaining = remaining.saturating_sub(width);
                }
                Constraint::Min(m) | Constraint::Max(m) => {
                    let width = (*m).min(remaining);
                    widths[i] = width;
                    remaining = remaining.saturating_sub(width);
                }
            }
        }

        // Second pass: distribute remaining space among Fill constraints
        if fill_count > 0 && remaining > 0 {
            let fill_width = remaining / fill_count as u16;
            let remainder = remaining % fill_count as u16;
            let mut remainder_distributed = 0;

            for (i, constraint) in self.widths.iter().enumerate() {
                if matches!(constraint, Constraint::Fill(_)) {
                    widths[i] = fill_width;
                    if remainder_distributed < remainder {
                        widths[i] += 1;
                        remainder_distributed += 1;
                    }
                }
            }
        }

        widths
    }

    /// Renders a single row at the given position.
    fn render_row(
        &self,
        buf: &mut Buffer,
        area: Rect,
        row: &Row,
        widths: &[u16],
        style: Style,
    ) {
        let mut x = area.x;
        let y = area.y;

        for (i, cell) in row.cells().iter().enumerate() {
            if i >= widths.len() {
                break;
            }

            let width = widths[i];
            if x >= area.right() || width == 0 {
                break;
            }

            // Determine cell style
            let cell_style = if cell.style != Style::default() {
                cell.style
            } else if row.style != Style::default() {
                row.style
            } else {
                style
            };

            // Render cell content
            let content = cell.content();
            let display_width = width as usize;

            // Truncate content if it's too wide
            let mut truncated = String::new();
            let mut current_width = 0;
            for ch in content.chars() {
                let ch_width = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
                if current_width + ch_width > display_width {
                    break;
                }
                truncated.push(ch);
                current_width += ch_width;
            }

            buf.set_string(x, y, &truncated, cell_style);

            x = x.saturating_add(width).saturating_add(self.column_spacing);
        }
    }
}

impl StatefulWidget for Table {
    type State = TableState;

    fn render(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if area.area() == 0 {
            return;
        }

        // Apply default style to the entire area
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                if let Some(cell) = buf.get_mut(x, y) {
                    cell.set_style(self.style);
                }
            }
        }

        let widths = self.calculate_widths(area.width);
        if widths.is_empty() {
            return;
        }

        let mut y = area.y;

        // Render header if present
        if let Some(ref header) = self.header {
            if y < area.bottom() {
                let header_area = Rect {
                    x: area.x,
                    y,
                    width: area.width,
                    height: header.height.min(area.bottom().saturating_sub(y)),
                };
                self.render_row(buf, header_area, header, &widths, self.style);
                y = y.saturating_add(header.total_height());
            }
        }

        // Render rows
        let mut row_index = state.offset;
        while row_index < self.rows.len() && y < area.bottom() {
            let row = &self.rows[row_index];
            let row_height = row.height.min(area.bottom().saturating_sub(y));

            if row_height == 0 {
                break;
            }

            let row_area = Rect {
                x: area.x,
                y,
                width: area.width,
                height: row_height,
            };

            // Determine row style (highlight if selected)
            let row_style = if Some(row_index) == state.selected {
                self.highlight_style
            } else {
                self.style
            };

            // Apply highlight to entire row if selected
            if Some(row_index) == state.selected {
                for row_y in row_area.top()..row_area.bottom() {
                    for row_x in row_area.left()..row_area.right() {
                        if let Some(cell) = buf.get_mut(row_x, row_y) {
                            cell.set_style(self.highlight_style);
                        }
                    }
                }
            }

            self.render_row(buf, row_area, row, &widths, row_style);
            y = y.saturating_add(row.total_height());
            row_index += 1;
        }
    }
}

impl Widget for Table {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let mut state = TableState::default();
        StatefulWidget::render(self, area, buf, &mut state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fusabi_tui_core::style::Color;

    #[test]
    fn test_table_cell_new() {
        let cell = TableCell::new("test");
        assert_eq!(cell.content(), "test");
        assert_eq!(cell.style, Style::default());
    }

    #[test]
    fn test_table_cell_width() {
        let cell = TableCell::new("hello");
        assert_eq!(cell.width(), 5);
    }

    #[test]
    fn test_table_cell_style() {
        let style = Style::default().fg(Color::Red);
        let cell = TableCell::new("test").style(style);
        assert_eq!(cell.style, style);
    }

    #[test]
    fn test_row_new() {
        let row = Row::new(vec!["a", "b", "c"]);
        assert_eq!(row.cells().len(), 3);
        assert_eq!(row.height, 1);
        assert_eq!(row.bottom_margin, 0);
    }

    #[test]
    fn test_row_total_height() {
        let row = Row::new(vec!["test"]).height(2).bottom_margin(1);
        assert_eq!(row.total_height(), 3);
    }

    #[test]
    fn test_table_state() {
        let mut state = TableState::new();
        assert_eq!(state.selected(), None);
        assert_eq!(state.offset(), 0);

        state.select(Some(5));
        assert_eq!(state.selected(), Some(5));

        state.set_offset(10);
        assert_eq!(state.offset(), 10);
    }

    #[test]
    fn test_table_new() {
        let table = Table::new(vec![
            Row::new(vec!["a", "b"]),
            Row::new(vec!["c", "d"]),
        ]);
        assert_eq!(table.rows.len(), 2);
        assert_eq!(table.header, None);
    }

    #[test]
    fn test_table_with_header() {
        let table = Table::new(vec![Row::new(vec!["a", "b"])])
            .header(Row::new(vec!["H1", "H2"]));
        assert!(table.header.is_some());
    }

    #[test]
    fn test_table_calculate_widths_length() {
        let table = Table::new(vec![Row::new(vec!["a", "b"])])
            .widths(&[Constraint::Length(10), Constraint::Length(20)]);
        let widths = table.calculate_widths(100);
        assert_eq!(widths, vec![10, 20]);
    }

    #[test]
    fn test_table_calculate_widths_percentage() {
        let table = Table::new(vec![Row::new(vec!["a", "b"])])
            .widths(&[Constraint::Percentage(50), Constraint::Percentage(50)])
            .column_spacing(0);
        let widths = table.calculate_widths(100);
        assert_eq!(widths, vec![50, 50]);
    }

    #[test]
    fn test_table_calculate_widths_fill() {
        let table = Table::new(vec![Row::new(vec!["a", "b", "c"])])
            .widths(&[
                Constraint::Length(10),
                Constraint::Fill(1),
                Constraint::Fill(1),
            ])
            .column_spacing(0);
        let widths = table.calculate_widths(100);
        assert_eq!(widths, vec![10, 45, 45]);
    }

    #[test]
    fn test_table_render() {
        let table = Table::new(vec![
            Row::new(vec!["a", "b"]),
            Row::new(vec!["c", "d"]),
        ])
        .widths(&[Constraint::Length(5), Constraint::Length(5)]);

        let area = Rect::new(0, 0, 20, 10);
        let mut buffer = Buffer::new(area);
        let mut state = TableState::default();

        StatefulWidget::render(&table, area, &mut buffer, &mut state);

        // Check that cells were written
        assert_eq!(buffer.get(0, 0).unwrap().symbol, "a");
        assert_eq!(buffer.get(0, 1).unwrap().symbol, "c");
    }

    #[test]
    fn test_table_render_with_selection() {
        let highlight_style = Style::default().fg(Color::Green);
        let table = Table::new(vec![
            Row::new(vec!["a", "b"]),
            Row::new(vec!["c", "d"]),
        ])
        .widths(&[Constraint::Length(5), Constraint::Length(5)])
        .highlight_style(highlight_style);

        let area = Rect::new(0, 0, 20, 10);
        let mut buffer = Buffer::new(area);
        let mut state = TableState::default();
        state.select(Some(1));

        StatefulWidget::render(&table, area, &mut buffer, &mut state);

        // Second row should have highlight style
        let cell = buffer.get(0, 1).unwrap();
        assert_eq!(cell.fg, Color::Green);
    }
}
