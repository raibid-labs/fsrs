//! Core widget trait definitions.
//!
//! This module defines the fundamental traits that all widgets must implement.

use fusabi_tui_core::{buffer::Buffer, layout::Rect};

/// Trait for stateless widgets.
///
/// A widget is any component that can be rendered to a terminal buffer.
/// Stateless widgets don't maintain any state between renders.
///
/// # Examples
///
/// ```rust
/// use fusabi_tui_core::{buffer::Buffer, layout::Rect};
/// use fusabi_tui_widgets::Widget;
///
/// struct MyWidget;
///
/// impl Widget for MyWidget {
///     fn render(&self, area: Rect, buf: &mut Buffer) {
///         buf.set_string(area.x, area.y, "Hello!", Default::default());
///     }
/// }
/// ```
pub trait Widget {
    /// Renders the widget to the given area in the buffer.
    ///
    /// # Arguments
    ///
    /// * `area` - The rectangular area where the widget should be rendered
    /// * `buf` - The buffer to render into
    fn render(&self, area: Rect, buf: &mut Buffer);
}

/// Trait for stateful widgets.
///
/// Stateful widgets maintain state between renders, such as scroll position
/// or selected item index.
///
/// # Examples
///
/// ```rust
/// use fusabi_tui_core::{buffer::Buffer, layout::Rect};
/// use fusabi_tui_widgets::StatefulWidget;
///
/// struct MyStatefulWidget;
///
/// struct MyState {
///     selected: usize,
/// }
///
/// impl StatefulWidget for MyStatefulWidget {
///     type State = MyState;
///
///     fn render(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
///         let text = format!("Selected: {}", state.selected);
///         buf.set_string(area.x, area.y, &text, Default::default());
///     }
/// }
/// ```
pub trait StatefulWidget {
    /// The state type associated with this widget.
    type State;

    /// Renders the widget to the given area in the buffer using the provided state.
    ///
    /// # Arguments
    ///
    /// * `area` - The rectangular area where the widget should be rendered
    /// * `buf` - The buffer to render into
    /// * `state` - Mutable reference to the widget's state
    fn render(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State);
}
