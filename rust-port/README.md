# Presidio Rust Port

A high-performance Rust port of Microsoft's Presidio Data Protection and PII De-identification SDK.

## Overview

Presidio is a context-aware, pluggable PII detection and anonymization SDK for text and images. This Rust port maintains full compatibility with the Python version while providing:

- **Performance**: Significantly faster processing through Rust's zero-cost abstractions
- **Safety**: Memory safety and thread safety guaranteed by the Rust compiler
- **Concurrency**: Native async/await support for high-throughput scenarios
- **Low Resource Usage**: Minimal memory footprint and CPU usage

## Project Structure

```
rust-port/
├── presidio-common/          # Shared types, traits, and utilities
├── presidio-analyzer/        # PII detection engine
├── presidio-anonymizer/      # PII transformation engine
├── presidio-image-redactor/  # Image PII redaction
├── presidio-structured/      # Structured data support (DataFrames, JSON)
└── presidio-cli/            # Command-line scanner tool
```

## Quick Start

### Prerequisites

- Rust 1.75 or later
- Cargo

### Building

```bash
# Build all crates
cargo build --release

# Build specific crate
cargo build --release -p presidio-analyzer

# Run tests
cargo test --workspace

# Run with all features
cargo build --all-features
```

### Running

```bash
# CLI tool
cargo run --bin presidio-cli -- /path/to/scan

# Run analyzer service
cargo run --bin presidio-analyzer-service

# Run anonymizer service
cargo run --bin presidio-anonymizer-service
```

## Features

### Analyzer

- **Context-aware PII detection** using NLP and regex patterns
- **50+ built-in recognizers** for various PII types
- **Multi-language support** (English, Spanish, French, German, Italian, Portuguese, etc.)
- **Extensible recognizer system** via traits
- **Country-specific recognizers** (US SSN, UK NHS, Indian Aadhaar, etc.)

### Anonymizer

- **Multiple transformation operators**: Replace, Redact, Mask, Hash, Encrypt
- **Conflict resolution** for overlapping entities
- **Reversible anonymization** with deanonymization support
- **Custom operators** via closures

### Image Redactor

- **OCR integration** (Tesseract, Azure Document Intelligence)
- **Standard image formats** (PNG, JPEG, etc.)
- **DICOM medical images** support

## Development

### Code Quality

```bash
# Format code
cargo fmt

# Lint code
cargo clippy -- -D warnings

# Security audit
cargo audit

# Generate documentation
cargo doc --no-deps --open
```

### Benchmarking

```bash
cargo bench
```

## Architecture

See [ARCHITECTURE.md](./docs/ARCHITECTURE.md) for detailed architecture documentation and interactive diagrams.

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md) for contribution guidelines.

## License

MIT License - see [LICENSE](../LICENSE) for details.

## Acknowledgments

This is a Rust port of Microsoft's [Presidio](https://github.com/microsoft/presidio) project, maintaining compatibility with the original Python implementation while leveraging Rust's performance and safety features.
