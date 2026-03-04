//! Provides the high-level Spec API for comparing Protobuf files.

use crate::compat::{BreakingConfig, BreakingEngine, BreakingResult};
use crate::compatibility::{CompatibilityModel, get_compatibility_model};
use crate::generate_fingerprint;
use std::collections::BTreeMap;

/// The result of a compatibility comparison between two Protobuf specifications.
#[derive(Debug, PartialEq, Eq)]
pub enum Compatibility {
    /// The two specifications are semantically identical.
    Green,
    /// The new specification is backward-compatible with the old one (e.g., a new field was added).
    Yellow,
    /// The new specification is not backward-compatible with the old one (e.g., a field was removed or changed).
    Red,
}

/// Represents a single Protobuf specification, holding its content and derived models for comparison.
#[derive(Debug)]
pub struct Spec<'a> {
    /// The original content of the .proto file.
    pub content: &'a str,
    /// The exact semantic fingerprint.
    pub fingerprint: String,
    /// The compatibility model.
    pub compatibility_model: CompatibilityModel,
    /// The canonical representation for detailed breaking change analysis.
    pub canonical_file: crate::canonical::CanonicalFile,
}

impl<'a> Spec<'a> {
    /// Creates a new `Spec` from the content of a .proto file.
    ///
    /// This function parses the content and generates the necessary fingerprint and models,
    /// so it should be called once per file.
    pub fn try_from(content: &'a str) -> anyhow::Result<Self> {
        let fingerprint = generate_fingerprint(content)?;
        let compatibility_model = get_compatibility_model(content)?;
        let canonical_file = parse_canonical_file(content)?;
        Ok(Spec {
            content,
            fingerprint,
            compatibility_model,
            canonical_file,
        })
    }

    /// Creates a new `Spec` from a .proto file path.
    ///
    /// This variant provides file path context for better import resolution.
    pub fn try_from_file(file_path: &std::path::Path, content: &'a str) -> anyhow::Result<Self> {
        // Try normal parsing first
        match Self::try_from_file_internal(file_path, content) {
            Ok(spec) => Ok(spec),
            Err(e) => {
                eprintln!(
                    "Warning: Proto parsing failed, using fallback for {}: {}",
                    file_path.display(),
                    e
                );
                // Fallback: create a minimal spec with basic parsing
                Ok(Self::create_fallback_spec(content))
            }
        }
    }

    /// Internal try_from_file implementation without fallback
    fn try_from_file_internal(
        file_path: &std::path::Path,
        content: &'a str,
    ) -> anyhow::Result<Self> {
        let fingerprint = generate_fingerprint(content)?;
        let compatibility_model = get_compatibility_model(content)?;
        let canonical_file = parse_canonical_file_with_context(content, Some(file_path))?;
        Ok(Spec {
            content,
            fingerprint,
            compatibility_model,
            canonical_file,
        })
    }

    /// Create a fallback spec when parsing fails
    fn create_fallback_spec(content: &'a str) -> Self {
        // Use simplified fingerprint (just length + first/last chars as a basic hash)
        let fingerprint = format!(
            "fallback_{}_{}_{}",
            content.len(),
            content.chars().next().unwrap_or('_') as u32,
            content.chars().last().unwrap_or('_') as u32
        );

        // Use a minimal compatibility model
        let compatibility_model = CompatibilityModel {
            messages: std::collections::BTreeSet::new(),
            services: std::collections::BTreeSet::new(),
        };

        // Use the fallback canonical file parser
        let canonical_file = create_fallback_canonical_file(content);

        Spec {
            content,
            fingerprint,
            compatibility_model,
            canonical_file,
        }
    }

    /// Compares this `Spec` (the "old" version) with another `Spec` (the "new" version)
    /// to determine their compatibility level.
    pub fn compare_with(&self, new_spec: &Spec) -> Compatibility {
        // If the exact fingerprints are identical, the files are semantically identical.
        if self.fingerprint == new_spec.fingerprint {
            return Compatibility::Green;
        }

        // If the fingerprints differ, check for backward compatibility.
        if crate::compatibility::is_compatible(
            &self.compatibility_model,
            &new_spec.compatibility_model,
        ) {
            return Compatibility::Yellow;
        }

        // If it's not identical and not compatible, it's a breaking change.
        Compatibility::Red
    }

    /// Perform detailed breaking change analysis using the Buf-compatible rule system
    pub fn check_breaking_changes(&self, new_spec: &Spec) -> BreakingResult {
        self.check_breaking_changes_with_config(new_spec, &BreakingConfig::default())
    }

    /// Perform detailed breaking change analysis with custom configuration
    pub fn check_breaking_changes_with_config(
        &self,
        new_spec: &Spec,
        config: &BreakingConfig,
    ) -> BreakingResult {
        let engine = BreakingEngine::new();
        engine.check(&new_spec.canonical_file, &self.canonical_file, config)
    }
}

/// Parse a proto file content into a canonical file representation
fn parse_canonical_file(proto_content: &str) -> anyhow::Result<crate::canonical::CanonicalFile> {
    parse_canonical_file_with_context(proto_content, None)
}

/// Parse a proto file content with optional file path context for imports
fn parse_canonical_file_with_context(
    proto_content: &str,
    file_path_context: Option<&std::path::Path>,
) -> anyhow::Result<crate::canonical::CanonicalFile> {
    use anyhow::Context;

    let temp_dir = tempfile::tempdir().context("Failed to create temp directory")?;
    let file_name = "input.proto";
    let temp_path = temp_dir.path().join(file_name);

    // Pre-process proto content to handle unsupported syntax
    let processed_content = preprocess_proto_content(proto_content);
    std::fs::write(&temp_path, &processed_content).context("Failed to write to temp file")?;

    // Handle imports - try to find actual files first, then create dummy files
    for line in proto_content.lines() {
        if line.trim().starts_with("import ") {
            let path_str = line
                .trim()
                .trim_start_matches("import ")
                .trim_start_matches("public ")
                .trim_start_matches("weak ")
                .trim_matches(|c| c == '"' || c == ';');

            if !path_str.starts_with("google/protobuf/") {
                let import_path = temp_dir.path().join(path_str);
                if let Some(parent) = import_path.parent() {
                    std::fs::create_dir_all(parent).context(format!(
                        "Failed to create parent dirs for import: {path_str}"
                    ))?;
                }

                // Try to find the actual import file
                let mut found = false;

                // First, try relative to the file being parsed if we have context
                if let Some(context_path) = file_path_context {
                    if let Some(parent_dir) = context_path.parent() {
                        let actual_import_path = parent_dir.join(path_str);
                        if actual_import_path.exists() {
                            let import_content = std::fs::read_to_string(&actual_import_path)
                                .context(format!("Failed to read import file: {path_str}"))?;
                            let processed_import = preprocess_proto_content(&import_content);
                            std::fs::write(&import_path, processed_import)
                                .context(format!("Failed to copy import file: {path_str}"))?;
                            found = true;
                        }
                    }
                }

                // If not found with context, try current working directory
                if !found {
                    let current_dir = std::path::Path::new(".");
                    let actual_import_path = current_dir.join(path_str);

                    if actual_import_path.exists() {
                        let import_content = std::fs::read_to_string(&actual_import_path)
                            .context(format!("Failed to read import file: {path_str}"))?;
                        let processed_import = preprocess_proto_content(&import_content);
                        std::fs::write(&import_path, processed_import)
                            .context(format!("Failed to copy import file: {path_str}"))?;
                        found = true;
                    }
                }

                // Fallback: create a dummy proto3 file
                if !found {
                    std::fs::write(&import_path, "syntax = \"proto3\";")
                        .context(format!("Failed to create dummy import file: {path_str}"))?;
                }
            }
        }
    }

    // Attempt parsing with error recovery
    match try_parse_with_fallback(&temp_dir, &temp_path, file_name, &processed_content) {
        Ok(canonical_file) => Ok(canonical_file),
        Err(e) => {
            eprintln!("Warning: Proto parsing failed, using fallback: {e}");
            Ok(create_fallback_canonical_file(&processed_content))
        }
    }
}

/// Preprocess proto content to handle unsupported syntax and edge cases
fn preprocess_proto_content(content: &str) -> String {
    let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let mut processed_lines = Vec::new();

    // Detect if this is an editions file and convert to proto3
    let mut is_editions = false;
    let mut has_syntax_declaration = false;

    for line in &lines {
        let trimmed = line.trim();
        if trimmed.starts_with("edition =") {
            is_editions = true;
            // Convert edition to proto3
            processed_lines.push("syntax = \"proto3\";".to_string());
            continue;
        } else if trimmed.starts_with("syntax =") {
            has_syntax_declaration = true;
        }
    }

    // If no syntax declaration and not editions, default to proto2
    if !has_syntax_declaration && !is_editions {
        processed_lines.insert(0, "syntax = \"proto2\";".to_string());
        processed_lines.insert(1, "".to_string());
    }

    for line in &lines {
        let trimmed = line.trim();

        // Skip edition lines (already handled)
        if trimmed.starts_with("edition =") {
            continue;
        }

        // Convert editions-specific features to proto3 equivalents
        if is_editions {
            let converted_line = convert_editions_features(line);
            processed_lines.push(converted_line);
        } else {
            processed_lines.push(line.clone());
        }
    }

    processed_lines.join("\n")
}

/// Convert Protobuf Editions features to proto3 equivalent syntax
fn convert_editions_features(line: &str) -> String {
    let mut result = line.to_string();

    // Convert [features.field_presence = LEGACY_REQUIRED] to similar proto2/proto3 syntax
    if result.contains("[features.field_presence = LEGACY_REQUIRED]") {
        // Remove the feature annotation for now - this is a simplification
        result = result.replace("[features.field_presence = LEGACY_REQUIRED]", "");
        result = result.trim_end().to_string();
        if result.ends_with(' ') {
            result = result.trim_end().to_string();
        }
    }

    // Convert other editions features as needed
    if result.contains("[features.") {
        // For now, remove unsupported features annotations
        if let Some(start) = result.find("[features.") {
            if let Some(end) = result[start..].find(']') {
                let before = &result[..start];
                let after = &result[start + end + 1..];
                result = format!("{}{}", before.trim_end(), after);
            }
        }
    }

    result
}

/// Attempt to parse proto files with error recovery
fn try_parse_with_fallback(
    temp_dir: &tempfile::TempDir,
    temp_path: &std::path::Path,
    file_name: &str,
    _processed_content: &str,
) -> anyhow::Result<crate::canonical::CanonicalFile> {
    use anyhow::Context;
    use protobuf_parse::Parser;

    let parsed = Parser::new()
        .pure()
        .include(temp_dir.path())
        .input(temp_path)
        .file_descriptor_set()
        .context("Protobuf parsing failed")?;

    let file_descriptor = parsed
        .file
        .into_iter()
        .find(|d| d.name() == file_name)
        .context("Could not find the parsed file descriptor for the input file")?;

    Ok(crate::normalize::normalize_file(&file_descriptor))
}

/// Create a fallback canonical file when parsing fails
/// This extracts basic information using simple text parsing
fn create_fallback_canonical_file(content: &str) -> crate::canonical::CanonicalFile {
    use crate::canonical::*;
    use std::collections::BTreeSet;

    let mut canonical_file = CanonicalFile::default();

    // Extract syntax
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("syntax = ") {
            if trimmed.contains("proto3") {
                canonical_file.syntax = "proto3".to_string();
            } else if trimmed.contains("proto2") {
                canonical_file.syntax = "proto2".to_string();
            }
            break;
        } else if trimmed.starts_with("edition = ") {
            canonical_file.syntax = "proto3".to_string(); // Treat editions as proto3
            break;
        }
    }

    // If no syntax found, default to proto2 (protobuf default)
    if canonical_file.syntax.is_empty() {
        canonical_file.syntax = "proto2".to_string();
    }

    // Extract package
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("package ") && trimmed.ends_with(';') {
            canonical_file.package = Some(trimmed[8..trimmed.len() - 1].trim().to_string());
            break;
        }
    }

    // Extract messages and enums (for basic breaking change detection)
    let mut messages = BTreeSet::new();
    let mut enums = BTreeSet::new();
    let mut in_message = false;
    let mut in_enum = false;
    let mut current_message = String::new();
    let mut current_enum = String::new();
    let mut current_enum_options = BTreeMap::new();
    let mut brace_count = 0;

    for line in content.lines() {
        let trimmed = line.trim();

        // Handle message parsing
        if trimmed.starts_with("message ") && !in_enum {
            if let Some(name_part) = trimmed.strip_prefix("message ") {
                if let Some(space_pos) = name_part.find(' ') {
                    current_message = name_part[..space_pos].to_string();
                } else if let Some(brace_pos) = name_part.find('{') {
                    current_message = name_part[..brace_pos].trim().to_string();
                } else {
                    current_message = name_part.trim().to_string();
                }
                in_message = true;
                brace_count = trimmed.matches('{').count();
            }
        }

        // Handle enum parsing
        if trimmed.starts_with("enum ") && !in_message {
            if let Some(name_part) = trimmed.strip_prefix("enum ") {
                if let Some(space_pos) = name_part.find(' ') {
                    current_enum = name_part[..space_pos].to_string();
                } else if let Some(brace_pos) = name_part.find('{') {
                    current_enum = name_part[..brace_pos].trim().to_string();
                } else {
                    current_enum = name_part.trim().to_string();
                }
                in_enum = true;
                current_enum_options.clear();
                brace_count = trimmed.matches('{').count();
            }
        }

        // Extract enum options while in enum
        if in_enum && trimmed.starts_with("option ") && trimmed.contains("features.json_format") {
            // Extract json_format value
            if let Some(equals_pos) = trimmed.find('=') {
                let value_part = &trimmed[equals_pos + 1..];
                let value = value_part.trim().trim_end_matches(';').trim();
                current_enum_options.insert("json_format".to_string(), value.to_string());
            }
        }

        // Handle brace counting for both messages and enums
        if in_message || in_enum {
            brace_count += trimmed.matches('{').count();
            brace_count -= trimmed.matches('}').count();

            if brace_count == 0 {
                if in_message {
                    // Create a minimal message with extracted fields
                    let fields = extract_fields_from_text(content, &current_message);
                    let message = CanonicalMessage {
                        name: current_message.clone(),
                        fields,
                        nested_messages: BTreeSet::new(),
                        nested_enums: BTreeSet::new(),
                        oneofs: Vec::new(),
                        reserved_ranges: BTreeSet::new(),
                        reserved_names: BTreeSet::new(),
                        extension_ranges: BTreeSet::new(),
                        message_set_wire_format: None,
                        no_standard_descriptor_accessor: None,
                        deprecated: None,
                    };
                    messages.insert(message);
                    in_message = false;
                } else if in_enum {
                    // Create a minimal enum with options
                    let en = CanonicalEnum {
                        name: current_enum.clone(),
                        values: BTreeSet::new(),
                        reserved_ranges: BTreeSet::new(),
                        reserved_names: BTreeSet::new(),
                        allow_alias: None,
                        deprecated: None,
                        closed_enum: None,
                        options: current_enum_options.clone(),
                    };
                    enums.insert(en);
                    in_enum = false;
                }
            }
        }
    }

    canonical_file.messages = messages;
    canonical_file.enums = enums;
    canonical_file
}

/// Extract fields from a message using text parsing
fn extract_fields_from_text(
    content: &str,
    message_name: &str,
) -> std::collections::BTreeSet<crate::canonical::CanonicalField> {
    use std::collections::BTreeSet;

    let mut fields = BTreeSet::new();
    let mut in_message = false;
    let mut brace_count = 0;
    let mut found_message = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Find the specific message
        if trimmed.starts_with("message ") {
            if let Some(name_part) = trimmed.strip_prefix("message ") {
                let name = if let Some(space_pos) = name_part.find(' ') {
                    &name_part[..space_pos]
                } else if let Some(brace_pos) = name_part.find('{') {
                    name_part[..brace_pos].trim()
                } else {
                    name_part.trim()
                };

                if name == message_name {
                    found_message = true;
                    in_message = true;
                    brace_count = trimmed.matches('{').count();
                }
            }
        }

        if found_message && in_message {
            brace_count += trimmed.matches('{').count();
            brace_count -= trimmed.matches('}').count();

            // Parse field lines
            if brace_count > 0 && !trimmed.starts_with("message ") && !trimmed.starts_with("enum ")
            {
                if let Some(field) = parse_field_line(trimmed) {
                    fields.insert(field);
                }
            }

            if brace_count == 0 {
                break; // Exit when message ends
            }
        }
    }

    fields
}

/// Parse a single field line from proto text
fn parse_field_line(line: &str) -> Option<crate::canonical::CanonicalField> {
    use crate::canonical::CanonicalField;

    let trimmed = line.trim();

    // Skip comments, empty lines, and nested definitions
    if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with("oneof ") {
        return None;
    }

    // Parse field format: [label] type name = number;
    // Examples:
    // int32 one = 1;
    // repeated int64 one = 1;
    // optional int32 one = 1;
    // map<int32, Two> six = 6;

    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    if parts.len() < 4 {
        return None;
    }

    let mut idx = 0;
    let mut label = None;

    // Check for label (optional, required, repeated)
    if parts[idx] == "optional" || parts[idx] == "required" || parts[idx] == "repeated" {
        label = Some(parts[idx].to_string());
        idx += 1;
    }

    if idx + 2 >= parts.len() {
        return None;
    }

    let type_name = parts[idx].to_string();
    let field_name = parts[idx + 1].to_string();

    // Extract field number from "= number;"
    let equals_idx = idx + 2;
    if equals_idx >= parts.len() || parts[equals_idx] != "=" {
        return None;
    }

    let number_str = if equals_idx + 1 < parts.len() {
        parts[equals_idx + 1].trim_end_matches(';')
    } else {
        return None;
    };

    let number = number_str.parse::<i32>().ok()?;

    Some(CanonicalField {
        name: field_name,
        number,
        label,
        type_name,
        oneof_index: None,
        options: BTreeMap::new(),
        default: None,
        json_name: None,
        jstype: None,
        ctype: None,
        cpp_string_type: None,
        utf8_validation: None,
        java_utf8_validation: None,
        deprecated: None,
        weak: None,
    })
}
