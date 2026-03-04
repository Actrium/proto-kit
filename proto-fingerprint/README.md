# Proto-Sign

A Rust tool for Protocol Buffers compatibility checking and semantic fingerprinting.

## Features

- **Breaking Change Detection**: Comprehensive rule-based checking for protobuf compatibility
- **Semantic Fingerprinting**: Generate stable fingerprints that ignore formatting changes
- **Buf Compatible**: Same rules and behavior as Buf's breaking change system
- **Flexible Configuration**: YAML-based configuration with predefined templates

## Installation

```bash
# Build from source
git clone https://github.com/your-org/proto-sign.git
cd proto-sign
cargo install --path .

# Extract test configurations (for development)
bash ./compat-configs/extract_buf_configs.sh
```

## Usage

### Breaking Change Detection

```bash
# Basic breaking change check
proto-sign breaking old.proto new.proto

# JSON output
proto-sign breaking old.proto new.proto --format json

# Use specific rule categories
proto-sign breaking old.proto new.proto --use-categories FILE,WIRE
```

### Quick Compatibility Check

```bash
# Three-level compatibility assessment (Green/Yellow/Red)
proto-sign compare old.proto new.proto
```

### Semantic Fingerprinting

```bash
# Generate semantic fingerprint
proto-sign fingerprint file.proto
```

## Configuration

Proto-Sign uses YAML configuration files. Copy a template to get started:

```bash
# Choose a configuration template
cp compat-configs/examples/strict-mode.yaml proto-sign.yaml

# Use custom configuration
proto-sign breaking old.proto new.proto --config proto-sign.yaml
```

### Configuration Templates

- **`strict-mode.yaml`** - All rule categories (recommended for public APIs)
- **`lenient-mode.yaml`** - Balanced mode for internal APIs  
- **`wire-only.yaml`** - Binary compatibility only
- **`specific-rules.yaml`** - Custom rule selection

### Configuration Format

```yaml
version: v1
breaking:
  use_categories:
    - FILE
    - PACKAGE
    - WIRE
    - WIRE_JSON
  except_rules:
    - FIELD_SAME_JSON_NAME
  ignore:
    - "generated/**"
  ignore_unstable_packages: true
```

## Rule Categories

- **FILE** - File-level changes (deletions, package changes)
- **PACKAGE** - Package-level changes (message/service deletions)
- **WIRE** - Binary encoding compatibility
- **WIRE_JSON** - JSON serialization compatibility

## Library Usage

```rust
use proto_sign::spec::{Spec, Compatibility};

let old_spec = Spec::try_from(old_proto_content)?;
let new_spec = Spec::try_from(new_proto_content)?;

match old_spec.compare_with(&new_spec) {
    Compatibility::Green => println!("No changes"),
    Compatibility::Yellow => println!("Backward compatible"),
    Compatibility::Red => println!("Breaking changes detected"),
}

// Detailed analysis
let result = old_spec.check_breaking_changes(&new_spec);
for change in result.changes {
    println!("{}: {}", change.rule_id, change.message);
}
```

## License

Apache License 2.0

## Acknowledgments

Breaking change detection rules ported from [Buf](https://github.com/bufbuild/buf).