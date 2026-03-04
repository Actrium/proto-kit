//! Bulk-generated SERVICE rules for service-level breaking change detection
//!
//! These rules handle service definitions, RPC methods, and their attributes.

use crate::canonical::{CanonicalFile, CanonicalMethod, CanonicalService};
use crate::compat::handlers::{create_breaking_change, create_location};
use crate::compat::types::{RuleContext, RuleResult};
use std::collections::HashMap;

// ========================================
// SERVICE Rules
// ========================================

/// SERVICE_NO_DELETE - checks services aren't deleted
pub fn check_service_no_delete(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    let prev_services = collect_all_services(previous);
    let curr_services = collect_all_services(current);

    for service_name in prev_services.keys() {
        if !curr_services.contains_key(service_name) {
            changes.push(create_breaking_change(
                "SERVICE_NO_DELETE",
                format!("Service \"{service_name}\" was deleted."),
                create_location(&context.current_file, "file", &context.current_file),
                Some(create_location(
                    context.previous_file.as_deref().unwrap_or(""),
                    "service",
                    service_name,
                )),
                vec!["SERVICE".to_string()],
            ));
        }
    }

    RuleResult::with_changes(changes)
}

/// RPC_NO_DELETE - checks RPC methods aren't deleted
pub fn check_rpc_no_delete(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    let prev_services = collect_all_services(previous);
    let curr_services = collect_all_services(current);

    for (service_name, prev_service) in &prev_services {
        if let Some(curr_service) = curr_services.get(service_name) {
            // Create maps for efficient lookup by method name
            let prev_methods: HashMap<String, &CanonicalMethod> = prev_service
                .methods
                .iter()
                .map(|m| (m.name.clone(), m))
                .collect();
            let curr_methods: HashMap<String, &CanonicalMethod> = curr_service
                .methods
                .iter()
                .map(|m| (m.name.clone(), m))
                .collect();

            // Find deleted methods
            for method_name in prev_methods.keys() {
                if !curr_methods.contains_key(method_name) {
                    changes.push(create_breaking_change(
                        "RPC_NO_DELETE",
                        format!(
                            "RPC \"{method_name}\" was deleted from service \"{service_name}\"."
                        ),
                        create_location(&context.current_file, "service", service_name),
                        Some(create_location(
                            context.previous_file.as_deref().unwrap_or(""),
                            "rpc",
                            method_name,
                        )),
                        vec!["RPC".to_string()],
                    ));
                }
            }
        }
    }

    RuleResult::with_changes(changes)
}

/// RPC_SAME_REQUEST_TYPE - checks RPC request types don't change
pub fn check_rpc_same_request_type(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    let prev_services = collect_all_services(previous);
    let curr_services = collect_all_services(current);

    for (service_name, prev_service) in &prev_services {
        if let Some(curr_service) = curr_services.get(service_name) {
            let prev_methods: HashMap<String, &CanonicalMethod> = prev_service
                .methods
                .iter()
                .map(|m| (m.name.clone(), m))
                .collect();
            let curr_methods: HashMap<String, &CanonicalMethod> = curr_service
                .methods
                .iter()
                .map(|m| (m.name.clone(), m))
                .collect();

            for (method_name, prev_method) in &prev_methods {
                if let Some(curr_method) = curr_methods.get(method_name) {
                    if prev_method.input_type != curr_method.input_type {
                        changes.push(create_breaking_change(
                            "RPC_SAME_REQUEST_TYPE",
                            format!(
                                "RPC \"{}\" request type changed from \"{}\" to \"{}\" in service \"{}\".",
                                method_name, prev_method.input_type, curr_method.input_type, service_name
                            ),
                            create_location(&context.current_file, "rpc", method_name),
                            Some(create_location(
                                context.previous_file.as_deref().unwrap_or(""),
                                "rpc",
                                method_name
                            )),
                            vec!["RPC".to_string()],
                        ));
                    }
                }
            }
        }
    }

    RuleResult::with_changes(changes)
}

/// RPC_SAME_RESPONSE_TYPE - checks RPC response types don't change
pub fn check_rpc_same_response_type(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    let prev_services = collect_all_services(previous);
    let curr_services = collect_all_services(current);

    for (service_name, prev_service) in &prev_services {
        if let Some(curr_service) = curr_services.get(service_name) {
            let prev_methods: HashMap<String, &CanonicalMethod> = prev_service
                .methods
                .iter()
                .map(|m| (m.name.clone(), m))
                .collect();
            let curr_methods: HashMap<String, &CanonicalMethod> = curr_service
                .methods
                .iter()
                .map(|m| (m.name.clone(), m))
                .collect();

            for (method_name, prev_method) in &prev_methods {
                if let Some(curr_method) = curr_methods.get(method_name) {
                    if prev_method.output_type != curr_method.output_type {
                        changes.push(create_breaking_change(
                            "RPC_SAME_RESPONSE_TYPE",
                            format!(
                                "RPC \"{}\" response type changed from \"{}\" to \"{}\" in service \"{}\".",
                                method_name, prev_method.output_type, curr_method.output_type, service_name
                            ),
                            create_location(&context.current_file, "rpc", method_name),
                            Some(create_location(
                                context.previous_file.as_deref().unwrap_or(""),
                                "rpc",
                                method_name
                            )),
                            vec!["RPC".to_string()],
                        ));
                    }
                }
            }
        }
    }

    RuleResult::with_changes(changes)
}

/// RPC_SAME_CLIENT_STREAMING - checks RPC client streaming flags don't change
pub fn check_rpc_same_client_streaming(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    let prev_services = collect_all_services(previous);
    let curr_services = collect_all_services(current);

    for (service_name, prev_service) in &prev_services {
        if let Some(curr_service) = curr_services.get(service_name) {
            let prev_methods: HashMap<String, &CanonicalMethod> = prev_service
                .methods
                .iter()
                .map(|m| (m.name.clone(), m))
                .collect();
            let curr_methods: HashMap<String, &CanonicalMethod> = curr_service
                .methods
                .iter()
                .map(|m| (m.name.clone(), m))
                .collect();

            for (method_name, prev_method) in &prev_methods {
                if let Some(curr_method) = curr_methods.get(method_name) {
                    if prev_method.client_streaming != curr_method.client_streaming {
                        changes.push(create_breaking_change(
                            "RPC_SAME_CLIENT_STREAMING",
                            format!(
                                "RPC \"{}\" client streaming changed from {} to {} in service \"{}\".",
                                method_name, prev_method.client_streaming, curr_method.client_streaming, service_name
                            ),
                            create_location(&context.current_file, "rpc", method_name),
                            Some(create_location(
                                context.previous_file.as_deref().unwrap_or(""),
                                "rpc",
                                method_name
                            )),
                            vec!["RPC".to_string()],
                        ));
                    }
                }
            }
        }
    }

    RuleResult::with_changes(changes)
}

/// RPC_SAME_SERVER_STREAMING - checks RPC server streaming flags don't change
pub fn check_rpc_same_server_streaming(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    let prev_services = collect_all_services(previous);
    let curr_services = collect_all_services(current);

    for (service_name, prev_service) in &prev_services {
        if let Some(curr_service) = curr_services.get(service_name) {
            let prev_methods: HashMap<String, &CanonicalMethod> = prev_service
                .methods
                .iter()
                .map(|m| (m.name.clone(), m))
                .collect();
            let curr_methods: HashMap<String, &CanonicalMethod> = curr_service
                .methods
                .iter()
                .map(|m| (m.name.clone(), m))
                .collect();

            for (method_name, prev_method) in &prev_methods {
                if let Some(curr_method) = curr_methods.get(method_name) {
                    if prev_method.server_streaming != curr_method.server_streaming {
                        changes.push(create_breaking_change(
                            "RPC_SAME_SERVER_STREAMING",
                            format!(
                                "RPC \"{}\" server streaming changed from {} to {} in service \"{}\".",
                                method_name, prev_method.server_streaming, curr_method.server_streaming, service_name
                            ),
                            create_location(&context.current_file, "rpc", method_name),
                            Some(create_location(
                                context.previous_file.as_deref().unwrap_or(""),
                                "rpc",
                                method_name
                            )),
                            vec!["RPC".to_string()],
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

pub const SERVICE_RULES: &[crate::compat::types::RuleEntry] = &[
    ("SERVICE_NO_DELETE", check_service_no_delete),
    ("RPC_NO_DELETE", check_rpc_no_delete),
    ("RPC_SAME_REQUEST_TYPE", check_rpc_same_request_type),
    ("RPC_SAME_RESPONSE_TYPE", check_rpc_same_response_type),
    ("RPC_SAME_CLIENT_STREAMING", check_rpc_same_client_streaming),
    ("RPC_SAME_SERVER_STREAMING", check_rpc_same_server_streaming),
];
