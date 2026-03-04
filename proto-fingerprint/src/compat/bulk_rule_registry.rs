//! Auto-generated bulk rule registry
//!
//! This module automatically registers all bulk-generated rules.

// Import all bulk rule modules
use crate::compat::bulk_enum_rules;
use crate::compat::bulk_extension_rules;
use crate::compat::bulk_field_rules;
use crate::compat::bulk_file_rules;
use crate::compat::bulk_message_rules;
use crate::compat::bulk_other_rules;
use crate::compat::bulk_package_rules;
use crate::compat::bulk_reserved_rules;
use crate::compat::bulk_service_rules;
// No longer using bulk_special_rules - removed for 1:1 Buf compatibility

/// Master rule registry combining all bulk-generated rules
pub fn get_bulk_rule_mapping() -> &'static [crate::compat::types::RuleEntry] {
    BULK_RULES
}

/// Static rule table exactly matching Buf's breaking rules (69 rules)
const BULK_RULES: &[crate::compat::types::RuleEntry] = &[
    // COMMENT rules (1 rule) - Buf specific
    ("COMMENT_ENUM", bulk_other_rules::check_comment_enum),
    // ENUM rules (3 rules)
    (
        "ENUM_SAME_JSON_FORMAT",
        bulk_other_rules::check_enum_same_json_format,
    ),
    ("ENUM_SAME_TYPE", bulk_other_rules::check_enum_same_type),
    (
        "ENUM_VALUE_NO_DELETE",
        bulk_enum_rules::check_enum_value_no_delete,
    ),
    (
        "ENUM_VALUE_NO_DELETE_UNLESS_NAME_RESERVED",
        bulk_reserved_rules::check_enum_value_no_delete_unless_name_reserved,
    ),
    (
        "ENUM_VALUE_NO_DELETE_UNLESS_NUMBER_RESERVED",
        bulk_reserved_rules::check_enum_value_no_delete_unless_number_reserved,
    ),
    (
        "ENUM_VALUE_SAME_NAME",
        bulk_other_rules::check_enum_value_same_name,
    ),
    // EXTENSION rules (2 rules)
    (
        "EXTENSION_MESSAGE_NO_DELETE",
        bulk_extension_rules::check_extension_message_no_delete,
    ),
    (
        "EXTENSION_NO_DELETE",
        bulk_extension_rules::check_extension_no_delete,
    ),
    // FIELD rules (17 rules)
    ("FIELD_NO_DELETE", bulk_message_rules::check_field_no_delete),
    (
        "FIELD_NO_DELETE_UNLESS_NAME_RESERVED",
        bulk_reserved_rules::check_field_no_delete_unless_name_reserved,
    ),
    (
        "FIELD_NO_DELETE_UNLESS_NUMBER_RESERVED",
        bulk_reserved_rules::check_field_no_delete_unless_number_reserved,
    ),
    (
        "FIELD_SAME_CARDINALITY",
        bulk_field_rules::check_field_same_cardinality,
    ),
    (
        "FIELD_SAME_CPP_STRING_TYPE",
        bulk_field_rules::check_field_same_cpp_string_type,
    ),
    ("FIELD_SAME_CTYPE", bulk_field_rules::check_field_same_ctype),
    (
        "FIELD_SAME_DEFAULT",
        bulk_field_rules::check_field_same_default,
    ),
    (
        "FIELD_SAME_JAVA_UTF8_VALIDATION",
        bulk_field_rules::check_field_same_java_utf8_validation,
    ),
    (
        "FIELD_SAME_JSON_NAME",
        bulk_field_rules::check_field_same_json_name,
    ),
    (
        "FIELD_SAME_JSTYPE",
        bulk_field_rules::check_field_same_jstype,
    ),
    ("FIELD_SAME_LABEL", bulk_field_rules::check_field_same_label),
    ("FIELD_SAME_NAME", bulk_message_rules::check_field_same_name),
    ("FIELD_SAME_ONEOF", bulk_field_rules::check_field_same_oneof),
    ("FIELD_SAME_TYPE", bulk_message_rules::check_field_same_type),
    (
        "FIELD_SAME_UTF8_VALIDATION",
        bulk_field_rules::check_field_same_utf8_validation,
    ),
    (
        "FIELD_WIRE_COMPATIBLE_CARDINALITY",
        bulk_field_rules::check_field_wire_compatible_cardinality,
    ),
    (
        "FIELD_WIRE_COMPATIBLE_TYPE",
        bulk_field_rules::check_field_wire_compatible_type,
    ),
    (
        "FIELD_WIRE_JSON_COMPATIBLE_CARDINALITY",
        bulk_field_rules::check_field_wire_json_compatible_cardinality,
    ),
    (
        "FIELD_WIRE_JSON_COMPATIBLE_TYPE",
        bulk_field_rules::check_field_wire_json_compatible_type,
    ),
    // FILE rules (17 rules)
    ("FILE_NO_DELETE", bulk_file_rules::check_file_no_delete),
    (
        "FILE_SAME_CC_ENABLE_ARENAS",
        bulk_file_rules::check_file_same_cc_enable_arenas,
    ),
    (
        "FILE_SAME_CC_GENERIC_SERVICES",
        bulk_file_rules::check_file_same_cc_generic_services,
    ),
    (
        "FILE_SAME_CSHARP_NAMESPACE",
        bulk_file_rules::check_file_same_csharp_namespace,
    ),
    (
        "FILE_SAME_GO_PACKAGE",
        bulk_file_rules::check_file_same_go_package,
    ),
    (
        "FILE_SAME_JAVA_GENERIC_SERVICES",
        bulk_file_rules::check_file_same_java_generic_services,
    ),
    (
        "FILE_SAME_JAVA_MULTIPLE_FILES",
        bulk_file_rules::check_file_same_java_multiple_files,
    ),
    (
        "FILE_SAME_JAVA_OUTER_CLASSNAME",
        bulk_file_rules::check_file_same_java_outer_classname,
    ),
    (
        "FILE_SAME_JAVA_PACKAGE",
        bulk_file_rules::check_file_same_java_package,
    ),
    (
        "FILE_SAME_JAVA_STRING_CHECK_UTF8",
        bulk_file_rules::check_file_same_java_string_check_utf8,
    ),
    (
        "FILE_SAME_OBJC_CLASS_PREFIX",
        bulk_file_rules::check_file_same_objc_class_prefix,
    ),
    (
        "FILE_SAME_OPTIMIZE_FOR",
        bulk_file_rules::check_file_same_optimize_for,
    ),
    (
        "FILE_SAME_PACKAGE",
        bulk_file_rules::check_file_same_package,
    ),
    (
        "FILE_SAME_PHP_CLASS_PREFIX",
        bulk_file_rules::check_file_same_php_class_prefix,
    ),
    (
        "FILE_SAME_PHP_GENERIC_SERVICES",
        bulk_file_rules::check_file_same_php_generic_services,
    ),
    (
        "FILE_SAME_PHP_METADATA_NAMESPACE",
        bulk_file_rules::check_file_same_php_metadata_namespace,
    ),
    (
        "FILE_SAME_PHP_NAMESPACE",
        bulk_file_rules::check_file_same_php_namespace,
    ),
    (
        "FILE_SAME_PY_GENERIC_SERVICES",
        bulk_file_rules::check_file_same_py_generic_services,
    ),
    (
        "FILE_SAME_RUBY_PACKAGE",
        bulk_file_rules::check_file_same_ruby_package,
    ),
    (
        "FILE_SAME_SWIFT_PREFIX",
        bulk_file_rules::check_file_same_swift_prefix,
    ),
    ("FILE_SAME_SYNTAX", bulk_file_rules::check_file_same_syntax),
    // MESSAGE rules (5 rules)
    (
        "MESSAGE_NO_DELETE",
        bulk_message_rules::check_message_no_delete,
    ),
    (
        "MESSAGE_NO_REMOVE_STANDARD_DESCRIPTOR_ACCESSOR",
        bulk_message_rules::check_message_no_remove_standard_descriptor_accessor,
    ),
    (
        "MESSAGE_SAME_JSON_FORMAT",
        bulk_other_rules::check_message_same_json_format,
    ),
    (
        "MESSAGE_SAME_MESSAGE_SET_WIRE_FORMAT",
        bulk_message_rules::check_message_same_message_set_wire_format,
    ),
    (
        "MESSAGE_SAME_REQUIRED_FIELDS",
        bulk_other_rules::check_message_same_required_fields,
    ),
    // ONEOF rules (1 rule)
    ("ONEOF_NO_DELETE", bulk_message_rules::check_oneof_no_delete),
    // PACKAGE rules (5 rules)
    (
        "PACKAGE_ENUM_NO_DELETE",
        bulk_package_rules::check_package_enum_no_delete,
    ),
    (
        "PACKAGE_EXTENSION_NO_DELETE",
        bulk_package_rules::check_package_extension_no_delete,
    ),
    (
        "PACKAGE_MESSAGE_NO_DELETE",
        bulk_package_rules::check_package_message_no_delete,
    ),
    (
        "PACKAGE_NO_DELETE",
        bulk_package_rules::check_package_no_delete,
    ),
    (
        "PACKAGE_SERVICE_NO_DELETE",
        bulk_package_rules::check_package_service_no_delete,
    ),
    // RESERVED rules (2 rules)
    (
        "RESERVED_ENUM_NO_DELETE",
        bulk_reserved_rules::check_reserved_enum_no_delete,
    ),
    (
        "RESERVED_MESSAGE_NO_DELETE",
        bulk_reserved_rules::check_reserved_message_no_delete,
    ),
    // RPC rules (6 rules)
    ("RPC_NO_DELETE", bulk_service_rules::check_rpc_no_delete),
    (
        "RPC_SAME_CLIENT_STREAMING",
        bulk_service_rules::check_rpc_same_client_streaming,
    ),
    (
        "RPC_SAME_IDEMPOTENCY_LEVEL",
        bulk_other_rules::check_rpc_same_idempotency_level,
    ),
    (
        "RPC_SAME_REQUEST_TYPE",
        bulk_service_rules::check_rpc_same_request_type,
    ),
    (
        "RPC_SAME_RESPONSE_TYPE",
        bulk_service_rules::check_rpc_same_response_type,
    ),
    (
        "RPC_SAME_SERVER_STREAMING",
        bulk_service_rules::check_rpc_same_server_streaming,
    ),
    // SERVICE rules (1 rule)
    (
        "SERVICE_NO_DELETE",
        bulk_service_rules::check_service_no_delete,
    ),
];

/// Get count of all bulk-generated rules
pub const fn get_bulk_rule_count() -> usize {
    BULK_RULES.len()
}

/// Verify rule consistency (for testing)
pub fn verify_bulk_rules() -> Result<(), String> {
    // Verify no duplicate rule IDs
    let mut seen = std::collections::HashSet::new();
    for (rule_id, _) in BULK_RULES {
        if !seen.insert(rule_id) {
            return Err(format!("Duplicate rule ID: {rule_id}"));
        }
    }

    // Verify expected count exactly matches Buf
    let expected_count = 69; // Exactly matching Buf's breaking rule count
    let actual_count = BULK_RULES.len();
    if actual_count != expected_count {
        return Err(format!(
            "Expected {expected_count} rules (Buf exact), but found {actual_count}"
        ));
    }

    Ok(())
}
