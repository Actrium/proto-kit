//! Bulk-generated PACKAGE rules for package-level breaking change detection
//!
//! These rules check for deletions at the package level across file sets.
//! Note: These rules require file-set analysis, not single-file comparison.

use crate::canonical::{CanonicalEnum, CanonicalFile, CanonicalMessage, CanonicalService};
use crate::compat::handlers::{create_breaking_change, create_location};
use crate::compat::types::{RuleContext, RuleResult};
use std::collections::{BTreeSet, HashMap};

// ========================================
// PACKAGE_* Rules - File-Set Level Analysis
// ========================================

/// PACKAGE_NO_DELETE - checks entire packages aren't deleted
/// Note: Single-file implementation - detects package name changes that may indicate deletion
pub fn check_package_no_delete(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    let prev_package = previous.package.as_deref().unwrap_or("");
    let curr_package = current.package.as_deref().unwrap_or("");

    // If package name changed and previous package is not empty,
    // this could indicate package deletion (though could also be renaming)
    if !prev_package.is_empty() && prev_package != curr_package {
        // Special case: if current package is empty, this is likely package deletion
        if curr_package.is_empty() {
            changes.push(create_breaking_change(
                "PACKAGE_NO_DELETE",
                format!(
                    "Package \"{prev_package}\" was deleted (file no longer declares package)."
                ),
                create_location(&context.current_file, "file", &context.current_file),
                Some(create_location(
                    context.previous_file.as_deref().unwrap_or(""),
                    "package",
                    prev_package,
                )),
                vec!["PACKAGE".to_string()],
            ));
        }
        // Note: Package renaming is also a breaking change but harder to distinguish
        // from deletion in single-file context. Full implementation requires multi-file analysis.
    }

    RuleResult::with_changes(changes)
}

/// PACKAGE_ENUM_NO_DELETE - checks enums aren't deleted from package
pub fn check_package_enum_no_delete(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    // Get package names
    let prev_package = previous.package.as_deref().unwrap_or("");
    let curr_package = current.package.as_deref().unwrap_or("");

    // Only check if same package
    if prev_package == curr_package && !prev_package.is_empty() {
        let prev_enums = collect_all_enums(previous);
        let curr_enums = collect_all_enums(current);

        // Find deleted enums
        for enum_path in prev_enums.keys() {
            if !curr_enums.contains_key(enum_path) {
                changes.push(create_breaking_change(
                    "PACKAGE_ENUM_NO_DELETE",
                    format!("Enum \"{enum_path}\" was deleted from package \"{prev_package}\"."),
                    create_location(&context.current_file, "enum", enum_path),
                    Some(create_location(
                        context.previous_file.as_deref().unwrap_or(""),
                        "enum",
                        enum_path,
                    )),
                    vec!["PACKAGE".to_string()],
                ));
            }
        }
    }

    RuleResult::with_changes(changes)
}

/// PACKAGE_MESSAGE_NO_DELETE - checks messages aren't deleted from package  
pub fn check_package_message_no_delete(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    let prev_package = previous.package.as_deref().unwrap_or("");
    let curr_package = current.package.as_deref().unwrap_or("");

    if prev_package == curr_package && !prev_package.is_empty() {
        let prev_messages = collect_all_messages(previous);
        let curr_messages = collect_all_messages(current);

        for message_path in prev_messages.keys() {
            if !curr_messages.contains_key(message_path) {
                changes.push(create_breaking_change(
                    "PACKAGE_MESSAGE_NO_DELETE",
                    format!(
                        "Message \"{message_path}\" was deleted from package \"{prev_package}\"."
                    ),
                    create_location(&context.current_file, "package", prev_package),
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

    RuleResult::with_changes(changes)
}

/// PACKAGE_SERVICE_NO_DELETE - checks services aren't deleted from package
pub fn check_package_service_no_delete(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    let prev_package = previous.package.as_deref().unwrap_or("");
    let curr_package = current.package.as_deref().unwrap_or("");

    if prev_package == curr_package && !prev_package.is_empty() {
        let prev_services = collect_all_services(previous);
        let curr_services = collect_all_services(current);

        for service_name in prev_services.keys() {
            if !curr_services.contains_key(service_name) {
                changes.push(create_breaking_change(
                    "PACKAGE_SERVICE_NO_DELETE",
                    format!(
                        "Service \"{service_name}\" was deleted from package \"{prev_package}\"."
                    ),
                    create_location(&context.current_file, "package", prev_package),
                    Some(create_location(
                        context.previous_file.as_deref().unwrap_or(""),
                        "service",
                        service_name,
                    )),
                    vec!["PACKAGE".to_string()],
                ));
            }
        }
    }

    RuleResult::with_changes(changes)
}

/// PACKAGE_EXTENSION_NO_DELETE - checks extensions aren't deleted from package
pub fn check_package_extension_no_delete(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    // Get package names
    let prev_package = previous.package.as_deref().unwrap_or("");
    let curr_package = current.package.as_deref().unwrap_or("");

    // Only check if same package
    if prev_package == curr_package && !prev_package.is_empty() {
        // Create maps for efficient lookup by extension key (extendee + number)
        let prev_extensions: HashMap<String, &crate::canonical::CanonicalExtension> = previous
            .extensions
            .iter()
            .map(|ext| (format!("{}.{}", ext.extendee, ext.number), ext))
            .collect();
        let curr_extensions: HashMap<String, &crate::canonical::CanonicalExtension> = current
            .extensions
            .iter()
            .map(|ext| (format!("{}.{}", ext.extendee, ext.number), ext))
            .collect();

        // Find deleted extensions
        for (ext_key, prev_ext) in &prev_extensions {
            if !curr_extensions.contains_key(ext_key) {
                changes.push(create_breaking_change(
                    "PACKAGE_EXTENSION_NO_DELETE",
                    format!(
                        "Extension \"{}\" with number {} extending \"{}\" was deleted from package \"{}\".",
                        prev_ext.name, prev_ext.number, prev_ext.extendee, prev_package
                    ),
                    create_location(&context.current_file, "package", prev_package),
                    Some(create_location(
                        context.previous_file.as_deref().unwrap_or(""),
                        "extension",
                        &prev_ext.name
                    )),
                    vec!["PACKAGE".to_string()],
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

fn collect_all_services(file: &CanonicalFile) -> HashMap<String, &CanonicalService> {
    let mut all_services = HashMap::new();
    for service in &file.services {
        all_services.insert(service.name.clone(), service);
    }
    all_services
}

// ========================================
// Rule Export Table
// ========================================

pub const PACKAGE_RULES: &[crate::compat::types::RuleEntry] = &[
    ("PACKAGE_NO_DELETE", check_package_no_delete),
    ("PACKAGE_ENUM_NO_DELETE", check_package_enum_no_delete),
    ("PACKAGE_MESSAGE_NO_DELETE", check_package_message_no_delete),
    ("PACKAGE_SERVICE_NO_DELETE", check_package_service_no_delete),
    (
        "PACKAGE_EXTENSION_NO_DELETE",
        check_package_extension_no_delete,
    ),
];
