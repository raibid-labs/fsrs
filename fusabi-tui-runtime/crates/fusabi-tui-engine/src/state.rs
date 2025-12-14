//! Dashboard state management for the Fusabi TUI engine.

use std::any::Any;
use std::collections::HashMap;

/// Represents the state of the entire dashboard.
#[derive(Debug)]
pub struct DashboardState {
    /// Widget states keyed by widget ID.
    pub widgets: HashMap<String, WidgetState>,

    /// The currently focused widget ID, if any.
    pub focus: Option<String>,

    /// Flag indicating if the dashboard needs re-rendering.
    pub dirty: bool,
}

impl DashboardState {
    /// Create a new empty dashboard state.
    pub fn new() -> Self {
        Self {
            widgets: HashMap::new(),
            focus: None,
            dirty: false,
        }
    }

    /// Mark the dashboard as dirty (needs re-rendering).
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Clear the dirty flag.
    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    /// Set the focused widget.
    pub fn set_focus(&mut self, widget_id: impl Into<String>) {
        self.focus = Some(widget_id.into());
        self.mark_dirty();
    }

    /// Clear the focus.
    pub fn clear_focus(&mut self) {
        self.focus = None;
        self.mark_dirty();
    }

    /// Get the focused widget ID.
    pub fn focused_widget(&self) -> Option<&str> {
        self.focus.as_deref()
    }

    /// Check if a widget is focused.
    pub fn is_focused(&self, widget_id: &str) -> bool {
        self.focus.as_deref() == Some(widget_id)
    }

    /// Insert or update a widget state.
    pub fn insert_widget(&mut self, id: impl Into<String>, state: WidgetState) {
        self.widgets.insert(id.into(), state);
        self.mark_dirty();
    }

    /// Get a widget state by ID.
    pub fn get_widget(&self, id: &str) -> Option<&WidgetState> {
        self.widgets.get(id)
    }

    /// Get a mutable widget state by ID.
    pub fn get_widget_mut(&mut self, id: &str) -> Option<&mut WidgetState> {
        self.widgets.get_mut(id)
    }

    /// Remove a widget state.
    pub fn remove_widget(&mut self, id: &str) -> Option<WidgetState> {
        let removed = self.widgets.remove(id);
        if removed.is_some() {
            self.mark_dirty();
        }
        removed
    }

    /// Clear all widget states.
    pub fn clear(&mut self) {
        self.widgets.clear();
        self.focus = None;
        self.mark_dirty();
    }
}

impl Default for DashboardState {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents the state of an individual widget.
#[derive(Debug)]
pub enum WidgetState {
    /// State for a List widget.
    List(ListState),

    /// State for a Table widget.
    Table(TableState),

    /// State for custom widgets.
    Custom(Box<dyn Any>),
}

/// State for a List widget.
#[derive(Debug, Clone)]
pub struct ListState {
    /// Index of the currently selected item.
    pub selected: Option<usize>,

    /// Scroll offset for the list.
    pub offset: usize,
}

impl ListState {
    /// Create a new list state with no selection.
    pub fn new() -> Self {
        Self {
            selected: None,
            offset: 0,
        }
    }

    /// Create a new list state with the first item selected.
    pub fn with_selected(index: usize) -> Self {
        Self {
            selected: Some(index),
            offset: 0,
        }
    }

    /// Select an item by index.
    pub fn select(&mut self, index: Option<usize>) {
        self.selected = index;
    }

    /// Select the next item.
    pub fn select_next(&mut self, total_items: usize) {
        if total_items == 0 {
            self.selected = None;
            return;
        }

        self.selected = Some(match self.selected {
            Some(i) => (i + 1).min(total_items - 1),
            None => 0,
        });
    }

    /// Select the previous item.
    pub fn select_previous(&mut self) {
        self.selected = Some(match self.selected {
            Some(i) => i.saturating_sub(1),
            None => 0,
        });
    }

    /// Get the selected index.
    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    /// Set the scroll offset.
    pub fn set_offset(&mut self, offset: usize) {
        self.offset = offset;
    }

    /// Get the scroll offset.
    pub fn offset(&self) -> usize {
        self.offset
    }
}

impl Default for ListState {
    fn default() -> Self {
        Self::new()
    }
}

/// State for a Table widget.
#[derive(Debug, Clone)]
pub struct TableState {
    /// Index of the currently selected row.
    pub selected_row: Option<usize>,

    /// Index of the currently selected column.
    pub selected_column: Option<usize>,

    /// Vertical scroll offset.
    pub row_offset: usize,

    /// Horizontal scroll offset.
    pub column_offset: usize,
}

impl TableState {
    /// Create a new table state with no selection.
    pub fn new() -> Self {
        Self {
            selected_row: None,
            selected_column: None,
            row_offset: 0,
            column_offset: 0,
        }
    }

    /// Create a new table state with a cell selected.
    pub fn with_selected(row: usize, column: usize) -> Self {
        Self {
            selected_row: Some(row),
            selected_column: Some(column),
            row_offset: 0,
            column_offset: 0,
        }
    }

    /// Select a cell by row and column.
    pub fn select(&mut self, row: Option<usize>, column: Option<usize>) {
        self.selected_row = row;
        self.selected_column = column;
    }

    /// Select the next row.
    pub fn select_next_row(&mut self, total_rows: usize) {
        if total_rows == 0 {
            self.selected_row = None;
            return;
        }

        self.selected_row = Some(match self.selected_row {
            Some(i) => (i + 1).min(total_rows - 1),
            None => 0,
        });
    }

    /// Select the previous row.
    pub fn select_previous_row(&mut self) {
        self.selected_row = Some(match self.selected_row {
            Some(i) => i.saturating_sub(1),
            None => 0,
        });
    }

    /// Select the next column.
    pub fn select_next_column(&mut self, total_columns: usize) {
        if total_columns == 0 {
            self.selected_column = None;
            return;
        }

        self.selected_column = Some(match self.selected_column {
            Some(i) => (i + 1).min(total_columns - 1),
            None => 0,
        });
    }

    /// Select the previous column.
    pub fn select_previous_column(&mut self) {
        self.selected_column = Some(match self.selected_column {
            Some(i) => i.saturating_sub(1),
            None => 0,
        });
    }

    /// Get the selected row index.
    pub fn selected_row(&self) -> Option<usize> {
        self.selected_row
    }

    /// Get the selected column index.
    pub fn selected_column(&self) -> Option<usize> {
        self.selected_column
    }

    /// Set the row offset.
    pub fn set_row_offset(&mut self, offset: usize) {
        self.row_offset = offset;
    }

    /// Set the column offset.
    pub fn set_column_offset(&mut self, offset: usize) {
        self.column_offset = offset;
    }

    /// Get the row offset.
    pub fn row_offset(&self) -> usize {
        self.row_offset
    }

    /// Get the column offset.
    pub fn column_offset(&self) -> usize {
        self.column_offset
    }
}

impl Default for TableState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dashboard_state_new() {
        let state = DashboardState::new();
        assert!(state.widgets.is_empty());
        assert!(state.focus.is_none());
        assert!(!state.dirty);
    }

    #[test]
    fn test_dashboard_state_mark_dirty() {
        let mut state = DashboardState::new();
        assert!(!state.dirty);
        state.mark_dirty();
        assert!(state.dirty);
        state.clear_dirty();
        assert!(!state.dirty);
    }

    #[test]
    fn test_dashboard_state_focus() {
        let mut state = DashboardState::new();
        assert!(state.focus.is_none());

        state.set_focus("widget1");
        assert_eq!(state.focused_widget(), Some("widget1"));
        assert!(state.is_focused("widget1"));
        assert!(!state.is_focused("widget2"));
        assert!(state.dirty);

        state.clear_dirty();
        state.clear_focus();
        assert!(state.focus.is_none());
        assert!(state.dirty);
    }

    #[test]
    fn test_dashboard_state_widgets() {
        let mut state = DashboardState::new();

        state.insert_widget("list1", WidgetState::List(ListState::new()));
        assert!(state.get_widget("list1").is_some());
        assert!(state.dirty);

        state.clear_dirty();
        let removed = state.remove_widget("list1");
        assert!(removed.is_some());
        assert!(state.dirty);
    }

    #[test]
    fn test_list_state_selection() {
        let mut state = ListState::new();
        assert_eq!(state.selected(), None);

        state.select(Some(5));
        assert_eq!(state.selected(), Some(5));

        state.select_next(10);
        assert_eq!(state.selected(), Some(6));

        state.select_next(10);
        assert_eq!(state.selected(), Some(7));

        state.select_previous();
        assert_eq!(state.selected(), Some(6));

        state.select_next(7);
        assert_eq!(state.selected(), Some(6)); // Clamped to max

        state.select_previous();
        state.select_previous();
        state.select_previous();
        state.select_previous();
        state.select_previous();
        state.select_previous();
        state.select_previous();
        assert_eq!(state.selected(), Some(0)); // Saturating sub
    }

    #[test]
    fn test_table_state_selection() {
        let mut state = TableState::new();
        assert_eq!(state.selected_row(), None);
        assert_eq!(state.selected_column(), None);

        state.select(Some(2), Some(3));
        assert_eq!(state.selected_row(), Some(2));
        assert_eq!(state.selected_column(), Some(3));

        state.select_next_row(10);
        assert_eq!(state.selected_row(), Some(3));

        state.select_previous_row();
        assert_eq!(state.selected_row(), Some(2));

        state.select_next_column(5);
        assert_eq!(state.selected_column(), Some(4));

        state.select_previous_column();
        assert_eq!(state.selected_column(), Some(3));
    }

    #[test]
    fn test_table_state_offsets() {
        let mut state = TableState::new();
        assert_eq!(state.row_offset(), 0);
        assert_eq!(state.column_offset(), 0);

        state.set_row_offset(10);
        state.set_column_offset(5);

        assert_eq!(state.row_offset(), 10);
        assert_eq!(state.column_offset(), 5);
    }
}
