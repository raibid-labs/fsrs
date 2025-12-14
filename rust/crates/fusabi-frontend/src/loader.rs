//! File loader for multi-file module system
//!
//! This module implements the `FileLoader` for loading and caching `.fsx` files
//! referenced by `#load` directives. It handles:
//! - Path resolution (relative and absolute paths)
//! - Circular dependency detection
//! - Caching of loaded files to avoid recompilation
//!
//! # Example
//!
//! ```rust
//! use fusabi_frontend::loader::FileLoader;
//! use std::path::PathBuf;
//!
//! let mut loader = FileLoader::new(PathBuf::from("."));
//! // Load a file (with circular dependency detection and caching)
//! // let loaded = loader.load("utils.fsx", &PathBuf::from("main.fsx")).unwrap();
//! ```

use crate::ast::Program;
use crate::lexer::{LexError, Lexer};
use crate::parser::{ParseError, Parser};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::path::{Path, PathBuf};

/// Errors that can occur during file loading
#[derive(Debug, Clone)]
pub enum LoadError {
    /// File not found at the specified path
    FileNotFound(PathBuf),
    /// Circular dependency detected
    CircularDependency(Vec<PathBuf>),
    /// Parse error in loaded file
    ParseError(PathBuf, ParseError),
    /// Lexer error in loaded file
    LexError(PathBuf, LexError),
    /// IO error reading file
    IoError(String),
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadError::FileNotFound(path) => write!(f, "File not found: {}", path.display()),
            LoadError::CircularDependency(paths) => {
                write!(f, "Circular dependency detected: ")?;
                for (i, path) in paths.iter().enumerate() {
                    if i > 0 {
                        write!(f, " -> ")?;
                    }
                    write!(f, "{}", path.display())?;
                }
                Ok(())
            }
            LoadError::ParseError(path, err) => {
                write!(f, "Parse error in {}: {}", path.display(), err)
            }
            LoadError::LexError(path, err) => write!(f, "Lex error in {}: {}", path.display(), err),
            LoadError::IoError(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

impl std::error::Error for LoadError {}

/// A loaded file with its parsed program
#[derive(Debug, Clone)]
pub struct LoadedFile {
    /// Canonical path to the file
    pub path: PathBuf,
    /// Parsed program AST
    pub program: Program,
}

/// File loader with caching and circular dependency detection
pub struct FileLoader {
    /// Cache of already-loaded files (canonical path -> LoadedFile)
    cache: HashMap<PathBuf, LoadedFile>,
    /// Currently loading files (for cycle detection)
    loading: HashSet<PathBuf>,
    /// Base directory for relative path resolution
    base_dir: PathBuf,
}

impl FileLoader {
    /// Create a new file loader with the given base directory
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            cache: HashMap::new(),
            loading: HashSet::new(),
            base_dir,
        }
    }

    /// Load a file and all its dependencies
    ///
    /// # Arguments
    /// * `path` - Path to load (can be relative or absolute)
    /// * `from_file` - The file containing the #load directive (for relative path resolution)
    ///
    /// # Returns
    /// Reference to the loaded file (from cache)
    pub fn load(&mut self, path: &str, from_file: &Path) -> Result<&LoadedFile, LoadError> {
        let resolved = self.resolve_path(path, from_file)?;

        // Check cache first
        if self.cache.contains_key(&resolved) {
            return Ok(self.cache.get(&resolved).unwrap());
        }

        // Check for cycles
        if self.loading.contains(&resolved) {
            let mut cycle: Vec<PathBuf> = self.loading.iter().cloned().collect();
            cycle.push(resolved.clone());
            return Err(LoadError::CircularDependency(cycle));
        }

        // Mark as loading
        self.loading.insert(resolved.clone());

        // Read and parse file
        let source = std::fs::read_to_string(&resolved)
            .map_err(|e| LoadError::IoError(format!("{}: {}", resolved.display(), e)))?;

        let mut lexer = Lexer::new(&source);
        let tokens = lexer
            .tokenize()
            .map_err(|e| LoadError::LexError(resolved.clone(), e))?;

        let mut parser = Parser::new(tokens);
        let program = parser
            .parse_program()
            .map_err(|e| LoadError::ParseError(resolved.clone(), e))?;

        // Recursively load dependencies
        for directive in &program.directives {
            self.load(&directive.path, &resolved)?;
        }

        // Create loaded file
        let loaded = LoadedFile {
            path: resolved.clone(),
            program,
        };

        // Remove from loading, add to cache
        self.loading.remove(&resolved);
        self.cache.insert(resolved.clone(), loaded);

        Ok(self.cache.get(&resolved).unwrap())
    }

    /// Resolve a path relative to a source file
    fn resolve_path(&self, path: &str, from_file: &Path) -> Result<PathBuf, LoadError> {
        let resolved = if path.starts_with('/') {
            // Absolute path
            PathBuf::from(path)
        } else {
            // Relative to from_file's directory
            from_file
                .parent()
                .unwrap_or(&self.base_dir)
                .join(path)
        };

        // Canonicalize to get absolute path and resolve symlinks
        resolved
            .canonicalize()
            .map_err(|_| LoadError::FileNotFound(resolved.clone()))
    }

    /// Get a loaded file from cache (if it exists)
    pub fn get_cached(&self, path: &Path) -> Option<&LoadedFile> {
        self.cache.get(path)
    }

    /// Clear the cache (useful for hot reload scenarios)
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Invalidate a specific file in the cache
    pub fn invalidate(&mut self, path: &Path) {
        self.cache.remove(path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_load_simple_file() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.fsx");
        let from_file = temp_dir.path().join("from.fsx");
        fs::write(&test_file, "let x = 42").unwrap();

        let mut loader = FileLoader::new(temp_dir.path().to_path_buf());
        let loaded = loader.load("test.fsx", &from_file).unwrap();

        assert_eq!(loaded.program.items.len(), 1);
    }

    #[test]
    fn test_circular_dependency_detection() {
        let temp_dir = TempDir::new().unwrap();
        let a_file = temp_dir.path().join("a.fsx");
        let b_file = temp_dir.path().join("b.fsx");

        fs::write(&a_file, "#load \"b.fsx\"\nlet a = 1").unwrap();
        fs::write(&b_file, "#load \"a.fsx\"\nlet b = 2").unwrap();

        let mut loader = FileLoader::new(temp_dir.path().to_path_buf());
        let result = loader.load("a.fsx", &a_file);

        assert!(matches!(result, Err(LoadError::CircularDependency(_))));
    }

    #[test]
    fn test_caching() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("cached.fsx");
        let from_file = temp_dir.path().join("from.fsx");
        fs::write(&test_file, "let x = 42").unwrap();

        let mut loader = FileLoader::new(temp_dir.path().to_path_buf());

        // Load twice
        loader.load("cached.fsx", &from_file).unwrap();
        loader.load("cached.fsx", &from_file).unwrap();

        // Should only be loaded once
        assert_eq!(loader.cache.len(), 1);
    }

    #[test]
    fn test_nested_loads() {
        let temp_dir = TempDir::new().unwrap();
        let utils_file = temp_dir.path().join("utils.fsx");
        let main_file = temp_dir.path().join("main.fsx");

        fs::write(&utils_file, "let helper x = x + 1").unwrap();
        fs::write(&main_file, "#load \"utils.fsx\"\nlet result = helper 42").unwrap();

        let mut loader = FileLoader::new(temp_dir.path().to_path_buf());
        let loaded = loader.load("main.fsx", &main_file).unwrap();

        // main.fsx should be loaded, and utils.fsx should be in cache
        assert_eq!(loaded.program.directives.len(), 1);
        assert_eq!(loader.cache.len(), 2); // both main and utils
    }

    #[test]
    fn test_file_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let from_file = temp_dir.path().join("from.fsx");
        let mut loader = FileLoader::new(temp_dir.path().to_path_buf());
        let result = loader.load("nonexistent.fsx", &from_file);

        assert!(matches!(result, Err(LoadError::FileNotFound(_))));
    }
}
