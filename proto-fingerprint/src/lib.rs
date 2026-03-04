pub mod canonical;
pub mod compat;
pub mod compatibility;
pub mod normalize;
pub mod spec;
pub mod test_bulk_count;

pub use spec::{Compatibility, Spec};

use anyhow::Context;
use protobuf_parse::Parser;
use sha2::{Digest, Sha256};

/// Generates a semantic fingerprint for a given Protobuf file content.
///
/// The fingerprint is a SHA-256 hash of the file's canonical, semantic
/// representation. This means it is insensitive to changes in comments,
/// formatting, or ordering of elements.
///
/// # Arguments
///
/// * `proto_content` - A string slice that holds the content of the .proto file.
///
/// # Returns
///
/// A `Result` containing the hex-encoded SHA-256 fingerprint string,
/// or an error if parsing or processing fails.
pub fn generate_fingerprint(proto_content: &str) -> anyhow::Result<String> {
    // The parser works with the filesystem, so we need to create a temporary
    // directory and file to hold the content.
    let temp_dir = tempfile::tempdir().context("Failed to create temp directory")?;
    let file_name = "input.proto";
    let temp_path = temp_dir.path().join(file_name);
    std::fs::write(&temp_path, proto_content).context("Failed to write to temp file")?;

    // To handle imports correctly without needing the entire dependency tree,
    // we create dummy files for each imported `.proto` file.
    for line in proto_content.lines() {
        if line.trim().starts_with("import ") {
            let path_str = line
                .trim()
                .trim_start_matches("import ")
                .trim_start_matches("public ")
                .trim_matches(|c| c == '"' || c == ';');

            // The parser has built-in knowledge of standard google.protobuf types.
            // If we create a dummy file, it will override the built-in and fail
            // because our dummy file is empty. So, we only create dummies for
            // non-standard imports.
            if !path_str.starts_with("google/protobuf/") {
                let import_path = temp_dir.path().join(path_str);
                if let Some(parent) = import_path.parent() {
                    std::fs::create_dir_all(parent).context(format!(
                        "Failed to create parent dirs for import: {path_str}"
                    ))?;
                }
                std::fs::write(&import_path, "syntax = \"proto3\";")
                    .context(format!("Failed to create dummy import file: {path_str}"))?;
            }
        }
    }

    // 1. Parse the proto file using the public `Parser` API.
    let parsed = Parser::new()
        .pure()
        .include(temp_dir.path()) // Search for imports in the temp dir.
        .input(&temp_path) // The file to parse.
        .file_descriptor_set()
        .context("Protobuf parsing failed")?;

    // The result contains all parsed files, including imports. We need to find the
    // one corresponding to our input file.
    let file_descriptor = parsed
        .file
        .into_iter()
        .find(|d| d.name() == file_name)
        .context("Could not find the parsed file descriptor for the input file")?;

    // 2. Normalize the AST into our canonical representation.
    let canonical_file = normalize::normalize_file(&file_descriptor);

    // 3. Serialize the canonical representation to a stable JSON string.
    let json_string = serde_json::to_string_pretty(&canonical_file)
        .context("Failed to serialize canonical representation to JSON")?;

    // 4. Compute the SHA-256 hash of the JSON string.
    let mut hasher = Sha256::new();
    hasher.update(json_string.as_bytes());
    let hash_result = hasher.finalize();

    // 5. Format as a hex string and return.
    Ok(format!("{hash_result:x}"))
}
