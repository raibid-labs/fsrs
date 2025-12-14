//! File loading and dependency tracking for the Fusabi TUI engine.

use crate::error::{LoadError, LoadResult};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Represents a loaded file with metadata and content.
#[derive(Debug, Clone)]
pub struct LoadedFile {
    /// Canonical path to the file.
    pub path: PathBuf,

    /// Raw file content as a string.
    pub content: String,

    /// Last modification time.
    pub modified: SystemTime,

    /// Files that this file depends on.
    pub dependencies: Vec<PathBuf>,
}

/// Manages file loading with dependency tracking and caching.
#[derive(Debug)]
pub struct FileLoader {
    /// Cache of loaded files keyed by canonical path.
    cache: HashMap<PathBuf, LoadedFile>,

    /// Reverse dependency map: file -> files that depend on it.
    dependents: HashMap<PathBuf, HashSet<PathBuf>>,
}

impl FileLoader {
    /// Create a new file loader.
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            dependents: HashMap::new(),
        }
    }

    /// Load a file from disk, using cache if available and up-to-date.
    ///
    /// This method will:
    /// - Check if the file is already cached and up-to-date
    /// - Read the file from disk if needed
    /// - Parse dependencies (can be extended for .fsx files)
    /// - Update the cache and dependency graph
    pub fn load(&mut self, path: &Path) -> LoadResult<&LoadedFile> {
        let canonical_path = path
            .canonicalize()
            .map_err(|_| LoadError::FileNotFound {
                path: path.to_path_buf(),
            })?;

        // Check if we have a valid cached version
        let needs_reload = if let Some(cached) = self.cache.get(&canonical_path) {
            if let Ok(metadata) = fs::metadata(&canonical_path) {
                if let Ok(modified) = metadata.modified() {
                    modified > cached.modified
                } else {
                    true
                }
            } else {
                true
            }
        } else {
            true
        };

        if !needs_reload {
            return Ok(self.cache.get(&canonical_path).unwrap());
        }

        // Load the file from disk
        let content = fs::read_to_string(&canonical_path).map_err(|e| {
            LoadError::ReadFailed {
                path: canonical_path.clone(),
                source: e,
            }
        })?;

        let metadata = fs::metadata(&canonical_path).map_err(|e| LoadError::ReadFailed {
            path: canonical_path.clone(),
            source: e,
        })?;

        let modified = metadata
            .modified()
            .map_err(|e| LoadError::ReadFailed {
                path: canonical_path.clone(),
                source: e,
            })?;

        // Parse dependencies (simplified - can be extended for .fsx imports)
        let dependencies = self.parse_dependencies(&canonical_path, &content)?;

        // Update reverse dependency map
        for dep in &dependencies {
            self.dependents
                .entry(dep.clone())
                .or_insert_with(HashSet::new)
                .insert(canonical_path.clone());
        }

        // Create and cache the loaded file
        let loaded = LoadedFile {
            path: canonical_path.clone(),
            content,
            modified,
            dependencies,
        };

        self.cache.insert(canonical_path.clone(), loaded);

        Ok(self.cache.get(&canonical_path).unwrap())
    }

    /// Invalidate a file in the cache and return all files that depend on it.
    ///
    /// This is useful when a file changes - you can invalidate it and reload
    /// all dependent files.
    pub fn invalidate(&mut self, path: &Path) -> Vec<PathBuf> {
        let canonical_path = match path.canonicalize() {
            Ok(p) => p,
            Err(_) => return Vec::new(),
        };

        // Remove from cache
        self.cache.remove(&canonical_path);

        // Get all dependents
        let dependents = self.get_dependents(&canonical_path);

        // Recursively invalidate dependents
        let mut all_invalidated = vec![canonical_path.clone()];
        for dep in &dependents {
            if !all_invalidated.contains(dep) {
                all_invalidated.push(dep.clone());
                self.cache.remove(dep);
            }
        }

        all_invalidated
    }

    /// Get all files that depend on the given file (transitively).
    pub fn get_dependents(&self, path: &Path) -> Vec<PathBuf> {
        let canonical_path = match path.canonicalize() {
            Ok(p) => p,
            Err(_) => return Vec::new(),
        };

        let mut visited = HashSet::new();
        let mut result = Vec::new();

        self.collect_dependents(&canonical_path, &mut visited, &mut result);

        result
    }

    /// Recursively collect all dependents.
    fn collect_dependents(
        &self,
        path: &Path,
        visited: &mut HashSet<PathBuf>,
        result: &mut Vec<PathBuf>,
    ) {
        if !visited.insert(path.to_path_buf()) {
            return;
        }

        if let Some(deps) = self.dependents.get(path) {
            for dep in deps {
                result.push(dep.clone());
                self.collect_dependents(dep, visited, result);
            }
        }
    }

    /// Parse dependencies from file content.
    ///
    /// This is a simplified implementation. In a real system, this would
    /// parse .fsx files for import/open statements and resolve relative paths.
    fn parse_dependencies(&self, _path: &Path, _content: &str) -> LoadResult<Vec<PathBuf>> {
        // TODO: Implement proper dependency parsing for .fsx files
        // For now, return empty dependencies
        Ok(Vec::new())
    }

    /// Check if a file is loaded in the cache.
    pub fn is_cached(&self, path: &Path) -> bool {
        if let Ok(canonical) = path.canonicalize() {
            self.cache.contains_key(&canonical)
        } else {
            false
        }
    }

    /// Get a loaded file from the cache without loading it.
    pub fn get(&self, path: &Path) -> Option<&LoadedFile> {
        let canonical = path.canonicalize().ok()?;
        self.cache.get(&canonical)
    }

    /// Clear all cached files.
    pub fn clear(&mut self) {
        self.cache.clear();
        self.dependents.clear();
    }

    /// Get the number of cached files.
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

impl Default for FileLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_file_loader_new() {
        let loader = FileLoader::new();
        assert_eq!(loader.len(), 0);
        assert!(loader.is_empty());
    }

    #[test]
    fn test_load_file() {
        let mut loader = FileLoader::new();

        // Create a temporary file
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "let x = 42").unwrap();
        temp_file.flush().unwrap();

        let path = temp_file.path();

        // Load the file
        let loaded = loader.load(path).unwrap();
        assert_eq!(loaded.content.trim(), "let x = 42");
        assert_eq!(loader.len(), 1);
    }

    #[test]
    fn test_load_nonexistent_file() {
        let mut loader = FileLoader::new();
        let result = loader.load(Path::new("/nonexistent/file.fsx"));
        assert!(result.is_err());
    }

    #[test]
    fn test_cache_hit() {
        let mut loader = FileLoader::new();

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "let x = 42").unwrap();
        temp_file.flush().unwrap();

        let path = temp_file.path();

        // First load
        let loaded1 = loader.load(path).unwrap();
        let ptr1 = loaded1 as *const LoadedFile;

        // Second load should use cache
        let loaded2 = loader.load(path).unwrap();
        let ptr2 = loaded2 as *const LoadedFile;

        assert_eq!(ptr1, ptr2);
    }

    #[test]
    fn test_invalidate() {
        let mut loader = FileLoader::new();

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "let x = 42").unwrap();
        temp_file.flush().unwrap();

        let path = temp_file.path();

        // Load the file
        loader.load(path).unwrap();
        assert!(loader.is_cached(path));

        // Invalidate it
        let invalidated = loader.invalidate(path);
        assert_eq!(invalidated.len(), 1);
        assert!(!loader.is_cached(path));
    }

    #[test]
    fn test_get_cached() {
        let mut loader = FileLoader::new();

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "let x = 42").unwrap();
        temp_file.flush().unwrap();

        let path = temp_file.path();

        // Before loading
        assert!(loader.get(path).is_none());

        // After loading
        loader.load(path).unwrap();
        assert!(loader.get(path).is_some());
    }

    #[test]
    fn test_clear() {
        let mut loader = FileLoader::new();

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "let x = 42").unwrap();
        temp_file.flush().unwrap();

        loader.load(temp_file.path()).unwrap();
        assert_eq!(loader.len(), 1);

        loader.clear();
        assert_eq!(loader.len(), 0);
        assert!(loader.is_empty());
    }
}
