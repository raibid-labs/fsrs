//! Scarab shared memory backend for Fusabi TUI.
//!
//! This crate provides integration between the Fusabi TUI framework and the
//! Scarab terminal emulator's shared memory protocol. It enables zero-copy
//! rendering of TUI applications directly to Scarab's terminal buffer.
//!
//! # Architecture
//!
//! Scarab uses a split-process architecture with shared memory for IPC:
//! - **Daemon**: Headless server that owns PTY processes
//! - **Client**: Bevy-based GUI that renders via shared memory
//! - **Plugins**: Can run in either process with Fusabi scripting
//!
//! This crate provides:
//! - `ScarabRenderer`: Renderer implementation that writes to shared memory
//! - `SharedCell` / `SharedState`: Zero-copy types matching Scarab's protocol
//! - Conversion utilities between Fusabi and Scarab types
//! - Plugin trait for building TUI applications
//!
//! # Examples
//!
//! ## Basic Rendering
//!
//! ```no_run
//! use fusabi_tui_scarab::renderer::ScarabRenderer;
//! use fusabi_tui_render::renderer::Renderer;
//! use fusabi_tui_core::buffer::Buffer;
//! use fusabi_tui_core::layout::Rect;
//! use fusabi_tui_core::style::{Style, Color};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Connect to Scarab's shared memory
//! let mut renderer = ScarabRenderer::connect(None)?;
//!
//! // Create a buffer and draw to it
//! let size = renderer.size()?;
//! let mut buffer = Buffer::new(size);
//!
//! let style = Style::new().fg(Color::Green);
//! buffer.set_string(0, 0, "Hello, Scarab!", style);
//!
//! // Render to shared memory
//! renderer.draw(&buffer)?;
//! renderer.flush()?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Plugin Development
//!
//! ```no_run
//! use fusabi_tui_scarab::plugin::{TuiPlugin, PluginContext, RenderContext, InputEvent, Action};
//! use fusabi_tui_scarab::error::Result;
//! use fusabi_tui_core::buffer::Buffer;
//! use fusabi_tui_core::style::{Style, Color};
//!
//! struct MyPlugin {
//!     counter: u32,
//! }
//!
//! impl TuiPlugin for MyPlugin {
//!     fn on_init(&mut self, ctx: &PluginContext) -> Result<()> {
//!         println!("Plugin initialized!");
//!         Ok(())
//!     }
//!
//!     fn on_render(&mut self, ctx: &RenderContext) -> Result<Buffer> {
//!         let mut buffer = Buffer::new(ctx.size);
//!         let style = Style::new().fg(Color::Cyan);
//!
//!         let text = format!("Frame: {}", ctx.frame);
//!         buffer.set_string(0, 0, &text, style);
//!
//!         Ok(buffer)
//!     }
//!
//!     fn on_input(&mut self, event: InputEvent) -> Result<Action> {
//!         self.counter += 1;
//!         Ok(Action::Redraw)
//!     }
//! }
//! ```
//!
//! # Shared Memory Layout
//!
//! The shared memory region contains a `SharedState` structure with:
//! - Sequence number for lock-free synchronization
//! - Cursor position
//! - 200x100 grid of cells (each 16 bytes)
//!
//! All structures use `#[repr(C)]` and `bytemuck::Pod` for zero-copy safety.
//!
//! # Features
//!
//! - `plugin`: Enables plugin system with JSON serialization (optional)

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod convert;
pub mod error;
pub mod plugin;
pub mod renderer;
pub mod shared;

// Re-export commonly used types
pub use error::{Result, ScarabError};
pub use renderer::ScarabRenderer;
pub use shared::{SharedCell, SharedState, GRID_HEIGHT, GRID_WIDTH, SHMEM_PATH};

#[cfg(feature = "plugin")]
pub use plugin::{
    Action, InputEvent, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent,
    MouseEventKind, PluginContext, RenderContext, TuiPlugin,
};

/// Prelude module for convenient imports.
pub mod prelude {
    //! Convenient re-exports for common types and traits.

    pub use crate::error::{Result, ScarabError};
    pub use crate::renderer::ScarabRenderer;
    pub use crate::shared::{SharedCell, SharedState};

    #[cfg(feature = "plugin")]
    pub use crate::plugin::{
        Action, InputEvent, KeyCode, KeyEvent, PluginContext, RenderContext, TuiPlugin,
    };

    // Re-export core TUI types
    pub use fusabi_tui_core::{
        buffer::{Buffer, Cell},
        layout::{Constraint, Direction, Layout, Rect},
        style::{Color, Modifier, Style},
    };

    // Re-export renderer trait
    pub use fusabi_tui_render::renderer::Renderer;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shared_cell_size() {
        // Verify SharedCell is exactly 16 bytes (required by Scarab protocol)
        assert_eq!(std::mem::size_of::<SharedCell>(), 16);
    }

    #[test]
    fn test_shared_state_size() {
        // Verify SharedState size matches expected layout
        let expected = std::mem::size_of::<u64>() // sequence_number
            + std::mem::size_of::<u8>()           // dirty_flag
            + std::mem::size_of::<u8>()           // error_mode
            + std::mem::size_of::<u16>()          // cursor_x
            + std::mem::size_of::<u16>()          // cursor_y
            + 2                                    // _padding2
            + (std::mem::size_of::<SharedCell>() * shared::BUFFER_SIZE); // cells

        assert_eq!(std::mem::size_of::<SharedState>(), expected);
    }
}
