//! Breaking change categories matching Buf's categorization system

use serde::{Deserialize, Serialize};

/// Breaking change categories that group related rules
/// These match exactly with Buf's category system
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BreakingCategory {
    /// FILE category - checks for source-code breaking changes at the per-file level
    File,
    /// PACKAGE category - checks for source-code breaking changes at the per-package level  
    Package,
    /// WIRE category - checks for wire breaking changes for the binary encoding
    Wire,
    /// WIRE_JSON category - checks for wire breaking changes for binary or JSON encodings
    WireJson,
}

impl BreakingCategory {
    /// Get the string identifier for this category (matches Buf exactly)
    pub fn id(&self) -> &'static str {
        match self {
            BreakingCategory::File => "FILE",
            BreakingCategory::Package => "PACKAGE",
            BreakingCategory::Wire => "WIRE",
            BreakingCategory::WireJson => "WIRE_JSON",
        }
    }

    /// Get the description for this category
    pub fn description(&self) -> &'static str {
        match self {
            BreakingCategory::File => {
                "Checks that there are no source-code breaking changes at the per-file level."
            }
            BreakingCategory::Package => {
                "Checks that there are no source-code breaking changes at the per-package level."
            }
            BreakingCategory::Wire => {
                "Checks that there are no wire breaking changes for the binary encoding."
            }
            BreakingCategory::WireJson => {
                "Checks that there are no wire breaking changes for the binary or JSON encodings."
            }
        }
    }

    /// Parse category from string ID
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "FILE" => Some(BreakingCategory::File),
            "PACKAGE" => Some(BreakingCategory::Package),
            "WIRE" => Some(BreakingCategory::Wire),
            "WIRE_JSON" => Some(BreakingCategory::WireJson),
            _ => None,
        }
    }

    /// Get all available categories
    pub fn all() -> Vec<Self> {
        vec![
            BreakingCategory::File,
            BreakingCategory::Package,
            BreakingCategory::Wire,
            BreakingCategory::WireJson,
        ]
    }
}

impl std::fmt::Display for BreakingCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id())
    }
}

impl std::str::FromStr for BreakingCategory {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_id(s).ok_or_else(|| format!("Unknown breaking category: {s}"))
    }
}
