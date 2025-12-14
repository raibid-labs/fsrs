//! Plugin trait for Scarab TUI plugins.
//!
//! This module defines the plugin interface for creating interactive TUI
//! applications that run inside Scarab terminal emulator.

use fusabi_tui_core::buffer::Buffer;
use fusabi_tui_core::layout::Rect;
use std::path::PathBuf;

use crate::error::Result;

/// Context provided to plugins during initialization.
#[derive(Debug, Clone)]
pub struct PluginContext {
    /// Path to the shared memory region
    pub shm_path: String,
    /// Optional path to plugin configuration file
    pub config_path: Option<PathBuf>,
    /// Terminal dimensions
    pub terminal_size: Rect,
}

impl PluginContext {
    /// Create a new plugin context.
    pub fn new(shm_path: String, terminal_size: Rect) -> Self {
        Self {
            shm_path,
            config_path: None,
            terminal_size,
        }
    }

    /// Set the configuration path.
    pub fn with_config(mut self, config_path: PathBuf) -> Self {
        self.config_path = Some(config_path);
        self
    }
}

/// Context provided during rendering.
#[derive(Debug, Clone)]
pub struct RenderContext {
    /// Current terminal size
    pub size: Rect,
    /// Time since last render (in milliseconds)
    pub delta_time_ms: u64,
    /// Frame number (increments each render)
    pub frame: u64,
}

impl RenderContext {
    /// Create a new render context.
    pub fn new(size: Rect, delta_time_ms: u64, frame: u64) -> Self {
        Self {
            size,
            delta_time_ms,
            frame,
        }
    }
}

/// Input events from the terminal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputEvent {
    /// Key press event
    Key(KeyEvent),
    /// Mouse event
    Mouse(MouseEvent),
    /// Terminal resize
    Resize { width: u16, height: u16 },
    /// Focus gained
    FocusGained,
    /// Focus lost
    FocusLost,
}

/// Key press event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyEvent {
    /// Key code
    pub code: KeyCode,
    /// Modifiers (Ctrl, Alt, Shift)
    pub modifiers: KeyModifiers,
}

/// Key code enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    /// Backspace key
    Backspace,
    /// Enter key
    Enter,
    /// Left arrow
    Left,
    /// Right arrow
    Right,
    /// Up arrow
    Up,
    /// Down arrow
    Down,
    /// Home key
    Home,
    /// End key
    End,
    /// Page up
    PageUp,
    /// Page down
    PageDown,
    /// Tab key
    Tab,
    /// Delete key
    Delete,
    /// Insert key
    Insert,
    /// Function key (F1-F12)
    F(u8),
    /// Character key
    Char(char),
    /// Escape key
    Esc,
}

/// Key modifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyModifiers {
    /// Control key held
    pub ctrl: bool,
    /// Alt key held
    pub alt: bool,
    /// Shift key held
    pub shift: bool,
}

impl KeyModifiers {
    pub const NONE: Self = Self {
        ctrl: false,
        alt: false,
        shift: false,
    };
}

/// Mouse event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MouseEvent {
    /// Mouse button
    pub kind: MouseEventKind,
    /// Column position
    pub column: u16,
    /// Row position
    pub row: u16,
    /// Modifiers
    pub modifiers: KeyModifiers,
}

/// Mouse event kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseEventKind {
    /// Mouse button pressed
    Down(MouseButton),
    /// Mouse button released
    Up(MouseButton),
    /// Mouse moved
    Moved,
    /// Mouse wheel scrolled
    ScrollUp,
    /// Mouse wheel scrolled
    ScrollDown,
}

/// Mouse button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    /// Left button
    Left,
    /// Right button
    Right,
    /// Middle button
    Middle,
}

/// Actions that can be returned from input handlers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Continue running
    Continue,
    /// Request redraw
    Redraw,
    /// Exit the plugin
    Exit,
    /// Custom action (plugin-specific)
    Custom(String),
}

/// Trait for TUI plugins that run in Scarab.
///
/// Plugins implement this trait to provide interactive TUI applications
/// that render to Scarab's shared memory.
pub trait TuiPlugin: Send + Sync {
    /// Initialize the plugin.
    ///
    /// Called once when the plugin is loaded.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Plugin initialization context
    ///
    /// # Errors
    ///
    /// Returns an error if initialization fails.
    fn on_init(&mut self, ctx: &PluginContext) -> Result<()>;

    /// Render the TUI.
    ///
    /// Called on each frame to render the plugin's UI.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Render context with timing and size information
    ///
    /// # Returns
    ///
    /// Returns a buffer containing the rendered content.
    ///
    /// # Errors
    ///
    /// Returns an error if rendering fails.
    fn on_render(&mut self, ctx: &RenderContext) -> Result<Buffer>;

    /// Handle input events.
    ///
    /// Called when the user provides input (keyboard, mouse, etc.).
    ///
    /// # Arguments
    ///
    /// * `event` - The input event to handle
    ///
    /// # Returns
    ///
    /// Returns an action indicating how to proceed.
    ///
    /// # Errors
    ///
    /// Returns an error if input handling fails.
    fn on_input(&mut self, _event: InputEvent) -> Result<Action> {
        // Default implementation: ignore all input
        Ok(Action::Continue)
    }

    /// Handle periodic tick.
    ///
    /// Called at regular intervals for plugin updates.
    ///
    /// # Errors
    ///
    /// Returns an error if the tick handler fails.
    fn on_tick(&mut self) -> Result<()> {
        // Default implementation: do nothing
        Ok(())
    }

    /// Cleanup on shutdown.
    ///
    /// Called when the plugin is being unloaded.
    fn on_shutdown(&mut self) {
        // Default implementation: do nothing
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestPlugin {
        initialized: bool,
    }

    impl TestPlugin {
        fn new() -> Self {
            Self {
                initialized: false,
            }
        }
    }

    impl TuiPlugin for TestPlugin {
        fn on_init(&mut self, _ctx: &PluginContext) -> Result<()> {
            self.initialized = true;
            Ok(())
        }

        fn on_render(&mut self, ctx: &RenderContext) -> Result<Buffer> {
            let buffer = Buffer::new(ctx.size);
            Ok(buffer)
        }
    }

    #[test]
    fn test_plugin_init() {
        let mut plugin = TestPlugin::new();
        assert!(!plugin.initialized);

        let ctx = PluginContext::new("/test_shm".to_string(), Rect::new(0, 0, 80, 24));
        plugin.on_init(&ctx).unwrap();
        assert!(plugin.initialized);
    }

    #[test]
    fn test_plugin_render() {
        let mut plugin = TestPlugin::new();
        let ctx = RenderContext::new(Rect::new(0, 0, 80, 24), 16, 0);
        let buffer = plugin.on_render(&ctx).unwrap();
        assert_eq!(buffer.area.width, 80);
        assert_eq!(buffer.area.height, 24);
    }

    #[test]
    fn test_default_input_handler() {
        let mut plugin = TestPlugin::new();
        let event = InputEvent::Key(KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
        });
        let action = plugin.on_input(event).unwrap();
        assert_eq!(action, Action::Continue);
    }
}
