//! Widget framework for Fusabi TUI.
//!
//! This crate provides a collection of reusable widgets for building terminal user interfaces
//! in the Fusabi ecosystem. It builds on top of `fusabi-tui-core` to provide higher-level
//! abstractions for common UI patterns.
//!
//! # Overview
//!
//! The crate is organized around the [`Widget`] and [`StatefulWidget`] traits, which define
//! how components can be rendered to a buffer.
//!
//! ## Modules
//!
//! - [`widget`] - Core widget trait definitions
//! - [`borders`] - Border types and styles
//! - [`block`] - Block widget for bordered containers
//! - [`table`] - Table widget for tabular data display
//! - [`gauge`] - Gauge widget for progress bars
//! - [`sparkline`] - Sparkline widget for inline mini-charts
//! - [`tabs`] - Tabs widget for tab navigation
//!
//! # Quick Start
//!
//! ```rust
//! use fusabi_tui_core::{buffer::Buffer, layout::Rect, style::Style};
//! use fusabi_tui_widgets::{
//!     block::Block,
//!     borders::{Borders, BorderType},
//!     widget::Widget,
//! };
//!
//! // Create a block with borders
//! let block = Block::default()
//!     .title("My Panel")
//!     .borders(Borders::ALL)
//!     .border_type(BorderType::Rounded);
//!
//! // Render it to a buffer
//! let area = Rect::new(0, 0, 20, 10);
//! let mut buffer = Buffer::new(area);
//! block.render(area, &mut buffer);
//! ```
//!
//! # Design Philosophy
//!
//! Widgets in this crate are designed to be:
//!
//! - **Composable**: Small widgets can be combined to build complex UIs
//! - **Immutable**: Widgets use builder patterns and don't mutate state
//! - **Efficient**: Rendering is done through zero-copy buffers
//! - **Flexible**: Extensive styling and customization options

#![deny(missing_docs)]
#![deny(unsafe_code)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod borders;
pub mod block;
pub mod gauge;
pub mod list;
pub mod paragraph;
pub mod sparkline;
pub mod table;
pub mod tabs;
pub mod text;
pub mod widget;

// Re-export commonly used types at the crate root for convenience
pub use block::{Block, Padding, Title, TitleAlignment, TitlePosition};
pub use borders::{BorderType, Borders};
pub use gauge::{Gauge, GaugeCharSet};
pub use list::{List, ListItem, ListState};
pub use paragraph::{Alignment, Paragraph, Wrap};
pub use sparkline::{Sparkline, SparklineBarSet};
pub use table::{Row, Table, TableCell, TableState};
pub use tabs::Tabs;
pub use text::{Line, Span, Text};
pub use widget::{StatefulWidget, Widget};
