//! Build functionality for the Fusabi Package Manager.
//!
//! This module handles compilation of Fusabi packages including dependency resolution.

use crate::manifest::{Dependency, DetailedDependency, Manifest};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Errors that can occur during the build process.
#[derive(Debug, Error)]
pub enum BuildError {
    #[error("Manifest not found: {0}")]
    ManifestNotFound(PathBuf),

    #[error("Main source file not found: {0}")]
    MainFileNotFound(PathBuf),

    #[error("Failed to read file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Manifest error: {0}")]
    ManifestError(#[from] crate::manifest::ManifestError),

    #[error("Compilation error: {0}")]
    CompileError(String),

    #[error("Dependency not found: {name} (expected at {path})")]
    DependencyNotFound { name: String, path: PathBuf },

    #[error("Cyclic dependency detected: {0}")]
    CyclicDependency(String),

    #[error("Dependency resolution failed: {0}")]
    DependencyResolutionFailed(String),
}

/// Result of a successful build.
#[derive(Debug)]
pub struct BuildResult {
    /// Path to the output bytecode file.
    pub output_path: PathBuf,
    /// Size of the output in bytes.
    pub output_size: usize,
    /// List of resolved dependencies.
    pub resolved_deps: Vec<ResolvedDependency>,
}

/// A resolved dependency with its path and metadata.
#[derive(Debug, Clone)]
pub struct ResolvedDependency {
    /// Name of the dependency.
    pub name: String,
    /// Resolved path to the dependency.
    pub path: PathBuf,
    /// Version if available.
    pub version: Option<String>,
}

/// Builder for Fusabi packages.
pub struct PackageBuilder {
    /// Root directory of the package.
    project_root: PathBuf,
    /// Path to fusabi_packages directory for installed dependencies.
    packages_dir: PathBuf,
    /// Verbose output flag.
    verbose: bool,
}

impl PackageBuilder {
    /// Creates a new package builder for the given project root.
    pub fn new(project_root: PathBuf) -> Self {
        let packages_dir = project_root.join("fusabi_packages");
        Self {
            project_root,
            packages_dir,
            verbose: false,
        }
    }

    /// Enables verbose output.
    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Builds the package.
    pub fn build(&self) -> Result<BuildResult, BuildError> {
        let manifest_path = self.project_root.join("fusabi.toml");
        if !manifest_path.exists() {
            return Err(BuildError::ManifestNotFound(manifest_path));
        }

        let manifest = Manifest::load(&manifest_path)?;

        if self.verbose {
            println!("Building {}...", manifest.package.name);
        }

        // Resolve dependencies
        let resolved_deps = self.resolve_dependencies(&manifest)?;

        if self.verbose && !resolved_deps.is_empty() {
            println!("Resolved {} dependencies:", resolved_deps.len());
            for dep in &resolved_deps {
                println!("  - {} @ {}", dep.name, dep.path.display());
            }
        }

        // Find and read main source file
        let main_path = self.project_root.join("src").join("main.fsx");
        if !main_path.exists() {
            return Err(BuildError::MainFileNotFound(main_path));
        }

        let source = fs::read_to_string(&main_path)?;

        // Compile to bytecode
        let bytecode = fusabi::compile_to_bytecode(&source)
            .map_err(|e| BuildError::CompileError(e.to_string()))?;

        // Create target directory
        let target_dir = self.project_root.join("target");
        if !target_dir.exists() {
            fs::create_dir_all(&target_dir)?;
        }

        // Write bytecode
        let output_path = target_dir.join(format!("{}.fzb", manifest.package.name));
        let output_size = bytecode.len();
        fs::write(&output_path, &bytecode)?;

        if self.verbose {
            println!("Build successful!");
            println!("Output: {} ({} bytes)", output_path.display(), output_size);
        }

        Ok(BuildResult {
            output_path,
            output_size,
            resolved_deps,
        })
    }

    /// Resolves all dependencies for the manifest.
    fn resolve_dependencies(
        &self,
        manifest: &Manifest,
    ) -> Result<Vec<ResolvedDependency>, BuildError> {
        let mut resolved = Vec::new();
        let mut visited = HashSet::new();

        for (name, dep) in &manifest.dependencies {
            self.resolve_dependency(name, dep, &mut resolved, &mut visited)?;
        }

        Ok(resolved)
    }

    /// Resolves a single dependency recursively.
    fn resolve_dependency(
        &self,
        name: &str,
        dependency: &Dependency,
        resolved: &mut Vec<ResolvedDependency>,
        visited: &mut HashSet<String>,
    ) -> Result<(), BuildError> {
        // Check for cycles
        if visited.contains(name) {
            return Err(BuildError::CyclicDependency(name.to_string()));
        }

        // Check if already resolved
        if resolved.iter().any(|d| d.name == name) {
            return Ok(());
        }

        visited.insert(name.to_string());

        let (dep_path, version) = self.locate_dependency(name, dependency)?;

        // Check if the dependency has its own manifest with dependencies
        let dep_manifest_path = dep_path.join("fusabi.toml");
        if dep_manifest_path.exists() {
            let dep_manifest = Manifest::load(&dep_manifest_path)?;

            // Recursively resolve transitive dependencies
            for (trans_name, trans_dep) in &dep_manifest.dependencies {
                self.resolve_dependency(trans_name, trans_dep, resolved, visited)?;
            }
        }

        resolved.push(ResolvedDependency {
            name: name.to_string(),
            path: dep_path,
            version,
        });

        visited.remove(name);

        Ok(())
    }

    /// Locates a dependency on the filesystem.
    fn locate_dependency(
        &self,
        name: &str,
        dependency: &Dependency,
    ) -> Result<(PathBuf, Option<String>), BuildError> {
        match dependency {
            Dependency::Simple(version) => {
                // Look in fusabi_packages directory
                let dep_path = self.packages_dir.join(name);
                if dep_path.exists() {
                    Ok((dep_path, Some(version.clone())))
                } else {
                    Err(BuildError::DependencyNotFound {
                        name: name.to_string(),
                        path: dep_path,
                    })
                }
            }
            Dependency::Detailed(DetailedDependency {
                path: Some(local_path),
                version,
                ..
            }) => {
                // Resolve relative path from project root
                let dep_path = if Path::new(local_path).is_absolute() {
                    PathBuf::from(local_path)
                } else {
                    self.project_root.join(local_path)
                };

                if dep_path.exists() {
                    Ok((dep_path, version.clone()))
                } else {
                    Err(BuildError::DependencyNotFound {
                        name: name.to_string(),
                        path: dep_path,
                    })
                }
            }
            Dependency::Detailed(DetailedDependency {
                git: Some(_git_url),
                ..
            }) => {
                // Git dependencies should be cloned to fusabi_packages
                let dep_path = self.packages_dir.join(name);
                if dep_path.exists() {
                    Ok((dep_path, None))
                } else {
                    Err(BuildError::DependencyResolutionFailed(format!(
                        "Git dependency '{}' not installed. Run 'fpm install' first.",
                        name
                    )))
                }
            }
            Dependency::Detailed(DetailedDependency { version, .. }) => {
                // Fallback to packages directory
                let dep_path = self.packages_dir.join(name);
                if dep_path.exists() {
                    Ok((dep_path, version.clone()))
                } else {
                    Err(BuildError::DependencyNotFound {
                        name: name.to_string(),
                        path: dep_path,
                    })
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::Package;
    use tempfile::TempDir;

    fn create_test_package(temp_dir: &TempDir, name: &str) -> PathBuf {
        let pkg_dir = temp_dir.path().join(name);
        fs::create_dir_all(pkg_dir.join("src")).unwrap();

        let manifest = Manifest::new(Package::new(name.to_string(), "0.1.0".to_string()));
        fs::write(pkg_dir.join("fusabi.toml"), manifest.to_toml().unwrap()).unwrap();

        fs::write(pkg_dir.join("src/main.fsx"), "42").unwrap();

        pkg_dir
    }

    #[test]
    fn test_build_simple_package() {
        let temp_dir = TempDir::new().unwrap();
        let pkg_dir = create_test_package(&temp_dir, "test-pkg");

        let builder = PackageBuilder::new(pkg_dir);
        let result = builder.build();

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.output_path.exists());
        assert!(result.resolved_deps.is_empty());
    }

    #[test]
    fn test_build_missing_manifest() {
        let temp_dir = TempDir::new().unwrap();

        let builder = PackageBuilder::new(temp_dir.path().to_path_buf());
        let result = builder.build();

        assert!(matches!(result, Err(BuildError::ManifestNotFound(_))));
    }

    #[test]
    fn test_build_missing_main_file() {
        let temp_dir = TempDir::new().unwrap();

        let manifest = Manifest::new(Package::new("test".to_string(), "0.1.0".to_string()));
        fs::write(
            temp_dir.path().join("fusabi.toml"),
            manifest.to_toml().unwrap(),
        )
        .unwrap();

        let builder = PackageBuilder::new(temp_dir.path().to_path_buf());
        let result = builder.build();

        assert!(matches!(result, Err(BuildError::MainFileNotFound(_))));
    }

    #[test]
    fn test_resolve_path_dependency() {
        let temp_dir = TempDir::new().unwrap();

        // Create the main package
        let main_pkg = create_test_package(&temp_dir, "main-pkg");

        // Create a dependency package
        let dep_pkg = create_test_package(&temp_dir, "dep-pkg");

        // Update main package manifest to depend on dep-pkg
        let mut manifest = Manifest::load(main_pkg.join("fusabi.toml")).unwrap();
        manifest.add_dependency(
            "dep-pkg".to_string(),
            Dependency::Detailed(DetailedDependency {
                path: Some(dep_pkg.to_string_lossy().to_string()),
                version: Some("0.1.0".to_string()),
                git: None,
                rev: None,
                optional: false,
            }),
        );
        fs::write(main_pkg.join("fusabi.toml"), manifest.to_toml().unwrap()).unwrap();

        let builder = PackageBuilder::new(main_pkg);
        let result = builder.build().unwrap();

        assert_eq!(result.resolved_deps.len(), 1);
        assert_eq!(result.resolved_deps[0].name, "dep-pkg");
    }
}
