//! Install command for cloning git dependencies.

use crate::manifest::{Dependency, Manifest, ManifestError};
use crate::registry::{Registry, RegistryError};
use git2::Repository;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InstallError {
    #[error("Manifest error: {0}")]
    Manifest(#[from] ManifestError),

    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Registry error: {0}")]
    Registry(#[from] RegistryError),

    #[error("{0}")]
    Other(String),
}

pub fn install_dependencies(project_dir: &Path) -> Result<(), InstallError> {
    let manifest_path = project_dir.join("fusabi.toml");
    if !manifest_path.exists() {
        return Err(InstallError::Other(
            "fusabi.toml not found. Run 'fpm init' first.".to_string(),
        ));
    }

    let manifest = Manifest::load(&manifest_path)?;
    let packages_dir = project_dir.join("fusabi_packages");

    if manifest.dependencies.is_empty() {
        println!("No dependencies to install.");
        return Ok(());
    }

    std::fs::create_dir_all(&packages_dir)?;

    let registry = Registry::new();

    for (name, dependency) in &manifest.dependencies {
        match dependency {
            Dependency::Detailed(detailed) => {
                if let Some(git_url) = &detailed.git {
                    install_git_dependency(name, git_url, detailed.rev.as_deref(), &packages_dir)?;
                } else if detailed.path.is_some() {
                    println!("Skipping local path dependency '{}'", name);
                } else if let Some(version) = &detailed.version {
                    install_registry_dependency(name, version, &packages_dir, &registry)?;
                } else {
                    println!(
                        "Skipping dependency '{}': no git, path, or version specified",
                        name
                    );
                }
            }
            Dependency::Simple(version) => {
                install_registry_dependency(name, version, &packages_dir, &registry)?;
            }
        }
    }

    println!("Install complete.");
    Ok(())
}

fn install_git_dependency(
    name: &str,
    git_url: &str,
    rev: Option<&str>,
    packages_dir: &Path,
) -> Result<(), InstallError> {
    let dep_path = packages_dir.join(name);

    if dep_path.exists() {
        println!("Dependency '{}' already exists, skipping.", name);
        return Ok(());
    }

    println!("Cloning '{}'...", name);

    let repo = Repository::clone(git_url, &dep_path)?;

    if let Some(rev) = rev {
        checkout_rev(&repo, rev)?;
    }

    println!("Installed '{}'", name);
    Ok(())
}

fn install_registry_dependency(
    name: &str,
    version_constraint: &str,
    packages_dir: &Path,
    registry: &Registry,
) -> Result<(), InstallError> {
    let dep_path = packages_dir.join(name);

    if dep_path.exists() {
        println!("Dependency '{}' already exists, skipping.", name);
        return Ok(());
    }

    println!("Resolving '{}' ({})...", name, version_constraint);

    let resolved = registry.resolve(name, version_constraint)?;

    println!(
        "Installing '{}' v{} from {}",
        resolved.name, resolved.version, resolved.git_url
    );

    let repo = Repository::clone(&resolved.git_url, &dep_path)?;

    if let Some(ref rev) = resolved.rev {
        if let Err(e) = checkout_rev(&repo, rev) {
            println!(
                "Warning: Could not checkout version tag '{}': {}. Using default branch.",
                rev, e
            );
        }
    }

    println!("Installed '{}' v{}", resolved.name, resolved.version);
    Ok(())
}

fn checkout_rev(repo: &Repository, rev: &str) -> Result<(), git2::Error> {
    let (object, reference) = repo.revparse_ext(rev)?;
    repo.checkout_tree(&object, None)?;

    match reference {
        Some(gref) => repo.set_head(gref.name().unwrap()),
        None => repo.set_head_detached(object.id()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_install_no_manifest() {
        let temp_dir = TempDir::new().unwrap();
        let result = install_dependencies(temp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_install_empty_dependencies() {
        let temp_dir = TempDir::new().unwrap();
        let manifest_content = r#"
[package]
name = "test"
version = "0.1.0"
"#;
        std::fs::write(temp_dir.path().join("fusabi.toml"), manifest_content).unwrap();

        let result = install_dependencies(temp_dir.path());
        assert!(result.is_ok());
    }
}
