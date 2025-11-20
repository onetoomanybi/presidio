# Presidio Rust Examples

This directory contains practical examples demonstrating how to use Presidio Rust.

## Running Examples

```bash
# From the rust-port directory
cargo run --example <example_name>
```

## Available Examples

### 1. Simple Analysis

**File**: `simple_analysis.rs`

Demonstrates basic PII detection in text using the default analyzer.

```bash
cargo run --example simple_analysis
```

**Key Concepts**:
- Creating an analyzer engine
- Analyzing text for PII
- Interpreting results

### 2. Simple Anonymization

**File**: `simple_anonymization.rs`

Shows how to anonymize PII using different operators.

```bash
cargo run --example simple_anonymization
```

**Key Concepts**:
- Creating an anonymizer engine
- Configuring operators
- Applying anonymization

### 3. Complete Pipeline

**File**: `complete_pipeline.rs`

Full workflow combining analysis and anonymization.

```bash
cargo run --example complete_pipeline
```

**Key Concepts**:
- End-to-end PII workflow
- Different anonymization strategies
- Operator configuration

### 4. Custom Recognizer

**File**: `custom_recognizer.rs`

Creating a custom entity recognizer for domain-specific PII.

```bash
cargo run --example custom_recognizer
```

**Key Concepts**:
- Implementing the `EntityRecognizer` trait
- Pattern-based recognition
- Custom entity types
- Integrating with analyzer

## Common Patterns

### Basic Analysis

```rust
use presidio_analyzer::AnalyzerEngine;
use presidio_common::Language;

let analyzer = AnalyzerEngine::with_defaults();
let results = analyzer.analyze(
    text,
    Language::En,
    None,  // All entity types
    None,  // No correlation ID
    0.5,   // Score threshold
    false, // Don't include decision process
)?;
```

### Basic Anonymization

```rust
use presidio_anonymizer::AnonymizerEngine;
use presidio_common::{EntityType, ConflictResolutionStrategy};
use serde_json::json;
use std::collections::HashMap;

let anonymizer = AnonymizerEngine::with_defaults();

let mut operators = HashMap::new();
operators.insert(
    EntityType::Email,
    ("replace".to_string(), json!({"new_value": "<EMAIL>"})),
);

let result = anonymizer.anonymize(
    text,
    &analyzer_results,
    &operators,
    ConflictResolutionStrategy::MergeFirstBeforeSecond,
)?;
```

### Custom Recognizer

```rust
use presidio_common::{EntityRecognizer, EntityType, Language, RecognizerResult, Result};

struct MyRecognizer {
    // Your fields
}

impl EntityRecognizer for MyRecognizer {
    fn name(&self) -> &str {
        "MyRecognizer"
    }

    fn supported_entities(&self) -> &[EntityType] {
        &[EntityType::Custom("MY_ENTITY".to_string())]
    }

    fn supported_languages(&self) -> &[Language] {
        &[Language::En]
    }

    fn analyze(
        &self,
        text: &str,
        entities: Option<&[EntityType]>,
        _nlp_artifacts: Option<&NlpArtifacts>,
        _context: Option<&[String]>,
    ) -> Result<Vec<RecognizerResult>> {
        // Your recognition logic
        Ok(vec![])
    }
}
```

## Entity Types

Common built-in entity types:
- `EMAIL`
- `PHONE_NUMBER`
- `CREDIT_CARD`
- `IP_ADDRESS`
- `URL`
- `US_SSN`
- `UK_NHS`
- And many more...

See `presidio-common/src/types.rs` for the full list.

## Anonymization Operators

Built-in operators:
- **replace**: Replace with a placeholder value
- **redact**: Remove completely
- **mask**: Mask characters (e.g., `***`)
- **hash**: Hash using SHA-256 or SHA-512
- **encrypt**: Encrypt using AES-256-GCM
- **keep**: Keep original value (no-op)

### Operator Parameters

**Replace**:
```json
{"new_value": "<PLACEHOLDER>"}
```

**Mask**:
```json
{
  "masking_char": "*",
  "chars_to_mask": 10,
  "from_end": true
}
```

**Hash**:
```json
{
  "hash_type": "sha256"  // or "sha512"
}
```

**Encrypt**:
```json
{
  "key": "base64-encoded-32-byte-key"
}
```

## Advanced Usage

### Filtering Entity Types

```rust
use presidio_common::EntityType;

let entities = vec![EntityType::Email, EntityType::PhoneNumber];
let results = analyzer.analyze(
    text,
    Language::En,
    Some(&entities),  // Only detect these types
    None,
    0.5,
    false,
)?;
```

### Score Threshold

```rust
// Only return high-confidence results
let results = analyzer.analyze(
    text,
    Language::En,
    None,
    None,
    0.9,  // 90% confidence minimum
    false,
)?;
```

### Conflict Resolution

When entities overlap, choose a resolution strategy:

```rust
use presidio_common::ConflictResolutionStrategy;

// Keep first entity when overlapping
ConflictResolutionStrategy::MergeFirstBeforeSecond

// Merge similar entities
ConflictResolutionStrategy::MergeSimilarOrContained

// Remove all overlaps
ConflictResolutionStrategy::RemoveIntersections
```

## Testing Your Code

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_detection() {
        let analyzer = AnalyzerEngine::with_defaults();
        let text = "Contact: john@example.com";

        let results = analyzer.analyze(
            text,
            Language::En,
            None,
            None,
            0.0,
            false,
        ).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].entity_type, EntityType::Email);
    }
}
```

## Next Steps

- Check out the [API Reference](../docs/API_REFERENCE.md)
- Read the [Architecture Guide](../docs/ARCHITECTURE.md)
- Try the [CLI tool](../presidio-cli/)
- Deploy with [Kubernetes](../k8s/)

## Need Help?

- Open an issue on GitHub
- Check the main documentation
- Review the inline docs with `cargo doc --open`
