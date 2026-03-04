//! Core types for breaking change detection

use crate::canonical::CanonicalFile;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a breaking change detected between two proto files
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BreakingChange {
    /// The rule ID that detected this change (matches Buf rule IDs exactly)
    pub rule_id: String,
    /// Human-readable description of the breaking change
    pub message: String,
    /// Location information for the change
    pub location: BreakingLocation,
    /// Previous location (for comparison-based rules)
    pub previous_location: Option<BreakingLocation>,
    /// Severity level of the breaking change
    pub severity: BreakingSeverity,
    /// Categories this rule belongs to
    pub categories: Vec<String>,
}

/// Location information for a breaking change
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BreakingLocation {
    /// File path
    pub file_path: String,
    /// Line number (1-based)
    pub line: Option<u32>,
    /// Column number (1-based)
    pub column: Option<u32>,
    /// Element type (e.g., "field", "message", "enum")
    pub element_type: String,
    /// Element name or identifier
    pub element_name: String,
}

/// Severity levels for breaking changes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BreakingSeverity {
    /// Critical breaking change that will definitely break clients
    Error,
    /// Warning about potential compatibility issues
    Warning,
}

/// Context for rule execution
#[derive(Debug, Clone)]
pub struct RuleContext {
    /// Current file being analyzed
    pub current_file: String,
    /// Previous file being compared against
    pub previous_file: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Result of a single rule check
#[derive(Debug, Clone)]
pub struct RuleResult {
    /// Breaking changes found by this rule
    pub changes: Vec<BreakingChange>,
    /// Whether the rule executed successfully
    pub success: bool,
    /// Error message if rule execution failed
    pub error: Option<String>,
}

impl RuleResult {
    pub fn success() -> Self {
        Self {
            changes: Vec::new(),
            success: true,
            error: None,
        }
    }

    pub fn with_changes(changes: Vec<BreakingChange>) -> Self {
        Self {
            changes,
            success: true,
            error: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            changes: Vec::new(),
            success: false,
            error: Some(message),
        }
    }
}

/// Type alias for breaking change rule functions to reduce type complexity
pub type RuleFn = fn(&CanonicalFile, &CanonicalFile, &RuleContext) -> RuleResult;

/// Type alias for rule registry entries
pub type RuleEntry = (&'static str, RuleFn);
