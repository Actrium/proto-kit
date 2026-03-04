//! Provides structures and functions for checking backward-compatibility of Protobuf files.

use serde::Serialize;
use std::collections::BTreeSet;

//==============================================================================
// Structures for Compatibility Analysis
//==============================================================================

/// Represents the backward-compatibility-relevant content of a .proto file.
#[derive(Debug, Default, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct CompatibilityModel {
    #[serde(skip_serializing_if = "BTreeSet::is_empty")]
    pub messages: BTreeSet<CompatibilityMessage>,
    #[serde(skip_serializing_if = "BTreeSet::is_empty")]
    pub services: BTreeSet<CompatibilityService>,
}

/// Represents a message for compatibility purposes.
#[derive(Debug, Default, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct CompatibilityMessage {
    pub name: String,
    #[serde(skip_serializing_if = "BTreeSet::is_empty")]
    pub fields: BTreeSet<CompatibilityField>,
}

/// Represents a field for compatibility purposes.
/// Note the absence of `name` and `label`.
#[derive(Debug, Default, Serialize, PartialEq, Eq)]
pub struct CompatibilityField {
    pub number: i32,
    pub type_name: String,
}

// Custom implementation of Ord for CompatibilityField to sort by `number` first.
impl Ord for CompatibilityField {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.number
            .cmp(&other.number)
            .then_with(|| self.type_name.cmp(&other.type_name))
    }
}

impl PartialOrd for CompatibilityField {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Represents a service for compatibility purposes.
#[derive(Debug, Default, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct CompatibilityService {
    pub name: String,
    #[serde(skip_serializing_if = "BTreeSet::is_empty")]
    pub methods: BTreeSet<CompatibilityMethod>,
}

/// Represents a service method for compatibility purposes.
#[derive(Debug, Default, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct CompatibilityMethod {
    pub name: String,
    pub input_type: String,
    pub output_type: String,
}

//==============================================================================
// Public API Functions
//==============================================================================

use crate::normalize;
use anyhow::Context;
use protobuf_parse::Parser;

/// Parses a `.proto` file content and returns its compatibility model.
pub fn get_compatibility_model(proto_content: &str) -> anyhow::Result<CompatibilityModel> {
    let temp_dir = tempfile::tempdir().context("Failed to create temp directory")?;
    let file_name = "input.proto";
    let temp_path = temp_dir.path().join(file_name);
    std::fs::write(&temp_path, proto_content).context("Failed to write to temp file")?;

    for line in proto_content.lines() {
        if line.trim().starts_with("import ") {
            let path_str = line
                .trim()
                .trim_start_matches("import ")
                .trim_start_matches("public ")
                .trim_matches(|c| c == '"' || c == ';');

            if !path_str.starts_with("google/protobuf/") {
                let import_path = temp_dir.path().join(path_str);
                if let Some(parent) = import_path.parent() {
                    std::fs::create_dir_all(parent).context(format!(
                        "Failed to create parent dirs for import: {path_str}"
                    ))?;
                }
                std::fs::write(&import_path, "syntax = \"proto3\";")
                    .context(format!("Failed to create dummy import file: {path_str}"))?;
            }
        }
    }

    let parsed = Parser::new()
        .pure()
        .include(temp_dir.path())
        .input(&temp_path)
        .file_descriptor_set()
        .context("Protobuf parsing failed")?;

    let file_descriptor = parsed
        .file
        .into_iter()
        .find(|d| d.name() == file_name)
        .context("Could not find the parsed file descriptor for the input file")?;

    Ok(normalize::normalize_compatibility_file(&file_descriptor))
}

/// Compares two compatibility models to see if `new_model` is backward-compatible
/// with `old_model`.
pub fn is_compatible(old_model: &CompatibilityModel, new_model: &CompatibilityModel) -> bool {
    // To be compatible, the new model must have every message and service
    // that the old model has, and their contents must be compatible.

    // Check messages
    for old_msg in &old_model.messages {
        // Find the corresponding message in the new model by name.
        if let Some(new_msg) = new_model.messages.iter().find(|m| m.name == old_msg.name) {
            // The new message's fields must be a superset of the old message's fields.
            if !old_msg.fields.is_subset(&new_msg.fields) {
                return false; // Breaking change: a field was removed or its type/number changed.
            }
        } else {
            return false; // Breaking change: a message was removed.
        }
    }

    // Check services
    for old_svc in &old_model.services {
        // Find the corresponding service in the new model by name.
        if let Some(new_svc) = new_model.services.iter().find(|s| s.name == old_svc.name) {
            // The new service's methods must be a superset of the old one's.
            if !old_svc.methods.is_subset(&new_svc.methods) {
                return false; // Breaking change: a method was removed or its signature changed.
            }
        } else {
            return false; // Breaking change: a service was removed.
        }
    }

    true
}
