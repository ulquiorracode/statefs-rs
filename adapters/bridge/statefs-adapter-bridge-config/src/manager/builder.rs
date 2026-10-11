use super::policy::PushPolicy;
use super::profile::Profile;
use crate::adapters::port::{ConfigSourcePort, SourceEntry};
use crate::error::ConfigError;
use crate::model::instance::Config;
use crate::storage::statefs::StateFsStorageAdapter;
use statefs_core::MemStore;
use std::sync::Arc;

/// Modern fluent configuration orchestrator and manager.
///
/// Combines multi-source stacking (`.push()`, `.merge()`, `.join()`),
/// sub-tree mounting (`.at()`, `.scope()`), and profile selection.
pub struct ConfigManager {
    profile: Profile,
    mount_prefix: String,
    sources: Vec<SourceEntry>,
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigManager {
    /// Creates a new configuration builder with default settings.
    pub fn new() -> Self {
        Self {
            profile: Profile::default(),
            mount_prefix: "/".to_string(),
            sources: Vec::new(),
        }
    }

    /// Alias for [`Self::new`].
    pub fn builder() -> Self {
        Self::new()
    }

    /// Sets the active profile (e.g. `"default"`, `"staging"`, `"production"`).
    pub fn profile(mut self, profile: impl Into<Profile>) -> Self {
        self.profile = profile.into();
        self
    }

    /// Sets the root mount prefix (e.g. `"/plugins/vip"`).
    pub fn mount(mut self, prefix: impl Into<String>) -> Self {
        self.mount_prefix = prefix.into();
        self
    }

    /// Core primitive: stacks a configuration source with explicit [`PushPolicy`].
    pub fn push<S: ConfigSourcePort + 'static>(mut self, policy: PushPolicy, source: S) -> Self {
        self.sources.push(SourceEntry {
            policy,
            source: Box::new(source),
            mount: None,
        });
        self
    }

    /// Idiomatic shortcut: Merges a source with [`PushPolicy::Merge`] (Prefer Incoming / Override).
    #[inline(always)]
    pub fn merge<S: ConfigSourcePort + 'static>(self, source: S) -> Self {
        self.push(PushPolicy::Merge, source)
    }

    /// Idiomatic shortcut: Joins a source with [`PushPolicy::Join`] (Prefer Existing / Fallback).
    #[inline(always)]
    pub fn join<S: ConfigSourcePort + 'static>(self, source: S) -> Self {
        self.push(PushPolicy::Join, source)
    }

    /// Nested builder: configures a sub-namespace or sub-tree with isolated relative paths.
    pub fn scope<F>(mut self, prefix: impl Into<String>, configure: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        let sub_manager = configure(Self {
            profile: self.profile.clone(),
            mount_prefix: prefix.into(),
            sources: Vec::new(),
        });

        // Absorb child sources into parent stack
        for mut entry in sub_manager.sources {
            if entry.mount.is_none() {
                entry.mount = Some(sub_manager.mount_prefix.clone());
            }
            self.sources.push(entry);
        }
        self
    }

    /// Compiles all stacked sources into an optimized in-memory StateFS [`MemStore`].
    pub fn compile(&self) -> Result<MemStore, ConfigError> {
        let mut store = MemStore::new();
        for entry in &self.sources {
            let active_mount = entry.mount.as_deref().unwrap_or(&self.mount_prefix);

            entry
                .source
                .apply(&mut store, &self.profile, active_mount, entry.policy)?;
        }
        Ok(store)
    }

    /// Builds an immutable [`Config`] container backed by StateFS.
    pub fn build(self) -> Result<Config<StateFsStorageAdapter>, ConfigError> {
        let profile_str: Arc<str> = Arc::from(self.profile.as_str());
        let mount_str: Arc<str> = Arc::from(self.mount_prefix.as_str());
        let store = self.compile()?;
        let storage = StateFsStorageAdapter::from_mem_store(store);
        Ok(Config::new(storage, profile_str, mount_str))
    }
}

/// Backward compatibility alias matching v0.1.1.
pub type StateFsConfig = ConfigManager;
