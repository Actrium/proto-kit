//! Converts the raw `FileDescriptorProto` AST from the `protobuf` crate
//! into a simplified, serializable `CanonicalFile` representation.

use crate::canonical::{
    CanonicalEnum, CanonicalEnumValue, CanonicalExtension, CanonicalField, CanonicalFile,
    CanonicalMessage, CanonicalMethod, CanonicalService, ReservedName, ReservedRange,
};
use crate::compatibility::{
    CompatibilityField, CompatibilityMessage, CompatibilityMethod, CompatibilityModel,
    CompatibilityService,
};
use protobuf::descriptor::{
    DescriptorProto, EnumDescriptorProto, EnumValueDescriptorProto, FieldDescriptorProto,
    FileDescriptorProto, MethodDescriptorProto, ServiceDescriptorProto, field_descriptor_proto,
};

pub fn normalize_file(file: &FileDescriptorProto) -> CanonicalFile {
    let mut canonical_file = CanonicalFile {
        package: file.package.clone(),
        ..Default::default()
    };

    // Extract syntax - defaults to "proto2" if not specified
    canonical_file.syntax = file.syntax.clone().unwrap_or_else(|| "proto2".to_string());

    for import in file.dependency.iter() {
        canonical_file.imports.insert(import.clone());
    }

    for msg in file.message_type.iter() {
        canonical_file.messages.insert(normalize_message(msg));
    }

    for en in file.enum_type.iter() {
        canonical_file.enums.insert(normalize_enum(en));
    }

    for svc in file.service.iter() {
        canonical_file.services.insert(normalize_service(svc));
    }

    // Extract extension field definitions
    for ext in file.extension.iter() {
        canonical_file.extensions.insert(normalize_extension(ext));
    }

    // Extract file options
    if let Some(file_options) = file.options.as_ref() {
        if file_options.has_go_package() {
            canonical_file.go_package = Some(file_options.go_package().to_string());
        }
        if file_options.has_java_package() {
            canonical_file.java_package = Some(file_options.java_package().to_string());
        }
        if file_options.has_csharp_namespace() {
            canonical_file.csharp_namespace = Some(file_options.csharp_namespace().to_string());
        }
        if file_options.has_java_multiple_files() {
            canonical_file.java_multiple_files = Some(file_options.java_multiple_files());
        }
        if file_options.has_java_outer_classname() {
            canonical_file.java_outer_classname =
                Some(file_options.java_outer_classname().to_string());
        }
        if file_options.has_java_string_check_utf8() {
            canonical_file.java_string_check_utf8 = Some(file_options.java_string_check_utf8());
        }
        if file_options.has_objc_class_prefix() {
            canonical_file.objc_class_prefix = Some(file_options.objc_class_prefix().to_string());
        }
        if file_options.has_php_class_prefix() {
            canonical_file.php_class_prefix = Some(file_options.php_class_prefix().to_string());
        }
        if file_options.has_php_namespace() {
            canonical_file.php_namespace = Some(file_options.php_namespace().to_string());
        }
        if file_options.has_php_metadata_namespace() {
            canonical_file.php_metadata_namespace =
                Some(file_options.php_metadata_namespace().to_string());
        }
        if file_options.has_ruby_package() {
            canonical_file.ruby_package = Some(file_options.ruby_package().to_string());
        }
        if file_options.has_swift_prefix() {
            canonical_file.swift_prefix = Some(file_options.swift_prefix().to_string());
        }
        if file_options.has_optimize_for() {
            canonical_file.optimize_for = Some(format!("{:?}", file_options.optimize_for()));
        }
        if file_options.has_cc_generic_services() {
            canonical_file.cc_generic_services = Some(file_options.cc_generic_services());
        }
        if file_options.has_java_generic_services() {
            canonical_file.java_generic_services = Some(file_options.java_generic_services());
        }
        if file_options.has_py_generic_services() {
            canonical_file.py_generic_services = Some(file_options.py_generic_services());
        }
        if file_options.has_php_generic_services() {
            canonical_file.php_generic_services = Some(file_options.php_generic_services());
        }
        if file_options.has_cc_enable_arenas() {
            canonical_file.cc_enable_arenas = Some(file_options.cc_enable_arenas());
        }
    }

    canonical_file
}

fn normalize_message(msg: &DescriptorProto) -> CanonicalMessage {
    let mut canonical_msg = CanonicalMessage {
        name: msg.name().to_string(),
        ..Default::default()
    };

    // Collect oneof names
    for oneof_decl in msg.oneof_decl.iter() {
        canonical_msg.oneofs.push(oneof_decl.name().to_string());
    }

    for field in msg.field.iter() {
        canonical_msg.fields.insert(normalize_field(field));
    }

    for nested in msg.nested_type.iter() {
        canonical_msg
            .nested_messages
            .insert(normalize_message(nested));
    }

    for nested_enum in msg.enum_type.iter() {
        canonical_msg
            .nested_enums
            .insert(normalize_enum(nested_enum));
    }

    // Extract reserved ranges
    for reserved_range in msg.reserved_range.iter() {
        canonical_msg.reserved_ranges.insert(ReservedRange {
            start: reserved_range.start(),
            end: reserved_range.end(),
        });
    }

    // Extract reserved names
    for reserved_name in msg.reserved_name.iter() {
        canonical_msg.reserved_names.insert(ReservedName {
            name: reserved_name.clone(),
        });
    }

    // Extract extension ranges
    for extension_range in msg.extension_range.iter() {
        canonical_msg.extension_ranges.insert(ReservedRange {
            start: extension_range.start(),
            end: extension_range.end(),
        });
    }

    // Extract message options
    if let Some(msg_options) = msg.options.as_ref() {
        if msg_options.has_message_set_wire_format() {
            canonical_msg.message_set_wire_format = Some(msg_options.message_set_wire_format());
        }
        if msg_options.has_no_standard_descriptor_accessor() {
            canonical_msg.no_standard_descriptor_accessor =
                Some(msg_options.no_standard_descriptor_accessor());
        }
        if msg_options.has_deprecated() {
            canonical_msg.deprecated = Some(msg_options.deprecated());
        }
    }

    canonical_msg
}

fn normalize_field(field: &FieldDescriptorProto) -> CanonicalField {
    let label = match field.label() {
        field_descriptor_proto::Label::LABEL_OPTIONAL => "optional",
        field_descriptor_proto::Label::LABEL_REQUIRED => "required",
        field_descriptor_proto::Label::LABEL_REPEATED => "repeated",
    };

    // For primitive types, `type_name` is empty and `type` is set.
    // For message/enum types, `type_name` is set and `type` is TYPE_MESSAGE/TYPE_ENUM.
    let type_name = if field.type_name().is_empty() {
        format!("{:?}", field.type_())
            .to_lowercase()
            .replace("type_", "")
    } else {
        // Keep the fully qualified name for message/enum types.
        field.type_name().to_string()
    };

    // Extract field options
    let mut options = std::collections::BTreeMap::new();
    let mut default = None;
    let mut json_name_opt = None;
    let mut jstype = None;
    let mut ctype = None;
    let cpp_string_type = None;
    let utf8_validation = None;
    let java_utf8_validation = None;
    let mut deprecated = None;
    let mut weak = None;

    if let Some(field_options) = field.options.as_ref() {
        // Extract ctype option
        if field_options.has_ctype() {
            let ctype_name = match field_options.ctype() {
                protobuf::descriptor::field_options::CType::STRING => "STRING",
                protobuf::descriptor::field_options::CType::CORD => "CORD",
                protobuf::descriptor::field_options::CType::STRING_PIECE => "STRING_PIECE",
            };
            options.insert("ctype".to_string(), ctype_name.to_string());
            ctype = Some(ctype_name.to_string());
        }

        // Extract jstype option
        if field_options.has_jstype() {
            let jstype_name = match field_options.jstype() {
                protobuf::descriptor::field_options::JSType::JS_NORMAL => "JS_NORMAL",
                protobuf::descriptor::field_options::JSType::JS_STRING => "JS_STRING",
                protobuf::descriptor::field_options::JSType::JS_NUMBER => "JS_NUMBER",
            };
            options.insert("jstype".to_string(), jstype_name.to_string());
            jstype = Some(jstype_name.to_string());
        }

        // Extract packed option
        if field_options.has_packed() {
            options.insert("packed".to_string(), field_options.packed().to_string());
        }

        // Extract deprecated option
        if field_options.has_deprecated() {
            options.insert(
                "deprecated".to_string(),
                field_options.deprecated().to_string(),
            );
            deprecated = Some(field_options.deprecated());
        }

        // Extract weak option
        if field_options.has_weak() {
            options.insert("weak".to_string(), field_options.weak().to_string());
            weak = Some(field_options.weak());
        }

        // Extract UTF8 validation options (for string/bytes fields)
        // Note: This might be available through uninterpreted_option for editions/proto3
        // For java_string_check_utf8, check file-level option

        // Note: default_value and json_name are on FieldDescriptorProto itself, not in options
        // We'll handle these at the field level instead
    }

    // Extract field-level options from FieldDescriptorProto
    if field.has_default_value() {
        let v = field.default_value().to_string();
        options.insert("default".to_string(), v.clone());
        default = Some(v);
    }

    if field.has_json_name() {
        let v = field.json_name().to_string();
        options.insert("json_name".to_string(), v.clone());
        json_name_opt = Some(v);
    }

    CanonicalField {
        name: field.name().to_string(),
        number: field.number(),
        // In proto3, optional is the default and not explicitly stated,
        // so we can consider omitting it for a cleaner canonical form.
        // However, for full semantic representation, it's better to keep it.
        // Let's remove it if it is the default 'optional'.
        label: if label == "optional" {
            None
        } else {
            Some(label.to_string())
        },
        type_name,
        oneof_index: field.oneof_index,
        // normalized fast-paths
        default,
        json_name: json_name_opt,
        jstype,
        ctype,
        cpp_string_type,
        utf8_validation,
        java_utf8_validation,
        deprecated,
        weak,
        // raw options snapshot
        options,
    }
}

fn normalize_enum(en: &EnumDescriptorProto) -> CanonicalEnum {
    let mut canonical_enum = CanonicalEnum {
        name: en.name().to_string(),
        ..Default::default()
    };

    for value in en.value.iter() {
        canonical_enum.values.insert(normalize_enum_value(value));
    }

    // Extract enum options (like json_format)
    let mut options = std::collections::BTreeMap::new();
    if let Some(enum_options) = en.options.as_ref() {
        // Extract standard enum options
        if enum_options.has_allow_alias() {
            options.insert(
                "allow_alias".to_string(),
                enum_options.allow_alias().to_string(),
            );
        }
        if enum_options.has_deprecated() {
            options.insert(
                "deprecated".to_string(),
                enum_options.deprecated().to_string(),
            );
        }

        // Extract custom options from uninterpreted_option
        for uninterpreted in &enum_options.uninterpreted_option {
            if let Some(name_part) = uninterpreted.name.first() {
                if name_part.has_name_part() {
                    let name = name_part.name_part();
                    // Handle Protobuf Editions features
                    if name == "features.json_format" {
                        if let Some(identifier_value) = uninterpreted.identifier_value.as_ref() {
                            options.insert("json_format".to_string(), identifier_value.clone());
                        }
                    } else {
                        // Handle other custom options
                        if let Some(string_value) = uninterpreted.string_value.as_ref() {
                            if let Ok(string_val) = String::from_utf8(string_value.clone()) {
                                options.insert(name.to_string(), string_val);
                            }
                        } else if let Some(identifier_value) =
                            uninterpreted.identifier_value.as_ref()
                        {
                            options.insert(name.to_string(), identifier_value.clone());
                        }
                    }
                }
            }
        }
    }
    canonical_enum.options = options;

    // Extract reserved ranges for enum values
    for reserved_range in en.reserved_range.iter() {
        canonical_enum.reserved_ranges.insert(ReservedRange {
            start: reserved_range.start(),
            end: reserved_range.end(),
        });
    }

    // Extract reserved names for enum values
    for reserved_name in en.reserved_name.iter() {
        canonical_enum.reserved_names.insert(ReservedName {
            name: reserved_name.clone(),
        });
    }

    // Extract enum-level options
    if let Some(enum_options) = en.options.as_ref() {
        if enum_options.has_allow_alias() {
            canonical_enum.allow_alias = Some(enum_options.allow_alias());
        }
        if enum_options.has_deprecated() {
            canonical_enum.deprecated = Some(enum_options.deprecated());
        }
    }

    canonical_enum
}

fn normalize_enum_value(val: &EnumValueDescriptorProto) -> CanonicalEnumValue {
    CanonicalEnumValue {
        name: val.name().to_string(),
        number: val.number(),
    }
}

fn normalize_service(svc: &ServiceDescriptorProto) -> CanonicalService {
    let mut canonical_svc = CanonicalService {
        name: svc.name().to_string(),
        ..Default::default()
    };

    for method in svc.method.iter() {
        canonical_svc.methods.insert(normalize_method(method));
    }

    canonical_svc
}

fn normalize_method(method: &MethodDescriptorProto) -> CanonicalMethod {
    let mut m = CanonicalMethod {
        name: method.name().to_string(),
        input_type: method.input_type().to_string(),
        output_type: method.output_type().to_string(),
        client_streaming: method.client_streaming(),
        server_streaming: method.server_streaming(),
        idempotency_level: None,
        deprecated: None,
    };

    if let Some(options) = method.options.as_ref() {
        if options.has_idempotency_level() {
            m.idempotency_level = Some(format!("{:?}", options.idempotency_level()));
        }
    }

    m
}

fn normalize_extension(ext: &FieldDescriptorProto) -> CanonicalExtension {
    let label = match ext.label() {
        field_descriptor_proto::Label::LABEL_OPTIONAL => "optional",
        field_descriptor_proto::Label::LABEL_REQUIRED => "required",
        field_descriptor_proto::Label::LABEL_REPEATED => "repeated",
    };

    // For primitive types, `type_name` is empty and `type` is set.
    // For message/enum types, `type_name` is set and `type` is TYPE_MESSAGE/TYPE_ENUM.
    let type_name = if ext.type_name().is_empty() {
        format!("{:?}", ext.type_())
            .to_lowercase()
            .replace("type_", "")
    } else {
        // Keep the fully qualified name for message/enum types.
        ext.type_name().to_string()
    };

    let mut default = None;
    let mut deprecated = None;

    if let Some(ext_options) = ext.options.as_ref() {
        if ext_options.has_deprecated() {
            deprecated = Some(ext_options.deprecated());
        }
    }

    // Extract default value if present
    if ext.has_default_value() {
        default = Some(ext.default_value().to_string());
    }

    CanonicalExtension {
        name: ext.name().to_string(),
        number: ext.number(),
        extendee: ext.extendee().to_string(), // The message being extended
        type_name,
        label: if label == "optional" {
            None
        } else {
            Some(label.to_string())
        },
        default,
        deprecated,
    }
}

//==============================================================================
// Normalization for Compatibility Fingerprinting
//==============================================================================

pub fn normalize_compatibility_file(file: &FileDescriptorProto) -> CompatibilityModel {
    let mut compat_model = CompatibilityModel::default();

    for msg in file.message_type.iter() {
        compat_model
            .messages
            .insert(normalize_compatibility_message(msg));
    }

    for svc in file.service.iter() {
        compat_model
            .services
            .insert(normalize_compatibility_service(svc));
    }

    compat_model
}

fn normalize_compatibility_message(msg: &DescriptorProto) -> CompatibilityMessage {
    let mut compat_msg = CompatibilityMessage {
        name: msg.name().to_string(),
        ..Default::default()
    };

    for field in msg.field.iter() {
        // For compatibility, we consider the presence of a field number and its type.
        // Removing any field is a breaking change. Changing a name or label is not.
        // Adding a new field is also not a breaking change, which is handled by comparing
        // the set of fields from an old version to a new one.
        compat_msg
            .fields
            .insert(normalize_compatibility_field(field));
    }

    // Note: We are intentionally not descending into nested messages here,
    // as their compatibility is handled when they are defined as top-level messages.

    compat_msg
}

fn normalize_compatibility_field(field: &FieldDescriptorProto) -> CompatibilityField {
    let type_name = if field.type_name().is_empty() {
        format!("{:?}", field.type_())
            .to_lowercase()
            .replace("type_", "")
    } else {
        field.type_name().to_string()
    };

    CompatibilityField {
        number: field.number(),
        type_name,
    }
}

fn normalize_compatibility_service(svc: &ServiceDescriptorProto) -> CompatibilityService {
    let mut compat_svc = CompatibilityService {
        name: svc.name().to_string(),
        ..Default::default()
    };

    for method in svc.method.iter() {
        compat_svc
            .methods
            .insert(normalize_compatibility_method(method));
    }

    compat_svc
}

fn normalize_compatibility_method(method: &MethodDescriptorProto) -> CompatibilityMethod {
    CompatibilityMethod {
        name: method.name().to_string(),
        input_type: method.input_type().to_string(),
        output_type: method.output_type().to_string(),
    }
}
