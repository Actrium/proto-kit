//! Bulk-generated SPECIAL rules for unique breaking change scenarios
//!
//! These rules handle special cases and advanced scenarios not covered by other rule categories.

use crate::canonical::{CanonicalEnum, CanonicalFile, CanonicalMessage};
use crate::compat::handlers::{create_breaking_change, create_location};
use crate::compat::types::{RuleContext, RuleResult};
use std::collections::{BTreeSet, HashMap};

// ========================================
// SPECIAL Rules
// ========================================

/// SYNTAX_SAME - checks file syntax doesn't change (proto2 vs proto3)
pub fn check_syntax_same(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    if current.syntax != previous.syntax {
        let prev_syntax = &previous.syntax;
        let curr_syntax = &current.syntax;

        RuleResult::with_changes(vec![create_breaking_change(
            "SYNTAX_SAME",
            format!("File syntax changed from \"{prev_syntax}\" to \"{curr_syntax}\"."),
            create_location(&context.current_file, "file", &context.current_file),
            Some(create_location(
                context.previous_file.as_deref().unwrap_or(""),
                "file",
                context.previous_file.as_deref().unwrap_or(""),
            )),
            vec!["FILE".to_string()],
        )])
    } else {
        RuleResult::success()
    }
}

/// IMPORT_NO_CYCLE - checks for circular import dependencies
pub fn check_import_no_cycle(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    // Compare import lists between previous and current
    let prev_imports: std::collections::HashSet<_> = previous.imports.iter().collect();
    let curr_imports: std::collections::HashSet<_> = current.imports.iter().collect();

    // Check for new imports that might create cycles
    for new_import in curr_imports.difference(&prev_imports) {
        // Simple cycle detection: if we're importing something that imports us
        // This is a simplified check - real cycle detection would require full dependency graph
        if new_import.contains(&context.current_file) {
            changes.push(create_breaking_change(
                "IMPORT_NO_CYCLE",
                format!("Adding import \"{new_import}\" might create a circular dependency."),
                create_location(&context.current_file, "file", &context.current_file),
                None,
                vec!["IMPORT".to_string()],
            ));
        }
    }

    RuleResult::with_changes(changes)
}

/// FIELD_NAME_SAME_CASE - checks field name case conventions don't change
pub fn check_field_name_same_case(
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
                if let Some(curr_field) = curr_fields.get(number) {
                    // Check if field name changed case style
                    if prev_field.name != curr_field.name {
                        let prev_snake_case = is_snake_case(&prev_field.name);
                        let curr_snake_case = is_snake_case(&curr_field.name);

                        if prev_snake_case != curr_snake_case {
                            changes.push(create_breaking_change(
                                "FIELD_NAME_SAME_CASE",
                                format!(
                                    "Field name \"{}\" changed case style to \"{}\" in message \"{}\".",
                                    prev_field.name, curr_field.name, message_path
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
    }

    RuleResult::with_changes(changes)
}

/// ENUM_ALLOW_ALIAS_SAME - checks enum allow_alias setting doesn't change
pub fn check_enum_allow_alias_same(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    let prev_enums = collect_all_enums(previous);
    let curr_enums = collect_all_enums(current);

    for (enum_path, prev_enum) in &prev_enums {
        if let Some(curr_enum) = curr_enums.get(enum_path) {
            if prev_enum.allow_alias != curr_enum.allow_alias {
                changes.push(create_breaking_change(
                    "ENUM_ALLOW_ALIAS_SAME",
                    format!(
                        "Enum \"{}\" allow_alias setting changed from {:?} to {:?}.",
                        enum_path, prev_enum.allow_alias, curr_enum.allow_alias
                    ),
                    create_location(&context.current_file, "enum", enum_path),
                    Some(create_location(
                        context.previous_file.as_deref().unwrap_or(""),
                        "enum",
                        enum_path,
                    )),
                    vec!["ENUM".to_string()],
                ));
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

fn is_snake_case(name: &str) -> bool {
    name.chars()
        .all(|c| c.is_lowercase() || c.is_numeric() || c == '_')
        && !name.starts_with('_')
        && !name.ends_with('_')
}

// ========================================
// Rule Export Table
// ========================================

pub const SPECIAL_RULES: &[crate::compat::types::RuleEntry] = &[
    ("SYNTAX_SAME", check_syntax_same),
    ("IMPORT_NO_CYCLE", check_import_no_cycle),
    ("FIELD_NAME_SAME_CASE", check_field_name_same_case),
    ("ENUM_ALLOW_ALIAS_SAME", check_enum_allow_alias_same),
];
