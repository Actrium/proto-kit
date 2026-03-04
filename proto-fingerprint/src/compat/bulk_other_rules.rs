//! Bulk-generated message, enum, service, and other rules
//!
//! This module implements all remaining non-field rules in one go.

use crate::canonical::{
    CanonicalEnum, CanonicalEnumValue, CanonicalFile, CanonicalMessage, CanonicalService,
};
use crate::compat::handlers::{create_breaking_change, create_location};
use crate::compat::types::{RuleContext, RuleResult};
use std::collections::{BTreeSet, HashMap};

// ========================================
// Message Rules
// ========================================

/// MESSAGE_SAME_JSON_FORMAT rule
pub fn check_message_same_json_format(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    let prev_messages = collect_all_messages(previous);
    let curr_messages = collect_all_messages(current);

    for (message_path, prev_message) in &prev_messages {
        if let Some(curr_message) = curr_messages.get(message_path) {
            // Check if message_set_wire_format changed (this affects JSON format)
            let prev_wire_format = prev_message.message_set_wire_format.unwrap_or(false);
            let curr_wire_format = curr_message.message_set_wire_format.unwrap_or(false);

            if prev_wire_format != curr_wire_format {
                changes.push(create_breaking_change(
                    "MESSAGE_SAME_JSON_FORMAT",
                    format!(
                        "Message \"{message_path}\" message_set_wire_format changed from {prev_wire_format} to {curr_wire_format} (affects JSON format)."
                    ),
                    create_location(&context.current_file, "message", message_path),
                    Some(create_location(
                        context.previous_file.as_deref().unwrap_or(""),
                        "message",
                        message_path
                    )),
                    vec!["WIRE_JSON".to_string()],
                ));
            }
        }
    }

    RuleResult::with_changes(changes)
}

/// MESSAGE_SAME_REQUIRED_FIELDS rule
pub fn check_message_same_required_fields(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    // Collect all messages with their required fields
    let prev_messages = collect_all_messages(previous);
    let curr_messages = collect_all_messages(current);

    for (message_path, prev_message) in &prev_messages {
        if let Some(curr_message) = curr_messages.get(message_path) {
            let prev_required = get_required_fields(prev_message);
            let curr_required = get_required_fields(curr_message);

            // Check for added required fields (breaking)
            for field_name in &curr_required {
                if !prev_required.contains(field_name) {
                    changes.push(create_breaking_change(
                        "MESSAGE_SAME_REQUIRED_FIELDS",
                        format!(
                            "Message \"{message_path}\" added required field \"{field_name}\"."
                        ),
                        create_location(&context.current_file, "message", message_path),
                        Some(create_location(
                            context.previous_file.as_deref().unwrap_or(""),
                            "message",
                            message_path,
                        )),
                        vec!["WIRE_JSON".to_string(), "WIRE".to_string()],
                    ));
                }
            }

            // Check for removed required fields (also breaking)
            for field_name in &prev_required {
                if !curr_required.contains(field_name) {
                    changes.push(create_breaking_change(
                        "MESSAGE_SAME_REQUIRED_FIELDS",
                        format!(
                            "Message \"{message_path}\" removed required field \"{field_name}\"."
                        ),
                        create_location(&context.current_file, "message", message_path),
                        Some(create_location(
                            context.previous_file.as_deref().unwrap_or(""),
                            "message",
                            message_path,
                        )),
                        vec!["WIRE_JSON".to_string(), "WIRE".to_string()],
                    ));
                }
            }
        }
    }

    RuleResult::with_changes(changes)
}

/// MESSAGE_NO_REMOVE_STANDARD_DESCRIPTOR_ACCESSOR rule
pub fn check_message_no_remove_standard_descriptor_accessor(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    let prev_messages = collect_all_messages(previous);
    let curr_messages = collect_all_messages(current);

    for (message_path, prev_message) in &prev_messages {
        if let Some(curr_message) = curr_messages.get(message_path) {
            let prev_no_accessor = prev_message
                .no_standard_descriptor_accessor
                .unwrap_or(false);
            let curr_no_accessor = curr_message
                .no_standard_descriptor_accessor
                .unwrap_or(false);

            // Breaking: changing from false to true (removing accessor)
            if !prev_no_accessor && curr_no_accessor {
                changes.push(create_breaking_change(
                    "MESSAGE_NO_REMOVE_STANDARD_DESCRIPTOR_ACCESSOR",
                    format!(
                        "Message \"{message_path}\" removed standard descriptor accessor by setting no_standard_descriptor_accessor to true."
                    ),
                    create_location(&context.current_file, "message", message_path),
                    Some(create_location(
                        context.previous_file.as_deref().unwrap_or(""),
                        "message",
                        message_path
                    )),
                    vec!["WIRE_JSON".to_string()],
                ));
            }
        }
    }

    RuleResult::with_changes(changes)
}

// ========================================
// Enum Rules
// ========================================

/// ENUM_SAME_TYPE rule (open vs closed enum)
pub fn check_enum_same_type(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    let prev_enums = collect_all_enums(previous);
    let curr_enums = collect_all_enums(current);

    for (enum_path, prev_enum) in &prev_enums {
        if let Some(curr_enum) = curr_enums.get(enum_path) {
            let prev_closed = prev_enum.closed_enum.unwrap_or(false);
            let curr_closed = curr_enum.closed_enum.unwrap_or(false);

            if prev_closed != curr_closed {
                let prev_type = if prev_closed { "closed" } else { "open" };
                let curr_type = if curr_closed { "closed" } else { "open" };

                changes.push(create_breaking_change(
                    "ENUM_SAME_TYPE",
                    format!(
                        "Enum \"{enum_path}\" type changed from \"{prev_type}\" to \"{curr_type}\"."
                    ),
                    create_location(&context.current_file, "enum", enum_path),
                    Some(create_location(
                        context.previous_file.as_deref().unwrap_or(""),
                        "enum",
                        enum_path,
                    )),
                    vec!["WIRE_JSON".to_string(), "WIRE".to_string()],
                ));
            }
        }
    }

    RuleResult::with_changes(changes)
}

/// ENUM_SAME_JSON_FORMAT rule
pub fn check_enum_same_json_format(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    let prev_enums = collect_all_enums(previous);
    let curr_enums = collect_all_enums(current);

    for (enum_path, prev_enum) in &prev_enums {
        if let Some(curr_enum) = curr_enums.get(enum_path) {
            // Check if json_format option changed
            let prev_json_format = prev_enum
                .options
                .get("json_format")
                .cloned()
                .unwrap_or_else(|| "ALLOW".to_string());
            let curr_json_format = curr_enum
                .options
                .get("json_format")
                .cloned()
                .unwrap_or_else(|| "ALLOW".to_string());

            if prev_json_format != curr_json_format {
                changes.push(create_breaking_change(
                    "ENUM_SAME_JSON_FORMAT",
                    format!(
                        "Enum \"{enum_path}\" json_format changed from \"{prev_json_format}\" to \"{curr_json_format}\"."
                    ),
                    create_location(&context.current_file, "enum", enum_path),
                    Some(create_location(
                        context.previous_file.as_deref().unwrap_or(""),
                        "enum",
                        enum_path,
                    )),
                    vec!["WIRE_JSON".to_string()],
                ));
            }

            // Also check if closed_enum setting changed (affects JSON representation)
            let prev_closed = prev_enum.closed_enum.unwrap_or(false);
            let curr_closed = curr_enum.closed_enum.unwrap_or(false);

            if prev_closed != curr_closed {
                changes.push(create_breaking_change(
                    "ENUM_SAME_JSON_FORMAT",
                    format!(
                        "Enum \"{enum_path}\" closed_enum changed from {prev_closed} to {curr_closed} (affects JSON format)."
                    ),
                    create_location(&context.current_file, "enum", enum_path),
                    Some(create_location(
                        context.previous_file.as_deref().unwrap_or(""),
                        "enum",
                        enum_path,
                    )),
                    vec!["WIRE_JSON".to_string()],
                ));
            }
        }
    }

    RuleResult::with_changes(changes)
}

/// ENUM_VALUE_SAME_NAME rule
pub fn check_enum_value_same_name(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    let prev_enums = collect_all_enums(previous);
    let curr_enums = collect_all_enums(current);

    for (enum_path, prev_enum) in &prev_enums {
        if let Some(curr_enum) = curr_enums.get(enum_path) {
            // Create number -> values mappings
            let prev_by_number = group_enum_values_by_number(&prev_enum.values);
            let curr_by_number = group_enum_values_by_number(&curr_enum.values);

            // Check if names changed for existing numbers
            for (number, prev_values) in &prev_by_number {
                if let Some(curr_values) = curr_by_number.get(number) {
                    let prev_names: BTreeSet<_> = prev_values.iter().map(|v| &v.name).collect();
                    let curr_names: BTreeSet<_> = curr_values.iter().map(|v| &v.name).collect();

                    if prev_names != curr_names {
                        changes.push(create_breaking_change(
                            "ENUM_VALUE_SAME_NAME",
                            format!(
                                "Enum \"{enum_path}\" value number {number} changed names from {prev_names:?} to {curr_names:?}."
                            ),
                            create_location(&context.current_file, "enum", enum_path),
                            Some(create_location(
                                context.previous_file.as_deref().unwrap_or(""),
                                "enum",
                                enum_path,
                            )),
                            vec!["WIRE_JSON".to_string()],
                        ));
                    }
                }
            }
        }
    }

    RuleResult::with_changes(changes)
}

// ========================================
// RPC Rules
// ========================================

/// RPC_SAME_IDEMPOTENCY_LEVEL rule
pub fn check_rpc_same_idempotency_level(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    let prev_services = collect_all_services(previous);
    let curr_services = collect_all_services(current);

    for (service_path, prev_service) in &prev_services {
        if let Some(curr_service) = curr_services.get(service_path) {
            // Check each method
            for prev_method in &prev_service.methods {
                if let Some(curr_method) = curr_service
                    .methods
                    .iter()
                    .find(|m| m.name == prev_method.name)
                {
                    let prev_level = prev_method
                        .idempotency_level
                        .as_deref()
                        .unwrap_or("UNKNOWN");
                    let curr_level = curr_method
                        .idempotency_level
                        .as_deref()
                        .unwrap_or("UNKNOWN");

                    if prev_level != curr_level {
                        let method_path = format!("{}.{}", service_path, prev_method.name);
                        changes.push(create_breaking_change(
                            "RPC_SAME_IDEMPOTENCY_LEVEL",
                            format!(
                                "RPC \"{method_path}\" idempotency level changed from \"{prev_level}\" to \"{curr_level}\"."
                            ),
                            create_location(
                                &context.current_file,
                                "rpc",
                                &method_path,
                            ),
                            Some(create_location(
                                context.previous_file.as_deref().unwrap_or(""),
                                "rpc",
                                &method_path,
                            )),
                            vec!["WIRE_JSON".to_string()],
                        ));
                    }
                }
            }
        }
    }

    RuleResult::with_changes(changes)
}

// ========================================
// Other Rules
// ========================================

/// ONEOF_NO_DELETE rule
pub fn check_oneof_no_delete(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    let prev_messages = collect_all_messages(previous);
    let curr_messages = collect_all_messages(current);

    for (message_path, prev_message) in &prev_messages {
        if let Some(curr_message) = curr_messages.get(message_path) {
            // Check for deleted oneofs
            for prev_oneof in &prev_message.oneofs {
                if !curr_message.oneofs.contains(prev_oneof) {
                    changes.push(create_breaking_change(
                        "ONEOF_NO_DELETE",
                        format!(
                            "Oneof \"{prev_oneof}\" was deleted from message \"{message_path}\"."
                        ),
                        create_location(&context.current_file, "message", message_path),
                        Some(create_location(
                            context.previous_file.as_deref().unwrap_or(""),
                            "message",
                            message_path,
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
// Helper Functions
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

fn collect_all_enums(file: &CanonicalFile) -> HashMap<String, &CanonicalEnum> {
    let mut all_enums = HashMap::new();

    // Top-level enums
    for enum_def in &file.enums {
        all_enums.insert(enum_def.name.clone(), enum_def);
    }

    // Nested enums in messages
    fn collect_from_messages<'a>(
        messages: &'a BTreeSet<CanonicalMessage>,
        prefix: &str,
        all_enums: &mut HashMap<String, &'a CanonicalEnum>,
    ) {
        for message in messages {
            let message_name = if prefix.is_empty() {
                message.name.clone()
            } else {
                format!("{}.{}", prefix, message.name)
            };

            for enum_def in &message.nested_enums {
                let enum_key = format!("{}.{}", message_name, enum_def.name);
                all_enums.insert(enum_key, enum_def);
            }

            collect_from_messages(&message.nested_messages, &message_name, all_enums);
        }
    }

    collect_from_messages(&file.messages, "", &mut all_enums);
    all_enums
}

fn collect_all_services(file: &CanonicalFile) -> HashMap<String, &CanonicalService> {
    let mut all_services = HashMap::new();
    for service in &file.services {
        all_services.insert(service.name.clone(), service);
    }
    all_services
}

fn get_required_fields(message: &CanonicalMessage) -> BTreeSet<String> {
    message
        .fields
        .iter()
        .filter(|field| field.label.as_deref() == Some("required"))
        .map(|field| field.name.clone())
        .collect()
}

fn group_enum_values_by_number(
    values: &BTreeSet<CanonicalEnumValue>,
) -> HashMap<i32, Vec<&CanonicalEnumValue>> {
    let mut by_number = HashMap::new();
    for value in values {
        by_number
            .entry(value.number)
            .or_insert_with(Vec::new)
            .push(value);
    }
    by_number
}

// ========================================
// Rule Export Table
// ========================================

/// COMMENT_ENUM rule - Buf specific comment handling  
pub fn check_comment_enum(
    _current: &CanonicalFile,
    _previous: &CanonicalFile,
    _context: &RuleContext,
) -> RuleResult {
    // This is a comment-related rule that Buf uses but isn't critical for breaking changes
    // For 1:1 compatibility, we implement as no-op since our model doesn't track comments
    RuleResult::success()
}

pub const OTHER_RULES: &[crate::compat::types::RuleEntry] = &[
    // Message rules
    ("MESSAGE_SAME_JSON_FORMAT", check_message_same_json_format),
    (
        "MESSAGE_SAME_REQUIRED_FIELDS",
        check_message_same_required_fields,
    ),
    (
        "MESSAGE_NO_REMOVE_STANDARD_DESCRIPTOR_ACCESSOR",
        check_message_no_remove_standard_descriptor_accessor,
    ),
    // Enum rules
    ("ENUM_SAME_TYPE", check_enum_same_type),
    ("ENUM_SAME_JSON_FORMAT", check_enum_same_json_format),
    ("ENUM_VALUE_SAME_NAME", check_enum_value_same_name),
    // RPC rules
    (
        "RPC_SAME_IDEMPOTENCY_LEVEL",
        check_rpc_same_idempotency_level,
    ),
    // Comment rules
    ("COMMENT_ENUM", check_comment_enum),
    // Other rules
    ("ONEOF_NO_DELETE", check_oneof_no_delete),
];
