//! Error types for type providers

use std::fmt;

pub type ProviderResult<T> = Result<T, ProviderError>;

#[derive(Debug, Clone)]
pub enum ProviderError {
    /// Schema fetch failed
    FetchError(String),
    /// Schema parsing failed
    ParseError(String),
    /// Type generation failed
    GenerationError(String),
    /// Unknown provider
    UnknownProvider(String),
    /// Invalid source URI
    InvalidSource(String),
    /// Cache error
    CacheError(String),
    /// IO error
    IoError(String),
}

impl fmt::Display for ProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FetchError(msg) => write!(f, "Schema fetch error: {}", msg),
            Self::ParseError(msg) => write!(f, "Schema parse error: {}", msg),
            Self::GenerationError(msg) => write!(f, "Type generation error: {}", msg),
            Self::UnknownProvider(name) => write!(f, "Unknown type provider: {}", name),
            Self::InvalidSource(uri) => write!(f, "Invalid source URI: {}", uri),
            Self::CacheError(msg) => write!(f, "Cache error: {}", msg),
            Self::IoError(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

impl std::error::Error for ProviderError {}
