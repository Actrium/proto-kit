//! Bulk-generated field rules using macro magic
//!
//! This module implements all remaining FIELD_* rules in one go.

use crate::canonical::{CanonicalField, CanonicalFile, CanonicalMessage};
use crate::compat::handlers::{create_breaking_change, create_location};
use crate::compat::types::{RuleContext, RuleResult};
use std::collections::{BTreeSet, HashMap};

// ========================================
// Helper Functions for Field Collection
// ========================================

fn collect_all_fields(file: &CanonicalFile) -> HashMap<String, &CanonicalField> {
    let mut all_fields = HashMap::new();

    fn collect_from_messages<'a>(
        messages: &'a BTreeSet<CanonicalMessage>,
        prefix: &str,
        all_fields: &mut HashMap<String, &'a CanonicalField>,
    ) {
        for message in messages {
            let message_name = if prefix.is_empty() {
                message.name.clone()
            } else {
                format!("{}.{}", prefix, message.name)
            };

            for field in &message.fields {
                let field_key = format!("{}.{}", message_name, field.name);
                all_fields.insert(field_key, field);
            }

            collect_from_messages(&message.nested_messages, &message_name, all_fields);
        }
    }

    collect_from_messages(&file.messages, "", &mut all_fields);
    all_fields
}

// ========================================
// MACRO MAGIC for Field Rules
// ========================================

macro_rules! generate_field_rules {
    (
        $(
            ($fn_name:ident, $rule_id:literal, $field_check:expr)
        ),* $(,)?
    ) => {
        $(
            pub fn $fn_name(
                current: &CanonicalFile,
                previous: &CanonicalFile,
                context: &RuleContext,
            ) -> RuleResult {
                let mut changes = Vec::new();

                let previous_fields = collect_all_fields(previous);
                let current_fields = collect_all_fields(current);

                for (field_path, prev_field) in &previous_fields {
                    if let Some(curr_field) = current_fields.get(field_path) {
                        if !($field_check)(prev_field, curr_field) {
                            changes.push(create_breaking_change(
                                $rule_id,
                                format!(
                                    "Field \"{}\" {}: was \"{}\", now \"{}\".",
                                    field_path,
                                    stringify!($fn_name).replace("check_field_", "").replace("_", " "),
                                    get_field_attribute_value(prev_field, $rule_id),
                                    get_field_attribute_value(curr_field, $rule_id)
                                ),
                                create_location(&context.current_file, "field", field_path),
                                Some(create_location(
                                    context.previous_file.as_deref().unwrap_or(""),
                                    "field",
                                    field_path
                                )),
                                vec!["WIRE_JSON".to_string()],
                            ));
                        }
                    }
                }

                RuleResult::with_changes(changes)
            }
        )*
    };
}

// ========================================
// Field Attribute Value Extractor
// ========================================

fn get_field_attribute_value(field: &CanonicalField, rule_id: &str) -> String {
    match rule_id {
        "FIELD_SAME_CARDINALITY" => field.label.as_deref().unwrap_or("optional").to_string(),
        "FIELD_SAME_ONEOF" => field
            .oneof_index
            .map(|i| i.to_string())
            .unwrap_or_else(|| "none".to_string()),
        "FIELD_SAME_JAVA_UTF8_VALIDATION" => field
            .java_utf8_validation
            .map(|b| b.to_string())
            .unwrap_or_else(|| "false".to_string()),
        "FIELD_SAME_UTF8_VALIDATION" => field
            .java_utf8_validation
            .map(|b| b.to_string())
            .unwrap_or_else(|| "false".to_string()),
        _ => "unknown".to_string(),
    }
}

// ========================================
// Bulk Generation: All Field Rules at Once!
// ========================================

generate_field_rules! {
    (check_field_same_cardinality, "FIELD_SAME_CARDINALITY",
        |prev: &CanonicalField, curr: &CanonicalField| {
            prev.label.as_deref().unwrap_or("optional") == curr.label.as_deref().unwrap_or("optional")
        }),

    (check_field_same_oneof, "FIELD_SAME_ONEOF",
        |prev: &CanonicalField, curr: &CanonicalField| {
            prev.oneof_index == curr.oneof_index
        }),

    (check_field_same_java_utf8_validation, "FIELD_SAME_JAVA_UTF8_VALIDATION",
        |prev: &CanonicalField, curr: &CanonicalField| {
            prev.java_utf8_validation == curr.java_utf8_validation
        }),

    (check_field_same_utf8_validation, "FIELD_SAME_UTF8_VALIDATION",
        |prev: &CanonicalField, curr: &CanonicalField| {
            // Generic UTF8 validation check (similar to Java version for now)
            prev.java_utf8_validation == curr.java_utf8_validation
        }),
}

// ========================================
// Complex Field Rules (Hand-coded for Special Logic)
// ========================================

/// FIELD_NO_DELETE_UNLESS_NAME_RESERVED rule
pub fn check_field_no_delete_unless_name_reserved(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    let previous_fields = collect_all_fields(previous);
    let current_fields = collect_all_fields(current);

    // Find deleted fields and check if names are reserved in current messages
    for field_path in previous_fields.keys() {
        if !current_fields.contains_key(field_path) {
            // Field was deleted, check if name is reserved in the message
            let parts: Vec<&str> = field_path.rsplitn(2, '.').collect();
            if parts.len() == 2 {
                let field_name = parts[0];
                let message_path = parts[1];

                let is_reserved =
                    check_field_name_reserved_in_message(current, message_path, field_name);
                if !is_reserved {
                    changes.push(create_breaking_change(
                        "FIELD_NO_DELETE_UNLESS_NAME_RESERVED",
                        format!(
                            "Previously present field \"{field_path}\" was deleted without reserving the name \"{field_name}\"."
                        ),
                        create_location(&context.current_file, "message", message_path),
                        Some(create_location(
                            context.previous_file.as_deref().unwrap_or(""),
                            "field",
                            field_path
                        )),
                        vec!["WIRE_JSON".to_string()],
                    ));
                }
            }
        }
    }

    RuleResult::with_changes(changes)
}

/// FIELD_NO_DELETE_UNLESS_NUMBER_RESERVED rule  
pub fn check_field_no_delete_unless_number_reserved(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    let previous_fields = collect_all_fields(previous);
    let current_fields = collect_all_fields(current);

    // Find deleted fields and check if numbers are reserved in current messages
    for (field_path, prev_field) in &previous_fields {
        if !current_fields.contains_key(field_path) {
            // Field was deleted, check if number is reserved in the message
            let parts: Vec<&str> = field_path.rsplitn(2, '.').collect();
            if parts.len() == 2 {
                let message_path = parts[1];

                let is_reserved = check_field_number_reserved_in_message(
                    current,
                    message_path,
                    prev_field.number,
                );
                if !is_reserved {
                    changes.push(create_breaking_change(
                        "FIELD_NO_DELETE_UNLESS_NUMBER_RESERVED",
                        format!(
                            "Previously present field \"{}\" was deleted without reserving the number \"{}\".",
                            field_path, prev_field.number
                        ),
                        create_location(&context.current_file, "message", message_path),
                        Some(create_location(
                            context.previous_file.as_deref().unwrap_or(""),
                            "field",
                            field_path
                        )),
                        vec!["WIRE_JSON".to_string(), "WIRE".to_string()],
                    ));
                }
            }
        }
    }

    RuleResult::with_changes(changes)
}

// ========================================
// Helper Functions for Reserved Checking
// ========================================

fn collect_all_messages(file: &CanonicalFile) -> HashMap<String, &CanonicalMessage> {
    let mut all_messages = HashMap::new();

    fn collect_from_messages<'a>(
        messages: &'a BTreeSet<CanonicalMessage>,
        prefix: &str,
        all_messages: &mut HashMap<String, &'a CanonicalMessage>,
    ) {
        for message in messages {
            let message_name = if prefix.is_empty() {
                message.name.clone()
            } else {
                format!("{}.{}", prefix, message.name)
            };

            all_messages.insert(message_name.clone(), message);
            collect_from_messages(&message.nested_messages, &message_name, all_messages);
        }
    }

    collect_from_messages(&file.messages, "", &mut all_messages);
    all_messages
}

fn check_field_name_reserved_in_message(
    file: &CanonicalFile,
    message_path: &str,
    field_name: &str,
) -> bool {
    let messages = collect_all_messages(file);
    if let Some(message) = messages.get(message_path) {
        message
            .reserved_names
            .iter()
            .any(|reserved| reserved.name == field_name)
    } else {
        false
    }
}

fn check_field_number_reserved_in_message(
    file: &CanonicalFile,
    message_path: &str,
    field_number: i32,
) -> bool {
    let messages = collect_all_messages(file);
    if let Some(message) = messages.get(message_path) {
        message
            .reserved_ranges
            .iter()
            .any(|range| field_number >= range.start && field_number <= range.end)
    } else {
        false
    }
}

// ========================================
// Wire Compatibility Rules (Simplified)
// ========================================

/// FIELD_WIRE_COMPATIBLE_TYPE - allows compatible type changes
pub fn check_field_wire_compatible_type(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    let previous_fields = collect_all_fields(previous);
    let current_fields = collect_all_fields(current);

    for (field_path, prev_field) in &previous_fields {
        if let Some(curr_field) = current_fields.get(field_path) {
            if !are_types_wire_compatible(&prev_field.type_name, &curr_field.type_name) {
                changes.push(create_breaking_change(
                    "FIELD_WIRE_COMPATIBLE_TYPE",
                    format!(
                        "Field \"{}\" type changed from \"{}\" to \"{}\" which are not wire-compatible.",
                        field_path, prev_field.type_name, curr_field.type_name
                    ),
                    create_location(&context.current_file, "field", field_path),
                    Some(create_location(
                        context.previous_file.as_deref().unwrap_or(""),
                        "field",
                        field_path
                    )),
                    vec!["WIRE".to_string()],
                ));
            }
        }
    }

    RuleResult::with_changes(changes)
}

/// FIELD_WIRE_COMPATIBLE_CARDINALITY - allows compatible cardinality changes
pub fn check_field_wire_compatible_cardinality(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    let previous_fields = collect_all_fields(previous);
    let current_fields = collect_all_fields(current);

    for (field_path, prev_field) in &previous_fields {
        if let Some(curr_field) = current_fields.get(field_path) {
            let prev_cardinality = prev_field.label.as_deref().unwrap_or("optional");
            let curr_cardinality = curr_field.label.as_deref().unwrap_or("optional");

            if !are_cardinalities_wire_compatible(prev_cardinality, curr_cardinality) {
                changes.push(create_breaking_change(
                    "FIELD_WIRE_COMPATIBLE_CARDINALITY",
                    format!(
                        "Field \"{field_path}\" cardinality changed from \"{prev_cardinality}\" to \"{curr_cardinality}\" which are not wire-compatible."
                    ),
                    create_location(&context.current_file, "field", field_path),
                    Some(create_location(
                        context.previous_file.as_deref().unwrap_or(""),
                        "field",
                        field_path
                    )),
                    vec!["WIRE".to_string()],
                ));
            }
        }
    }

    RuleResult::with_changes(changes)
}

// ========================================
// Compatibility Check Functions
// ========================================

fn are_types_wire_compatible(prev_type: &str, curr_type: &str) -> bool {
    if prev_type == curr_type {
        return true;
    }

    // Wire-compatible type pairs (simplified)
    matches!(
        (prev_type, curr_type),
        ("int32", "uint32")
            | ("uint32", "int32")
            | ("int64", "uint64")
            | ("uint64", "int64")
            | ("sint32", "int32")
            | ("int32", "sint32")
            | ("sint64", "int64")
            | ("int64", "sint64")
            | ("fixed32", "uint32")
            | ("uint32", "fixed32")
            | ("fixed64", "uint64")
            | ("uint64", "fixed64")
            | ("sfixed32", "int32")
            | ("int32", "sfixed32")
            | ("sfixed64", "int64")
            | ("int64", "sfixed64")
    )
}

fn are_cardinalities_wire_compatible(prev_cardinality: &str, curr_cardinality: &str) -> bool {
    if prev_cardinality == curr_cardinality {
        return true;
    }

    // Compatible cardinality changes
    matches!(
        (prev_cardinality, curr_cardinality),
        ("required", "optional") | ("optional", "repeated") // Simplified
    )
}

/// FIELD_WIRE_JSON_COMPATIBLE_TYPE - allows JSON+wire compatible type changes
pub fn check_field_wire_json_compatible_type(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    let previous_fields = collect_all_fields(previous);
    let current_fields = collect_all_fields(current);

    for (field_path, prev_field) in &previous_fields {
        if let Some(curr_field) = current_fields.get(field_path) {
            if !are_types_wire_json_compatible(&prev_field.type_name, &curr_field.type_name) {
                changes.push(create_breaking_change(
                    "FIELD_WIRE_JSON_COMPATIBLE_TYPE",
                    format!(
                        "Field \"{}\" type changed from \"{}\" to \"{}\" which are not wire+JSON compatible.",
                        field_path, prev_field.type_name, curr_field.type_name
                    ),
                    create_location(&context.current_file, "field", field_path),
                    Some(create_location(
                        context.previous_file.as_deref().unwrap_or(""),
                        "field",
                        field_path
                    )),
                    vec!["WIRE_JSON".to_string(), "WIRE".to_string()],
                ));
            }
        }
    }

    RuleResult::with_changes(changes)
}

/// FIELD_WIRE_JSON_COMPATIBLE_CARDINALITY - allows JSON+wire compatible cardinality changes  
pub fn check_field_wire_json_compatible_cardinality(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    let previous_fields = collect_all_fields(previous);
    let current_fields = collect_all_fields(current);

    for (field_path, prev_field) in &previous_fields {
        if let Some(curr_field) = current_fields.get(field_path) {
            let prev_cardinality = prev_field.label.as_deref().unwrap_or("optional");
            let curr_cardinality = curr_field.label.as_deref().unwrap_or("optional");

            if !are_cardinalities_wire_json_compatible(prev_cardinality, curr_cardinality) {
                changes.push(create_breaking_change(
                    "FIELD_WIRE_JSON_COMPATIBLE_CARDINALITY",
                    format!(
                        "Field \"{field_path}\" cardinality changed from \"{prev_cardinality}\" to \"{curr_cardinality}\" which are not wire+JSON compatible."
                    ),
                    create_location(&context.current_file, "field", field_path),
                    Some(create_location(
                        context.previous_file.as_deref().unwrap_or(""),
                        "field",
                        field_path
                    )),
                    vec!["WIRE_JSON".to_string(), "WIRE".to_string()],
                ));
            }
        }
    }

    RuleResult::with_changes(changes)
}

// ========================================
// Enhanced Compatibility Functions
// ========================================

fn are_types_wire_json_compatible(prev_type: &str, curr_type: &str) -> bool {
    if prev_type == curr_type {
        return true;
    }

    // Wire+JSON compatible types (more restrictive than wire-only)
    matches!(
        (prev_type, curr_type),
        ("int32", "uint32") | ("uint32", "int32") | ("int64", "uint64") | ("uint64", "int64") // Note: JSON compatibility is more restrictive than wire-only
                                                                                              // Some wire-compatible changes break JSON representation
    )
}

fn are_cardinalities_wire_json_compatible(prev_cardinality: &str, curr_cardinality: &str) -> bool {
    if prev_cardinality == curr_cardinality {
        return true;
    }

    // JSON+Wire compatible cardinality changes (very restrictive)
    matches!(
        (prev_cardinality, curr_cardinality),
        ("optional", "repeated") // Limited compatibility
    )
}

/// FIELD_SAME_DEFAULT - checks field default values don't change
pub fn check_field_same_default(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    let previous_fields = collect_all_fields(previous);
    let current_fields = collect_all_fields(current);

    for (field_path, prev_field) in &previous_fields {
        if let Some(curr_field) = current_fields.get(field_path) {
            if prev_field.default != curr_field.default {
                changes.push(create_breaking_change(
                    "FIELD_SAME_DEFAULT",
                    format!(
                        "Field \"{}\" default value changed from \"{}\" to \"{}\".",
                        field_path,
                        prev_field.default.as_deref().unwrap_or(""),
                        curr_field.default.as_deref().unwrap_or("")
                    ),
                    create_location(&context.current_file, "field", field_path),
                    Some(create_location(
                        context.previous_file.as_deref().unwrap_or(""),
                        "field",
                        field_path,
                    )),
                    vec!["WIRE_JSON".to_string()],
                ));
            }
        }
    }

    RuleResult::with_changes(changes)
}

/// FIELD_SAME_JSON_NAME - checks field JSON names don't change
pub fn check_field_same_json_name(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    let previous_fields = collect_all_fields(previous);
    let current_fields = collect_all_fields(current);

    for (field_path, prev_field) in &previous_fields {
        if let Some(curr_field) = current_fields.get(field_path) {
            if prev_field.json_name != curr_field.json_name {
                changes.push(create_breaking_change(
                    "FIELD_SAME_JSON_NAME",
                    format!(
                        "Field \"{}\" JSON name changed from \"{}\" to \"{}\".",
                        field_path,
                        prev_field.json_name.as_deref().unwrap_or(""),
                        curr_field.json_name.as_deref().unwrap_or("")
                    ),
                    create_location(&context.current_file, "field", field_path),
                    Some(create_location(
                        context.previous_file.as_deref().unwrap_or(""),
                        "field",
                        field_path,
                    )),
                    vec!["WIRE_JSON".to_string()],
                ));
            }
        }
    }

    RuleResult::with_changes(changes)
}

/// FIELD_SAME_JSTYPE - checks field JSType options don't change
pub fn check_field_same_jstype(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    let previous_fields = collect_all_fields(previous);
    let current_fields = collect_all_fields(current);

    for (field_path, prev_field) in &previous_fields {
        if let Some(curr_field) = current_fields.get(field_path) {
            if prev_field.jstype != curr_field.jstype {
                changes.push(create_breaking_change(
                    "FIELD_SAME_JSTYPE",
                    format!(
                        "Field \"{}\" JSType changed from \"{}\" to \"{}\".",
                        field_path,
                        prev_field.jstype.as_deref().unwrap_or("JS_NORMAL"),
                        curr_field.jstype.as_deref().unwrap_or("JS_NORMAL")
                    ),
                    create_location(&context.current_file, "field", field_path),
                    Some(create_location(
                        context.previous_file.as_deref().unwrap_or(""),
                        "field",
                        field_path,
                    )),
                    vec!["WIRE_JSON".to_string()],
                ));
            }
        }
    }

    RuleResult::with_changes(changes)
}

/// FIELD_SAME_CTYPE - checks field CType options don't change
pub fn check_field_same_ctype(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    let previous_fields = collect_all_fields(previous);
    let current_fields = collect_all_fields(current);

    for (field_path, prev_field) in &previous_fields {
        if let Some(curr_field) = current_fields.get(field_path) {
            if prev_field.ctype != curr_field.ctype {
                changes.push(create_breaking_change(
                    "FIELD_SAME_CTYPE",
                    format!(
                        "Field \"{}\" CType changed from \"{}\" to \"{}\".",
                        field_path,
                        prev_field.ctype.as_deref().unwrap_or("STRING"),
                        curr_field.ctype.as_deref().unwrap_or("STRING")
                    ),
                    create_location(&context.current_file, "field", field_path),
                    Some(create_location(
                        context.previous_file.as_deref().unwrap_or(""),
                        "field",
                        field_path,
                    )),
                    vec!["WIRE_JSON".to_string()],
                ));
            }
        }
    }

    RuleResult::with_changes(changes)
}

/// FIELD_SAME_CPP_STRING_TYPE - checks field C++ string type options don't change
pub fn check_field_same_cpp_string_type(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    let previous_fields = collect_all_fields(previous);
    let current_fields = collect_all_fields(current);

    for (field_path, prev_field) in &previous_fields {
        if let Some(curr_field) = current_fields.get(field_path) {
            if prev_field.cpp_string_type != curr_field.cpp_string_type {
                changes.push(create_breaking_change(
                    "FIELD_SAME_CPP_STRING_TYPE",
                    format!(
                        "Field \"{}\" C++ string type changed from \"{}\" to \"{}\".",
                        field_path,
                        prev_field.cpp_string_type.as_deref().unwrap_or(""),
                        curr_field.cpp_string_type.as_deref().unwrap_or("")
                    ),
                    create_location(&context.current_file, "field", field_path),
                    Some(create_location(
                        context.previous_file.as_deref().unwrap_or(""),
                        "field",
                        field_path,
                    )),
                    vec!["WIRE_JSON".to_string()],
                ));
            }
        }
    }

    RuleResult::with_changes(changes)
}

/// FIELD_SAME_LABEL - checks field labels (required/optional/repeated) don't change  
pub fn check_field_same_label(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    let previous_fields = collect_all_fields(previous);
    let current_fields = collect_all_fields(current);

    for (field_path, prev_field) in &previous_fields {
        if let Some(curr_field) = current_fields.get(field_path) {
            let prev_label = prev_field.label.as_deref().unwrap_or("optional");
            let curr_label = curr_field.label.as_deref().unwrap_or("optional");

            if prev_label != curr_label {
                changes.push(create_breaking_change(
                    "FIELD_SAME_LABEL",
                    format!(
                        "Field \"{field_path}\" label changed from \"{prev_label}\" to \"{curr_label}\"."
                    ),
                    create_location(&context.current_file, "field", field_path),
                    Some(create_location(
                        context.previous_file.as_deref().unwrap_or(""),
                        "field",
                        field_path,
                    )),
                    vec!["WIRE_JSON".to_string(), "WIRE".to_string()],
                ));
            }
        }
    }

    RuleResult::with_changes(changes)
}

// ========================================
// Rule Export Table
// ========================================

pub const FIELD_RULES: &[crate::compat::types::RuleEntry] = &[
    ("FIELD_SAME_CARDINALITY", check_field_same_cardinality),
    ("FIELD_SAME_ONEOF", check_field_same_oneof),
    (
        "FIELD_SAME_JAVA_UTF8_VALIDATION",
        check_field_same_java_utf8_validation,
    ),
    (
        "FIELD_NO_DELETE_UNLESS_NAME_RESERVED",
        check_field_no_delete_unless_name_reserved,
    ),
    (
        "FIELD_NO_DELETE_UNLESS_NUMBER_RESERVED",
        check_field_no_delete_unless_number_reserved,
    ),
    (
        "FIELD_WIRE_COMPATIBLE_TYPE",
        check_field_wire_compatible_type,
    ),
    (
        "FIELD_WIRE_COMPATIBLE_CARDINALITY",
        check_field_wire_compatible_cardinality,
    ),
    (
        "FIELD_WIRE_JSON_COMPATIBLE_TYPE",
        check_field_wire_json_compatible_type,
    ),
    (
        "FIELD_WIRE_JSON_COMPATIBLE_CARDINALITY",
        check_field_wire_json_compatible_cardinality,
    ),
    ("FIELD_SAME_DEFAULT", check_field_same_default),
    ("FIELD_SAME_JSON_NAME", check_field_same_json_name),
    ("FIELD_SAME_JSTYPE", check_field_same_jstype),
    ("FIELD_SAME_CTYPE", check_field_same_ctype),
    (
        "FIELD_SAME_CPP_STRING_TYPE",
        check_field_same_cpp_string_type,
    ),
    ("FIELD_SAME_LABEL", check_field_same_label),
    (
        "FIELD_SAME_UTF8_VALIDATION",
        check_field_same_utf8_validation,
    ),
];
