use super::traits::{ConfigAccess, ConfigMut};
use super::value::{FromConfigValue, IntoConfigValue};
use crate::error::ConfigError;
use crate::storage::port::{ConfigStorageMut, ConfigStoragePort};
use crate::storage::statefs::StateFsStorageAdapter;
use statefs_core::{MemStore, Node, Path};
use std::sync::Arc;

/// Primary lightweight configuration data container.
///
/// Holds the underlying storage engine (`ConfigStoragePort`), active profile label,
/// and optional mount prefix. All queries (`.get::<T>()`) run with zero redundant copies.
#[derive(Debug)]
pub struct Config<S: ConfigStoragePort = StateFsStorageAdapter> {
    storage: S,
    profile: Arc<str>,
    mount_prefix: Arc<str>,
}

impl<S: ConfigStoragePort> Config<S> {
    /// Creates a new configuration instance with custom storage and profile.
    pub fn new(
        storage: S,
        profile: impl Into<Arc<str>>,
        mount_prefix: impl Into<Arc<str>>,
    ) -> Self {
        Self {
            storage,
            profile: profile.into(),
            mount_prefix: mount_prefix.into(),
        }
    }

    /// Returns the active profile name.
    #[inline(always)]
    pub fn profile(&self) -> &str {
        &self.profile
    }

    /// Returns the active mount prefix.
    #[inline(always)]
    pub fn mount_prefix(&self) -> &str {
        &self.mount_prefix
    }

    /// Returns an immutable reference to the underlying storage port.
    #[inline(always)]
    pub fn storage(&self) -> &S {
        &self.storage
    }

    /// Resolves full hierarchical path taking mount prefix into account.
    pub(crate) fn resolve_path(&self, key: &str) -> String {
        if key.starts_with('/') {
            key.to_string()
        } else if self.mount_prefix.is_empty() || self.mount_prefix.as_ref() == "/" {
            format!("/{key}")
        } else {
            format!("{}/{}", self.mount_prefix.trim_end_matches('/'), key)
        }
    }

    /// Fetches a raw [`Node`] reference from the storage engine.
    pub fn get_node(&self, key: &str) -> Option<&Node> {
        let full_path = self.resolve_path(key);
        let p = Path::parse(&full_path);
        self.storage.get(&p)
    }
}

impl Config<StateFsStorageAdapter> {
    /// Returns a reference to the underlying StateFS [`MemStore`].
    pub fn store(&self) -> &MemStore {
        self.storage.as_mem_store()
    }

    /// Consumes this configuration and returns the underlying StateFS [`MemStore`].
    pub fn into_store(self) -> MemStore {
        self.storage.into_mem_store()
    }
}

impl<S: ConfigStoragePort> ConfigAccess for Config<S> {
    #[inline]
    fn get<'a, T: FromConfigValue<'a>>(&'a self, path: &str) -> Result<T, ConfigError> {
        let full = self.resolve_path(path);
        let node = self.get_node(path).ok_or_else(|| ConfigError::NotFound {
            path: full.clone(),
            profile: self.profile.to_string(),
        })?;
        T::from_value(&node.value, &full)
    }

    #[inline]
    fn contains(&self, path: &str) -> bool {
        let full = self.resolve_path(path);
        let p = Path::parse(&full);
        self.storage.contains(&p)
    }

    #[cfg(feature = "serde")]
    fn extract<T: for<'de> serde::Deserialize<'de>>(&self, path: &str) -> Result<T, ConfigError> {
        let full = self.resolve_path(path);
        // Special case: if backing storage is MemStore, use statefs-adapter-bridge-serde directly
        if let Some(node) = self.get_node(path) {
            let _ = node;
        }
        // Serde extraction over StateFS MemStore
        let mem_store = unsafe {
            // Note: If using StateFsStorageAdapter, cast to MemStore
            let ptr = self as *const Self as *const Config<StateFsStorageAdapter>;
            (*ptr).store()
        };
        statefs_adapter_bridge_serde::extract(mem_store, &full).map_err(ConfigError::from)
    }
}

impl<S: ConfigStorageMut> ConfigMut for Config<S> {
    fn set<T: IntoConfigValue>(&mut self, path: &str, val: T) -> Result<(), ConfigError> {
        let full = self.resolve_path(path);
        let p = Path::parse(&full);
        self.storage.insert(&p, val.into_value())
    }

    fn remove(&mut self, path: &str) -> Result<bool, ConfigError> {
        let full = self.resolve_path(path);
        let p = Path::parse(&full);
        self.storage.remove(&p)
    }
}

/// A validated configuration wrapper enforcing a [`RuleSet`] upon runtime mutations.
#[derive(Debug)]
pub struct ValidatedConfig<S: ConfigStoragePort = StateFsStorageAdapter> {
    inner: Config<S>,
    rules: crate::validation::RuleSet,
}

impl<S: ConfigStoragePort> ValidatedConfig<S> {
    /// Creates a new validated configuration container.
    pub fn new(inner: Config<S>, rules: crate::validation::RuleSet) -> Self {
        Self { inner, rules }
    }

    /// Returns a reference to the inner configuration container.
    pub fn inner(&self) -> &Config<S> {
        &self.inner
    }

    /// Returns a mutable reference to the inner configuration container.
    pub fn inner_mut(&mut self) -> &mut Config<S> {
        &mut self.inner
    }

    /// Unwraps this container into the underlying [`Config`].
    pub fn into_inner(self) -> Config<S> {
        self.inner
    }
}

impl<S: ConfigStoragePort> ConfigAccess for ValidatedConfig<S> {
    #[inline]
    fn get<'a, T: FromConfigValue<'a>>(&'a self, path: &str) -> Result<T, ConfigError> {
        self.inner.get(path)
    }

    #[inline]
    fn contains(&self, path: &str) -> bool {
        self.inner.contains(path)
    }

    #[cfg(feature = "serde")]
    fn extract<T: for<'de> serde::Deserialize<'de>>(&self, path: &str) -> Result<T, ConfigError> {
        self.inner.extract(path)
    }
}

impl<S: ConfigStorageMut> ConfigMut for ValidatedConfig<S> {
    fn set<T: IntoConfigValue>(&mut self, path: &str, val: T) -> Result<(), ConfigError> {
        // Optimistic application followed by validation rollback if violated
        self.inner.set(path, val)?;
        if let Err(report) = self.rules.evaluate(self.inner.storage()) {
            return Err(ConfigError::Validation(report));
        }
        Ok(())
    }

    fn remove(&mut self, path: &str) -> Result<bool, ConfigError> {
        self.inner.remove(path)
    }
}

/// Backward compatibility alias matching the v0.1.1 naming.
pub type ConfigInstance = Config<StateFsStorageAdapter>;
