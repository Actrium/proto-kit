//! Bulk-generated RESERVED rules for reserved field/range protection
//!
//! These rules ensure that reserved fields, ranges, and names cannot be violated.

use crate::canonical::{CanonicalEnum, CanonicalFile, CanonicalMessage};
use crate::compat::handlers::{create_breaking_change, create_location};
use crate::compat::types::{RuleContext, RuleResult};
use std::collections::{BTreeSet, HashMap};

// ========================================
// RESERVED Rules
// ========================================

/// RESERVED_ENUM_NO_DELETE - checks reserved enum ranges aren't deleted
pub fn check_reserved_enum_no_delete(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    let prev_enums = collect_all_enums(previous);
    let curr_enums = collect_all_enums(current);

    for (enum_path, prev_enum) in &prev_enums {
        if let Some(curr_enum) = curr_enums.get(enum_path) {
            // Check reserved ranges
            for prev_range in &prev_enum.reserved_ranges {
                if !curr_enum.reserved_ranges.contains(prev_range) {
                    changes.push(create_breaking_change(
                        "RESERVED_ENUM_NO_DELETE",
                        format!(
                            "Reserved range {}-{} was deleted from enum \"{}\".",
                            prev_range.start, prev_range.end, enum_path
                        ),
                        create_location(&context.current_file, "enum", enum_path),
                        Some(create_location(
                            context.previous_file.as_deref().unwrap_or(""),
                            "enum",
                            enum_path,
                        )),
                        vec!["RESERVED".to_string()],
                    ));
                }
            }

            // Check reserved names
            for prev_name in &prev_enum.reserved_names {
                if !curr_enum.reserved_names.contains(prev_name) {
                    changes.push(create_breaking_change(
                        "RESERVED_ENUM_NO_DELETE",
                        format!(
                            "Reserved name \"{}\" was deleted from enum \"{}\".",
                            prev_name.name, enum_path
                        ),
                        create_location(&context.current_file, "enum", enum_path),
                        Some(create_location(
                            context.previous_file.as_deref().unwrap_or(""),
                            "enum",
                            enum_path,
                        )),
                        vec!["RESERVED".to_string()],
                    ));
                }
            }
        }
    }

    RuleResult::with_changes(changes)
}

/// RESERVED_MESSAGE_NO_DELETE - checks reserved message ranges aren't deleted
pub fn check_reserved_message_no_delete(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    let prev_messages = collect_all_messages(previous);
    let curr_messages = collect_all_messages(current);

    for (message_path, prev_message) in &prev_messages {
        if let Some(curr_message) = curr_messages.get(message_path) {
            // Check reserved ranges
            for prev_range in &prev_message.reserved_ranges {
                if !curr_message.reserved_ranges.contains(prev_range) {
                    changes.push(create_breaking_change(
                        "RESERVED_MESSAGE_NO_DELETE",
                        format!(
                            "Reserved range {}-{} was deleted from message \"{}\".",
                            prev_range.start, prev_range.end, message_path
                        ),
                        create_location(&context.current_file, "message", message_path),
                        Some(create_location(
                            context.previous_file.as_deref().unwrap_or(""),
                            "message",
                            message_path,
                        )),
                        vec!["RESERVED".to_string()],
                    ));
                }
            }

            // Check reserved names
            for prev_name in &prev_message.reserved_names {
                if !curr_message.reserved_names.contains(prev_name) {
                    changes.push(create_breaking_change(
                        "RESERVED_MESSAGE_NO_DELETE",
                        format!(
                            "Reserved name \"{}\" was deleted from message \"{}\".",
                            prev_name.name, message_path
                        ),
                        create_location(&context.current_file, "message", message_path),
                        Some(create_location(
                            context.previous_file.as_deref().unwrap_or(""),
                            "message",
                            message_path,
                        )),
                        vec!["RESERVED".to_string()],
                    ));
                }
            }
        }
    }

    RuleResult::with_changes(changes)
}

/// FIELD_NO_DELETE_UNLESS_NAME_RESERVED - allows field deletion if name becomes reserved
pub fn check_field_no_delete_unless_name_reserved(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    let prev_messages = collect_all_messages(previous);
    let curr_messages = collect_all_messages(current);

    for (message_path, prev_message) in &prev_messages {
        if let Some(curr_message) = curr_messages.get(message_path) {
            let prev_fields: HashMap<i32, _> =
                prev_message.fields.iter().map(|f| (f.number, f)).collect();
            let curr_fields: HashMap<i32, _> =
                curr_message.fields.iter().map(|f| (f.number, f)).collect();

            for (number, prev_field) in &prev_fields {
                if !curr_fields.contains_key(number) {
                    // Field was deleted - check if name is now reserved
                    let reserved_name = crate::canonical::ReservedName {
                        name: prev_field.name.clone(),
                    };
                    if !curr_message.reserved_names.contains(&reserved_name) {
                        changes.push(create_breaking_change(
                            "FIELD_NO_DELETE_UNLESS_NAME_RESERVED",
                            format!(
                                "Field \"{}\" with number {} was deleted from message \"{}\", but the name is not reserved.",
                                prev_field.name, number, message_path
                            ),
                            create_location(&context.current_file, "message", message_path),
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

/// FIELD_NO_DELETE_UNLESS_NUMBER_RESERVED - allows field deletion if number becomes reserved  
pub fn check_field_no_delete_unless_number_reserved(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    let prev_messages = collect_all_messages(previous);
    let curr_messages = collect_all_messages(current);

    for (message_path, prev_message) in &prev_messages {
        if let Some(curr_message) = curr_messages.get(message_path) {
            let prev_fields: HashMap<i32, _> =
                prev_message.fields.iter().map(|f| (f.number, f)).collect();
            let curr_fields: HashMap<i32, _> =
                curr_message.fields.iter().map(|f| (f.number, f)).collect();

            for (number, prev_field) in &prev_fields {
                if !curr_fields.contains_key(number) {
                    // Field was deleted - check if number is now reserved
                    let number_reserved = curr_message
                        .reserved_ranges
                        .iter()
                        .any(|range| *number >= range.start && *number <= range.end);

                    if !number_reserved {
                        changes.push(create_breaking_change(
                            "FIELD_NO_DELETE_UNLESS_NUMBER_RESERVED",
                            format!(
                                "Field \"{}\" with number {} was deleted from message \"{}\", but the number is not reserved.",
                                prev_field.name, number, message_path
                            ),
                            create_location(&context.current_file, "message", message_path),
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

/// ENUM_VALUE_NO_DELETE_UNLESS_NAME_RESERVED - allows enum value deletion if name becomes reserved
pub fn check_enum_value_no_delete_unless_name_reserved(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    let prev_enums = collect_all_enums(previous);
    let curr_enums = collect_all_enums(current);

    for (enum_path, prev_enum) in &prev_enums {
        if let Some(curr_enum) = curr_enums.get(enum_path) {
            let prev_values: HashMap<i32, _> =
                prev_enum.values.iter().map(|v| (v.number, v)).collect();
            let curr_values: HashMap<i32, _> =
                curr_enum.values.iter().map(|v| (v.number, v)).collect();

            for (number, prev_value) in &prev_values {
                if !curr_values.contains_key(number) {
                    // Enum value was deleted - check if name is now reserved
                    let reserved_name = crate::canonical::ReservedName {
                        name: prev_value.name.clone(),
                    };
                    if !curr_enum.reserved_names.contains(&reserved_name) {
                        changes.push(create_breaking_change(
                            "ENUM_VALUE_NO_DELETE_UNLESS_NAME_RESERVED",
                            format!(
                                "Enum value \"{}\" with number {} was deleted from enum \"{}\", but the name is not reserved.",
                                prev_value.name, number, enum_path
                            ),
                            create_location(&context.current_file, "enum", enum_path),
                            Some(create_location(
                                context.previous_file.as_deref().unwrap_or(""),
                                "enum_value",
                                &prev_value.name
                            )),
                            vec!["ENUM_VALUE".to_string()],
                        ));
                    }
                }
            }
        }
    }

    RuleResult::with_changes(changes)
}

/// ENUM_VALUE_NO_DELETE_UNLESS_NUMBER_RESERVED - allows enum value deletion if number becomes reserved
pub fn check_enum_value_no_delete_unless_number_reserved(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    let prev_enums = collect_all_enums(previous);
    let curr_enums = collect_all_enums(current);

    for (enum_path, prev_enum) in &prev_enums {
        if let Some(curr_enum) = curr_enums.get(enum_path) {
            let prev_values: HashMap<i32, _> =
                prev_enum.values.iter().map(|v| (v.number, v)).collect();
            let curr_values: HashMap<i32, _> =
                curr_enum.values.iter().map(|v| (v.number, v)).collect();

            for (number, prev_value) in &prev_values {
                if !curr_values.contains_key(number) {
                    // Enum value was deleted - check if number is now reserved
                    let number_reserved = curr_enum
                        .reserved_ranges
                        .iter()
                        .any(|range| *number >= range.start && *number <= range.end);

                    if !number_reserved {
                        changes.push(create_breaking_change(
                            "ENUM_VALUE_NO_DELETE_UNLESS_NUMBER_RESERVED",
                            format!(
                                "Enum value \"{}\" with number {} was deleted from enum \"{}\", but the number is not reserved.",
                                prev_value.name, number, enum_path
                            ),
                            create_location(&context.current_file, "enum", enum_path),
                            Some(create_location(
                                context.previous_file.as_deref().unwrap_or(""),
                                "enum_value",
                                &prev_value.name
                            )),
                            vec!["ENUM_VALUE".to_string()],
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

// ========================================
// Rule Export Table
// ========================================

pub const RESERVED_RULES: &[crate::compat::types::RuleEntry] = &[
    ("RESERVED_ENUM_NO_DELETE", check_reserved_enum_no_delete),
    (
        "RESERVED_MESSAGE_NO_DELETE",
        check_reserved_message_no_delete,
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
        "ENUM_VALUE_NO_DELETE_UNLESS_NAME_RESERVED",
        check_enum_value_no_delete_unless_name_reserved,
    ),
    (
        "ENUM_VALUE_NO_DELETE_UNLESS_NUMBER_RESERVED",
        check_enum_value_no_delete_unless_number_reserved,
    ),
];
