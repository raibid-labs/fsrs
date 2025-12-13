//! Type provider registry

use super::{TypeProvider, ProviderParams, GeneratedTypes};
use super::error::{ProviderError, ProviderResult};
use super::cache::SchemaCache;
use std::collections::HashMap;
use std::sync::Arc;

/// Registry of available type providers
pub struct ProviderRegistry {
    providers: HashMap<String, Arc<dyn TypeProvider>>,
    cache: SchemaCache,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            cache: SchemaCache::new(),
        }
    }

    /// Register a type provider
    pub fn register(&mut self, provider: Arc<dyn TypeProvider>) {
        self.providers.insert(provider.name().to_string(), provider);
    }

    /// Get a provider by name
    pub fn get(&self, name: &str) -> Option<Arc<dyn TypeProvider>> {
        self.providers.get(name).cloned()
    }

    /// Resolve types from a provider declaration
    pub fn resolve(
        &self,
        provider_name: &str,
        source: &str,
        namespace: &str,
        params: &ProviderParams,
    ) -> ProviderResult<GeneratedTypes> {
        let provider = self.providers.get(provider_name)
            .ok_or_else(|| ProviderError::UnknownProvider(provider_name.to_string()))?;

        // Check cache
        let cache_key = format!("{}:{}", provider_name, source);
        if let Some(cached) = self.cache.get(&cache_key) {
            return Ok(cached);
        }

        // Resolve and generate
        let schema = provider.resolve_schema(source, params)?;
        let types = provider.generate_types(&schema, namespace)?;

        // Cache result
        self.cache.insert(&cache_key, types.clone());

        Ok(types)
    }

    /// List registered providers
    pub fn list_providers(&self) -> Vec<&str> {
        self.providers.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}
