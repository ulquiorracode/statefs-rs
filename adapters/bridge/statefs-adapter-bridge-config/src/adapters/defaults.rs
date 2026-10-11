use super::port::ConfigSourcePort;
use crate::error::ConfigError;
use crate::manager::policy::PushPolicy;
use crate::manager::profile::Profile;
use statefs_core::{MemStore, Path, Store, Value};
use std::collections::BTreeMap;

/// In-memory default values provider.
#[derive(Debug, Clone, Default)]
pub struct Defaults {
    entries: BTreeMap<String, Value>,
}

impl Defaults {
    /// Creates an empty in-memory defaults container.
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends a key-value pair to the defaults container.
    pub fn set(mut self, path: impl Into<String>, value: impl Into<Value>) -> Self {
        self.entries.insert(path.into(), value.into());
        self
    }
}

impl ConfigSourcePort for Defaults {
    fn origin(&self) -> String {
        "Defaults (in-memory)".to_string()
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
