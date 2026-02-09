//! Package management for NeuroScript
//!
//! This module provides infrastructure for working with NeuroScript packages:
//! - Manifest parsing (Axon.toml)
//! - Package initialization
//! - Lockfile management (Axon.lock)
//! - Dependency resolution
//! - Registry interaction and caching

pub mod init;
pub mod lockfile;
pub mod manifest;
pub mod registry;
pub mod resolver;
pub mod security;

pub use init::{init_package, InitError, InitOptions};
pub use lockfile::{Lockfile, LockfileError, LockedPackage, PackageSource};
pub use manifest::{Dependency, DependencyDetail, Manifest, ManifestError, PackageMetadata};
pub use registry::{Registry, RegistryError};
pub use resolver::{AvailablePackage, Resolver, ResolverError};
pub use security::{SecurityError, VerificationReport};
