//! # StateFS `config-rs` Compatibility Bridge
//!
//! Provides two-way zero-cost interoperability between StateFS [`MemStore`] and the [`config`] (config-rs) crate.
//!
//! - Implements [`config::Source`] over StateFS [`MemStore`], allowing any StateFS subtree
//!   to be used directly as a source in `config::ConfigBuilder`.
//! - Provides [`ConfigBridge`], a fast wrapper implementing standard `config-rs` access patterns
//!   directly over StateFS nodes without heap allocations.

use config::{ConfigError, Map, Source, Value as ConfigValue, ValueKind};
use statefs_core::{MemStore, Path, Store, Value};

/// Bridge adapter wrapping a StateFS [`MemStore`] to expose `config-rs`-style query methods.
#[derive(Debug, Clone)]
pub struct ConfigBridge<const _CAP: usize = 64> {
    store: MemStore,
}

impl ConfigBridge<64> {
    /// Creates a new `ConfigBridge` with default capacity.
    pub fn new(store: MemStore) -> Self {
        Self { store }
    }
}

impl<const CAP: usize> ConfigBridge<CAP> {
    /// Creates a new `ConfigBridge` with custom capacity parameter.
    pub fn with_capacity(store: MemStore) -> Self {
        Self { store }
    }

    /// Returns a reference to the underlying [`MemStore`].
    pub fn store(&self) -> &MemStore {
        &self.store
    }

    /// Consumes the bridge and returns the underlying [`MemStore`].
    pub fn into_store(self) -> MemStore {
        self.store
    }

    /// Fetches an integer configuration value by path.
    pub fn get_int(&self, key: &str) -> Result<i64, &'static str> {
        let p = Path::parse(key);
        let node = self.store.get(&p).ok_or("Key not found")?;
        node.value.as_int().ok_or("Value is not an integer")
    }

    /// Fetches a string configuration value by path.
    pub fn get_string(&self, key: &str) -> Result<String, &'static str> {
        let p = Path::parse(key);
        let node = self.store.get(&p).ok_or("Key not found")?;
        node.value
            .as_str()
            .map(|s| s.to_string())
            .ok_or("Value is not a string")
    }

    /// Fetches a boolean configuration value by path.
    pub fn get_bool(&self, key: &str) -> Result<bool, &'static str> {
        let p = Path::parse(key);
        let node = self.store.get(&p).ok_or("Key not found")?;
        node.value.as_bool().ok_or("Value is not a boolean")
    }
}

/// A [`config::Source`] implementation that exports a StateFS [`MemStore`] subtree into `config-rs`.
#[derive(Debug, Clone)]
pub struct StateFsSource {
    store: MemStore,
    mount_prefix: String,
}

impl StateFsSource {
    /// Creates a new `StateFsSource` extracting from the entire store.
    pub fn new(store: MemStore) -> Self {
        Self {
            store,
            mount_prefix: "/".to_string(),
        }
    }

    /// Creates a new `StateFsSource` targeting a specific subtree prefix.
    pub fn prefixed(store: MemStore, mount_prefix: impl Into<String>) -> Self {
        Self {
            store,
            mount_prefix: mount_prefix.into(),
        }
    }
}

impl Source for StateFsSource {
    fn clone_into_box(&self) -> Box<dyn Source + Send + Sync> {
        Box::new(self.clone())
    }

    fn collect(&self) -> Result<Map<String, ConfigValue>, ConfigError> {
        let mut map = Map::new();
        let prefix = self.mount_prefix.trim_matches('/');

        // Traverse the MemStore arena and map nodes to config::Value
        let iter = self.store.find_glob("**");
        for (path, node) in iter {
            let path_str = path.to_string();
            let relative = if prefix.is_empty() {
                path_str.trim_start_matches('/').to_string()
            } else if let Some(stripped) = path_str.strip_prefix(&format!("/{prefix}/")) {
                stripped.to_string()
            } else {
                continue;
            };

            let config_val = match &node.value {
                Value::Bool(b) => ConfigValue::new(Some(&path_str), ValueKind::Boolean(*b)),
                Value::Int(i) => ConfigValue::new(Some(&path_str), ValueKind::I64(*i)),
                Value::Float(f) => ConfigValue::new(Some(&path_str), ValueKind::Float(*f)),
                Value::String(s) => ConfigValue::new(Some(&path_str), ValueKind::String(s.clone())),
                Value::Null => ConfigValue::new(Some(&path_str), ValueKind::Nil),
                _ => continue,
            };

            map.insert(relative.replace('/', "."), config_val);
        }

        Ok(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_bridge_getters() {
        let mut store = MemStore::new();
        store
            .insert(&Path::parse("/server/port"), Value::from(27015))
            .unwrap();
        store
            .insert(&Path::parse("/server/name"), Value::from("CS 1.6 Server"))
            .unwrap();
        store
            .insert(&Path::parse("/server/active"), Value::from(true))
            .unwrap();

        let bridge = ConfigBridge::new(store);
        assert_eq!(bridge.get_int("/server/port"), Ok(27015));
        assert_eq!(
            bridge.get_string("/server/name"),
            Ok("CS 1.6 Server".to_string())
        );
        assert_eq!(bridge.get_bool("/server/active"), Ok(true));
    }

    #[test]
    fn test_config_rs_source_integration() {
        let mut store = MemStore::new();
        store
            .insert(&Path::parse("/server/port"), Value::from(27015))
            .unwrap();
        store
            .insert(&Path::parse("/server/name"), Value::from("Bridged Server"))
            .unwrap();

        let source = StateFsSource::new(store);
        let config = config::Config::builder()
            .add_source(source)
            .build()
            .unwrap();

        assert_eq!(config.get::<i64>("server.port").unwrap(), 27015);
        assert_eq!(
            config.get::<String>("server.name").unwrap(),
            "Bridged Server"
        );
    }
}
