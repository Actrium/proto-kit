//! Bulk-generated ENUM rules for enum-level breaking change detection
//!
//! These rules handle enum definitions, values, and reserved ranges.

use crate::canonical::{CanonicalEnum, CanonicalEnumValue, CanonicalFile};
use crate::compat::handlers::{create_breaking_change, create_location};
use crate::compat::types::{RuleContext, RuleResult};
use std::collections::{BTreeSet, HashMap};

// ========================================
// ENUM Rules
// ========================================

/// ENUM_VALUE_NO_DELETE - checks enum values aren't deleted
pub fn check_enum_value_no_delete(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    let prev_enums = collect_all_enums(previous);
    let curr_enums = collect_all_enums(current);

    for (enum_path, prev_enum) in &prev_enums {
        if let Some(curr_enum) = curr_enums.get(enum_path) {
            // Create maps for efficient lookup by number
            let prev_values: HashMap<i32, &CanonicalEnumValue> =
                prev_enum.values.iter().map(|v| (v.number, v)).collect();
            let curr_values: HashMap<i32, &CanonicalEnumValue> =
                curr_enum.values.iter().map(|v| (v.number, v)).collect();

            // Find deleted values
            for (number, prev_value) in &prev_values {
                if !curr_values.contains_key(number) {
                    changes.push(create_breaking_change(
                        "ENUM_VALUE_NO_DELETE",
                        format!(
                            "Enum value \"{}\" with number {} was deleted from enum \"{}\".",
                            prev_value.name, number, enum_path
                        ),
                        create_location(&context.current_file, "enum", enum_path),
                        Some(create_location(
                            context.previous_file.as_deref().unwrap_or(""),
                            "enum_value",
                            &prev_value.name,
                        )),
                        vec!["ENUM_VALUE".to_string()],
                    ));
                }
            }
        }
    }

    RuleResult::with_changes(changes)
}

/// ENUM_NO_DELETE - checks enums aren't deleted
pub fn check_enum_no_delete(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    let prev_enums = collect_all_enums(previous);
    let curr_enums = collect_all_enums(current);

    for enum_path in prev_enums.keys() {
        if !curr_enums.contains_key(enum_path) {
            changes.push(create_breaking_change(
                "ENUM_NO_DELETE",
                format!("Enum \"{enum_path}\" was deleted."),
                create_location(&context.current_file, "enum", enum_path),
                Some(create_location(
                    context.previous_file.as_deref().unwrap_or(""),
                    "enum",
                    enum_path,
                )),
                vec!["FILE".to_string()],
            ));
        }
    }

    RuleResult::with_changes(changes)
}

/// ENUM_FIRST_VALUE_SAME - checks first enum value remains the same  
pub fn check_enum_first_value_same(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    let prev_enums = collect_all_enums(previous);
    let curr_enums = collect_all_enums(current);

    for (enum_path, prev_enum) in &prev_enums {
        if let Some(curr_enum) = curr_enums.get(enum_path) {
            // Get first values by number (not by definition order)
            let prev_first = prev_enum.values.iter().min_by_key(|v| v.number);
            let curr_first = curr_enum.values.iter().min_by_key(|v| v.number);

            match (prev_first, curr_first) {
                (Some(prev), Some(curr)) => {
                    if prev.number != curr.number || prev.name != curr.name {
                        changes.push(create_breaking_change(
                            "ENUM_FIRST_VALUE_SAME",
                            format!(
                                "First enum value changed from \"{}\" ({}) to \"{}\" ({}) in enum \"{}\".",
                                prev.name, prev.number, curr.name, curr.number, enum_path
                            ),
                            create_location(&context.current_file, "enum_value", &curr.name),
                            Some(create_location(
                                context.previous_file.as_deref().unwrap_or(""),
                                "enum_value",
                                &prev.name
                            )),
                            vec!["ENUM_VALUE".to_string()],
                        ));
                    }
                }
                (Some(prev), None) => {
                    changes.push(create_breaking_change(
                        "ENUM_FIRST_VALUE_SAME",
                        format!(
                            "Enum \"{}\" first value \"{}\" ({}) was deleted.",
                            enum_path, prev.name, prev.number
                        ),
                        create_location(&context.current_file, "enum", enum_path),
                        Some(create_location(
                            context.previous_file.as_deref().unwrap_or(""),
                            "enum_value",
                            &prev.name,
                        )),
                        vec!["ENUM_VALUE".to_string()],
                    ));
                }
                _ => {} // If previous had no values, no constraint
            }
        }
    }

    RuleResult::with_changes(changes)
}

/// ENUM_VALUE_SAME_NUMBER - checks enum value numbers don't change
pub fn check_enum_value_same_number(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    let prev_enums = collect_all_enums(previous);
    let curr_enums = collect_all_enums(current);

    for (enum_path, prev_enum) in &prev_enums {
        if let Some(curr_enum) = curr_enums.get(enum_path) {
            // Create maps by name for efficient lookup
            let prev_values: HashMap<String, &CanonicalEnumValue> = prev_enum
                .values
                .iter()
                .map(|v| (v.name.clone(), v))
                .collect();
            let curr_values: HashMap<String, &CanonicalEnumValue> = curr_enum
                .values
                .iter()
                .map(|v| (v.name.clone(), v))
                .collect();

            // Find values with changed numbers
            for (name, prev_value) in &prev_values {
                if let Some(curr_value) = curr_values.get(name) {
                    if prev_value.number != curr_value.number {
                        changes.push(create_breaking_change(
                            "ENUM_VALUE_SAME_NUMBER",
                            format!(
                                "Enum value \"{}\" number changed from {} to {} in enum \"{}\".",
                                name, prev_value.number, curr_value.number, enum_path
                            ),
                            create_location(&context.current_file, "enum_value", name),
                            Some(create_location(
                                context.previous_file.as_deref().unwrap_or(""),
                                "enum_value",
                                name,
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

/// ENUM_ZERO_VALUE_SAME - checks enum zero value remains the same
pub fn check_enum_zero_value_same(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    let prev_enums = collect_all_enums(previous);
    let curr_enums = collect_all_enums(current);

    for (enum_path, prev_enum) in &prev_enums {
        if let Some(curr_enum) = curr_enums.get(enum_path) {
            // Find zero values
            let prev_zero = prev_enum.values.iter().find(|v| v.number == 0);
            let curr_zero = curr_enum.values.iter().find(|v| v.number == 0);

            match (prev_zero, curr_zero) {
                (Some(prev), Some(curr)) => {
                    if prev.name != curr.name {
                        changes.push(create_breaking_change(
                            "ENUM_ZERO_VALUE_SAME",
                            format!(
                                "Enum zero value name changed from \"{}\" to \"{}\" in enum \"{}\".",
                                prev.name, curr.name, enum_path
                            ),
                            create_location(&context.current_file, "enum_value", &curr.name),
                            Some(create_location(
                                context.previous_file.as_deref().unwrap_or(""),
                                "enum_value",
                                &prev.name
                            )),
                            vec!["ENUM_VALUE".to_string()],
                        ));
                    }
                }
                (Some(prev), None) => {
                    changes.push(create_breaking_change(
                        "ENUM_ZERO_VALUE_SAME",
                        format!(
                            "Enum \"{}\" zero value \"{}\" was deleted.",
                            enum_path, prev.name
                        ),
                        create_location(&context.current_file, "enum", enum_path),
                        Some(create_location(
                            context.previous_file.as_deref().unwrap_or(""),
                            "enum_value",
                            &prev.name,
                        )),
                        vec!["ENUM_VALUE".to_string()],
                    ));
                }
                (None, Some(curr)) => {
                    changes.push(create_breaking_change(
                        "ENUM_ZERO_VALUE_SAME",
                        format!(
                            "Enum \"{}\" added new zero value \"{}\" where none existed before.",
                            enum_path, curr.name
                        ),
                        create_location(&context.current_file, "enum_value", &curr.name),
                        None,
                        vec!["ENUM_VALUE".to_string()],
                    ));
                }
                _ => {} // Both had no zero value - no constraint
            }
        }
    }

    RuleResult::with_changes(changes)
}

// ========================================
// Helper Functions
// ========================================

fn collect_all_enums(file: &CanonicalFile) -> HashMap<String, &CanonicalEnum> {
    let mut all_enums = HashMap::new();

    // Top-level enums
    for enum_def in &file.enums {
        all_enums.insert(enum_def.name.clone(), enum_def);
    }

    // Nested enums in messages
    fn collect_from_messages<'a>(
        messages: &'a BTreeSet<crate::canonical::CanonicalMessage>,
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

pub const ENUM_RULES: &[crate::compat::types::RuleEntry] = &[
    ("ENUM_VALUE_NO_DELETE", check_enum_value_no_delete),
    ("ENUM_NO_DELETE", check_enum_no_delete),
    ("ENUM_FIRST_VALUE_SAME", check_enum_first_value_same),
    ("ENUM_VALUE_SAME_NUMBER", check_enum_value_same_number),
    ("ENUM_ZERO_VALUE_SAME", check_enum_zero_value_same),
];
