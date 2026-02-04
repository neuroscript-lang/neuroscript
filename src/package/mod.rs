//! Package management for NeuroScript
//!
//! This module provides infrastructure for working with NeuroScript packages:
//! - Manifest parsing (Axon.toml)
//! - Package initialization
//! - Dependency resolution (future)
//! - Registry interaction (future)

pub mod init;
pub mod manifest;

pub use init::{init_package, InitError, InitOptions};
pub use manifest::{Dependency, Manifest, ManifestError, PackageMetadata};
