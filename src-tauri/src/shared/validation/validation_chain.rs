use crate::domain::entities::AnimeDetailed;
use crate::domain::value_objects::AnimeProvider;
use crate::shared::errors::AppResult;
use async_trait::async_trait;
use std::sync::Arc;

/// Result of a validation rule check
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn valid() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn invalid(error: String) -> Self {
        Self {
            is_valid: false,
            errors: vec![error],
            warnings: Vec::new(),
        }
    }

    pub fn with_warning(mut self, warning: String) -> Self {
        self.warnings.push(warning);
        self
    }

    pub fn with_error(mut self, error: String) -> Self {
        self.errors.push(error);
        self.is_valid = false;
        self
    }

    /// Merge two validation results
    pub fn merge(mut self, other: ValidationResult) -> Self {
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
        self.is_valid = self.is_valid && other.is_valid;
        self
    }
}

/// Context passed through the validation chain
#[derive(Debug, Clone)]
pub struct ValidationContext {
    pub anime: AnimeDetailed,
    pub provider: AnimeProvider,
    pub input_title: String,
    pub strict_mode: bool,
    pub provider_specific_checks: bool,
}

/// Trait for validation rules in the chain of responsibility pattern
#[async_trait]
pub trait ValidationRule: Send + Sync {
    /// Execute this validation rule
    async fn validate(&self, context: &ValidationContext) -> AppResult<ValidationResult>;

    /// Get the name of this validation rule for logging
    fn rule_name(&self) -> &'static str;

    /// Check if this rule should be skipped based on context
    fn should_skip(&self, _context: &ValidationContext) -> bool {
        // Default: never skip
        false
    }
}

/// Chain of validation rules implementing Chain of Responsibility pattern
#[derive(Clone)]
pub struct ValidationChain {
    rules: Vec<Arc<dyn ValidationRule>>,
    stop_on_first_error: bool,
}

impl ValidationChain {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            stop_on_first_error: false,
        }
    }

    /// Add a validation rule to the chain
    pub fn add_rule(mut self, rule: Arc<dyn ValidationRule>) -> Self {
        self.rules.push(rule);
        self
    }

    /// Set whether to stop validation on first error
    pub fn stop_on_first_error(mut self, stop: bool) -> Self {
        self.stop_on_first_error = stop;
        self
    }

    /// Execute all validation rules in the chain
    pub async fn validate(&self, context: &ValidationContext) -> AppResult<ValidationResult> {
        let mut combined_result = ValidationResult::valid();

        for rule in &self.rules {
            // Skip rule if conditions are met
            if rule.should_skip(context) {
                continue;
            }

            let rule_result = rule.validate(context).await?;

            // Log rule execution for debugging
            if !rule_result.is_valid {
                log::debug!(
                    "Validation rule '{}' failed for '{}'",
                    rule.rule_name(),
                    context.input_title
                );
            }

            combined_result = combined_result.merge(rule_result);

            // Stop on first error if configured
            if self.stop_on_first_error && !combined_result.is_valid {
                break;
            }
        }

        Ok(combined_result)
    }

    /// Get the number of rules in the chain
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }
}

impl Default for ValidationChain {
    fn default() -> Self {
        Self::new()
    }
}
