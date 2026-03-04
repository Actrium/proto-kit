use serde::Serialize;

// Note: Using BTreeSet for sorted, unique collections.
// This requires `Ord` to be derived.
use std::collections::{BTreeMap, BTreeSet};

//==============================================================================
// Reserved Types for Breaking Change Detection
//==============================================================================

/// Represents a range of reserved numbers (for fields or enum values).
#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReservedRange {
    pub start: i32,
    pub end: i32,
}

/// Represents a reserved name.
#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReservedName {
    pub name: String,
}

//==============================================================================
// Structs for Exact Semantic Fingerprinting
//==============================================================================

/// Represents the semantically significant content of a .proto file.
#[derive(Debug, Default, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct CanonicalFile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub syntax: String, // "proto2", "proto3", "editions"
    #[serde(skip_serializing_if = "BTreeSet::is_empty")]
    pub imports: BTreeSet<String>,
    #[serde(skip_serializing_if = "BTreeSet::is_empty")]
    pub messages: BTreeSet<CanonicalMessage>,
    #[serde(skip_serializing_if = "BTreeSet::is_empty")]
    pub enums: BTreeSet<CanonicalEnum>,
    #[serde(skip_serializing_if = "BTreeSet::is_empty")]
    pub services: BTreeSet<CanonicalService>,
    #[serde(skip_serializing_if = "BTreeSet::is_empty")]
    pub extensions: BTreeSet<CanonicalExtension>, // Extension field definitions

    // ========================================
    // File Options - Complete Set for All Rules
    // ========================================

    // Language-specific package options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub go_package: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub java_package: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub csharp_namespace: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ruby_package: Option<String>,

    // Java-specific options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub java_multiple_files: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub java_outer_classname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub java_string_check_utf8: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub java_generic_services: Option<bool>,

    // Objective-C options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub objc_class_prefix: Option<String>,

    // PHP options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub php_class_prefix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub php_namespace: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub php_metadata_namespace: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub php_generic_services: Option<bool>, // Deprecated

    // Swift options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub swift_prefix: Option<String>,

    // C++ options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cc_generic_services: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cc_enable_arenas: Option<bool>,

    // Python options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub py_generic_services: Option<bool>,

    // Optimization options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optimize_for: Option<String>, // "SPEED", "CODE_SIZE", "LITE_RUNTIME"
}

/// Represents a Protobuf message.
#[derive(Debug, Default, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct CanonicalMessage {
    pub name: String,
    #[serde(skip_serializing_if = "BTreeSet::is_empty")]
    pub fields: BTreeSet<CanonicalField>,
    #[serde(skip_serializing_if = "BTreeSet::is_empty")]
    pub nested_messages: BTreeSet<CanonicalMessage>,
    #[serde(skip_serializing_if = "BTreeSet::is_empty")]
    pub nested_enums: BTreeSet<CanonicalEnum>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub oneofs: Vec<String>,
    #[serde(skip_serializing_if = "BTreeSet::is_empty")]
    pub reserved_ranges: BTreeSet<ReservedRange>,
    #[serde(skip_serializing_if = "BTreeSet::is_empty")]
    pub reserved_names: BTreeSet<ReservedName>,
    #[serde(skip_serializing_if = "BTreeSet::is_empty")]
    pub extension_ranges: BTreeSet<ReservedRange>, // Extensions use same range format

    // Message-level options for breaking change rules
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_set_wire_format: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_standard_descriptor_accessor: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<bool>,
}

/// Represents a field within a Protobuf message.
/// The sort order is primarily by field number.
#[derive(Debug, Default, Serialize, PartialEq, Eq)]
pub struct CanonicalField {
    pub name: String,
    pub number: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>, // "optional", "required", "repeated"
    pub type_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oneof_index: Option<i32>,

    // ========================================
    // Field Options - Complete Set for All Rules
    // ========================================

    // Basic field attributes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json_name: Option<String>,

    // Type-specific options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jstype: Option<String>, // "JS_NORMAL", "JS_STRING", "JS_NUMBER"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ctype: Option<String>, // Deprecated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpp_string_type: Option<String>, // Replacement for ctype

    // Validation options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub utf8_validation: Option<String>, // "VERIFY", "NONE"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub java_utf8_validation: Option<bool>,

    // Field state
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weak: Option<bool>,

    // Generic options map for any unrecognized options
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub options: BTreeMap<String, String>,
}

// Custom implementation of Ord for CanonicalField to sort by `number` first.
impl Ord for CanonicalField {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.number
            .cmp(&other.number)
            .then_with(|| self.name.cmp(&other.name))
    }
}

impl PartialOrd for CanonicalField {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Represents a Protobuf enum.
#[derive(Debug, Default, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct CanonicalEnum {
    pub name: String,
    #[serde(skip_serializing_if = "BTreeSet::is_empty")]
    pub values: BTreeSet<CanonicalEnumValue>,
    #[serde(skip_serializing_if = "BTreeSet::is_empty")]
    pub reserved_ranges: BTreeSet<ReservedRange>,
    #[serde(skip_serializing_if = "BTreeSet::is_empty")]
    pub reserved_names: BTreeSet<ReservedName>,

    // Enum-level options for breaking change rules
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_alias: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub closed_enum: Option<bool>, // For editions/proto3 open vs closed

    // Generic options map
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub options: BTreeMap<String, String>,
}

/// Represents a single value within a Protobuf enum.
/// The sort order is primarily by number.
#[derive(Debug, Default, Serialize, PartialEq, Eq)]
pub struct CanonicalEnumValue {
    pub name: String,
    pub number: i32,
}

/// Represents a protobuf extension field definition.
/// Extensions are fields that extend existing messages.
#[derive(Debug, Default, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct CanonicalExtension {
    pub name: String,
    pub number: i32,
    pub extendee: String, // The message being extended (fully qualified)
    pub type_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>, // "optional", "required", "repeated"

    // Extension options (similar to field options)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<bool>,
}

// Custom implementation of Ord for CanonicalEnumValue to sort by `number` first.
impl Ord for CanonicalEnumValue {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.number
            .cmp(&other.number)
            .then_with(|| self.name.cmp(&other.name))
    }
}

impl PartialOrd for CanonicalEnumValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Represents a Protobuf service.
#[derive(Debug, Default, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct CanonicalService {
    pub name: String,
    #[serde(skip_serializing_if = "BTreeSet::is_empty")]
    pub methods: BTreeSet<CanonicalMethod>,
}

/// Represents a method within a service.
#[derive(Debug, Default, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct CanonicalMethod {
    pub name: String,
    pub input_type: String,
    pub output_type: String,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub client_streaming: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub server_streaming: bool,

    // Method options for breaking change rules
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_level: Option<String>, // "NO_SIDE_EFFECTS", "IDEMPOTENT", "UNKNOWN"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<bool>,
}
