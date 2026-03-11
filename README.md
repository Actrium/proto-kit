# proto-kit

`proto-kit` is a Rust workspace for Protocol Buffers tooling.

It currently contains two CLI tools:

- `proto-fmt`: normalize, merge, inspect, and re-render `.proto` files.
- `proto-fingerprint`: compare protobuf schema changes and compute semantic fingerprints.

## Workspace Layout

- `proto-fmt/`: protobuf formatting and normalization utilities
- `proto-fingerprint/`: compatibility checking and fingerprint generation

## Build

```bash
cargo build
```

## Quick Start

Run the formatter on a single file:

```bash
cargo run -p proto-fmt -- normalize path/to/file.proto
```

Check compatibility between two schema versions:

```bash
cargo run -p proto-fingerprint -- compare old.proto new.proto
```

Generate a semantic fingerprint:

```bash
cargo run -p proto-fingerprint -- fingerprint path/to/file.proto
```

## More Details

Each tool has its own README:

- [proto-fmt](./proto-fmt/README.md)
- [proto-fingerprint](./proto-fingerprint/README.md)

## License

Apache-2.0
