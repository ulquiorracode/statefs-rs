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

// -------------------------------------------------------------------------------------------------
// Legacy config-rs Compatibility Bridge
// -------------------------------------------------------------------------------------------------

use statefs_core::{MemStore, Path, Store};

/// High-performance StateFS wrapper matching the API ergonomics of legacy `config::Config`.
pub struct ConfigBridge<const _CAP: usize = 64> {
    store: MemStore,
}

impl ConfigBridge<64> {
    pub fn new(store: MemStore) -> Self {
        Self { store }
    }
}

impl<const CAP: usize> ConfigBridge<CAP> {
    pub fn with_capacity(store: MemStore) -> Self {
        Self { store }
    }

    #[inline(always)]
    pub fn get_int(&mut self, key: &str) -> Result<i64, &'static str> {
        let p = Path::parse(key);
        let node = self.store.get(&p).ok_or("Key not found")?;
        node.value.as_int().ok_or("Value is not an integer")
    }

    #[inline(always)]
    pub fn get_string(&mut self, key: &str) -> Result<String, &'static str> {
        let p = Path::parse(key);
        let node = self.store.get(&p).ok_or("Key not found")?;
        node.value
            .as_str()
            .map(|s| s.to_string())
            .ok_or("Value is not a string")
    }

    #[inline(always)]
    pub fn get_bool(&mut self, key: &str) -> Result<bool, &'static str> {
        let p = Path::parse(key);
        let node = self.store.get(&p).ok_or("Key not found")?;
        node.value.as_bool().ok_or("Value is not a boolean")
    }
}
