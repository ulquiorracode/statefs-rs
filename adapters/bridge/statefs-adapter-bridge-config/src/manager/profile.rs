use std::fmt;

/// High-level profile selector for layered configurations (e.g. `"default"`, `"production"`, `"test"`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Profile(pub String);

impl Profile {
    /// The canonical default fallback profile.
    pub const DEFAULT: &'static str = "default";

    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for Profile {
    fn default() -> Self {
        Self(Self::DEFAULT.to_string())
    }
}

impl fmt::Display for Profile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<S: Into<String>> From<S> for Profile {
    fn from(s: S) -> Self {
        Self::new(s)
    }
}
