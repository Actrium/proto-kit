//! Rule handlers implementing the actual breaking change detection logic
//!
//! This module contains the concrete implementations of breaking change rules,
//! ported from Buf's handlers to maintain exact compatibility.

use crate::canonical::{
    CanonicalEnum, CanonicalField, CanonicalFile, CanonicalMessage, CanonicalService,
};
use crate::compat::types::{
    BreakingChange, BreakingLocation, BreakingSeverity, RuleContext, RuleResult,
};
use std::collections::{HashMap, HashSet};

// ============================================================================
// FIELD TYPE COMPATIBILITY HELPERS - 1:1 PORT FROM BUF GO
// ============================================================================

/// Extract proto field kind from type_name for 1:1 Buf compatibility
fn get_proto_field_kind(type_name: &str) -> String {
    // Handle well-known types and primitives exactly as Buf does
    match type_name {
        "double" => "double".to_string(),
        "float" => "float".to_string(),
        "int64" => "int64".to_string(),
        "uint64" => "uint64".to_string(),
        "int32" => "int32".to_string(),
        "fixed64" => "fixed64".to_string(),
        "fixed32" => "fixed32".to_string(),
        "bool" => "bool".to_string(),
        "string" => "string".to_string(),
        "bytes" => "bytes".to_string(),
        "uint32" => "uint32".to_string(),
        "sfixed32" => "sfixed32".to_string(),
        "sfixed64" => "sfixed64".to_string(),
        "sint32" => "sint32".to_string(),
        "sint64" => "sint64".to_string(),
        _ => {
            // Handle map types
            if type_name.starts_with("map<") {
                "map".to_string()
            }
            // Handle message types (including nested) - anything with dots or complex names
            else if type_name.contains('.')
                || (!type_name.contains('<') && !is_primitive_type(type_name))
            {
                "message".to_string()
            }
            // Assume enum if not primitive and not message
            else {
                "enum".to_string()
            }
        }
    }
}

fn is_primitive_type(type_name: &str) -> bool {
    matches!(
        type_name,
        "double"
            | "float"
            | "int64"
            | "uint64"
            | "int32"
            | "fixed64"
            | "fixed32"
            | "bool"
            | "string"
            | "bytes"
            | "uint32"
            | "sfixed32"
            | "sfixed64"
            | "sint32"
            | "sint64"
    )
}

/// Check if two types are compatible at kind level (matching Buf's descriptor.Kind() comparison)
fn types_have_same_kind(type1: &str, type2: &str) -> bool {
    get_proto_field_kind(type1) == get_proto_field_kind(type2)
}

/// Check if type requires typename comparison (enum, group, message types)
fn type_requires_typename_comparison(type_name: &str) -> bool {
    let kind = get_proto_field_kind(type_name);
    matches!(kind.as_str(), "enum" | "message") || type_name.starts_with("map<") // Maps also need typename comparison
}

/// Helper function to create a breaking change
pub fn create_breaking_change(
    rule_id: &str,
    message: String,
    location: BreakingLocation,
    previous_location: Option<BreakingLocation>,
    categories: Vec<String>,
) -> BreakingChange {
    BreakingChange {
        rule_id: rule_id.to_string(),
        message,
        location,
        previous_location,
        severity: BreakingSeverity::Error,
        categories,
    }
}

/// Helper function to create a location
pub fn create_location(
    file_path: &str,
    element_type: &str,
    element_name: &str,
) -> BreakingLocation {
    BreakingLocation {
        file_path: file_path.to_string(),
        line: None,
        column: None,
        element_type: element_type.to_string(),
        element_name: element_name.to_string(),
    }
}

/// Recursively collect all enums (both top-level and nested) with their full names
fn collect_all_enums(
    messages: &std::collections::BTreeSet<CanonicalMessage>,
    enums: &std::collections::BTreeSet<CanonicalEnum>,
    prefix: &str,
) -> HashMap<String, String> {
    let mut all_enums = HashMap::new();

    // Add top-level enums
    for enum_def in enums {
        let full_name = if prefix.is_empty() {
            enum_def.name.clone()
        } else {
            format!("{}.{}", prefix, enum_def.name)
        };
        all_enums.insert(full_name, enum_def.name.clone());
    }

    // Add nested enums from messages
    for message in messages {
        let message_prefix = if prefix.is_empty() {
            message.name.clone()
        } else {
            format!("{}.{}", prefix, message.name)
        };

        // Add nested enums
        for nested_enum in &message.nested_enums {
            let full_name = format!("{}.{}", message_prefix, nested_enum.name);
            all_enums.insert(full_name, nested_enum.name.clone());
        }

        // Recursively process nested messages
        let nested = collect_all_enums(
            &message.nested_messages,
            &std::collections::BTreeSet::new(),
            &message_prefix,
        );
        all_enums.extend(nested);
    }

    all_enums
}

/// Check for deleted enums (ENUM_NO_DELETE)
pub fn check_enum_no_delete(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    // Collect all enums (top-level and nested) from both files
    let current_enums = collect_all_enums(&current.messages, &current.enums, "");
    let previous_enums = collect_all_enums(&previous.messages, &previous.enums, "");

    for full_name in previous_enums.keys() {
        if !current_enums.contains_key(full_name) {
            let change = create_breaking_change(
                "ENUM_NO_DELETE",
                format!("Enum \"{full_name}\" was deleted."),
                create_location(&context.current_file, "enum", full_name),
                Some(create_location(
                    context
                        .previous_file
                        .as_ref()
                        .unwrap_or(&"previous".to_string()),
                    "enum",
                    full_name,
                )),
                vec!["FILE".to_string()],
            );
            changes.push(change);
        }
    }

    RuleResult::with_changes(changes)
}

/// Helper function to collect all messages (top-level and nested) with their full names
fn collect_all_messages(
    messages: &std::collections::BTreeSet<CanonicalMessage>,
    prefix: &str,
) -> HashMap<String, String> {
    let mut all_messages = HashMap::new();

    for message in messages {
        let full_name = if prefix.is_empty() {
            message.name.clone()
        } else {
            format!("{}.{}", prefix, message.name)
        };

        all_messages.insert(full_name.clone(), message.name.clone());

        // Recursively collect nested messages
        let nested = collect_all_messages(&message.nested_messages, &full_name);
        all_messages.extend(nested);
    }

    all_messages
}

/// Check for deleted messages (MESSAGE_NO_DELETE)
pub fn check_message_no_delete(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    // Collect all messages (top-level and nested) from both current and previous
    let current_messages = collect_all_messages(&current.messages, "");
    let previous_messages = collect_all_messages(&previous.messages, "");

    for full_name in previous_messages.keys() {
        if !current_messages.contains_key(full_name) {
            let change = create_breaking_change(
                "MESSAGE_NO_DELETE",
                format!("Message \"{full_name}\" was deleted."),
                create_location(&context.current_file, "message", full_name),
                Some(create_location(
                    context
                        .previous_file
                        .as_ref()
                        .unwrap_or(&"previous".to_string()),
                    "message",
                    full_name,
                )),
                vec!["FILE".to_string()],
            );
            changes.push(change);
        }
    }

    RuleResult::with_changes(changes)
}

/// Check for deleted services (SERVICE_NO_DELETE)
pub fn check_service_no_delete(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    let current_services: HashSet<&String> = current.services.iter().map(|s| &s.name).collect();

    for previous_service in &previous.services {
        if !current_services.contains(&previous_service.name) {
            let change = create_breaking_change(
                "SERVICE_NO_DELETE",
                format!("Service \"{}\" was deleted.", previous_service.name),
                create_location(&context.current_file, "service", &previous_service.name),
                Some(create_location(
                    context
                        .previous_file
                        .as_ref()
                        .unwrap_or(&"previous".to_string()),
                    "service",
                    &previous_service.name,
                )),
                vec!["FILE".to_string()],
            );
            changes.push(change);
        }
    }

    RuleResult::with_changes(changes)
}

/// Check for deleted fields (FIELD_NO_DELETE)
pub fn check_field_no_delete(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    // Create maps for efficient lookup
    let current_messages: HashMap<&String, &CanonicalMessage> =
        current.messages.iter().map(|m| (&m.name, m)).collect();

    for previous_message in &previous.messages {
        if let Some(current_message) = current_messages.get(&previous_message.name) {
            let current_fields: HashSet<i32> =
                current_message.fields.iter().map(|f| f.number).collect();

            for previous_field in &previous_message.fields {
                if !current_fields.contains(&previous_field.number) {
                    let change = create_breaking_change(
                        "FIELD_NO_DELETE",
                        format!(
                            "Field \"{}\" with number {} was deleted from message \"{}\".",
                            previous_field.name, previous_field.number, previous_message.name
                        ),
                        create_location(
                            &context.current_file,
                            "field",
                            &format!("{}.{}", previous_message.name, previous_field.name),
                        ),
                        Some(create_location(
                            context
                                .previous_file
                                .as_ref()
                                .unwrap_or(&"previous".to_string()),
                            "field",
                            &format!("{}.{}", previous_message.name, previous_field.name),
                        )),
                        vec!["FILE".to_string(), "PACKAGE".to_string()],
                    );
                    changes.push(change);
                }
            }
        }
    }

    RuleResult::with_changes(changes)
}

/// Helper to find enum by full name in nested structure
fn find_enum_by_name<'a>(
    messages: &'a std::collections::BTreeSet<CanonicalMessage>,
    enums: &'a std::collections::BTreeSet<CanonicalEnum>,
    full_name: &str,
) -> Option<&'a CanonicalEnum> {
    // Check top-level enums first
    for enum_def in enums {
        if enum_def.name == full_name {
            return Some(enum_def);
        }
    }

    // Check nested enums
    let parts: Vec<&str> = full_name.split('.').collect();
    if parts.len() >= 2 {
        // Try to find nested enum
        fn search_nested_enum<'b>(
            messages: &'b std::collections::BTreeSet<CanonicalMessage>,
            path: &[&str],
        ) -> Option<&'b CanonicalEnum> {
            if path.len() < 2 {
                return None;
            }

            let message_name = path[0];
            for message in messages {
                if message.name == message_name {
                    if path.len() == 2 {
                        // Direct nested enum
                        let enum_name = path[1];
                        for nested_enum in &message.nested_enums {
                            if nested_enum.name == enum_name {
                                return Some(nested_enum);
                            }
                        }
                    } else {
                        // Continue searching in nested messages
                        return search_nested_enum(&message.nested_messages, &path[1..]);
                    }
                }
            }
            None
        }

        return search_nested_enum(messages, &parts);
    }

    None
}

/// Check for deleted enum values (ENUM_VALUE_NO_DELETE)
pub fn check_enum_value_no_delete(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    // Get all enums from both files
    let previous_enums = collect_all_enums(&previous.messages, &previous.enums, "");

    for full_name in previous_enums.keys() {
        if let Some(previous_enum) =
            find_enum_by_name(&previous.messages, &previous.enums, full_name)
        {
            if let Some(current_enum) =
                find_enum_by_name(&current.messages, &current.enums, full_name)
            {
                let current_values: HashSet<i32> =
                    current_enum.values.iter().map(|v| v.number).collect();

                for previous_value in &previous_enum.values {
                    if !current_values.contains(&previous_value.number) {
                        let change = create_breaking_change(
                            "ENUM_VALUE_NO_DELETE",
                            format!(
                                "Enum value \"{}\" with number {} was deleted from enum \"{}\".",
                                previous_value.name, previous_value.number, full_name
                            ),
                            create_location(
                                &context.current_file,
                                "enum_value",
                                &format!("{}.{}", full_name, previous_value.name),
                            ),
                            Some(create_location(
                                context
                                    .previous_file
                                    .as_ref()
                                    .unwrap_or(&"previous".to_string()),
                                "enum_value",
                                &format!("{}.{}", full_name, previous_value.name),
                            )),
                            vec!["FILE".to_string(), "PACKAGE".to_string()],
                        );
                        changes.push(change);
                    }
                }
            }
        }
    }

    RuleResult::with_changes(changes)
}

/// Check for field type changes (FIELD_SAME_TYPE) - 1:1 PORT FROM BUF GO
pub fn check_field_same_type(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    let current_messages: HashMap<&String, &CanonicalMessage> =
        current.messages.iter().map(|m| (&m.name, m)).collect();

    for previous_message in &previous.messages {
        if let Some(current_message) = current_messages.get(&previous_message.name) {
            let current_fields: HashMap<i32, &CanonicalField> = current_message
                .fields
                .iter()
                .map(|f| (f.number, f))
                .collect();

            for previous_field in &previous_message.fields {
                if let Some(current_field) = current_fields.get(&previous_field.number) {
                    // First check: Kind level comparison (matching Buf's descriptor.Kind())
                    // This is the primary check that catches int32->string, message->enum, etc.
                    if !types_have_same_kind(&previous_field.type_name, &current_field.type_name) {
                        let previous_kind = get_proto_field_kind(&previous_field.type_name);
                        let current_kind = get_proto_field_kind(&current_field.type_name);

                        let change = create_breaking_change(
                            "FIELD_SAME_TYPE",
                            format!(
                                "Field \"{}\" on message \"{}\" changed type from \"{}\" to \"{}\".",
                                previous_field.name,
                                previous_message.name,
                                previous_kind,
                                current_kind
                            ),
                            create_location(
                                &context.current_file,
                                "field",
                                &format!("{}.{}", previous_message.name, current_field.name),
                            ),
                            Some(create_location(
                                context
                                    .previous_file
                                    .as_ref()
                                    .unwrap_or(&"previous".to_string()),
                                "field",
                                &format!("{}.{}", previous_message.name, previous_field.name),
                            )),
                            vec![
                                "FILE".to_string(),
                                "PACKAGE".to_string(),
                                "WIRE_JSON".to_string(),
                                "WIRE".to_string(),
                            ],
                        );
                        changes.push(change);
                    }
                    // Second check: TypeName comparison for complex types (enum, message, map)
                    // This catches same-kind but different-type changes like Message1->Message2
                    else if type_requires_typename_comparison(&previous_field.type_name)
                        && current_field.type_name != previous_field.type_name
                    {
                        // Clean type names by removing leading dots (matching Buf's behavior)
                        let clean_previous = previous_field.type_name.trim_start_matches('.');
                        let clean_current = current_field.type_name.trim_start_matches('.');

                        let change = create_breaking_change(
                            "FIELD_SAME_TYPE",
                            format!(
                                "Field \"{}\" on message \"{}\" changed type from \"{}\" to \"{}\".",
                                previous_field.name,
                                previous_message.name,
                                clean_previous,
                                clean_current
                            ),
                            create_location(
                                &context.current_file,
                                "field",
                                &format!("{}.{}", previous_message.name, current_field.name),
                            ),
                            Some(create_location(
                                context
                                    .previous_file
                                    .as_ref()
                                    .unwrap_or(&"previous".to_string()),
                                "field",
                                &format!("{}.{}", previous_message.name, previous_field.name),
                            )),
                            vec![
                                "FILE".to_string(),
                                "PACKAGE".to_string(),
                                "WIRE_JSON".to_string(),
                                "WIRE".to_string(),
                            ],
                        );
                        changes.push(change);
                    }
                }
            }
        }
    }

    RuleResult::with_changes(changes)
}

/// Check for field name changes (FIELD_SAME_NAME)
pub fn check_field_same_name(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    let current_messages: HashMap<&String, &CanonicalMessage> =
        current.messages.iter().map(|m| (&m.name, m)).collect();

    for previous_message in &previous.messages {
        if let Some(current_message) = current_messages.get(&previous_message.name) {
            let current_fields: HashMap<i32, &CanonicalField> = current_message
                .fields
                .iter()
                .map(|f| (f.number, f))
                .collect();

            for previous_field in &previous_message.fields {
                if let Some(current_field) = current_fields.get(&previous_field.number) {
                    if current_field.name != previous_field.name {
                        let change = create_breaking_change(
                            "FIELD_SAME_NAME",
                            format!(
                                "Field {} on message \"{}\" changed name from \"{}\" to \"{}\".",
                                previous_field.number,
                                previous_message.name,
                                previous_field.name,
                                current_field.name
                            ),
                            create_location(
                                &context.current_file,
                                "field",
                                &format!("{}.{}", previous_message.name, current_field.name),
                            ),
                            Some(create_location(
                                context
                                    .previous_file
                                    .as_ref()
                                    .unwrap_or(&"previous".to_string()),
                                "field",
                                &format!("{}.{}", previous_message.name, previous_field.name),
                            )),
                            vec![
                                "FILE".to_string(),
                                "PACKAGE".to_string(),
                                "WIRE_JSON".to_string(),
                            ],
                        );
                        changes.push(change);
                    }
                }
            }
        }
    }

    RuleResult::with_changes(changes)
}

/// Check for package changes (FILE_SAME_PACKAGE)
pub fn check_file_same_package(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    if current.package != previous.package {
        let current_pkg = current.package.as_deref().unwrap_or("<no package>");
        let previous_pkg = previous.package.as_deref().unwrap_or("<no package>");

        let change = create_breaking_change(
            "FILE_SAME_PACKAGE",
            format!("File package changed from \"{previous_pkg}\" to \"{current_pkg}\"."),
            create_location(&context.current_file, "file", "package"),
            Some(create_location(
                context
                    .previous_file
                    .as_ref()
                    .unwrap_or(&"previous".to_string()),
                "file",
                "package",
            )),
            vec!["FILE".to_string()],
        );
        changes.push(change);
    }

    RuleResult::with_changes(changes)
}

/// Check for deleted RPCs (RPC_NO_DELETE)
pub fn check_rpc_no_delete(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    let current_services: HashMap<&String, &CanonicalService> =
        current.services.iter().map(|s| (&s.name, s)).collect();

    for previous_service in &previous.services {
        if let Some(current_service) = current_services.get(&previous_service.name) {
            let current_methods: HashSet<&String> =
                current_service.methods.iter().map(|m| &m.name).collect();

            for previous_method in &previous_service.methods {
                if !current_methods.contains(&previous_method.name) {
                    let change = create_breaking_change(
                        "RPC_NO_DELETE",
                        format!(
                            "RPC \"{}\" was deleted from service \"{}\".",
                            previous_method.name, previous_service.name
                        ),
                        create_location(
                            &context.current_file,
                            "rpc",
                            &format!("{}.{}", previous_service.name, previous_method.name),
                        ),
                        Some(create_location(
                            context
                                .previous_file
                                .as_ref()
                                .unwrap_or(&"previous".to_string()),
                            "rpc",
                            &format!("{}.{}", previous_service.name, previous_method.name),
                        )),
                        vec!["FILE".to_string(), "PACKAGE".to_string()],
                    );
                    changes.push(change);
                }
            }
        }
    }

    RuleResult::with_changes(changes)
}

/// Check for RPC signature changes (RPC_SAME_VALUES)
pub fn check_rpc_same_values(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    let current_services: HashMap<&String, &CanonicalService> =
        current.services.iter().map(|s| (&s.name, s)).collect();

    for previous_service in &previous.services {
        if let Some(current_service) = current_services.get(&previous_service.name) {
            let current_methods: HashMap<&String, &crate::canonical::CanonicalMethod> =
                current_service
                    .methods
                    .iter()
                    .map(|m| (&m.name, m))
                    .collect();

            for previous_method in &previous_service.methods {
                if let Some(current_method) = current_methods.get(&previous_method.name) {
                    // Check if input type changed
                    if current_method.input_type != previous_method.input_type {
                        let change = create_breaking_change(
                            "RPC_SAME_VALUES",
                            format!(
                                "RPC \"{}\" on service \"{}\" changed input type from \"{}\" to \"{}\".",
                                previous_method.name,
                                previous_service.name,
                                previous_method.input_type,
                                current_method.input_type
                            ),
                            create_location(
                                &context.current_file,
                                "rpc",
                                &format!("{}.{}", previous_service.name, current_method.name),
                            ),
                            Some(create_location(
                                context
                                    .previous_file
                                    .as_ref()
                                    .unwrap_or(&"previous".to_string()),
                                "rpc",
                                &format!("{}.{}", previous_service.name, previous_method.name),
                            )),
                            vec!["FILE".to_string(), "PACKAGE".to_string()],
                        );
                        changes.push(change);
                    }

                    // Check if output type changed
                    if current_method.output_type != previous_method.output_type {
                        let change = create_breaking_change(
                            "RPC_SAME_VALUES",
                            format!(
                                "RPC \"{}\" on service \"{}\" changed output type from \"{}\" to \"{}\".",
                                previous_method.name,
                                previous_service.name,
                                previous_method.output_type,
                                current_method.output_type
                            ),
                            create_location(
                                &context.current_file,
                                "rpc",
                                &format!("{}.{}", previous_service.name, current_method.name),
                            ),
                            Some(create_location(
                                context
                                    .previous_file
                                    .as_ref()
                                    .unwrap_or(&"previous".to_string()),
                                "rpc",
                                &format!("{}.{}", previous_service.name, previous_method.name),
                            )),
                            vec!["FILE".to_string(), "PACKAGE".to_string()],
                        );
                        changes.push(change);
                    }

                    // Check if client streaming changed
                    if current_method.client_streaming != previous_method.client_streaming {
                        let change = create_breaking_change(
                            "RPC_SAME_VALUES",
                            format!(
                                "RPC \"{}\" on service \"{}\" changed client streaming from {} to {}.",
                                previous_method.name,
                                previous_service.name,
                                previous_method.client_streaming,
                                current_method.client_streaming
                            ),
                            create_location(
                                &context.current_file,
                                "rpc",
                                &format!("{}.{}", previous_service.name, current_method.name),
                            ),
                            Some(create_location(
                                context
                                    .previous_file
                                    .as_ref()
                                    .unwrap_or(&"previous".to_string()),
                                "rpc",
                                &format!("{}.{}", previous_service.name, previous_method.name),
                            )),
                            vec!["FILE".to_string(), "PACKAGE".to_string()],
                        );
                        changes.push(change);
                    }

                    // Check if server streaming changed
                    if current_method.server_streaming != previous_method.server_streaming {
                        let change = create_breaking_change(
                            "RPC_SAME_VALUES",
                            format!(
                                "RPC \"{}\" on service \"{}\" changed server streaming from {} to {}.",
                                previous_method.name,
                                previous_service.name,
                                previous_method.server_streaming,
                                current_method.server_streaming
                            ),
                            create_location(
                                &context.current_file,
                                "rpc",
                                &format!("{}.{}", previous_service.name, current_method.name),
                            ),
                            Some(create_location(
                                context
                                    .previous_file
                                    .as_ref()
                                    .unwrap_or(&"previous".to_string()),
                                "rpc",
                                &format!("{}.{}", previous_service.name, previous_method.name),
                            )),
                            vec!["FILE".to_string(), "PACKAGE".to_string()],
                        );
                        changes.push(change);
                    }
                }
            }
        }
    }

    RuleResult::with_changes(changes)
}

/// Check for package-level message deletions (PACKAGE_MESSAGE_NO_DELETE)
/// This is different from MESSAGE_NO_DELETE as it checks across the entire package
pub fn check_package_message_no_delete(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    // For package-level checks, we only care if the package has changed
    // If the package is the same, use the same logic as MESSAGE_NO_DELETE but with different categories
    if current.package == previous.package {
        let current_messages = collect_all_messages(&current.messages, "");
        let previous_messages = collect_all_messages(&previous.messages, "");

        for full_name in previous_messages.keys() {
            if !current_messages.contains_key(full_name) {
                let change = create_breaking_change(
                    "PACKAGE_MESSAGE_NO_DELETE",
                    format!("Message \"{full_name}\" was deleted from package."),
                    create_location(&context.current_file, "message", full_name),
                    Some(create_location(
                        context
                            .previous_file
                            .as_ref()
                            .unwrap_or(&"previous".to_string()),
                        "message",
                        full_name,
                    )),
                    vec!["PACKAGE".to_string()],
                );
                changes.push(change);
            }
        }
    }

    RuleResult::with_changes(changes)
}

/// Check for enum value name changes (ENUM_VALUE_SAME_NAME)
pub fn check_enum_value_same_name(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    let previous_enums = collect_all_enums(&previous.messages, &previous.enums, "");

    for full_name in previous_enums.keys() {
        if let Some(previous_enum) =
            find_enum_by_name(&previous.messages, &previous.enums, full_name)
        {
            if let Some(current_enum) =
                find_enum_by_name(&current.messages, &current.enums, full_name)
            {
                let current_values: HashMap<i32, &crate::canonical::CanonicalEnumValue> =
                    current_enum.values.iter().map(|v| (v.number, v)).collect();

                for previous_value in &previous_enum.values {
                    if let Some(current_value) = current_values.get(&previous_value.number) {
                        if current_value.name != previous_value.name {
                            let change = create_breaking_change(
                                "ENUM_VALUE_SAME_NAME",
                                format!(
                                    "Enum value {} on enum \"{}\" changed name from \"{}\" to \"{}\".",
                                    previous_value.number,
                                    full_name,
                                    previous_value.name,
                                    current_value.name
                                ),
                                create_location(
                                    &context.current_file,
                                    "enum_value",
                                    &format!("{}.{}", full_name, current_value.name),
                                ),
                                Some(create_location(
                                    context
                                        .previous_file
                                        .as_ref()
                                        .unwrap_or(&"previous".to_string()),
                                    "enum_value",
                                    &format!("{}.{}", full_name, previous_value.name),
                                )),
                                vec![
                                    "FILE".to_string(),
                                    "PACKAGE".to_string(),
                                    "WIRE_JSON".to_string(),
                                ],
                            );
                            changes.push(change);
                        }
                    }
                }
            }
        }
    }

    RuleResult::with_changes(changes)
}

/// Check for field cardinality changes (FIELD_SAME_CARDINALITY)
pub fn check_field_same_cardinality(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    let current_messages: HashMap<&String, &CanonicalMessage> =
        current.messages.iter().map(|m| (&m.name, m)).collect();

    for previous_message in &previous.messages {
        if let Some(current_message) = current_messages.get(&previous_message.name) {
            let current_fields: HashMap<i32, &CanonicalField> = current_message
                .fields
                .iter()
                .map(|f| (f.number, f))
                .collect();

            for previous_field in &previous_message.fields {
                if let Some(current_field) = current_fields.get(&previous_field.number) {
                    // Compare label (cardinality): optional, required, repeated
                    if current_field.label != previous_field.label {
                        let current_label = current_field.label.as_deref().unwrap_or("optional");
                        let previous_label = previous_field.label.as_deref().unwrap_or("optional");

                        let change = create_breaking_change(
                            "FIELD_SAME_CARDINALITY",
                            format!(
                                "Field \"{}\" on message \"{}\" changed cardinality from \"{}\" to \"{}\".",
                                previous_field.name,
                                previous_message.name,
                                previous_label,
                                current_label
                            ),
                            create_location(
                                &context.current_file,
                                "field",
                                &format!("{}.{}", previous_message.name, current_field.name),
                            ),
                            Some(create_location(
                                context
                                    .previous_file
                                    .as_ref()
                                    .unwrap_or(&"previous".to_string()),
                                "field",
                                &format!("{}.{}", previous_message.name, previous_field.name),
                            )),
                            vec![
                                "FILE".to_string(),
                                "PACKAGE".to_string(),
                                "WIRE_JSON".to_string(),
                                "WIRE".to_string(),
                            ],
                        );
                        changes.push(change);
                    }
                }
            }
        }
    }

    RuleResult::with_changes(changes)
}
