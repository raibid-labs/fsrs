//! Fusabi Package Manager (fpm)
//!
//! A package manager for the Fusabi language.

pub mod build;
pub mod install;
pub mod manifest;
pub mod publish;
pub mod registry;

pub use build::{BuildError, BuildResult, PackageBuilder, ResolvedDependency};
pub use install::{install_dependencies, InstallError};
pub use manifest::{Dependencies, Dependency, Manifest, Package};
pub use publish::{print_publish_instructions, publish_package, PublishError, PublishResult};
pub use registry::{Registry, RegistryError, RegistryPackage};
