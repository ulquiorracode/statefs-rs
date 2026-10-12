use super::port::ConfigSourcePort;
use crate::error::ConfigError;
use crate::manager::policy::PushPolicy;
use crate::manager::profile::Profile;
use statefs_core::{MemStore, Path, Store, Value};
use std::collections::BTreeMap;

/// Dynamic dictionary source for ad-hoc or CLI parameter injections.
#[derive(Debug, Clone, Default)]
pub struct DictSource {
    label: String,
    entries: BTreeMap<String, Value>,
}

impl DictSource {
    /// Creates an empty ad-hoc dictionary source.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            entries: BTreeMap::new(),
        }
    }

    /// Inserts a key-value pair.
    pub fn insert(mut self, path: impl Into<String>, value: impl Into<Value>) -> Self {
        self.entries.insert(path.into(), value.into());
        self
    }
}

impl ConfigSourcePort for DictSource {
    fn origin(&self) -> String {
        format!("Dict(\"{}\")", self.label)
    }

    fn apply(
        &self,
        store: &mut MemStore,
        _profile: &Profile,
        mount_prefix: &str,
        policy: PushPolicy,
    ) -> Result<(), ConfigError> {
        for (rel_path, val) in &self.entries {
            let full_path = if mount_prefix.is_empty() || mount_prefix == "/" {
                format!("/{}", rel_path.trim_start_matches('/'))
            } else {
                format!(
                    "{}/{}",
                    mount_prefix.trim_end_matches('/'),
                    rel_path.trim_start_matches('/')
                )
            };
            let p = Path::parse(&full_path);

            match policy {
                PushPolicy::Merge => {
                    store.insert(&p, val.clone())?;
                }
                PushPolicy::Join => {
                    if store.get(&p).is_none() {
                        store.insert(&p, val.clone())?;
                    }
                }
            }
        }
        Ok(())
    }
}
