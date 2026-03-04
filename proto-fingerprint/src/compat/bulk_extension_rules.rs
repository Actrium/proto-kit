//! Bulk-generated EXTENSION rules for protobuf extension breaking change detection
//!
//! These rules handle protobuf extensions which extend existing messages.

use crate::canonical::{CanonicalExtension, CanonicalFile, CanonicalMessage};
use crate::compat::handlers::{create_breaking_change, create_location};
use crate::compat::types::{RuleContext, RuleResult};
use std::collections::{BTreeSet, HashMap};

// ========================================
// EXTENSION Rules (Placeholder Implementation)
// ========================================

/// EXTENSION_NO_DELETE - checks extensions aren't deleted
pub fn check_extension_no_delete(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    // Create maps for efficient lookup by extension key (extendee + number)
    let prev_extensions: HashMap<String, &CanonicalExtension> = previous
        .extensions
        .iter()
        .map(|ext| (format!("{}.{}", ext.extendee, ext.number), ext))
        .collect();
    let curr_extensions: HashMap<String, &CanonicalExtension> = current
        .extensions
        .iter()
        .map(|ext| (format!("{}.{}", ext.extendee, ext.number), ext))
        .collect();

    // Find deleted extensions
    for (ext_key, prev_ext) in &prev_extensions {
        if !curr_extensions.contains_key(ext_key) {
            changes.push(create_breaking_change(
                "EXTENSION_NO_DELETE",
                format!(
                    "Extension \"{}\" with number {} extending \"{}\" was deleted.",
                    prev_ext.name, prev_ext.number, prev_ext.extendee
                ),
                create_location(&context.current_file, "file", &context.current_file),
                Some(create_location(
                    context.previous_file.as_deref().unwrap_or(""),
                    "extension",
                    &prev_ext.name,
                )),
                vec!["PACKAGE".to_string()],
            ));
        }
    }

    RuleResult::with_changes(changes)
}

/// EXTENSION_MESSAGE_NO_DELETE - checks extension ranges aren't deleted from messages  
pub fn check_extension_message_no_delete(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    let prev_messages = collect_all_messages(previous);
    let curr_messages = collect_all_messages(current);

    for (message_path, prev_message) in &prev_messages {
        if let Some(curr_message) = curr_messages.get(message_path) {
            // Check if any extension ranges were deleted
            for prev_range in &prev_message.extension_ranges {
                let range_exists = curr_message.extension_ranges.iter().any(|curr_range| {
                    curr_range.start <= prev_range.start && curr_range.end >= prev_range.end
                });

                if !range_exists {
                    changes.push(create_breaking_change(
                        "EXTENSION_MESSAGE_NO_DELETE",
                        format!(
                            "Extension range \"{}-{}\" was deleted from message \"{}\".",
                            prev_range.start, prev_range.end, message_path
                        ),
                        create_location(&context.current_file, "message", message_path),
                        Some(create_location(
                            context.previous_file.as_deref().unwrap_or(""),
                            "message",
                            message_path,
                        )),
                        vec!["PACKAGE".to_string()],
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

// ========================================
// Rule Export Table
// ========================================

pub const EXTENSION_RULES: &[crate::compat::types::RuleEntry] = &[
    ("EXTENSION_NO_DELETE", check_extension_no_delete),
    (
        "EXTENSION_MESSAGE_NO_DELETE",
        check_extension_message_no_delete,
    ),
];
