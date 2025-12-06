//! Package publishing functionality for fpm.
//!
//! Handles generating registry entries and creating PRs to the fusabi-community registry.

use crate::manifest::{Manifest, ManifestError, Package};
use std::path::Path;
use std::process::Command;
use thiserror::Error;

const REGISTRY_REPO: &str = "fusabi-lang/fusabi-community";
const REGISTRY_FILE: &str = "registry/index.toml";

#[derive(Debug, Error)]
pub enum PublishError {
    #[error("Manifest error: {0}")]
    Manifest(#[from] ManifestError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Package validation failed: {0}")]
    Validation(String),

    #[error("Missing required field: {0}")]
    MissingField(String),
}

/// Result of a successful publish preparation.
#[derive(Debug)]
pub struct PublishResult {
    pub package_name: String,
    pub version: String,
    pub registry_entry: String,
    pub gh_available: bool,
}

/// Validates that a package has all required fields for publishing.
fn validate_package(package: &Package) -> Result<(), PublishError> {
    if package.name.is_empty() {
        return Err(PublishError::MissingField("name".to_string()));
    }

    if package.version.is_empty() {
        return Err(PublishError::MissingField("version".to_string()));
    }

    if package.description.is_none() {
        return Err(PublishError::Validation(
            "Package must have a description for publishing".to_string(),
        ));
    }

    if package.repository.is_none() {
        return Err(PublishError::Validation(
            "Package must have a repository URL for publishing".to_string(),
        ));
    }

    Ok(())
}

/// Generates a TOML entry for the registry index.
fn generate_registry_entry(package: &Package) -> String {
    let mut entry = format!(
        r#"[[packages]]
name = "{}"
version = "{}"
"#,
        package.name, package.version
    );

    if let Some(ref desc) = package.description {
        entry.push_str(&format!("description = \"{}\"\n", desc));
    }

    if let Some(ref repo) = package.repository {
        entry.push_str(&format!("repository = \"{}\"\n", repo));
    }

    if let Some(ref license) = package.license {
        entry.push_str(&format!("license = \"{}\"\n", license));
    }

    if !package.authors.is_empty() {
        let authors: Vec<String> = package
            .authors
            .iter()
            .map(|a| format!("\"{}\"", a))
            .collect();
        entry.push_str(&format!("authors = [{}]\n", authors.join(", ")));
    }

    entry
}

/// Checks if the GitHub CLI is available.
fn is_gh_available() -> bool {
    Command::new("gh")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Publishes a package by generating a registry entry and providing PR instructions.
pub fn publish_package<P: AsRef<Path>>(project_dir: P) -> Result<PublishResult, PublishError> {
    let manifest_path = project_dir.as_ref().join("fusabi.toml");
    let manifest = Manifest::load(&manifest_path)?;

    validate_package(&manifest.package)?;

    let registry_entry = generate_registry_entry(&manifest.package);
    let gh_available = is_gh_available();

    Ok(PublishResult {
        package_name: manifest.package.name,
        version: manifest.package.version,
        registry_entry,
        gh_available,
    })
}

/// Prints instructions for creating a PR to the registry.
pub fn print_publish_instructions(result: &PublishResult) {
    println!(
        "📦 Preparing to publish {} v{}",
        result.package_name, result.version
    );
    println!();
    println!("Registry entry to add to {}:", REGISTRY_FILE);
    println!("─────────────────────────────────────────");
    println!("{}", result.registry_entry);
    println!("─────────────────────────────────────────");
    println!();

    if result.gh_available {
        println!("GitHub CLI detected! You can create a PR with:");
        println!();
        println!("  1. Fork and clone https://github.com/{}", REGISTRY_REPO);
        println!("  2. Add the above entry to {}", REGISTRY_FILE);
        println!("  3. Create a PR:");
        println!(
            "     gh pr create --title \"Add {} v{}\" --body \"Adds {} package to the registry\"",
            result.package_name, result.version, result.package_name
        );
    } else {
        println!("To publish your package:");
        println!();
        println!("  1. Fork https://github.com/{}", REGISTRY_REPO);
        println!("  2. Clone your fork locally");
        println!("  3. Add the above entry to {}", REGISTRY_FILE);
        println!("  4. Commit and push your changes");
        println!("  5. Create a pull request to the main repository");
    }

    println!();
    println!("Once your PR is merged, users can install your package with:");
    println!("  fpm add {}", result.package_name);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_package_missing_description() {
        let package = Package {
            name: "test".to_string(),
            version: "0.1.0".to_string(),
            authors: vec![],
            description: None,
            license: None,
            repository: Some("https://github.com/test/test".to_string()),
        };

        let result = validate_package(&package);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PublishError::Validation(_)));
    }

    #[test]
    fn test_validate_package_missing_repository() {
        let package = Package {
            name: "test".to_string(),
            version: "0.1.0".to_string(),
            authors: vec![],
            description: Some("A test package".to_string()),
            license: None,
            repository: None,
        };

        let result = validate_package(&package);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PublishError::Validation(_)));
    }

    #[test]
    fn test_validate_package_success() {
        let package = Package {
            name: "test".to_string(),
            version: "0.1.0".to_string(),
            authors: vec!["Author".to_string()],
            description: Some("A test package".to_string()),
            license: Some("MIT".to_string()),
            repository: Some("https://github.com/test/test".to_string()),
        };

        let result = validate_package(&package);
        assert!(result.is_ok());
    }

    #[test]
    fn test_generate_registry_entry() {
        let package = Package {
            name: "my-package".to_string(),
            version: "1.0.0".to_string(),
            authors: vec!["Alice".to_string(), "Bob".to_string()],
            description: Some("A cool package".to_string()),
            license: Some("MIT".to_string()),
            repository: Some("https://github.com/test/my-package".to_string()),
        };

        let entry = generate_registry_entry(&package);

        assert!(entry.contains("name = \"my-package\""));
        assert!(entry.contains("version = \"1.0.0\""));
        assert!(entry.contains("description = \"A cool package\""));
        assert!(entry.contains("repository = \"https://github.com/test/my-package\""));
        assert!(entry.contains("license = \"MIT\""));
        assert!(entry.contains("authors = [\"Alice\", \"Bob\"]"));
    }

    #[test]
    fn test_generate_registry_entry_minimal() {
        let package = Package {
            name: "minimal".to_string(),
            version: "0.1.0".to_string(),
            authors: vec![],
            description: Some("Minimal".to_string()),
            license: None,
            repository: Some("https://github.com/test/minimal".to_string()),
        };

        let entry = generate_registry_entry(&package);

        assert!(entry.contains("name = \"minimal\""));
        assert!(entry.contains("version = \"0.1.0\""));
        assert!(!entry.contains("license"));
        assert!(!entry.contains("authors"));
    }
}
