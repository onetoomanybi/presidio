# Presidio Rust - API Reference

## Table of Contents

- [presidio-common](#presidio-common)
- [presidio-analyzer](#presidio-analyzer)
- [Quick Start Examples](#quick-start-examples)

---

## presidio-common

Core types, traits, and utilities shared across all Presidio components.

### Types

#### `RecognizerResult`

Represents a detected PII entity in text.

```rust
pub struct RecognizerResult {
    pub entity_type: EntityType,
    pub start: usize,
    pub end: usize,
    pub score: f32,
    pub analysis_explanation: Option<AnalysisExplanation>,
    pub recognition_metadata: HashMap<String, serde_json::Value>,
}
```

**Methods:**

```rust
// Create a new recognizer result
pub fn new(entity_type: EntityType, start: usize, end: usize, score: f32) -> Self

// Create with explanation
pub fn with_explanation(
    entity_type: EntityType,
    start: usize,
    end: usize,
    score: f32,
    explanation: AnalysisExplanation,
) -> Self

// Check if this result overlaps with another
pub fn overlaps_with(&self, other: &RecognizerResult) -> bool

// Check if this result contains another
pub fn contains(&self, other: &RecognizerResult) -> bool

// Get the length of the detected entity
pub fn len(&self) -> usize

// Check if the entity is empty
pub fn is_empty(&self) -> bool
```

**Example:**

```rust
use presidio_common::{RecognizerResult, EntityType};

let result = RecognizerResult::new(
    EntityType::Email,
    10,
    28,
    0.95,
);

assert_eq!(result.entity_type, EntityType::Email);
assert_eq!(result.start, 10);
assert_eq!(result.end, 28);
assert_eq!(result.score, 0.95);
assert_eq!(result.len(), 18);
```

#### `EntityType`

Enumeration of all supported entity types.

```rust
pub enum EntityType {
    // Generic entities
    Email,
    Phone,
    Url,
    IpAddress,
    CreditCard,
    Iban,
    Crypto,
    Date,
    Time,
    Person,
    Location,
    Organization,

    // US-specific
    UsSsn,
    UsDriverLicense,
    UsPassport,
    UsItin,
    UsBankAccount,
    UsMedicalLicense,

    // UK-specific
    UkNhs,
    UkNino,

    // ... more country-specific types

    // Custom entity type
    Custom(String),
}
```

**Methods:**

```rust
pub fn as_str(&self) -> &str
pub fn from_str(s: &str) -> Self
```

**Example:**

```rust
use presidio_common::EntityType;

let email = EntityType::Email;
assert_eq!(email.as_str(), "EMAIL");

let parsed = EntityType::from_str("CREDIT_CARD");
assert_eq!(parsed, EntityType::CreditCard);

let custom = EntityType::Custom("MY_CUSTOM_TYPE".to_string());
assert_eq!(custom.as_str(), "MY_CUSTOM_TYPE");
```

#### `Language`

Supported languages for PII detection.

```rust
pub enum Language {
    En,  // English
    Es,  // Spanish
    Fr,  // French
    De,  // German
    It,  // Italian
    Pt,  // Portuguese
    Nl,  // Dutch
    Pl,  // Polish
    Fi,  // Finnish
    Ko,  // Korean
    Th,  // Thai
}
```

**Methods:**

```rust
pub fn as_str(&self) -> &'static str
pub fn from_str(s: &str) -> Option<Self>
```

**Example:**

```rust
use presidio_common::Language;

let lang = Language::En;
assert_eq!(lang.as_str(), "en");

let parsed = Language::from_str("english").unwrap();
assert_eq!(parsed, Language::En);
```

### Traits

#### `EntityRecognizer`

Trait for PII entity recognizers.

```rust
pub trait EntityRecognizer: Send + Sync {
    fn name(&self) -> &str;
    fn supported_entities(&self) -> &[EntityType];
    fn supported_languages(&self) -> &[Language];

    fn analyze(
        &self,
        text: &str,
        entities: Option<&[EntityType]>,
        nlp_artifacts: Option<&NlpArtifacts>,
        context: Option<&[String]>,
    ) -> Result<Vec<RecognizerResult>>;

    fn validate_result(&self, text: &str) -> bool {
        true
    }

    fn supports_language(&self, language: Language) -> bool {
        self.supported_languages().contains(&language)
    }

    fn supports_entity(&self, entity_type: &EntityType) -> bool {
        self.supported_entities().contains(entity_type)
    }
}
```

**Example Implementation:**

```rust
use presidio_common::{EntityRecognizer, EntityType, Language, RecognizerResult, Result};

struct MyRecognizer;

impl EntityRecognizer for MyRecognizer {
    fn name(&self) -> &str {
        "MyRecognizer"
    }

    fn supported_entities(&self) -> &[EntityType] {
        &[EntityType::Email]
    }

    fn supported_languages(&self) -> &[Language] {
        &[Language::En, Language::Es]
    }

    fn analyze(
        &self,
        text: &str,
        _entities: Option<&[EntityType]>,
        _nlp_artifacts: Option<&NlpArtifacts>,
        _context: Option<&[String]>,
    ) -> Result<Vec<RecognizerResult>> {
        // Detection logic here
        Ok(vec![])
    }
}
```

### Utilities

#### `remove_duplicates`

Removes duplicate and overlapping results.

```rust
pub fn remove_duplicates(results: Vec<RecognizerResult>) -> Vec<RecognizerResult>
```

**Example:**

```rust
use presidio_common::{remove_duplicates, RecognizerResult, EntityType};

let results = vec![
    RecognizerResult::new(EntityType::Email, 0, 10, 0.9),
    RecognizerResult::new(EntityType::Email, 0, 10, 0.8), // Duplicate
];

let dedup = remove_duplicates(results);
assert_eq!(dedup.len(), 1);
assert_eq!(dedup[0].score, 0.9); // Keeps highest score
```

#### `luhn_checksum`

Validates a number using the Luhn algorithm.

```rust
pub fn luhn_checksum(number: &str) -> bool
```

**Example:**

```rust
use presidio_common::luhn_checksum;

assert!(luhn_checksum("4532015112830366")); // Valid Visa
assert!(!luhn_checksum("1234567890123456")); // Invalid
```

#### `merge_overlapping`

Merges overlapping results using a conflict resolution strategy.

```rust
pub fn merge_overlapping(
    results: Vec<RecognizerResult>,
    strategy: ConflictResolutionStrategy,
) -> Vec<RecognizerResult>
```

**Example:**

```rust
use presidio_common::{merge_overlapping, ConflictResolutionStrategy, RecognizerResult, EntityType};

let results = vec![
    RecognizerResult::new(EntityType::Email, 0, 10, 0.9),
    RecognizerResult::new(EntityType::Email, 5, 15, 0.8),
];

let merged = merge_overlapping(results, ConflictResolutionStrategy::HighestScore);
assert_eq!(merged.len(), 1);
assert_eq!(merged[0].score, 0.9);
```

---

## presidio-analyzer

PII detection engine for identifying personally identifiable information in text.

### Main Components

#### `AnalyzerEngine`

The main analyzer engine for detecting PII in text.

```rust
pub struct AnalyzerEngine {
    registry: RecognizerRegistry,
    nlp_engine: Option<Arc<dyn NlpEngine>>,
}
```

**Methods:**

```rust
// Create a new analyzer engine with a custom registry
pub fn new(registry: RecognizerRegistry) -> Self

// Create with default recognizers
pub fn default() -> Self

// Add an NLP engine
pub fn with_nlp_engine(self, nlp_engine: Arc<dyn NlpEngine>) -> Self

// Analyze text and detect PII
pub fn analyze(
    &self,
    text: &str,
    language: Language,
    entities: Option<&[EntityType]>,
    correlation_id: Option<&str>,
    score_threshold: f32,
    return_decision_process: bool,
) -> Result<Vec<RecognizerResult>>

// Get the recognizer registry
pub fn registry(&self) -> &RecognizerRegistry

// Get a mutable reference to the registry
pub fn registry_mut(&mut self) -> &mut RecognizerRegistry
```

**Example:**

```rust
use presidio_analyzer::{AnalyzerEngine, RecognizerRegistry};
use presidio_common::Language;

// Create engine with default recognizers
let engine = AnalyzerEngine::default();

// Analyze text
let text = "My email is john@example.com and my phone is 555-1234";
let results = engine.analyze(
    text,
    Language::En,
    None,           // All entity types
    None,           // No correlation ID
    0.5,            // Minimum score threshold
    false,          // Don't return decision process
)?;

println!("Found {} PII entities", results.len());
for result in results {
    println!("{:?} at {}-{} (score: {})",
        result.entity_type,
        result.start,
        result.end,
        result.score
    );
}
```

#### `RecognizerRegistry`

Registry for managing entity recognizers.

```rust
pub struct RecognizerRegistry {
    recognizers: HashMap<String, Arc<dyn EntityRecognizer>>,
}
```

**Methods:**

```rust
// Create an empty registry
pub fn new() -> Self

// Create with default recognizers
pub fn with_defaults() -> Self

// Add a recognizer
pub fn add_recognizer(&mut self, recognizer: Arc<dyn EntityRecognizer>)

// Remove a recognizer
pub fn remove_recognizer(&mut self, name: &str) -> Result<()>

// Get a specific recognizer
pub fn get_recognizer(&self, name: &str) -> Option<Arc<dyn EntityRecognizer>>

// Get recognizers filtered by language and entity types
pub fn get_recognizers(
    &self,
    language: Language,
    entities: Option<&[EntityType]>,
) -> Vec<Arc<dyn EntityRecognizer>>

// Get all recognizer names
pub fn get_recognizer_names(&self) -> Vec<String>

// Get all supported entity types
pub fn get_supported_entities(&self) -> Vec<EntityType>

// Get all supported languages
pub fn get_supported_languages(&self) -> Vec<Language>

// Get the number of recognizers
pub fn len(&self) -> usize

// Check if empty
pub fn is_empty(&self) -> bool
```

**Example:**

```rust
use presidio_analyzer::{RecognizerRegistry, recognizers::EmailRecognizer};
use presidio_common::Language;
use std::sync::Arc;

// Create custom registry
let mut registry = RecognizerRegistry::new();

// Add recognizers
registry.add_recognizer(Arc::new(EmailRecognizer::new()));

// Get recognizers for English
let recognizers = registry.get_recognizers(Language::En, None);
println!("Found {} recognizers for English", recognizers.len());

// Remove a recognizer
registry.remove_recognizer("EmailRecognizer")?;
```

### Built-in Recognizers

#### `EmailRecognizer`

Detects email addresses with context-aware scoring.

```rust
pub struct EmailRecognizer;

impl EmailRecognizer {
    pub fn new() -> Self
}
```

**Features:**
- Regex-based email detection
- Context boosting for words like "email", "mail", "contact"
- Multi-language support

**Example:**

```rust
use presidio_analyzer::recognizers::EmailRecognizer;
use presidio_common::{EntityRecognizer, Language};

let recognizer = EmailRecognizer::new();
let text = "Please email me at john@example.com";
let results = recognizer.analyze(text, None, None, None)?;

assert_eq!(results.len(), 1);
assert_eq!(results[0].entity_type, EntityType::Email);
```

#### `CreditCardRecognizer`

Detects credit card numbers with Luhn validation.

```rust
pub struct CreditCardRecognizer;

impl CreditCardRecognizer {
    pub fn new() -> Self
}
```

**Features:**
- Detects 16-digit card numbers (with or without spaces/dashes)
- Luhn checksum validation
- High confidence score (0.8) for validated cards
- Context boosting for words like "card", "credit"

**Example:**

```rust
use presidio_analyzer::recognizers::CreditCardRecognizer;

let recognizer = CreditCardRecognizer::new();
let text = "Card: 4532 0151 1283 0366"; // Valid Visa test number
let results = recognizer.analyze(text, None, None, None)?;

assert_eq!(results.len(), 1);
assert!(results[0].score >= 0.8); // High confidence
```

#### `PhoneRecognizer`

Detects North American phone numbers.

```rust
pub struct PhoneRecognizer;

impl PhoneRecognizer {
    pub fn new() -> Self
}
```

**Patterns Supported:**
- (555) 123-4567
- 555-123-4567
- 555.123.4567
- +1-555-123-4567

**Example:**

```rust
use presidio_analyzer::recognizers::PhoneRecognizer;

let recognizer = PhoneRecognizer::new();
let text = "Call me at (555) 123-4567";
let results = recognizer.analyze(text, None, None, None)?;

assert_eq!(results.len(), 1);
```

#### `IpAddressRecognizer`

Detects IPv4 and IPv6 addresses.

```rust
pub struct IpAddressRecognizer;

impl IpAddressRecognizer {
    pub fn new() -> Self
}
```

**Example:**

```rust
use presidio_analyzer::recognizers::IpAddressRecognizer;

let recognizer = IpAddressRecognizer::new();
let text = "Server IP: 192.168.1.1";
let results = recognizer.analyze(text, None, None, None)?;

assert_eq!(results.len(), 1);
```

#### `UrlRecognizer`

Detects HTTP/HTTPS and www URLs.

```rust
pub struct UrlRecognizer;

impl UrlRecognizer {
    pub fn new() -> Self
}
```

**Example:**

```rust
use presidio_analyzer::recognizers::UrlRecognizer;

let recognizer = UrlRecognizer::new();
let text = "Visit https://example.com";
let results = recognizer.analyze(text, None, None, None)?;

assert_eq!(results.len(), 1);
```

---

## Quick Start Examples

### Basic Text Analysis

```rust
use presidio_analyzer::{AnalyzerEngine, RecognizerRegistry};
use presidio_common::{Language, EntityType};

fn main() -> Result<()> {
    // Create engine with default recognizers
    let engine = AnalyzerEngine::default();

    // Analyze text
    let text = "Contact John at john@example.com or call 555-123-4567";
    let results = engine.analyze(
        text,
        Language::En,
        None,
        None,
        0.5,
        false,
    )?;

    // Print results
    for result in results {
        let detected_text = &text[result.start..result.end];
        println!(
            "Found {}: '{}' (confidence: {})",
            result.entity_type.as_str(),
            detected_text,
            result.score
        );
    }

    Ok(())
}
```

### Custom Entity Filtering

```rust
// Only detect emails and phones
let entity_filter = vec![EntityType::Email, EntityType::Phone];

let results = engine.analyze(
    text,
    Language::En,
    Some(&entity_filter),
    None,
    0.5,
    false,
)?;
```

### Adding Custom Recognizer

```rust
use presidio_common::{EntityRecognizer, EntityType, Language, RecognizerResult, Result};
use std::sync::Arc;

struct SsnRecognizer;

impl EntityRecognizer for SsnRecognizer {
    fn name(&self) -> &str {
        "SsnRecognizer"
    }

    fn supported_entities(&self) -> &[EntityType] {
        &[EntityType::UsSsn]
    }

    fn supported_languages(&self) -> &[Language] {
        &[Language::En]
    }

    fn analyze(
        &self,
        text: &str,
        _entities: Option<&[EntityType]>,
        _nlp_artifacts: Option<&NlpArtifacts>,
        _context: Option<&[String]>,
    ) -> Result<Vec<RecognizerResult>> {
        // SSN detection logic
        let pattern = regex::Regex::new(r"\b\d{3}-\d{2}-\d{4}\b")?;
        let results = pattern
            .find_iter(text)
            .map(|m| RecognizerResult::new(
                EntityType::UsSsn,
                m.start(),
                m.end(),
                0.8,
            ))
            .collect();
        Ok(results)
    }
}

// Add to registry
let mut registry = RecognizerRegistry::new();
registry.add_recognizer(Arc::new(SsnRecognizer));
let engine = AnalyzerEngine::new(registry);
```

### Score Threshold Filtering

```rust
// Only return high-confidence detections (>= 0.8)
let results = engine.analyze(
    text,
    Language::En,
    None,
    None,
    0.8,  // Higher threshold
    false,
)?;
```

---

## Error Handling

All fallible operations return `Result<T, PresidioError>`.

```rust
use presidio_common::{PresidioError, Result};

match engine.analyze(text, Language::En, None, None, 0.5, false) {
    Ok(results) => {
        println!("Found {} entities", results.len());
    }
    Err(PresidioError::UnsupportedLanguage(lang)) => {
        eprintln!("Language not supported: {}", lang);
    }
    Err(PresidioError::RecognizerNotFound(name)) => {
        eprintln!("Recognizer not found: {}", name);
    }
    Err(e) => {
        eprintln!("Error: {}", e);
    }
}
```

---

## Performance Tips

1. **Reuse AnalyzerEngine**: Create once and reuse for multiple analyses
2. **Use Score Thresholds**: Filter low-confidence results early
3. **Limit Entity Types**: Only search for needed entities
4. **Batch Processing**: Process multiple texts in parallel using Rayon
5. **Regex Caching**: Recognizers use lazy_static for regex compilation

```rust
use rayon::prelude::*;

let texts = vec![
    "Text 1 with email@example.com",
    "Text 2 with 555-1234",
    // ...
];

let results: Vec<_> = texts
    .par_iter()
    .map(|text| engine.analyze(text, Language::En, None, None, 0.5, false))
    .collect();
```

---

## Additional Resources

- [Architecture Documentation](ARCHITECTURE.md)
- [Interactive Visualization](visualization.html)
- [GitHub Repository](https://github.com/microsoft/presidio)
- [Original Python Documentation](https://microsoft.github.io/presidio/)
