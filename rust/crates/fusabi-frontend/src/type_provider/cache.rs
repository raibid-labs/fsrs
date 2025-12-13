//! Schema caching for type providers

use super::generator::GeneratedTypes;
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

struct CacheEntry {
    types: GeneratedTypes,
    created_at: Instant,
}

/// In-memory cache for resolved schemas
pub struct SchemaCache {
    entries: RwLock<HashMap<String, CacheEntry>>,
    default_ttl: Duration,
}

impl SchemaCache {
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            default_ttl: Duration::from_secs(300), // 5 minutes
        }
    }

    pub fn with_ttl(ttl_secs: u64) -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            default_ttl: Duration::from_secs(ttl_secs),
        }
    }

    /// Get cached types if not expired
    pub fn get(&self, key: &str) -> Option<GeneratedTypes> {
        let entries = self.entries.read().ok()?;
        let entry = entries.get(key)?;

        if entry.created_at.elapsed() < self.default_ttl {
            Some(entry.types.clone())
        } else {
            None
        }
    }

    /// Insert types into cache
    pub fn insert(&self, key: &str, types: GeneratedTypes) {
        if let Ok(mut entries) = self.entries.write() {
            entries.insert(key.to_string(), CacheEntry {
                types,
                created_at: Instant::now(),
            });
        }
    }

    /// Clear all cached entries
    pub fn clear(&self) {
        if let Ok(mut entries) = self.entries.write() {
            entries.clear();
        }
    }

    /// Remove expired entries
    pub fn evict_stale(&self) {
        if let Ok(mut entries) = self.entries.write() {
            entries.retain(|_, entry| entry.created_at.elapsed() < self.default_ttl);
        }
    }
}

impl Default for SchemaCache {
    fn default() -> Self {
        Self::new()
    }
}
