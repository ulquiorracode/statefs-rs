//! # StateFS Configuration Engine & Bridge (`statefs-config`)
//!
//! High-performance, zero-allocation hierarchical configuration management
//! engine for Rust 2024, built on the principles of Sewing Machine Architecture (SMA).
//!
//! ## Key Pillars
//! - **Ports & Adapters Architecture**:
//!   - Storage engine is decoupled via [`ConfigStoragePort`] ([`StateFsStorageAdapter`] by default).
//!   - Ingestion sources are decoupled via [`ConfigSourcePort`] ([`Toml`], [`Json`], [`Env`], [`Defaults`]).
//! - **Layered Stacking & Policies**:
//!   - Explicit [`PushPolicy`] controlling conflict resolution:
//!     - [`PushPolicy::Merge`]: Prefer incoming / override (Figment `merge`).
//!     - [`PushPolicy::Join`]: Prefer existing / fallback defaults (Figment `join`).
//! - **Capability Composition**:
//!   - [`ConfigAccess`]: Fast typed queries (`.get::<T>()`) with zero-copy string slice borrowing.
//!   - [`ConfigMut`]: In-memory runtime mutation (`.set::<T>()`, `.remove()`).
//!   - [`ConfigPersist`]: Disk persistence (`.save_to()`, `.reload()`).
//! - **Subtree Mounts & Scopes**:
//!   - `.at("/plugins/vip")` adapter combinators and `.scope("/prefix", |sub| ...)` nested builders.

pub mod adapters;
pub mod error;
pub mod manager;
pub mod model;
pub mod storage;
pub mod validation;

// Re-export primary types for top-level ergonomics
pub use error::ConfigError;
pub use manager::{ConfigManager, Profile, PushPolicy, StateFsConfig};
pub use model::{
    Config, ConfigAccess, ConfigInstance, ConfigMut, ConfigPersist, FromConfigValue,
    IntoConfigValue, ValidatedConfig,
};
pub use storage::{ConfigStorageMut, ConfigStoragePort, StateFsStorageAdapter};
pub use validation::{ConfigRule, Rule, RuleSet, ValidationError, ValidationReport};

// Re-export adapters
pub use adapters::{ConfigSourcePort, Defaults, DictSource};
#[cfg(feature = "env")]
pub use adapters::{Env, EnvSource};
#[cfg(feature = "json")]
pub use adapters::{Json, JsonSource};
#[cfg(feature = "toml")]
pub use adapters::{Toml, TomlSource};
