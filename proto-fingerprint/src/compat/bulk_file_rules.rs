//! Bulk-generated file option rules using macro magic
//!
//! This module uses macros to generate all FILE_SAME_* option rules in one go,
//! drastically reducing code duplication and implementation time.

use crate::canonical::CanonicalFile;
use crate::compat::handlers::{create_breaking_change, create_location};
use crate::compat::types::{RuleContext, RuleResult};

// ========================================
// MACRO MAGIC: Generate Rules in Bulk
// ========================================

macro_rules! generate_file_option_rules {
    (
        $(
            ($fn_name:ident, $rule_id:literal, $field:ident, $field_type:tt, $default:expr)
        ),* $(,)?
    ) => {
        $(
            generate_file_option_rule!($fn_name, $rule_id, $field, $field_type, $default);
        )*
    };
}

macro_rules! generate_file_option_rule {
    ($fn_name:ident, $rule_id:literal, $field:ident, string, $default:expr) => {
        /// Auto-generated file option rule
        pub fn $fn_name(
            current: &CanonicalFile,
            previous: &CanonicalFile,
            context: &RuleContext,
        ) -> RuleResult {
            let previous_value = previous.$field.as_deref().unwrap_or($default);
            let current_value = current.$field.as_deref().unwrap_or($default);

            if previous_value != current_value {
                let option_name = stringify!($field);
                let change = create_breaking_change(
                    $rule_id,
                    format!(
                        "File option \"{}\" changed from \"{}\" to \"{}\".",
                        option_name, previous_value, current_value
                    ),
                    create_location(&context.current_file, "file", ""),
                    Some(create_location(
                        context.previous_file.as_deref().unwrap_or(""),
                        "file",
                        "",
                    )),
                    vec!["FILE".to_string()],
                );
                return RuleResult::with_changes(vec![change]);
            }

            RuleResult::success()
        }
    };

    ($fn_name:ident, $rule_id:literal, $field:ident, bool, $default:expr) => {
        /// Auto-generated file option rule (boolean)
        pub fn $fn_name(
            current: &CanonicalFile,
            previous: &CanonicalFile,
            context: &RuleContext,
        ) -> RuleResult {
            let previous_value = previous.$field.unwrap_or($default);
            let current_value = current.$field.unwrap_or($default);

            if previous_value != current_value {
                let option_name = stringify!($field);
                let change = create_breaking_change(
                    $rule_id,
                    format!(
                        "File option \"{}\" changed from \"{}\" to \"{}\".",
                        option_name, previous_value, current_value
                    ),
                    create_location(&context.current_file, "file", ""),
                    Some(create_location(
                        context.previous_file.as_deref().unwrap_or(""),
                        "file",
                        "",
                    )),
                    vec!["FILE".to_string()],
                );
                return RuleResult::with_changes(vec![change]);
            }

            RuleResult::success()
        }
    };
}

// ========================================
// BATCH GENERATION: All 13 File Option Rules at Once!
// ========================================

generate_file_option_rules! {
    // Language-specific package options
    (check_file_same_go_package, "FILE_SAME_GO_PACKAGE", go_package, string, ""),
    (check_file_same_java_package, "FILE_SAME_JAVA_PACKAGE", java_package, string, ""),
    (check_file_same_csharp_namespace, "FILE_SAME_CSHARP_NAMESPACE", csharp_namespace, string, ""),
    (check_file_same_ruby_package, "FILE_SAME_RUBY_PACKAGE", ruby_package, string, ""),

    // Java-specific options
    (check_file_same_java_multiple_files, "FILE_SAME_JAVA_MULTIPLE_FILES", java_multiple_files, bool, false),
    (check_file_same_java_outer_classname, "FILE_SAME_JAVA_OUTER_CLASSNAME", java_outer_classname, string, ""),
    (check_file_same_java_string_check_utf8, "FILE_SAME_JAVA_STRING_CHECK_UTF8", java_string_check_utf8, bool, false),

    // Other language prefixes
    (check_file_same_objc_class_prefix, "FILE_SAME_OBJC_CLASS_PREFIX", objc_class_prefix, string, ""),
    (check_file_same_php_class_prefix, "FILE_SAME_PHP_CLASS_PREFIX", php_class_prefix, string, ""),
    (check_file_same_php_namespace, "FILE_SAME_PHP_NAMESPACE", php_namespace, string, ""),
    (check_file_same_php_metadata_namespace, "FILE_SAME_PHP_METADATA_NAMESPACE", php_metadata_namespace, string, ""),
    (check_file_same_swift_prefix, "FILE_SAME_SWIFT_PREFIX", swift_prefix, string, ""),
}

// ========================================
// Special Rules (Not macro-generated due to unique logic)
// ========================================

/// FILE_SAME_SYNTAX rule with special proto2/proto3/editions handling
pub fn check_file_same_syntax(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    // Default to "proto2" if not specified
    let previous_syntax = if previous.syntax.is_empty() {
        "proto2"
    } else {
        &previous.syntax
    };
    let current_syntax = if current.syntax.is_empty() {
        "proto2"
    } else {
        &current.syntax
    };

    if previous_syntax != current_syntax {
        let change = create_breaking_change(
            "FILE_SAME_SYNTAX",
            format!("File syntax changed from \"{previous_syntax}\" to \"{current_syntax}\"."),
            create_location(&context.current_file, "file", ""),
            Some(create_location(
                context.previous_file.as_deref().unwrap_or(""),
                "file",
                "",
            )),
            vec![
                "FILE".to_string(),
                "WIRE_JSON".to_string(),
                "WIRE".to_string(),
            ],
        );
        return RuleResult::with_changes(vec![change]);
    }

    RuleResult::success()
}

/// FILE_NO_DELETE rule - checks entire files aren't deleted from the file set
/// Note: Single-file implementation - detects complete file emptying as potential deletion indicator
pub fn check_file_no_delete(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    let mut changes = Vec::new();

    // Check if file went from having content to being essentially empty
    let prev_has_content = !previous.messages.is_empty()
        || !previous.enums.is_empty()
        || !previous.services.is_empty()
        || !previous.extensions.is_empty();

    let curr_has_content = !current.messages.is_empty()
        || !current.enums.is_empty()
        || !current.services.is_empty()
        || !current.extensions.is_empty();

    // If previous file had content but current file is empty,
    // this could indicate file deletion or complete clearing
    if prev_has_content && !curr_has_content {
        // Additional check: if package also disappeared, stronger signal of deletion
        let prev_package = previous.package.as_deref().unwrap_or("");
        let curr_package = current.package.as_deref().unwrap_or("");

        if !prev_package.is_empty() && curr_package.is_empty() {
            changes.push(create_breaking_change(
                "FILE_NO_DELETE",
                "File appears to have been deleted (all content and package declaration removed)."
                    .to_string(),
                create_location(&context.current_file, "file", &context.current_file),
                Some(create_location(
                    context.previous_file.as_deref().unwrap_or(""),
                    "file",
                    context.previous_file.as_deref().unwrap_or(""),
                )),
                vec!["FILE".to_string()],
            ));
        }
    }

    // Note: True FILE_NO_DELETE detection requires multi-file project analysis.
    // This single-file implementation can only detect certain patterns.

    RuleResult::with_changes(changes)
}

/// FILE_SAME_OPTIMIZE_FOR rule - checks file optimize_for option doesn't change
pub fn check_file_same_optimize_for(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    if current.optimize_for != previous.optimize_for {
        RuleResult::with_changes(vec![create_breaking_change(
            "FILE_SAME_OPTIMIZE_FOR",
            format!(
                "File optimize_for changed from {:?} to {:?}.",
                previous.optimize_for, current.optimize_for
            ),
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

/// FILE_SAME_PACKAGE rule - checks file package doesn't change
pub fn check_file_same_package(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    if current.package != previous.package {
        RuleResult::with_changes(vec![create_breaking_change(
            "FILE_SAME_PACKAGE",
            format!(
                "File package changed from \"{}\" to \"{}\".",
                previous.package.as_deref().unwrap_or(""),
                current.package.as_deref().unwrap_or("")
            ),
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

/// FILE_SAME_CC_GENERIC_SERVICES rule - checks cc_generic_services option doesn't change
pub fn check_file_same_cc_generic_services(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    if current.cc_generic_services != previous.cc_generic_services {
        RuleResult::with_changes(vec![create_breaking_change(
            "FILE_SAME_CC_GENERIC_SERVICES",
            format!(
                "File cc_generic_services changed from {:?} to {:?}.",
                previous.cc_generic_services, current.cc_generic_services
            ),
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

/// FILE_SAME_CC_ENABLE_ARENAS - checks CC enable arenas option doesn't change
pub fn check_file_same_cc_enable_arenas(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    if current.cc_enable_arenas != previous.cc_enable_arenas {
        RuleResult::with_changes(vec![create_breaking_change(
            "FILE_SAME_CC_ENABLE_ARENAS",
            format!(
                "File cc_enable_arenas changed from {:?} to {:?}.",
                previous.cc_enable_arenas, current.cc_enable_arenas
            ),
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

/// FILE_SAME_JAVA_GENERIC_SERVICES - checks Java generic services option doesn't change
pub fn check_file_same_java_generic_services(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    if current.java_generic_services != previous.java_generic_services {
        RuleResult::with_changes(vec![create_breaking_change(
            "FILE_SAME_JAVA_GENERIC_SERVICES",
            format!(
                "File java_generic_services changed from {:?} to {:?}.",
                previous.java_generic_services, current.java_generic_services
            ),
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

/// FILE_SAME_PHP_GENERIC_SERVICES - checks PHP generic services option doesn't change  
pub fn check_file_same_php_generic_services(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    if current.php_generic_services != previous.php_generic_services {
        RuleResult::with_changes(vec![create_breaking_change(
            "FILE_SAME_PHP_GENERIC_SERVICES",
            format!(
                "File php_generic_services changed from {:?} to {:?}.",
                previous.php_generic_services, current.php_generic_services
            ),
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

/// FILE_SAME_PY_GENERIC_SERVICES - checks Python generic services option doesn't change
pub fn check_file_same_py_generic_services(
    current: &CanonicalFile,
    previous: &CanonicalFile,
    context: &RuleContext,
) -> RuleResult {
    if current.py_generic_services != previous.py_generic_services {
        RuleResult::with_changes(vec![create_breaking_change(
            "FILE_SAME_PY_GENERIC_SERVICES",
            format!(
                "File py_generic_services changed from {:?} to {:?}.",
                previous.py_generic_services, current.py_generic_services
            ),
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

// ========================================
// Rule Export Table for Bulk Registration
// ========================================

pub const FILE_OPTION_RULES: &[crate::compat::types::RuleEntry] = &[
    // Generated rules
    ("FILE_SAME_GO_PACKAGE", check_file_same_go_package),
    ("FILE_SAME_JAVA_PACKAGE", check_file_same_java_package),
    (
        "FILE_SAME_CSHARP_NAMESPACE",
        check_file_same_csharp_namespace,
    ),
    ("FILE_SAME_RUBY_PACKAGE", check_file_same_ruby_package),
    (
        "FILE_SAME_JAVA_MULTIPLE_FILES",
        check_file_same_java_multiple_files,
    ),
    (
        "FILE_SAME_JAVA_OUTER_CLASSNAME",
        check_file_same_java_outer_classname,
    ),
    (
        "FILE_SAME_JAVA_STRING_CHECK_UTF8",
        check_file_same_java_string_check_utf8,
    ),
    (
        "FILE_SAME_OBJC_CLASS_PREFIX",
        check_file_same_objc_class_prefix,
    ),
    (
        "FILE_SAME_PHP_CLASS_PREFIX",
        check_file_same_php_class_prefix,
    ),
    ("FILE_SAME_PHP_NAMESPACE", check_file_same_php_namespace),
    (
        "FILE_SAME_PHP_METADATA_NAMESPACE",
        check_file_same_php_metadata_namespace,
    ),
    ("FILE_SAME_SWIFT_PREFIX", check_file_same_swift_prefix),
    // Special rules
    ("FILE_SAME_SYNTAX", check_file_same_syntax),
    ("FILE_NO_DELETE", check_file_no_delete),
];
