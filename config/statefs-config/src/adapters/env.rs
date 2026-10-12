use super::port::{ConfigSourcePort, copy_subtree_with_policy};
use crate::error::ConfigError;
use crate::manager::policy::PushPolicy;
use crate::manager::profile::Profile;
use statefs_core::{MemStore, StateSource};

/// Environment variable provider with prefix and separator stripping.
#[derive(Debug, Clone)]
pub struct EnvSource {
    inner: statefs_adapter_bridge_env::EnvSource,
    prefix_label: String,
}

impl EnvSource {
    /// Creates an environment source with given prefix (e.g. `"APP_"` or `"GOLDSRC_"`).
    pub fn prefixed(prefix: impl Into<String>) -> Self {
        let p = prefix.into();
        Self {
            inner: statefs_adapter_bridge_env::EnvSource::new().with_prefix(&p),
            prefix_label: p,
        }
    }

    /// Sets custom segment separator (e.g. `"__"` for `APP__SERVER__PORT`).
    pub fn with_separator(mut self, sep: impl Into<String>) -> Self {
        self.inner = self.inner.with_separator(sep);
        self
    }
}

impl ConfigSourcePort for EnvSource {
    fn origin(&self) -> String {
        format!("Env(prefix = \"{}\")", self.prefix_label)
    }

    fn apply(
        &self,
        store: &mut MemStore,
        _profile: &Profile,
        mount_prefix: &str,
        policy: PushPolicy,
    ) -> Result<(), ConfigError> {
        let mut temp_store = MemStore::new();
        self.inner.apply_to_store(&mut temp_store)?;
        copy_subtree_with_policy(&temp_store, "/", store, mount_prefix, policy)?;
        Ok(())
    }
}

/// Ergonomic shortcut type alias matching the ports & adapters design.
pub type Env = EnvSource;
