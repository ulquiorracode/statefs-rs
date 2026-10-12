use crate::error::ConfigError;
use crate::manager::policy::PushPolicy;
use crate::manager::profile::Profile;
use statefs_core::{MemStore, Path, Store};

/// Inward SPI port for all pluggable configuration sources.
///
/// Adheres strictly to SMA Clean Separation of Policy from Mechanism:
/// The source is purely responsible for ingesting its domain representation
/// into a target StateFS [`MemStore`] according to the given [`PushPolicy`].
pub trait ConfigSourcePort: Send + Sync {
    /// Human-readable origin label for error diagnostics.
    fn origin(&self) -> String;

    /// Ingests configuration nodes into the target store.
    fn apply(
        &self,
        store: &mut MemStore,
        profile: &Profile,
        mount_prefix: &str,
        policy: PushPolicy,
    ) -> Result<(), ConfigError>;

    /// Combinator: mounts this source at a specific sub-tree path (e.g. `"/plugins/vip"`).
    fn at(self, mount_prefix: impl Into<String>) -> MountedSource<Self>
    where
        Self: Sized,
    {
        MountedSource {
            inner: self,
            prefix: mount_prefix.into(),
        }
    }
}

/// An entry in the manager's source stack.
pub struct SourceEntry {
    pub policy: PushPolicy,
    pub source: Box<dyn ConfigSourcePort>,
    pub mount: Option<String>,
}

/// A configuration source decorated with an explicit mount prefix.
pub struct MountedSource<S> {
    inner: S,
    prefix: String,
}

impl<S: ConfigSourcePort> ConfigSourcePort for MountedSource<S> {
    fn origin(&self) -> String {
        format!("{} [mounted at \"{}\"]", self.inner.origin(), self.prefix)
    }

    fn apply(
        &self,
        store: &mut MemStore,
        profile: &Profile,
        mount_prefix: &str,
        policy: PushPolicy,
    ) -> Result<(), ConfigError> {
        let combined = if mount_prefix.is_empty() || mount_prefix == "/" {
            self.prefix.clone()
        } else {
            format!(
                "{}/{}",
                mount_prefix.trim_end_matches('/'),
                self.prefix.trim_start_matches('/')
            )
        };
        self.inner.apply(store, profile, &combined, policy)
    }
}

/// Helper function to copy/mount a subtree from a source store into a target store
/// honoring the specified [`PushPolicy`] (Merge vs Join).
pub(crate) fn copy_subtree_with_policy(
    src: &MemStore,
    src_root: &str,
    dst: &mut MemStore,
    mount_prefix: &str,
    policy: PushPolicy,
) -> Result<(), ConfigError> {
    let src_clean = if src_root == "/" {
        ""
    } else {
        src_root.trim_end_matches('/')
    };
    let glob_query = if src_clean.is_empty() {
        "/**".to_string()
    } else {
        format!("{src_clean}/**")
    };

    for (p, node) in src.find_glob(&glob_query) {
        let path_str = p.to_string();
        let rel_suffix = if src_clean.is_empty() {
            path_str.as_str()
        } else {
            &path_str[src_clean.len()..]
        };

        let target_path_str = if mount_prefix.is_empty() || mount_prefix == "/" {
            format!("/{}", rel_suffix.trim_start_matches('/'))
        } else {
            format!(
                "{}/{}",
                mount_prefix.trim_end_matches('/'),
                rel_suffix.trim_start_matches('/')
            )
        };

        let target_path = Path::parse(&target_path_str);

        match policy {
            PushPolicy::Merge => {
                // Merge: incoming node overwrites existing node
                dst.insert_node(&target_path, node.clone())?;
            }
            PushPolicy::Join => {
                // Join: only insert if node does NOT already exist in destination
                if dst.get(&target_path).is_none() {
                    dst.insert_node(&target_path, node.clone())?;
                }
            }
        }
    }

    Ok(())
}
