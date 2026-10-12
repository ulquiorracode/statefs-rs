use std::fmt;
use std::sync::Arc;

use super::rule::{ConfigRule, ValidationError};
use crate::storage::port::ConfigStoragePort;

/// Aggregated validation report containing all detected violations.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ValidationReport {
    /// List of validation failures.
    pub errors: Vec<ValidationError>,
}

impl ValidationReport {
    /// Returns true if no errors were found.
    #[inline]
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }

    /// Appends a validation error.
    pub fn add(&mut self, error: ValidationError) {
        self.errors.push(error);
    }
}

impl fmt::Display for ValidationReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "configuration validation failed with {} error(s):",
            self.errors.len()
        )?;
        for (idx, err) in self.errors.iter().enumerate() {
            writeln!(f, "  {}. [{}] {}", idx + 1, err.path, err.message)?;
        }
        Ok(())
    }
}

impl std::error::Error for ValidationReport {}

/// Pluggable set of validation rules executed in an SMA U-cycle barrier.
#[derive(Default, Clone)]
pub struct RuleSet {
    rules: Vec<Arc<dyn ConfigRule>>,
}

impl fmt::Debug for RuleSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RuleSet")
            .field("rule_count", &self.rules.len())
            .finish()
    }
}

impl RuleSet {
    /// Creates an empty rule set.
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Adds a rule to this validation set.
    pub fn add<R: ConfigRule + 'static>(&mut self, rule: R) -> &mut Self {
        self.rules.push(Arc::new(rule));
        self
    }

    /// Evaluates all rules against the target storage, collecting all errors.
    pub fn evaluate(&self, storage: &dyn ConfigStoragePort) -> Result<(), ValidationReport> {
        let mut report = ValidationReport::default();

        for rule in &self.rules {
            if let Err(err) = rule.validate(storage) {
                report.add(err);
            }
        }

        if report.is_ok() { Ok(()) } else { Err(report) }
    }
}
