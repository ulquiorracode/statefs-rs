use statefs_core::{Path, Value};
use std::fmt;

use crate::storage::port::ConfigStoragePort;

/// Diagnostic details describing an individual rule violation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    /// Path of the violating configuration key.
    pub path: String,
    /// Human-readable explanation of why the value is invalid.
    pub message: String,
}

impl ValidationError {
    /// Creates a new validation error.
    pub fn new(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "configuration validation error at '{}': {}",
            self.path, self.message
        )
    }
}

impl std::error::Error for ValidationError {}

/// Inward SPI trait for any validation rule inspecting a `ConfigStoragePort`.
pub trait ConfigRule: Send + Sync {
    /// Evaluates the rule against the specified configuration storage.
    fn validate(&self, storage: &dyn ConfigStoragePort) -> Result<(), ValidationError>;
}

/// Declarative, zero-cost configuration validation rule.
#[derive(Clone)]
pub enum Rule {
    /// Specifies that a key must exist in the configuration tree.
    Required {
        /// Target key path.
        path: String,
    },
    /// Numerical boundary constraint (inclusive range `[min, max]`).
    Range {
        /// Target key path.
        path: String,
        /// Minimum permissible value.
        min: f64,
        /// Maximum permissible value.
        max: f64,
    },
    /// Discrete allowed choices for string or scalar values.
    Choices {
        /// Target key path.
        path: String,
        /// List of allowed string representations.
        allowed: Vec<String>,
    },
    /// Custom predicate closure or function pointer.
    Custom {
        /// Target key path.
        path: String,
        /// Arbitrary validation logic over the retrieved `Value`.
        validator: fn(Option<&Value>) -> Result<(), String>,
    },
}

impl Rule {
    /// Creates a `Required` rule.
    pub fn required(path: impl Into<String>) -> Self {
        Self::Required { path: path.into() }
    }

    /// Creates a numerical `Range` rule.
    pub fn range(path: impl Into<String>, min: f64, max: f64) -> Self {
        Self::Range {
            path: path.into(),
            min,
            max,
        }
    }

    /// Creates a discrete `Choices` rule.
    pub fn choices<I, S>(path: impl Into<String>, choices: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self::Choices {
            path: path.into(),
            allowed: choices.into_iter().map(Into::into).collect(),
        }
    }

    /// Creates a `Custom` validation rule.
    pub fn custom(
        path: impl Into<String>,
        validator: fn(Option<&Value>) -> Result<(), String>,
    ) -> Self {
        Self::Custom {
            path: path.into(),
            validator,
        }
    }
}

impl ConfigRule for Rule {
    fn validate(&self, storage: &dyn ConfigStoragePort) -> Result<(), ValidationError> {
        let normalize_path = |raw: &str| -> Path {
            let normalized = if raw.starts_with('/') {
                raw.to_string()
            } else {
                format!("/{raw}")
            };
            Path::parse(&normalized)
        };

        match self {
            Self::Required { path } => {
                let p = normalize_path(path);
                if !storage.contains(&p) {
                    return Err(ValidationError::new(
                        path,
                        "required configuration key is missing",
                    ));
                }
                Ok(())
            }
            Self::Range { path, min, max } => {
                let p = normalize_path(path);
                let Some(node) = storage.get(&p) else {
                    return Ok(()); // Absent keys are handled by Required rule if mandatory
                };

                let num: Option<f64> = node.value.as_float();

                match num {
                    Some(n) if n < *min || n > *max => Err(ValidationError::new(
                        path,
                        format!("value {n} is outside allowed range [{min}, {max}]"),
                    )),
                    Some(_) => Ok(()),
                    None => Err(ValidationError::new(
                        path,
                        format!(
                            "expected numeric value for range check, found '{:?}'",
                            node.value
                        ),
                    )),
                }
            }
            Self::Choices { path, allowed } => {
                let p = normalize_path(path);
                let Some(node) = storage.get(&p) else {
                    return Ok(());
                };

                let actual_str = match &node.value {
                    Value::String(s) => s.as_str(),
                    Value::Bool(b) => {
                        if *b {
                            "true"
                        } else {
                            "false"
                        }
                    }
                    Value::Int(i) => {
                        let s = i.to_string();
                        return if allowed.iter().any(|a| a == &s) {
                            Ok(())
                        } else {
                            Err(ValidationError::new(
                                path,
                                format!("value '{i}' is not in allowed choices: {allowed:?}"),
                            ))
                        };
                    }
                    _ => {
                        return Err(ValidationError::new(
                            path,
                            format!(
                                "unsupported value type for choices validation: '{:?}'",
                                node.value
                            ),
                        ));
                    }
                };

                if !allowed.iter().any(|a| a == actual_str) {
                    return Err(ValidationError::new(
                        path,
                        format!("value '{actual_str}' is not in allowed choices: {allowed:?}"),
                    ));
                }
                Ok(())
            }
            Self::Custom { path, validator } => {
                let p = normalize_path(path);
                let val = storage.get(&p).map(|node| &node.value);
                validator(val).map_err(|err| ValidationError::new(path, err))
            }
        }
    }
}
