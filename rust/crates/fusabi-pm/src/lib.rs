//! Fusabi Package Manager (fpm)
//!
//! A package manager for the Fusabi language.

pub mod manifest;

pub use manifest::{Dependencies, Dependency, Manifest, Package};
