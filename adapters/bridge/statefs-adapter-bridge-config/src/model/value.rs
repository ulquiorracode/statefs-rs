use crate::error::ConfigError;
use statefs_core::Value;

/// Trait for types that can be extracted from a configuration [`Value`].
///
/// Provides zero-copy conversion where possible (e.g. `&str`),
/// primitive scalar mapping, and optional generic Serde deserialization.
pub trait FromConfigValue<'a>: Sized {
    /// Converts a reference to [`Value`] into the target type.
    fn from_value(val: &'a Value, path: &str) -> Result<Self, ConfigError>;
}

// -----------------------------------------------------------------------------
// Primitive Scalar Implementations (Zero Heap Allocation)
// -----------------------------------------------------------------------------

impl<'a> FromConfigValue<'a> for bool {
    fn from_value(val: &'a Value, path: &str) -> Result<Self, ConfigError> {
        val.as_bool().ok_or_else(|| ConfigError::TypeMismatch {
            path: path.to_string(),
            expected: "bool",
            actual: format!("{val:?}"),
        })
    }
}

impl<'a> FromConfigValue<'a> for i64 {
    fn from_value(val: &'a Value, path: &str) -> Result<Self, ConfigError> {
        val.as_int().ok_or_else(|| ConfigError::TypeMismatch {
            path: path.to_string(),
            expected: "integer (i64)",
            actual: format!("{val:?}"),
        })
    }
}

impl<'a> FromConfigValue<'a> for i32 {
    fn from_value(val: &'a Value, path: &str) -> Result<Self, ConfigError> {
        let v = i64::from_value(val, path)?;
        i32::try_from(v).map_err(|_| ConfigError::TypeMismatch {
            path: path.to_string(),
            expected: "i32",
            actual: format!("{v} (out of i32 range)"),
        })
    }
}

impl<'a> FromConfigValue<'a> for i16 {
    fn from_value(val: &'a Value, path: &str) -> Result<Self, ConfigError> {
        let v = i64::from_value(val, path)?;
        i16::try_from(v).map_err(|_| ConfigError::TypeMismatch {
            path: path.to_string(),
            expected: "i16",
            actual: format!("{v} (out of i16 range)"),
        })
    }
}

impl<'a> FromConfigValue<'a> for i8 {
    fn from_value(val: &'a Value, path: &str) -> Result<Self, ConfigError> {
        let v = i64::from_value(val, path)?;
        i8::try_from(v).map_err(|_| ConfigError::TypeMismatch {
            path: path.to_string(),
            expected: "i8",
            actual: format!("{v} (out of i8 range)"),
        })
    }
}

impl<'a> FromConfigValue<'a> for u64 {
    fn from_value(val: &'a Value, path: &str) -> Result<Self, ConfigError> {
        let v = i64::from_value(val, path)?;
        u64::try_from(v).map_err(|_| ConfigError::TypeMismatch {
            path: path.to_string(),
            expected: "u64",
            actual: format!("{v} (negative value)"),
        })
    }
}

impl<'a> FromConfigValue<'a> for u32 {
    fn from_value(val: &'a Value, path: &str) -> Result<Self, ConfigError> {
        let v = i64::from_value(val, path)?;
        u32::try_from(v).map_err(|_| ConfigError::TypeMismatch {
            path: path.to_string(),
            expected: "u32",
            actual: format!("{v} (out of u32 range)"),
        })
    }
}

impl<'a> FromConfigValue<'a> for u16 {
    fn from_value(val: &'a Value, path: &str) -> Result<Self, ConfigError> {
        let v = i64::from_value(val, path)?;
        u16::try_from(v).map_err(|_| ConfigError::TypeMismatch {
            path: path.to_string(),
            expected: "u16",
            actual: format!("{v} (out of u16 range)"),
        })
    }
}

impl<'a> FromConfigValue<'a> for u8 {
    fn from_value(val: &'a Value, path: &str) -> Result<Self, ConfigError> {
        let v = i64::from_value(val, path)?;
        u8::try_from(v).map_err(|_| ConfigError::TypeMismatch {
            path: path.to_string(),
            expected: "u8",
            actual: format!("{v} (out of u8 range)"),
        })
    }
}

impl<'a> FromConfigValue<'a> for usize {
    fn from_value(val: &'a Value, path: &str) -> Result<Self, ConfigError> {
        let v = i64::from_value(val, path)?;
        usize::try_from(v).map_err(|_| ConfigError::TypeMismatch {
            path: path.to_string(),
            expected: "usize",
            actual: format!("{v} (out of usize range)"),
        })
    }
}

impl<'a> FromConfigValue<'a> for f64 {
    fn from_value(val: &'a Value, path: &str) -> Result<Self, ConfigError> {
        val.as_float().ok_or_else(|| ConfigError::TypeMismatch {
            path: path.to_string(),
            expected: "float (f64)",
            actual: format!("{val:?}"),
        })
    }
}

impl<'a> FromConfigValue<'a> for f32 {
    fn from_value(val: &'a Value, path: &str) -> Result<Self, ConfigError> {
        f64::from_value(val, path).map(|v| v as f32)
    }
}

impl<'a> FromConfigValue<'a> for &'a str {
    fn from_value(val: &'a Value, path: &str) -> Result<Self, ConfigError> {
        val.as_str().ok_or_else(|| ConfigError::TypeMismatch {
            path: path.to_string(),
            expected: "&str (string slice)",
            actual: format!("{val:?}"),
        })
    }
}

impl<'a> FromConfigValue<'a> for String {
    fn from_value(val: &'a Value, path: &str) -> Result<Self, ConfigError> {
        <&'a str>::from_value(val, path).map(|s| s.to_string())
    }
}

impl<'a, T: FromConfigValue<'a>> FromConfigValue<'a> for Option<T> {
    fn from_value(val: &'a Value, path: &str) -> Result<Self, ConfigError> {
        T::from_value(val, path).map(Some)
    }
}

/// Trait for types that can be converted into a configuration [`Value`].
pub trait IntoConfigValue {
    fn into_value(self) -> Value;
}

impl IntoConfigValue for Value {
    fn into_value(self) -> Value {
        self
    }
}

impl IntoConfigValue for bool {
    fn into_value(self) -> Value {
        Value::Bool(self)
    }
}

impl IntoConfigValue for i64 {
    fn into_value(self) -> Value {
        Value::Int(self)
    }
}

impl IntoConfigValue for i32 {
    fn into_value(self) -> Value {
        Value::Int(self as i64)
    }
}

impl IntoConfigValue for u32 {
    fn into_value(self) -> Value {
        Value::Int(self as i64)
    }
}

impl IntoConfigValue for u16 {
    fn into_value(self) -> Value {
        Value::Int(self as i64)
    }
}

impl IntoConfigValue for f64 {
    fn into_value(self) -> Value {
        Value::Float(self)
    }
}

impl IntoConfigValue for f32 {
    fn into_value(self) -> Value {
        Value::Float(self as f64)
    }
}

impl IntoConfigValue for String {
    fn into_value(self) -> Value {
        Value::from(self)
    }
}

impl IntoConfigValue for &str {
    fn into_value(self) -> Value {
        Value::from(self)
    }
}
