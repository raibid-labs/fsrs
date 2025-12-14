//! Comprehensive error types for the Fusabi TUI engine.

use fusabi_tui_render::error::RenderError;
use std::path::PathBuf;
use thiserror::Error;

/// Main error type for the dashboard engine.
#[derive(Debug, Error)]
pub enum EngineError {
    #[error("Failed to load file: {0}")]
    LoadError(#[from] LoadError),

    #[error("Failed to watch file: {0}")]
    WatchError(#[from] WatchError),

    #[error("Render error: {0}")]
    Render(#[from] RenderError),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Widget not found: {0}")]
    WidgetNotFound(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Custom error: {0}")]
    Custom(String),
}

/// Error type for file loading operations.
#[derive(Debug, Error)]
pub enum LoadError {
    #[error("File not found: {path}")]
    FileNotFound { path: PathBuf },

    #[error("Failed to read file: {path}: {source}")]
    ReadFailed {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("Failed to parse file: {path}: {reason}")]
    ParseFailed { path: PathBuf, reason: String },

    #[error("Circular dependency detected: {0:?}")]
    CircularDependency(Vec<PathBuf>),

    #[error("Invalid file format: {path}")]
    InvalidFormat { path: PathBuf },

    #[error("Dependency not found: {dependency} required by {dependent}")]
    DependencyNotFound {
        dependency: PathBuf,
        dependent: PathBuf,
    },
}

/// Error type for file watching operations.
#[derive(Debug, Error)]
pub enum WatchError {
    #[error("Failed to initialize file watcher: {0}")]
    InitFailed(String),

    #[error("Failed to watch path: {path}: {reason}")]
    WatchFailed { path: PathBuf, reason: String },

    #[error("Failed to unwatch path: {path}: {reason}")]
    UnwatchFailed { path: PathBuf, reason: String },

    #[error("Watcher channel closed")]
    ChannelClosed,

    #[error("Notify error: {0}")]
    Notify(#[from] notify::Error),
}

/// Result type using EngineError.
pub type EngineResult<T> = Result<T, EngineError>;

/// Result type using LoadError.
pub type LoadResult<T> = Result<T, LoadError>;

/// Result type using WatchError.
pub type WatchResult<T> = Result<T, WatchError>;
