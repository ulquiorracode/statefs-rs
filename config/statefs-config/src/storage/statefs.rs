use super::port::{ConfigStorageMut, ConfigStoragePort};
use crate::error::ConfigError;
use statefs_core::{MemStore, Node, Path, Store, Value};

/// Flagship storage adapter wrapping StateFS in-memory nanokernel (`MemStore`).
#[derive(Debug, Clone, Default)]
pub struct StateFsStorageAdapter {
    inner: MemStore,
}

impl StateFsStorageAdapter {
    /// Creates a new empty StateFS storage adapter.
    pub fn new() -> Self {
        Self {
            inner: MemStore::new(),
        }
    }

    /// Wraps an existing `MemStore`.
    pub fn from_mem_store(store: MemStore) -> Self {
        Self { inner: store }
    }

    /// Borrows the underlying `MemStore`.
    pub fn as_mem_store(&self) -> &MemStore {
        &self.inner
    }

    /// Mutably borrows the underlying `MemStore`.
    pub fn as_mem_store_mut(&mut self) -> &mut MemStore {
        &mut self.inner
    }

    /// Consumes this adapter and returns the inner `MemStore`.
    pub fn into_mem_store(self) -> MemStore {
        self.inner
    }
}

impl ConfigStoragePort for StateFsStorageAdapter {
    fn get(&self, path: &Path) -> Option<&Node> {
        self.inner.get(path)
    }

    fn find_glob<'a>(&'a self, pattern: &str) -> Box<dyn Iterator<Item = (Path, &'a Node)> + 'a> {
        Box::new(self.inner.find_glob(pattern).into_iter())
    }
}

impl ConfigStorageMut for StateFsStorageAdapter {
    fn insert(&mut self, path: &Path, value: Value) -> Result<(), ConfigError> {
        self.inner.insert(path, value)?;
        Ok(())
    }

    fn insert_node(&mut self, path: &Path, node: Node) -> Result<(), ConfigError> {
        self.inner.insert_node(path, node)?;
        Ok(())
    }

    fn remove(&mut self, path: &Path) -> Result<bool, ConfigError> {
        Ok(self.inner.remove(path)?.is_some())
    }
}
