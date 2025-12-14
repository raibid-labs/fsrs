//! Hot reload engine and dashboard runtime for Fusabi TUI.
//!
//! This crate provides the core engine for building hot-reloadable TUI applications
//! with the Fusabi framework. It includes:
//!
//! - **DashboardEngine**: Main orchestration engine for TUI applications
//! - **FileWatcher**: File system watching with debouncing
//! - **FileLoader**: Smart file loading with dependency tracking and caching
//! - **Event handling**: Comprehensive input event types and actions
//! - **State management**: Dashboard and widget state management
//!
//! # Features
//!
//! - **Hot Reload**: Automatically reload dashboards when files change
//! - **Dependency Tracking**: Track and reload dependent files
//! - **File Watching**: Efficient file system monitoring with debouncing
//! - **Event System**: Rich event handling with keyboard, mouse, and custom events
//! - **State Management**: Flexible state management for widgets
//!
//! # Example
//!
//! ```no_run
//! use fusabi_tui_engine::prelude::*;
//! use fusabi_tui_render::test::TestRenderer;
//! use std::path::{Path, PathBuf};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a dashboard engine
//! let renderer = TestRenderer::new(80, 24);
//! let mut engine = DashboardEngine::new(renderer, PathBuf::from("."));
//!
//! // Enable hot reload
//! engine.enable_hot_reload()?;
//!
//! // Load a dashboard file
//! engine.load(Path::new("dashboard.fsx"))?;
//!
//! // Render the dashboard
//! engine.render()?;
//!
//! // Check for file changes
//! if let Some(changes) = engine.poll_changes() {
//!     if !changes.is_empty() {
//!         println!("Files changed: {:?}", changes);
//!         engine.reload()?;
//!         engine.render()?;
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Hot Reload Example
//!
//! ```no_run
//! use fusabi_tui_engine::prelude::*;
//! use fusabi_tui_render::test::TestRenderer;
//! use std::path::{Path, PathBuf};
//! use std::time::Duration;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let renderer = TestRenderer::new(80, 24);
//! let mut engine = DashboardEngine::new(renderer, PathBuf::from("."));
//!
//! // Enable hot reload with 200ms debounce
//! engine.enable_hot_reload_with_debounce(200)?;
//!
//! // Load dashboard
//! engine.load(Path::new("dashboard.fsx"))?;
//!
//! // Main loop
//! loop {
//!     // Handle events
//!     // let event = read_event()?;
//!     // let action = engine.handle_event(event)?;
//!     // if action.is_quit() { break; }
//!
//!     // Check for file changes
//!     if let Some(changes) = engine.poll_changes() {
//!         if !changes.is_empty() {
//!             engine.reload()?;
//!             engine.render()?;
//!         }
//!     }
//!
//!     // Render if needed
//!     if engine.state().dirty {
//!         engine.render()?;
//!     }
//!
//!     std::thread::sleep(Duration::from_millis(16)); // ~60 FPS
//! }
//! # Ok(())
//! # }
//! ```

#![warn(clippy::all)]
#![warn(missing_docs)]

pub mod dashboard;
pub mod error;
pub mod event;
pub mod loader;
pub mod state;
pub mod watcher;

/// Convenient re-exports for common types and traits.
pub mod prelude {
    //! Prelude module with commonly used types.

    pub use crate::dashboard::DashboardEngine;
    pub use crate::error::{EngineError, EngineResult, LoadError, WatchError};
    pub use crate::event::{
        Action, Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
    };
    pub use crate::loader::{FileLoader, LoadedFile};
    pub use crate::state::{DashboardState, ListState, TableState, WidgetState};
    pub use crate::watcher::FileWatcher;
}