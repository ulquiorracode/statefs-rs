use super::value::{FromConfigValue, IntoConfigValue};
use crate::error::ConfigError;
use std::path::Path as StdPath;

/// Primary trait for accessing configuration values and checking existence.
pub trait ConfigAccess {
    /// Generic getter extracting a value at `path`.
    ///
    /// Supports automatic type inference or explicit turbofish:
    /// ```ignore
    /// let port: u16 = config.get("server/port")?;
    /// let host = config.get::<&str>("server/host")?;
    /// ```
    fn get<'a, T: FromConfigValue<'a>>(&'a self, path: &str) -> Result<T, ConfigError>;

    /// Checks if a key or path exists in the configuration.
    fn contains(&self, path: &str) -> bool;

    /// Attempts to extract a typed Serde structure from a path prefix.
    #[cfg(feature = "serde")]
    fn extract<T: for<'de> serde::Deserialize<'de>>(&self, path: &str) -> Result<T, ConfigError>;
}

/// Trait for mutable configuration updates.
pub trait ConfigMut: ConfigAccess {
    /// Inserts or replaces a configuration value at `path`.
    fn set<T: IntoConfigValue>(&mut self, path: &str, val: T) -> Result<(), ConfigError>;

    /// Removes a configuration value or branch at `path`.
    fn remove(&mut self, path: &str) -> Result<bool, ConfigError>;
}

/// Trait for persisting configuration changes to external storage.
pub trait ConfigPersist {
    /// Saves the current configuration state to a disk file.
    fn save_to(&self, path: impl AsRef<StdPath>) -> Result<(), ConfigError>;

    /// Reloads configuration sources from disk.
    fn reload(&mut self) -> Result<(), ConfigError>;
}
