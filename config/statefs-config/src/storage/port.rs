use crate::error::ConfigError;
use statefs_core::{Node, Path, Value};

/// Storage engine read-only abstraction for configuration trees.
pub trait ConfigStoragePort: Send + Sync {
    /// Retrieves a reference to a node by path.
    fn get(&self, path: &Path) -> Option<&Node>;

    /// Checks if a node exists at path.
    fn contains(&self, path: &Path) -> bool {
        self.get(path).is_some()
    }

    /// Iterates over all matching path-node pairs matching a glob pattern.
    fn find_glob<'a>(&'a self, pattern: &str) -> Box<dyn Iterator<Item = (Path, &'a Node)> + 'a>;
}

/// Storage engine mutation capability.
pub trait ConfigStorageMut: ConfigStoragePort {
    /// Inserts a value at path, creating intermediate directory nodes if necessary.
    fn insert(&mut self, path: &Path, value: Value) -> Result<(), ConfigError>;

    /// Inserts or replaces a pre-constructed node at path.
    fn insert_node(&mut self, path: &Path, node: Node) -> Result<(), ConfigError>;

    /// Removes a node at path.
    fn remove(&mut self, path: &Path) -> Result<bool, ConfigError>;
}
