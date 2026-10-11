use super::port::{ConfigSourcePort, copy_subtree_with_policy};
use crate::error::ConfigError;
use crate::manager::policy::PushPolicy;
use crate::manager::profile::Profile;
use statefs_core::MemStore;
use std::path::Path as StdPath;

/// JSON file or string configuration source adapter.
#[derive(Debug, Clone)]
pub struct JsonSource {
    origin_name: String,
    content: String,
}

impl JsonSource {
    /// Loads from raw JSON string content.
    pub fn string(content: impl Into<String>) -> Self {
        Self {
            origin_name: "<string.json>".to_string(),
            content: content.into(),
        }
    }

    /// Loads from disk file.
    pub fn file(path: impl AsRef<StdPath>) -> Result<Self, ConfigError> {
        let path_ref = path.as_ref();
        let content = std::fs::read_to_string(path_ref).map_err(|e| ConfigError::Io {
            path: path_ref.to_path_buf(),
            source: e,
        })?;
        Ok(Self {
            origin_name: path_ref.display().to_string(),
            content,
        })
    }
}

impl ConfigSourcePort for JsonSource {
    fn origin(&self) -> String {
        self.origin_name.clone()
    }

    fn apply(
        &self,
        store: &mut MemStore,
        profile: &Profile,
        mount_prefix: &str,
        policy: PushPolicy,
    ) -> Result<(), ConfigError> {
        let mut temp_store = MemStore::new();
        statefs_codec_json::ingest_json(&mut temp_store, &self.content).map_err(|e| {
            ConfigError::Parse {
                origin: self.origin_name.clone(),
                message: e.to_string(),
            }
        })?;

        let default_profile_prefix = format!("/{}", Profile::DEFAULT);
        let target_profile_prefix = format!("/{}", profile.as_str());

        let has_profiles = temp_store
            .find_arena_index(&default_profile_prefix)
            .map(|idx| idx != 0)
            .unwrap_or(false)
            || temp_store
                .find_arena_index(&target_profile_prefix)
                .map(|idx| idx != 0)
                .unwrap_or(false);

        if has_profiles {
            copy_subtree_with_policy(
                &temp_store,
                &default_profile_prefix,
                store,
                mount_prefix,
                policy,
            )?;
            if profile.as_str() != Profile::DEFAULT {
                copy_subtree_with_policy(
                    &temp_store,
                    &target_profile_prefix,
                    store,
                    mount_prefix,
                    policy,
                )?;
            }
        } else {
            copy_subtree_with_policy(&temp_store, "/", store, mount_prefix, policy)?;
        }

        Ok(())
    }
}

/// Ergonomic shortcut type alias matching the ports & adapters design.
pub type Json = JsonSource;
