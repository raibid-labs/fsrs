//! Registry handling for the fusabi-community package registry.
//!
//! Provides functionality to fetch and parse the registry index, and resolve
//! package names to git URLs with version constraints.

use git2::Repository;
use serde::Deserialize;
use std::cmp::Ordering;
use std::path::{Path, PathBuf};
use thiserror::Error;

const REGISTRY_URL: &str = "https://github.com/fusabi-lang/fusabi-community";
const REGISTRY_INDEX_PATH: &str = "registry/index.toml";

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to parse registry index: {0}")]
    Parse(#[from] toml::de::Error),

    #[error("Package '{0}' not found in registry")]
    PackageNotFound(String),

    #[error("No version of '{0}' satisfies constraint '{1}'")]
    NoMatchingVersion(String, String),

    #[error("Invalid version format: {0}")]
    InvalidVersion(String),
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegistryIndex {
    #[serde(default)]
    pub packages: Vec<RegistryPackage>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegistryPackage {
    pub name: String,
    pub version: String,
    pub repository: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub authors: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ResolvedPackage {
    pub name: String,
    pub version: String,
    pub git_url: String,
    pub rev: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemVer {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
}

impl SemVer {
    pub fn parse(version: &str) -> Result<Self, RegistryError> {
        let version = version.trim_start_matches('v');
        let parts: Vec<&str> = version.split('.').collect();

        let major = parts
            .first()
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| RegistryError::InvalidVersion(version.to_string()))?;

        let minor = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);

        let patch = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);

        Ok(SemVer {
            major,
            minor,
            patch,
        })
    }

    pub fn satisfies(&self, constraint: &str) -> bool {
        let constraint = constraint.trim();

        if constraint.is_empty() || constraint == "*" {
            return true;
        }

        if constraint.starts_with('^') {
            if let Ok(base) = SemVer::parse(&constraint[1..]) {
                return self.major == base.major
                    && (self.minor > base.minor
                        || (self.minor == base.minor && self.patch >= base.patch));
            }
        }

        if constraint.starts_with('~') {
            if let Ok(base) = SemVer::parse(&constraint[1..]) {
                return self.major == base.major
                    && self.minor == base.minor
                    && self.patch >= base.patch;
            }
        }

        if constraint.starts_with(">=") {
            if let Ok(base) = SemVer::parse(&constraint[2..]) {
                return *self >= base;
            }
        }

        if constraint.starts_with('>') {
            if let Ok(base) = SemVer::parse(&constraint[1..]) {
                return *self > base;
            }
        }

        if constraint.starts_with("<=") {
            if let Ok(base) = SemVer::parse(&constraint[2..]) {
                return *self <= base;
            }
        }

        if constraint.starts_with('<') {
            if let Ok(base) = SemVer::parse(&constraint[1..]) {
                return *self < base;
            }
        }

        if constraint.starts_with('=') {
            if let Ok(base) = SemVer::parse(&constraint[1..]) {
                return *self == base;
            }
        }

        if let Ok(base) = SemVer::parse(constraint) {
            return *self == base;
        }

        false
    }
}

impl PartialOrd for SemVer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SemVer {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.major.cmp(&other.major) {
            Ordering::Equal => match self.minor.cmp(&other.minor) {
                Ordering::Equal => self.patch.cmp(&other.patch),
                ord => ord,
            },
            ord => ord,
        }
    }
}

pub struct Registry {
    cache_dir: PathBuf,
}

impl Registry {
    pub fn new() -> Self {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("fusabi")
            .join("registry");

        Self { cache_dir }
    }

    pub fn with_cache_dir<P: AsRef<Path>>(cache_dir: P) -> Self {
        Self {
            cache_dir: cache_dir.as_ref().to_path_buf(),
        }
    }

    pub fn update(&self) -> Result<(), RegistryError> {
        std::fs::create_dir_all(&self.cache_dir)?;

        let repo_path = self.cache_dir.join("fusabi-community");

        if repo_path.exists() {
            println!("Updating registry index...");
            let repo = Repository::open(&repo_path)?;
            let mut remote = repo.find_remote("origin")?;
            remote.fetch(&["main"], None, None)?;

            let fetch_head = repo.find_reference("FETCH_HEAD")?;
            let commit = repo.reference_to_annotated_commit(&fetch_head)?;
            let (analysis, _) = repo.merge_analysis(&[&commit])?;

            if analysis.is_fast_forward() || analysis.is_normal() {
                let refname = "refs/heads/main";
                let mut reference = repo.find_reference(refname)?;
                reference.set_target(commit.id(), "Fast-forward")?;
                repo.set_head(refname)?;
                repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
            }
        } else {
            println!("Cloning registry index...");
            Repository::clone(REGISTRY_URL, &repo_path)?;
        }

        println!("Registry updated.");
        Ok(())
    }

    pub fn load_index(&self) -> Result<RegistryIndex, RegistryError> {
        let index_path = self
            .cache_dir
            .join("fusabi-community")
            .join(REGISTRY_INDEX_PATH);

        if !index_path.exists() {
            self.update()?;
        }

        let contents = std::fs::read_to_string(&index_path)?;
        let index: RegistryIndex = toml::from_str(&contents)?;
        Ok(index)
    }

    pub fn resolve(
        &self,
        name: &str,
        version_constraint: &str,
    ) -> Result<ResolvedPackage, RegistryError> {
        let index = self.load_index()?;

        let matching_packages: Vec<&RegistryPackage> =
            index.packages.iter().filter(|p| p.name == name).collect();

        if matching_packages.is_empty() {
            return Err(RegistryError::PackageNotFound(name.to_string()));
        }

        let mut candidates: Vec<(&RegistryPackage, SemVer)> = matching_packages
            .into_iter()
            .filter_map(|p| {
                let semver = SemVer::parse(&p.version).ok()?;
                if semver.satisfies(version_constraint) {
                    Some((p, semver))
                } else {
                    None
                }
            })
            .collect();

        candidates.sort_by(|a, b| b.1.cmp(&a.1));

        let (package, _) = candidates.into_iter().next().ok_or_else(|| {
            RegistryError::NoMatchingVersion(name.to_string(), version_constraint.to_string())
        })?;

        let rev = Some(format!("v{}", package.version));

        Ok(ResolvedPackage {
            name: package.name.clone(),
            version: package.version.clone(),
            git_url: package.repository.clone(),
            rev,
        })
    }

    pub fn search(&self, query: &str) -> Result<Vec<RegistryPackage>, RegistryError> {
        let index = self.load_index()?;
        let query_lower = query.to_lowercase();

        Ok(index
            .packages
            .into_iter()
            .filter(|p| {
                p.name.to_lowercase().contains(&query_lower)
                    || p.description
                        .as_ref()
                        .is_some_and(|d| d.to_lowercase().contains(&query_lower))
            })
            .collect())
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semver_parse() {
        let v = SemVer::parse("1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);

        let v = SemVer::parse("v2.0").unwrap();
        assert_eq!(v.major, 2);
        assert_eq!(v.minor, 0);
        assert_eq!(v.patch, 0);

        let v = SemVer::parse("1").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 0);
        assert_eq!(v.patch, 0);
    }

    #[test]
    fn test_semver_satisfies_star() {
        let v = SemVer::parse("1.2.3").unwrap();
        assert!(v.satisfies("*"));
        assert!(v.satisfies(""));
    }

    #[test]
    fn test_semver_satisfies_caret() {
        let v100 = SemVer::parse("1.0.0").unwrap();
        let v110 = SemVer::parse("1.1.0").unwrap();
        let v200 = SemVer::parse("2.0.0").unwrap();

        assert!(v100.satisfies("^1.0.0"));
        assert!(v110.satisfies("^1.0.0"));
        assert!(!v200.satisfies("^1.0.0"));
        assert!(!v100.satisfies("^1.1.0"));
    }

    #[test]
    fn test_semver_satisfies_tilde() {
        let v100 = SemVer::parse("1.0.0").unwrap();
        let v101 = SemVer::parse("1.0.1").unwrap();
        let v110 = SemVer::parse("1.1.0").unwrap();

        assert!(v100.satisfies("~1.0.0"));
        assert!(v101.satisfies("~1.0.0"));
        assert!(!v110.satisfies("~1.0.0"));
    }

    #[test]
    fn test_semver_satisfies_exact() {
        let v = SemVer::parse("1.2.3").unwrap();
        assert!(v.satisfies("1.2.3"));
        assert!(v.satisfies("=1.2.3"));
        assert!(!v.satisfies("1.2.4"));
    }

    #[test]
    fn test_semver_satisfies_comparison() {
        let v = SemVer::parse("1.5.0").unwrap();

        assert!(v.satisfies(">=1.0.0"));
        assert!(v.satisfies(">1.0.0"));
        assert!(v.satisfies("<=2.0.0"));
        assert!(v.satisfies("<2.0.0"));
        assert!(!v.satisfies(">2.0.0"));
        assert!(!v.satisfies("<1.0.0"));
    }

    #[test]
    fn test_semver_ordering() {
        let v100 = SemVer::parse("1.0.0").unwrap();
        let v101 = SemVer::parse("1.0.1").unwrap();
        let v110 = SemVer::parse("1.1.0").unwrap();
        let v200 = SemVer::parse("2.0.0").unwrap();

        assert!(v100 < v101);
        assert!(v101 < v110);
        assert!(v110 < v200);
    }

    #[test]
    fn test_registry_index_parse() {
        let toml = r#"
[[packages]]
name = "test-pkg"
version = "1.0.0"
repository = "https://github.com/test/test-pkg"
description = "A test package"

[[packages]]
name = "another-pkg"
version = "0.1.0"
repository = "https://github.com/test/another"
"#;

        let index: RegistryIndex = toml::from_str(toml).unwrap();
        assert_eq!(index.packages.len(), 2);
        assert_eq!(index.packages[0].name, "test-pkg");
        assert_eq!(index.packages[1].name, "another-pkg");
    }
}
