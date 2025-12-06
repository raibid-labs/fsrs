//! Manifest file handling for Fusabi packages.
//!
//! This module provides types and functions for working with fusabi.toml manifest files.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use thiserror::Error;

/// Type alias for package dependencies.
pub type Dependencies = HashMap<String, Dependency>;

/// Errors that can occur when working with manifests.
#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("Failed to read manifest file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed to parse manifest: {0}")]
    ParseError(#[from] toml::de::Error),

    #[error("Failed to serialize manifest: {0}")]
    SerializeError(#[from] toml::ser::Error),

    #[error("Invalid manifest: {0}")]
    InvalidManifest(String),
}

/// Represents a fusabi.toml manifest file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Manifest {
    /// Package metadata.
    pub package: Package,

    /// Package dependencies.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub dependencies: Dependencies,

    /// Development dependencies.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub dev_dependencies: Dependencies,
}

impl Manifest {
    /// Creates a new manifest with the given package information.
    pub fn new(package: Package) -> Self {
        Self {
            package,
            dependencies: HashMap::new(),
            dev_dependencies: HashMap::new(),
        }
    }

    /// Loads a manifest from a file path.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ManifestError> {
        let contents = fs::read_to_string(path)?;
        Self::parse(&contents)
    }

    /// Parses a manifest from a TOML string.
    pub fn parse(toml_str: &str) -> Result<Self, ManifestError> {
        let manifest: Manifest = toml::from_str(toml_str)?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Serializes the manifest to a TOML string.
    pub fn to_toml(&self) -> Result<String, ManifestError> {
        Ok(toml::to_string_pretty(self)?)
    }

    /// Validates the manifest structure.
    fn validate(&self) -> Result<(), ManifestError> {
        if self.package.name.is_empty() {
            return Err(ManifestError::InvalidManifest(
                "Package name cannot be empty".to_string(),
            ));
        }

        if self.package.version.is_empty() {
            return Err(ManifestError::InvalidManifest(
                "Package version cannot be empty".to_string(),
            ));
        }

        Ok(())
    }

    /// Adds a dependency to the manifest.
    pub fn add_dependency(&mut self, name: String, dependency: Dependency) {
        self.dependencies.insert(name, dependency);
    }

    /// Adds a development dependency to the manifest.
    pub fn add_dev_dependency(&mut self, name: String, dependency: Dependency) {
        self.dev_dependencies.insert(name, dependency);
    }
}

/// Package metadata from the manifest.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Package {
    /// The package name.
    pub name: String,

    /// The package version.
    pub version: String,

    /// Package authors.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub authors: Vec<String>,

    /// Package description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Package license.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,

    /// Package repository URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
}

impl Package {
    /// Creates a new package with the given name and version.
    pub fn new(name: String, version: String) -> Self {
        Self {
            name,
            version,
            authors: Vec::new(),
            description: None,
            license: None,
            repository: None,
        }
    }
}

/// Represents a dependency specification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Dependency {
    /// Simple version string (e.g., "1.0.0" or "^1.0").
    Simple(String),

    /// Detailed dependency specification.
    Detailed(DetailedDependency),
}

/// Detailed dependency specification with multiple options.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DetailedDependency {
    /// Version requirement.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Local file path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    /// Git repository URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git: Option<String>,

    /// Git revision (commit, branch, or tag).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rev: Option<String>,

    /// Whether this dependency is optional.
    #[serde(default, skip_serializing_if = "is_false")]
    pub optional: bool,
}

/// Helper function for serde to skip serializing false values.
fn is_false(b: &bool) -> bool {
    !b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_manifest() {
        let toml = r#"
[package]
name = "test-package"
version = "0.1.0"
authors = ["Test Author <test@example.com>"]
description = "A test package"
license = "MIT"
repository = "https://github.com/test/test"
"#;

        let manifest = Manifest::parse(toml).unwrap();
        assert_eq!(manifest.package.name, "test-package");
        assert_eq!(manifest.package.version, "0.1.0");
        assert_eq!(manifest.package.authors.len(), 1);
        assert_eq!(
            manifest.package.description,
            Some("A test package".to_string())
        );
        assert_eq!(manifest.package.license, Some("MIT".to_string()));
        assert_eq!(
            manifest.package.repository,
            Some("https://github.com/test/test".to_string())
        );
    }

    #[test]
    fn test_parse_manifest_with_dependencies() {
        let toml = r#"
[package]
name = "test-package"
version = "0.1.0"

[dependencies]
simple-dep = "1.0.0"

[dependencies.detailed-dep]
version = "2.0.0"
optional = true
"#;

        let manifest = Manifest::parse(toml).unwrap();
        assert_eq!(manifest.dependencies.len(), 2);

        let simple_dep = manifest.dependencies.get("simple-dep").unwrap();
        assert!(matches!(simple_dep, Dependency::Simple(v) if v == "1.0.0"));

        let detailed_dep = manifest.dependencies.get("detailed-dep").unwrap();
        match detailed_dep {
            Dependency::Detailed(d) => {
                assert_eq!(d.version, Some("2.0.0".to_string()));
                assert_eq!(d.optional, true);
            }
            _ => panic!("Expected detailed dependency"),
        }
    }

    #[test]
    fn test_parse_manifest_with_git_dependency() {
        let toml = r#"
[package]
name = "test-package"
version = "0.1.0"

[dependencies.git-dep]
git = "https://github.com/example/repo"
rev = "abc123"
"#;

        let manifest = Manifest::parse(toml).unwrap();
        let git_dep = manifest.dependencies.get("git-dep").unwrap();
        match git_dep {
            Dependency::Detailed(d) => {
                assert_eq!(d.git, Some("https://github.com/example/repo".to_string()));
                assert_eq!(d.rev, Some("abc123".to_string()));
            }
            _ => panic!("Expected detailed dependency"),
        }
    }

    #[test]
    fn test_parse_manifest_with_path_dependency() {
        let toml = r#"
[package]
name = "test-package"
version = "0.1.0"

[dependencies.local-dep]
path = "../local-package"
"#;

        let manifest = Manifest::parse(toml).unwrap();
        let local_dep = manifest.dependencies.get("local-dep").unwrap();
        match local_dep {
            Dependency::Detailed(d) => {
                assert_eq!(d.path, Some("../local-package".to_string()));
            }
            _ => panic!("Expected detailed dependency"),
        }
    }

    #[test]
    fn test_manifest_validation_empty_name() {
        let toml = r#"
[package]
name = ""
version = "0.1.0"
"#;

        let result = Manifest::parse(toml);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ManifestError::InvalidManifest(_)
        ));
    }

    #[test]
    fn test_manifest_validation_empty_version() {
        let toml = r#"
[package]
name = "test"
version = ""
"#;

        let result = Manifest::parse(toml);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ManifestError::InvalidManifest(_)
        ));
    }

    #[test]
    fn test_manifest_to_toml() {
        let package = Package::new("test-package".to_string(), "0.1.0".to_string());
        let mut manifest = Manifest::new(package);
        manifest.add_dependency("dep1".to_string(), Dependency::Simple("1.0.0".to_string()));

        let toml_str = manifest.to_toml().unwrap();
        assert!(toml_str.contains("name = \"test-package\""));
        assert!(toml_str.contains("version = \"0.1.0\""));
        assert!(toml_str.contains("dep1"));
    }

    #[test]
    fn test_add_dependency() {
        let package = Package::new("test".to_string(), "0.1.0".to_string());
        let mut manifest = Manifest::new(package);

        manifest.add_dependency("dep1".to_string(), Dependency::Simple("1.0.0".to_string()));
        assert_eq!(manifest.dependencies.len(), 1);

        manifest.add_dev_dependency(
            "dev-dep1".to_string(),
            Dependency::Simple("2.0.0".to_string()),
        );
        assert_eq!(manifest.dev_dependencies.len(), 1);
    }

    #[test]
    fn test_new_package() {
        let package = Package::new("my-package".to_string(), "1.0.0".to_string());
        assert_eq!(package.name, "my-package");
        assert_eq!(package.version, "1.0.0");
        assert!(package.authors.is_empty());
        assert!(package.description.is_none());
        assert!(package.license.is_none());
        assert!(package.repository.is_none());
    }
}
