use statefs_core::StoreError;
use std::fmt;
use std::path::PathBuf;

/// Rich contextual errors produced during configuration building and extraction.
#[derive(Debug)]
pub enum ConfigError {
    /// Key was not found in the layered store.
    NotFound { path: String, profile: String },
    /// Value at path did not match expected scalar or structure type.
    TypeMismatch {
        path: String,
        expected: &'static str,
        actual: String,
    },
    /// An I/O error occurred while reading a file source.
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    /// Codec parse error (TOML, JSON, etc.).
    Parse { origin: String, message: String },
    /// Serde extraction error.
    Serde(String),
    /// Underlying StateFS store failure.
    Store(StoreError),
    /// Contract validation error.
    Validation(crate::validation::ValidationReport),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound { path, profile } => {
                write!(
                    f,
                    "configuration key '{path}' not found (active profile: '{profile}')"
                )
            }
            Self::TypeMismatch {
                path,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "type mismatch at '{path}': expected {expected}, found {actual}"
                )
            }
            Self::Io { path, source } => {
                write!(
                    f,
                    "failed to read configuration file at '{}': {source}",
                    path.display()
                )
            }
            Self::Parse { origin, message } => {
                write!(f, "failed to parse configuration from {origin}: {message}")
            }
            Self::Serde(msg) => write!(f, "deserialization error: {msg}"),
            Self::Store(e) => write!(f, "storage engine error: {e}"),
            Self::Validation(report) => write!(f, "{report}"),
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl From<StoreError> for ConfigError {
    fn from(e: StoreError) -> Self {
        Self::Store(e)
    }
}

#[cfg(feature = "serde")]
impl From<statefs_adapter_bridge_serde::DeError> for ConfigError {
    fn from(e: statefs_adapter_bridge_serde::DeError) -> Self {
        Self::Serde(e.to_string())
    }
}
