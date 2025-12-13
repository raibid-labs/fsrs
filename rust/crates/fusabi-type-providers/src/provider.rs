//! Core type provider trait and infrastructure

use crate::error::{ProviderResult};
use crate::generator::GeneratedTypes;
use std::collections::HashMap;

/// Parameters passed to type providers
#[derive(Debug, Clone, Default)]
pub struct ProviderParams {
    pub cache_ttl_secs: Option<u64>,
    pub custom: HashMap<String, String>,
}

/// Result of schema resolution
#[derive(Debug, Clone)]
pub enum Schema {
    JsonSchema(serde_json::Value),
    OpenApi(serde_json::Value),
    Custom(String),
}

/// Core trait that all type providers must implement
pub trait TypeProvider: Send + Sync {
    /// Provider identifier
    fn name(&self) -> &str;

    /// Resolve schema from source URI
    fn resolve_schema(&self, source: &str, params: &ProviderParams) -> ProviderResult<Schema>;

    /// Generate Fusabi types from schema
    fn generate_types(&self, schema: &Schema, namespace: &str) -> ProviderResult<GeneratedTypes>;

    /// Get documentation for a type path (optional)
    fn get_documentation(&self, _type_path: &str) -> Option<String> {
        None
    }
}
