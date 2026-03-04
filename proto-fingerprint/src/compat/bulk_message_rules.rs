//! Bulk-generated MESSAGE rules for message-level breaking change detection
//!
//! These rules handle message definitions, fields, oneofs, and reserved ranges.

use crate::canonical::{CanonicalField, CanonicalFile, CanonicalMessage};
use crate::compat::handlers::{create_breaking_change, create_location};
use crate::compat::types::{RuleContext, RuleResult};
use std::collections::{BTreeSet, HashMap};

// ========================================
// MESSAGE Rules
// ========================================

/// MESSAGE_NO_DELETE - checks messages aren't deleted
pub fn check_message_no_delete(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    let prev_messages = collect_all_messages(previous);
    let curr_messages = collect_all_messages(current);

    for message_path in prev_messages.keys() {
        if !curr_messages.contains_key(message_path) {
            changes.push(create_breaking_change(
                "MESSAGE_NO_DELETE",
                format!("Message \"{message_path}\" was deleted."),
                create_location(&context.current_file, "file", &context.current_file),
                Some(create_location(
                    context.previous_file.as_deref().unwrap_or(""),
                    "message",
                    message_path,
                )),
                vec!["FILE".to_string()],
            ));
        }
    }

    RuleResult::with_changes(changes)
}

/// MESSAGE_NO_REMOVE_STANDARD_DESCRIPTOR_ACCESSOR - checks standard descriptor accessor isn't removed
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
            // Check if no_standard_descriptor_accessor was added (went from false to true)
            if !prev_message
                .no_standard_descriptor_accessor
                .unwrap_or(false)
                && curr_message
                    .no_standard_descriptor_accessor
                    .unwrap_or(false)
            {
                changes.push(create_breaking_change(
                    "MESSAGE_NO_REMOVE_STANDARD_DESCRIPTOR_ACCESSOR",
                    format!(
                        "Message \"{message_path}\" removed standard descriptor accessor (no_standard_descriptor_accessor was set)."
                    ),
                    create_location(&context.current_file, "message", message_path),
                    Some(create_location(
                        context.previous_file.as_deref().unwrap_or(""),
                        "message",
                        message_path
                    )),
                    vec!["FILE".to_string()],
                ));
            }
        }
    }

    RuleResult::with_changes(changes)
}

/// MESSAGE_SAME_MESSAGE_SET_WIRE_FORMAT - checks MessageSet wire format doesn't change
pub fn check_message_same_message_set_wire_format(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    let prev_messages = collect_all_messages(previous);
    let curr_messages = collect_all_messages(current);

    for (message_path, prev_message) in &prev_messages {
        if let Some(curr_message) = curr_messages.get(message_path) {
            if prev_message.message_set_wire_format != curr_message.message_set_wire_format {
                changes.push(create_breaking_change(
                    "MESSAGE_SAME_MESSAGE_SET_WIRE_FORMAT",
                    format!(
                        "Message \"{}\" MessageSet wire format changed from {:?} to {:?}.",
                        message_path,
                        prev_message.message_set_wire_format,
                        curr_message.message_set_wire_format
                    ),
                    create_location(&context.current_file, "message", message_path),
                    Some(create_location(
                        context.previous_file.as_deref().unwrap_or(""),
                        "message",
                        message_path,
                    )),
                    vec!["FILE".to_string()],
                ));
            }
        }
    }

    RuleResult::with_changes(changes)
}

/// ONEOF_NO_DELETE - checks oneof groups aren't deleted
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
            let prev_oneofs: std::collections::HashSet<_> = prev_message.oneofs.iter().collect();
            let curr_oneofs: std::collections::HashSet<_> = curr_message.oneofs.iter().collect();

            for prev_oneof in &prev_oneofs {
                if !curr_oneofs.contains(prev_oneof) {
                    changes.push(create_breaking_change(
                        "ONEOF_NO_DELETE",
                        format!(
                            "Oneof \"{prev_oneof}\" was deleted from message \"{message_path}\"."
                        ),
                        create_location(&context.current_file, "message", message_path),
                        Some(create_location(
                            context.previous_file.as_deref().unwrap_or(""),
                            "oneof",
                            prev_oneof,
                        )),
                        vec!["ONEOF".to_string()],
                    ));
                }
            }
        }
    }

    RuleResult::with_changes(changes)
}

/// FIELD_NO_DELETE - checks fields aren't deleted from messages
pub fn check_field_no_delete(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    let prev_messages = collect_all_messages(previous);
    let curr_messages = collect_all_messages(current);

    for (message_path, prev_message) in &prev_messages {
        if let Some(curr_message) = curr_messages.get(message_path) {
            // Create maps for efficient lookup by field number
            let prev_fields: HashMap<i32, &CanonicalField> =
                prev_message.fields.iter().map(|f| (f.number, f)).collect();
            let curr_fields: HashMap<i32, &CanonicalField> =
                curr_message.fields.iter().map(|f| (f.number, f)).collect();

            // Find deleted fields
            for (number, prev_field) in &prev_fields {
                if !curr_fields.contains_key(number) {
                    changes.push(create_breaking_change(
                        "FIELD_NO_DELETE",
                        format!(
                            "Field \"{}\" with number {} was deleted from message \"{}\".",
                            prev_field.name, number, message_path
                        ),
                        create_location(&context.current_file, "message", message_path),
                        Some(create_location(
                            context.previous_file.as_deref().unwrap_or(""),
                            "field",
                            &prev_field.name,
                        )),
                        vec!["FIELD".to_string()],
                    ));
                }
            }
        }
    }

    RuleResult::with_changes(changes)
}

/// FIELD_SAME_NAME - checks field names don't change
pub fn check_field_same_name(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    let prev_messages = collect_all_messages(previous);
    let curr_messages = collect_all_messages(current);

    for (message_path, prev_message) in &prev_messages {
        if let Some(curr_message) = curr_messages.get(message_path) {
            // Create maps for efficient lookup by field number
            let prev_fields: HashMap<i32, &CanonicalField> =
                prev_message.fields.iter().map(|f| (f.number, f)).collect();
            let curr_fields: HashMap<i32, &CanonicalField> =
                curr_message.fields.iter().map(|f| (f.number, f)).collect();

            // Find fields with changed names
            for (number, prev_field) in &prev_fields {
                if let Some(curr_field) = curr_fields.get(number) {
                    if prev_field.name != curr_field.name {
                        changes.push(create_breaking_change(
                            "FIELD_SAME_NAME",
                            format!(
                                "Field {} name changed from \"{}\" to \"{}\" in message \"{}\".",
                                number, prev_field.name, curr_field.name, message_path
                            ),
                            create_location(&context.current_file, "field", &curr_field.name),
                            Some(create_location(
                                context.previous_file.as_deref().unwrap_or(""),
                                "field",
                                &prev_field.name,
                            )),
                            vec!["FIELD".to_string()],
                        ));
                    }
                }
            }
        }
    }

    RuleResult::with_changes(changes)
}

/// FIELD_SAME_TYPE - checks field types don't change
pub fn check_field_same_type(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    let prev_messages = collect_all_messages(previous);
    let curr_messages = collect_all_messages(current);

    for (message_path, prev_message) in &prev_messages {
        if let Some(curr_message) = curr_messages.get(message_path) {
            // Create maps for efficient lookup by field number
            let prev_fields: HashMap<i32, &CanonicalField> =
                prev_message.fields.iter().map(|f| (f.number, f)).collect();
            let curr_fields: HashMap<i32, &CanonicalField> =
                curr_message.fields.iter().map(|f| (f.number, f)).collect();

            // Find fields with changed types
            for (number, prev_field) in &prev_fields {
                if let Some(curr_field) = curr_fields.get(number) {
                    if prev_field.type_name != curr_field.type_name {
                        changes.push(create_breaking_change(
                            "FIELD_SAME_TYPE",
                            format!(
                                "Field \"{}\" type changed from \"{}\" to \"{}\" in message \"{}\".",
                                prev_field.name, prev_field.type_name, curr_field.type_name, message_path
                            ),
                            create_location(&context.current_file, "field", &curr_field.name),
                            Some(create_location(
                                context.previous_file.as_deref().unwrap_or(""),
                                "field",
                                &prev_field.name
                            )),
                            vec!["FIELD".to_string()],
                        ));
                    }
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

// ========================================
// Rule Export Table
// ========================================

pub const MESSAGE_RULES: &[crate::compat::types::RuleEntry] = &[
    ("MESSAGE_NO_DELETE", check_message_no_delete),
    (
        "MESSAGE_NO_REMOVE_STANDARD_DESCRIPTOR_ACCESSOR",
        check_message_no_remove_standard_descriptor_accessor,
    ),
    (
        "MESSAGE_SAME_MESSAGE_SET_WIRE_FORMAT",
        check_message_same_message_set_wire_format,
    ),
    ("ONEOF_NO_DELETE", check_oneof_no_delete),
    ("FIELD_NO_DELETE", check_field_no_delete),
    ("FIELD_SAME_NAME", check_field_same_name),
    ("FIELD_SAME_TYPE", check_field_same_type),
];
