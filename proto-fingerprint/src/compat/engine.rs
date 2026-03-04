//! Breaking change detection engine
//!
//! This module provides the main engine for detecting breaking changes between
//! two Protocol Buffer files, using the simplified bulk rule registry system.

use crate::canonical::CanonicalFile;
use crate::compat::bulk_rule_registry;
use crate::compat::types::{BreakingChange, RuleContext};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for breaking change detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakingConfig {
    /// Categories to enable (if empty, uses default rules)
    #[serde(default)]
    pub use_categories: Vec<String>,
    /// Specific rules to enable (overrides categories if specified)
    #[serde(default)]
    pub use_rules: Vec<String>,
    /// Rules to explicitly disable
    #[serde(default)]
    pub except_rules: Vec<String>,
    /// Files or directories to ignore
    #[serde(default)]
    pub ignore: Vec<String>,
    /// Rule-specific file ignores
    #[serde(default)]
    pub ignore_only: std::collections::HashMap<String, Vec<String>>,
    /// Whether to ignore unstable packages
    #[serde(default)]
    pub ignore_unstable_packages: bool,
    /// Service name suffixes that cannot be changed
    #[serde(default)]
    pub service_no_change_suffixes: Vec<String>,
    /// Message name suffixes that cannot be changed
    #[serde(default)]
    pub message_no_change_suffixes: Vec<String>,
    /// Enum name suffixes that cannot be changed
    #[serde(default)]
    pub enum_no_change_suffixes: Vec<String>,
}

impl BreakingConfig {
    /// Load configuration from YAML file
    pub fn from_yaml_file<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::from_yaml_str(&content)
    }

    /// Load configuration from YAML string
    pub fn from_yaml_str(yaml: &str) -> anyhow::Result<Self> {
        #[derive(serde::Deserialize)]
        struct ConfigFile {
            breaking: Option<BreakingConfig>,
        }

        let config_file: ConfigFile = serde_yaml::from_str(yaml)?;
        Ok(config_file.breaking.unwrap_or_default())
    }
}

impl Default for BreakingConfig {
    fn default() -> Self {
        Self {
            use_categories: vec!["FILE".to_string(), "PACKAGE".to_string()],
            use_rules: Vec::new(),
            except_rules: Vec::new(),
            ignore: Vec::new(),
            ignore_only: HashMap::new(),
            ignore_unstable_packages: false,
            service_no_change_suffixes: Vec::new(),
            message_no_change_suffixes: Vec::new(),
            enum_no_change_suffixes: Vec::new(),
        }
    }
}

/// Result of breaking change detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakingResult {
    /// All breaking changes found
    pub changes: Vec<BreakingChange>,
    /// Whether any breaking changes were found
    pub has_breaking_changes: bool,
    /// Summary by category
    pub summary: HashMap<String, usize>,
    /// Rules that were executed
    pub executed_rules: Vec<String>,
    /// Rules that failed to execute
    pub failed_rules: Vec<String>,
}

impl BreakingResult {
    /// Create a new empty result
    pub fn new() -> Self {
        Self {
            changes: Vec::new(),
            has_breaking_changes: false,
            summary: HashMap::new(),
            executed_rules: Vec::new(),
            failed_rules: Vec::new(),
        }
    }

    /// Add breaking changes to the result
    pub fn add_changes(&mut self, new_changes: Vec<BreakingChange>) {
        self.has_breaking_changes = !new_changes.is_empty() || self.has_breaking_changes;

        // Update summary BEFORE moving changes
        for change in &new_changes {
            for category in &change.categories {
                *self.summary.entry(category.clone()).or_insert(0) += 1;
            }
        }

        // Now add to changes list
        self.changes.extend(new_changes);
    }

    /// Mark a rule as executed successfully
    pub fn mark_rule_executed(&mut self, rule_id: String) {
        self.executed_rules.push(rule_id);
    }

    /// Mark a rule as failed
    pub fn mark_rule_failed(&mut self, rule_id: String) {
        self.failed_rules.push(rule_id);
    }
}

impl Default for BreakingResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Main engine for breaking change detection
pub struct BreakingEngine {
    // Engine is stateless, uses bulk_rule_registry directly
}

impl BreakingEngine {
    /// Create a new breaking change engine
    pub fn new() -> Self {
        Self {}
    }

    /// Check for breaking changes between two canonical files
    pub fn check(
        &self,
        current: &CanonicalFile,
        previous: &CanonicalFile,
        config: &BreakingConfig,
    ) -> BreakingResult {
        let mut result = BreakingResult::new();

        // Create rule context
        let context = RuleContext {
            current_file: "current".to_string(),
            previous_file: Some("previous".to_string()),
            metadata: HashMap::new(),
        };

        // Get all rules from bulk registry
        let all_rules = bulk_rule_registry::get_bulk_rule_mapping();

        // Execute selected rules based on configuration
        for (rule_id, rule_fn) in all_rules.iter() {
            // Skip rules that are explicitly excluded
            if config.except_rules.contains(&rule_id.to_string()) {
                continue;
            }

            // If specific rules are specified, only run those
            if !config.use_rules.is_empty() && !config.use_rules.contains(&rule_id.to_string()) {
                continue;
            }

            // If using categories, check if rule belongs to enabled categories
            // For now, if use_rules is empty and use_categories is specified, we run based on categories
            // This is a simplified implementation - real Buf logic is more complex
            if config.use_rules.is_empty() && !config.use_categories.is_empty() {
                // Simplified category matching - could be improved based on actual Buf logic
                let rule_categories = get_rule_categories(rule_id);
                let should_run = config
                    .use_categories
                    .iter()
                    .any(|cat| rule_categories.contains(cat));
                if !should_run {
                    continue;
                }
            }

            let rule_result = rule_fn(current, previous, &context);

            if rule_result.success {
                result.mark_rule_executed(rule_id.to_string());
                result.add_changes(rule_result.changes);
            } else {
                result.mark_rule_failed(rule_id.to_string());
            }
        }

        result
    }

    /// Get rule count from bulk registry
    pub fn get_rule_count(&self) -> usize {
        bulk_rule_registry::get_bulk_rule_count()
    }

    /// Verify bulk rules integrity
    pub fn verify_rules(&self) -> Result<(), String> {
        bulk_rule_registry::verify_bulk_rules()
    }
}

/// Get categories for a rule (simplified mapping)
fn get_rule_categories(rule_id: &str) -> Vec<String> {
    match rule_id {
        // FILE category rules
        "FILE_SAME_PACKAGE"
        | "FILE_NO_DELETE"
        | "FILE_SAME_SYNTAX"
        | "FILE_SAME_GO_PACKAGE"
        | "FILE_SAME_JAVA_PACKAGE"
        | "FILE_SAME_CSHARP_NAMESPACE"
        | "FILE_SAME_RUBY_PACKAGE"
        | "FILE_SAME_JAVA_MULTIPLE_FILES"
        | "FILE_SAME_JAVA_OUTER_CLASSNAME"
        | "FILE_SAME_OBJC_CLASS_PREFIX"
        | "FILE_SAME_PHP_CLASS_PREFIX"
        | "FILE_SAME_PHP_NAMESPACE"
        | "FILE_SAME_PHP_METADATA_NAMESPACE"
        | "FILE_SAME_SWIFT_PREFIX"
        | "FILE_SAME_OPTIMIZE_FOR"
        | "FILE_SAME_CC_GENERIC_SERVICES" => vec!["FILE".to_string()],

        // MESSAGE/FIELD rules in FILE category
        "MESSAGE_NO_DELETE"
        | "FIELD_NO_DELETE"
        | "FIELD_SAME_NAME"
        | "FIELD_SAME_TYPE"
        | "ONEOF_NO_DELETE"
        | "MESSAGE_NO_REMOVE_STANDARD_DESCRIPTOR_ACCESSOR"
        | "MESSAGE_SAME_MESSAGE_SET_WIRE_FORMAT" => vec!["FILE".to_string()],

        // ENUM rules in FILE category
        "ENUM_NO_DELETE"
        | "ENUM_VALUE_NO_DELETE"
        | "ENUM_FIRST_VALUE_SAME"
        | "ENUM_VALUE_SAME_NUMBER"
        | "ENUM_ZERO_VALUE_SAME"
        | "ENUM_ALLOW_ALIAS_SAME" => vec!["FILE".to_string()],

        // SERVICE/RPC rules in FILE category
        "SERVICE_NO_DELETE"
        | "RPC_NO_DELETE"
        | "RPC_SAME_REQUEST_TYPE"
        | "RPC_SAME_RESPONSE_TYPE"
        | "RPC_SAME_CLIENT_STREAMING"
        | "RPC_SAME_SERVER_STREAMING" => vec!["FILE".to_string()],

        // PACKAGE category rules
        "PACKAGE_NO_DELETE"
        | "PACKAGE_ENUM_NO_DELETE"
        | "PACKAGE_MESSAGE_NO_DELETE"
        | "PACKAGE_SERVICE_NO_DELETE"
        | "PACKAGE_EXTENSION_NO_DELETE" => vec!["PACKAGE".to_string()],

        // WIRE category rules
        "FIELD_WIRE_COMPATIBLE_TYPE" | "FIELD_WIRE_COMPATIBLE_CARDINALITY" => {
            vec!["WIRE".to_string()]
        }

        // WIRE_JSON category rules
        "FIELD_WIRE_JSON_COMPATIBLE_TYPE" | "FIELD_WIRE_JSON_COMPATIBLE_CARDINALITY" => {
            vec!["WIRE_JSON".to_string()]
        }

        // Default to FILE category for unknown rules
        _ => vec!["FILE".to_string()],
    }
}

impl Default for BreakingEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = BreakingConfig::default();
        assert_eq!(config.use_categories, vec!["FILE", "PACKAGE"]);
        assert!(config.except_rules.is_empty());
    }

    #[test]
    fn test_engine_creation() {
        let engine = BreakingEngine::new();
        assert!(engine.get_rule_count() > 0);
    }

    #[test]
    fn test_rule_selection() {
        let engine = BreakingEngine::new();
        let config = BreakingConfig::default();

        // Create empty canonical files for testing
        let current = CanonicalFile::default();
        let previous = CanonicalFile::default();

        let result = engine.check(&current, &previous, &config);

        // Should execute rules without errors
        assert!(!result.executed_rules.is_empty());
    }

    #[test]
    fn test_rule_exclusion() {
        let engine = BreakingEngine::new();
        let mut config = BreakingConfig::default();
        config.except_rules.push("FILE_SAME_PACKAGE".to_string());

        let current = CanonicalFile::default();
        let previous = CanonicalFile::default();

        let result = engine.check(&current, &previous, &config);

        // Should not include the excluded rule
        assert!(
            !result
                .executed_rules
                .contains(&"FILE_SAME_PACKAGE".to_string())
        );
    }

    #[test]
    fn test_empty_check() {
        let engine = BreakingEngine::new();
        let config = BreakingConfig::default();
        let current = CanonicalFile::default();
        let previous = CanonicalFile::default();

        let result = engine.check(&current, &previous, &config);

        // Empty files should have no breaking changes
        assert!(!result.has_breaking_changes);
        assert!(result.changes.is_empty());
    }
}
