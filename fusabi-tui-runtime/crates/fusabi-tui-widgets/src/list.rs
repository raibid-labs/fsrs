//! List widget for displaying scrollable lists with selection.

use fusabi_tui_core::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
};

use crate::{
    block::Block,
    text::Text,
    widget::{StatefulWidget, Widget},
};

/// A single item in a list.
///
/// # Examples
///
/// ```rust
/// use fusabi_tui_core::style::{Color, Style};
/// use fusabi_tui_widgets::ListItem;
///
/// let item = ListItem::new("Item 1")
///     .style(Style::default().fg(Color::White));
/// ```
#[derive(Debug, Clone)]
pub struct ListItem<'a> {
    /// The content of the item
    content: Text<'a>,
    /// The style of the item
    style: Style,
}

impl<'a> ListItem<'a> {
    /// Creates a new list item with the given content.
    pub fn new<T>(content: T) -> Self
    where
        T: Into<Text<'a>>,
    {
        Self {
            content: content.into(),
            style: Style::default(),
        }
    }

    /// Sets the style of the item.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Returns the height of the item in lines.
    pub fn height(&self) -> usize {
        self.content.height()
    }
}

impl<'a> From<&'a str> for ListItem<'a> {
    fn from(s: &'a str) -> Self {
        Self::new(s)
    }
}

impl From<String> for ListItem<'static> {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl<'a> From<Text<'a>> for ListItem<'a> {
    fn from(text: Text<'a>) -> Self {
        Self::new(text)
    }
}

/// State for a stateful list widget.
///
/// Tracks the currently selected item and scroll offset.
///
/// # Examples
///
/// ```rust
/// use fusabi_tui_widgets::ListState;
///
/// let mut state = ListState::default();
/// state.select(Some(0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct ListState {
    /// The index of the selected item
    selected: Option<usize>,
    /// The offset for scrolling
    offset: usize,
}

impl ListState {
    /// Creates a new list state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the currently selected index.
    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    /// Selects an item by index.
    pub fn select(&mut self, index: Option<usize>) {
        self.selected = index;
    }

    /// Returns the current scroll offset.
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Sets the scroll offset.
    pub fn set_offset(&mut self, offset: usize) {
        self.offset = offset;
    }

    /// Selects the next item in the list.
    ///
    /// If no item is selected, selects the first item.
    /// If the last item is selected, wraps around to the first item.
    pub fn select_next(&mut self, len: usize) {
        if len == 0 {
            self.selected = None;
            return;
        }

        self.selected = Some(match self.selected {
            Some(i) => (i + 1) % len,
            None => 0,
        });
    }

    /// Selects the previous item in the list.
    ///
    /// If no item is selected, selects the last item.
    /// If the first item is selected, wraps around to the last item.
    pub fn select_previous(&mut self, len: usize) {
        if len == 0 {
            self.selected = None;
            return;
        }

        self.selected = Some(match self.selected {
            Some(0) => len - 1,
            Some(i) => i - 1,
            None => len - 1,
        });
    }

    /// Selects the first item in the list.
    pub fn select_first(&mut self) {
        self.selected = Some(0);
        self.offset = 0;
    }

    /// Selects the last item in the list.
    pub fn select_last(&mut self, len: usize) {
        if len > 0 {
            self.selected = Some(len - 1);
        }
    }
}

/// A scrollable list widget with selection support.
///
/// # Examples
///
/// ```rust
/// use fusabi_tui_core::{buffer::Buffer, layout::Rect, style::{Color, Style}};
/// use fusabi_tui_widgets::{List, ListItem, ListState, StatefulWidget};
///
/// let items = vec![
///     ListItem::new("Item 1"),
///     ListItem::new("Item 2"),
///     ListItem::new("Item 3"),
/// ];
///
/// let list = List::new(items)
///     .highlight_style(Style::default().fg(Color::Yellow))
///     .highlight_symbol("> ");
///
/// let mut state = ListState::default();
/// state.select(Some(0));
///
/// let area = Rect::new(0, 0, 20, 5);
/// let mut buffer = Buffer::new(area);
/// list.render(area, &mut buffer, &mut state);
/// ```
#[derive(Debug, Clone)]
pub struct List<'a> {
    /// The items in the list
    items: Vec<ListItem<'a>>,
    /// Optional block to wrap the list
    block: Option<Block>,
    /// Style for the list
    style: Style,
    /// Style for the highlighted item
    highlight_style: Style,
    /// Symbol to show before the highlighted item
    highlight_symbol: Option<&'a str>,
}

impl<'a> List<'a> {
    /// Creates a new list with the given items.
    pub fn new<T>(items: T) -> Self
    where
        T: Into<Vec<ListItem<'a>>>,
    {
        Self {
            items: items.into(),
            block: None,
            style: Style::default(),
            highlight_style: Style::default(),
            highlight_symbol: None,
        }
    }

    /// Wraps the list in a block.
    #[must_use]
    pub fn block(mut self, block: Block) -> Self {
        self.block = Some(block);
        self
    }

    /// Sets the style for the list.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Sets the style for the highlighted item.
    #[must_use]
    pub fn highlight_style(mut self, style: Style) -> Self {
        self.highlight_style = style;
        self
    }

    /// Sets the symbol to show before the highlighted item.
    #[must_use]
    pub fn highlight_symbol(mut self, symbol: &'a str) -> Self {
        self.highlight_symbol = Some(symbol);
        self
    }

    /// Returns the number of items in the list.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns whether the list is empty.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl StatefulWidget for List<'_> {
    type State = ListState;

    fn render(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        // Render block and get inner area
        let list_area = if let Some(ref block) = self.block {
            block.render(area, buf);
            block.inner(area)
        } else {
            buf.set_style(area, self.style);
            area
        };

        if list_area.width == 0 || list_area.height == 0 || self.items.is_empty() {
            return;
        }

        // Calculate visible range
        let list_height = list_area.height as usize;
        let selected = state.selected();
        
        // Auto-scroll to keep selected item visible
        if let Some(selected_idx) = selected {
            if selected_idx < state.offset {
                state.offset = selected_idx;
            } else if selected_idx >= state.offset + list_height {
                state.offset = selected_idx.saturating_sub(list_height - 1);
            }
        }

        let start_idx = state.offset;
        let end_idx = (start_idx + list_height).min(self.items.len());

        // Render visible items
        let mut current_y = list_area.y;
        let highlight_symbol_width = self.highlight_symbol
            .map(|s| s.len())
            .unwrap_or(0) as u16;

        for (idx, item) in self.items[start_idx..end_idx].iter().enumerate() {
            let item_idx = start_idx + idx;
            let is_selected = Some(item_idx) == selected;

            // Determine item style
            let item_style = if is_selected {
                self.highlight_style
            } else {
                item.style
            };

            // Render highlight symbol
            let mut x = list_area.x;
            if let Some(symbol) = self.highlight_symbol {
                if is_selected {
                    buf.set_string(x, current_y, symbol, self.highlight_style);
                }
                x += highlight_symbol_width;
            }

            // Render item content
            let content_width = list_area.width.saturating_sub(highlight_symbol_width);
            
            for (line_idx, line) in item.content.lines.iter().enumerate() {
                if current_y >= list_area.y + list_area.height {
                    break;
                }

                let mut line_x = x;
                for span in &line.spans {
                    let span_style = if is_selected {
                        // Apply highlight style to the entire line
                        self.highlight_style
                    } else {
                        span.style
                    };

                    // Render span content
                    for ch in span.content.chars() {
                        if line_x >= x + content_width {
                            break;
                        }
                        buf.set_string(line_x, current_y, &ch.to_string(), span_style);
                        line_x += unicode_width::UnicodeWidthStr::width(ch.to_string().as_str()) as u16;
                    }
                }

                // Fill remaining space with highlight style if selected
                if is_selected {
                    while line_x < list_area.x + list_area.width {
                        buf.set_string(line_x, current_y, " ", self.highlight_style);
                        line_x += 1;
                    }
                }

                current_y += 1;
                
                // Don't render more lines if this is a multi-line item
                if line_idx == 0 {
                    break;
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
    fn test_list_item_creation() {
        let item = ListItem::new("test");
        assert_eq!(item.content.lines.len(), 1);
    }

    #[test]
    fn test_list_item_from_str() {
        let item: ListItem = "test".into();
        assert_eq!(item.content.lines.len(), 1);
    }

    #[test]
    fn test_list_state_select() {
        let mut state = ListState::default();
        assert_eq!(state.selected(), None);

        state.select(Some(2));
        assert_eq!(state.selected(), Some(2));
    }

    #[test]
    fn test_list_state_select_next() {
        let mut state = ListState::default();
        
        state.select_next(3);
        assert_eq!(state.selected(), Some(0));

        state.select_next(3);
        assert_eq!(state.selected(), Some(1));

        state.select_next(3);
        assert_eq!(state.selected(), Some(2));

        state.select_next(3);
        assert_eq!(state.selected(), Some(0)); // Wraps around
    }

    #[test]
    fn test_list_state_select_previous() {
        let mut state = ListState::default();
        
        state.select_previous(3);
        assert_eq!(state.selected(), Some(2)); // Starts at last

        state.select_previous(3);
        assert_eq!(state.selected(), Some(1));

        state.select_previous(3);
        assert_eq!(state.selected(), Some(0));

        state.select_previous(3);
        assert_eq!(state.selected(), Some(2)); // Wraps around
    }

    #[test]
    fn test_list_creation() {
        let items = vec![
            ListItem::new("Item 1"),
            ListItem::new("Item 2"),
        ];
        let list = List::new(items);
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_list_with_highlight() {
        let items = vec![ListItem::new("Item 1")];
        let list = List::new(items)
            .highlight_symbol("> ")
            .highlight_style(Style::default().fg(Color::Yellow));

        let mut state = ListState::default();
        state.select(Some(0));

        let area = Rect::new(0, 0, 10, 5);
        let mut buffer = Buffer::new(area);
        list.render(area, &mut buffer, &mut state);

        // Check highlight symbol is rendered
        assert_eq!(buffer.get(0, 0).unwrap().symbol, ">");
        assert_eq!(buffer.get(1, 0).unwrap().symbol, " ");
    }

    #[test]
    fn test_list_auto_scroll() {
        let items: Vec<ListItem> = (0..10)
            .map(|i| ListItem::new(format!("Item {}", i)))
            .collect();

        let list = List::new(items);
        let mut state = ListState::default();

        // Select an item beyond visible area
        state.select(Some(8));

        let area = Rect::new(0, 0, 10, 5);
        let mut buffer = Buffer::new(area);
        list.render(area, &mut buffer, &mut state);

        // Offset should be adjusted to keep selected item visible
        assert!(state.offset() > 0);
    }

    #[test]
    fn test_list_state_first_last() {
        let mut state = ListState::default();

        state.select_first();
        assert_eq!(state.selected(), Some(0));

        state.select_last(5);
        assert_eq!(state.selected(), Some(4));
    }
}
