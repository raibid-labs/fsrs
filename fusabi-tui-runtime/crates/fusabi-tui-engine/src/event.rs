//! Event handling and action types for the Fusabi TUI engine.

use std::path::PathBuf;

/// Input events that the dashboard engine can process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// Keyboard input event.
    Key(KeyEvent),

    /// Mouse input event.
    Mouse(MouseEvent),

    /// Terminal resize event with new dimensions (width, height).
    Resize(u16, u16),

    /// File change notification from the file watcher.
    FileChange(PathBuf),

    /// Periodic tick event for animations or timed updates.
    Tick,

    /// Custom event with string payload.
    Custom(String),
}

/// Keyboard event details.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyEvent {
    /// The key code that was pressed.
    pub code: KeyCode,

    /// Modifier keys that were held during the event.
    pub modifiers: KeyModifiers,
}

/// Key codes for keyboard input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    /// Character keys.
    Char(char),

    /// Function keys (F1-F12).
    F(u8),

    /// Enter/Return key.
    Enter,

    /// Escape key.
    Esc,

    /// Backspace key.
    Backspace,

    /// Tab key.
    Tab,

    /// Arrow keys.
    Up,
    Down,
    Left,
    Right,

    /// Home key.
    Home,

    /// End key.
    End,

    /// Page Up key.
    PageUp,

    /// Page Down key.
    PageDown,

    /// Delete key.
    Delete,

    /// Insert key.
    Insert,

    /// Space bar.
    Space,
}

/// Modifier keys for keyboard events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyModifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
}

impl KeyModifiers {
    /// Create a new KeyModifiers with no modifiers pressed.
    pub const fn none() -> Self {
        Self {
            shift: false,
            ctrl: false,
            alt: false,
            meta: false,
        }
    }

    /// Create a new KeyModifiers with only Ctrl pressed.
    pub const fn ctrl() -> Self {
        Self {
            shift: false,
            ctrl: true,
            alt: false,
            meta: false,
        }
    }

    /// Create a new KeyModifiers with only Shift pressed.
    pub const fn shift() -> Self {
        Self {
            shift: true,
            ctrl: false,
            alt: false,
            meta: false,
        }
    }

    /// Create a new KeyModifiers with only Alt pressed.
    pub const fn alt() -> Self {
        Self {
            shift: false,
            ctrl: false,
            alt: true,
            meta: false,
        }
    }
}

/// Mouse event details.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MouseEvent {
    /// The type of mouse event.
    pub kind: MouseEventKind,

    /// X coordinate (column).
    pub x: u16,

    /// Y coordinate (row).
    pub y: u16,

    /// Modifier keys held during the event.
    pub modifiers: KeyModifiers,
}

/// Types of mouse events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseEventKind {
    /// Mouse button was pressed.
    Down(MouseButton),

    /// Mouse button was released.
    Up(MouseButton),

    /// Mouse was moved.
    Moved,

    /// Mouse wheel was scrolled.
    ScrollUp,
    ScrollDown,
}

/// Mouse buttons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// Actions that can be returned from event handling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// No action needed.
    None,

    /// Request a re-render of the dashboard.
    Render,

    /// Request a reload of files.
    Reload,

    /// Request to quit the application.
    Quit,

    /// Custom action with string identifier.
    Custom(String),

    /// Multiple actions to be performed in sequence.
    Batch(Vec<Action>),
}

impl Action {
    /// Check if the action is None.
    pub fn is_none(&self) -> bool {
        matches!(self, Action::None)
    }

    /// Check if the action requires rendering.
    pub fn requires_render(&self) -> bool {
        match self {
            Action::Render => true,
            Action::Reload => true,
            Action::Batch(actions) => actions.iter().any(|a| a.requires_render()),
            _ => false,
        }
    }

    /// Check if the action is a quit request.
    pub fn is_quit(&self) -> bool {
        matches!(self, Action::Quit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_modifiers() {
        let mods = KeyModifiers::none();
        assert!(!mods.shift && !mods.ctrl && !mods.alt && !mods.meta);

        let mods = KeyModifiers::ctrl();
        assert!(mods.ctrl && !mods.shift && !mods.alt && !mods.meta);
    }

    #[test]
    fn test_action_none() {
        assert!(Action::None.is_none());
        assert!(!Action::Render.is_none());
    }

    #[test]
    fn test_action_requires_render() {
        assert!(Action::Render.requires_render());
        assert!(Action::Reload.requires_render());
        assert!(!Action::None.requires_render());
        assert!(!Action::Quit.requires_render());

        let batch = Action::Batch(vec![Action::None, Action::Render]);
        assert!(batch.requires_render());
    }

    #[test]
    fn test_action_is_quit() {
        assert!(Action::Quit.is_quit());
        assert!(!Action::None.is_quit());
        assert!(!Action::Render.is_quit());
    }

    #[test]
    fn test_event_equality() {
        let event1 = Event::Key(KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::none(),
        });
        let event2 = Event::Key(KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::none(),
        });
        assert_eq!(event1, event2);
    }
}
