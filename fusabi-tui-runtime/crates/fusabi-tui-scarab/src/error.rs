//! Error types for Scarab shared memory integration.

/// Errors that can occur when working with Scarab shared memory.
#[derive(Debug, thiserror::Error)]
pub enum ScarabError {
    /// Failed to open or create shared memory
    #[error("Shared memory error: {0}")]
    SharedMemory(String),

    /// Shared memory size mismatch
    #[error("Shared memory size mismatch: expected {expected}, got {actual}")]
    SizeMismatch { expected: usize, actual: usize },

    /// Invalid shared memory layout
    #[error("Invalid shared memory layout: {0}")]
    InvalidLayout(String),

    /// Rendering error from the underlying renderer
    #[error("Render error: {0}")]
    Render(#[from] fusabi_tui_render::error::RenderError),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Plugin error
    #[cfg(feature = "plugin")]
    #[error("Plugin error: {0}")]
    Plugin(String),

    /// Buffer conversion error
    #[error("Buffer conversion error: {0}")]
    Conversion(String),
}

/// Type alias for Results in the Scarab integration.
pub type Result<T> = std::result::Result<T, ScarabError>;

impl From<shared_memory::ShmemError> for ScarabError {
    fn from(err: shared_memory::ShmemError) -> Self {
        ScarabError::SharedMemory(err.to_string())
    }
}
