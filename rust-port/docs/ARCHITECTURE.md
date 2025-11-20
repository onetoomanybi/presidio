# Presidio Rust Port - Architecture

## Overview

This document provides a comprehensive architectural overview of the Presidio Rust port, including system design, component interactions, and data flow.

## System Architecture

```mermaid
graph TB
    subgraph "Client Layer"
        CLI[CLI Tool]
        REST[REST API Client]
        LIB[Library User]
    end

    subgraph "API Layer"
        ANALYZER_API[Analyzer Service<br/>Port 3000]
        ANONYMIZER_API[Anonymizer Service<br/>Port 3001]
        IMAGE_API[Image Redactor Service<br/>Port 3002]
    end

    subgraph "Core Engines"
        ANALYZER[Analyzer Engine]
        ANONYMIZER[Anonymizer Engine]
        IMAGE_REDACTOR[Image Redactor Engine]
        STRUCTURED[Structured Engine]
    end

    subgraph "Recognition Layer"
        REGISTRY[Recognizer Registry]
        PATTERN[Pattern Recognizers]
        NLP[NLP Recognizers]
        CUSTOM[Custom Recognizers]
    end

    subgraph "Operator Layer"
        REPLACE[Replace Operator]
        REDACT[Redact Operator]
        MASK[Mask Operator]
        HASH[Hash Operator]
        ENCRYPT[Encrypt Operator]
        CUSTOM_OP[Custom Operators]
    end

    subgraph "Foundation Layer"
        COMMON[presidio-common<br/>Types, Traits, Utils]
    end

    subgraph "External Services"
        NLP_EXT[External NLP Service]
        OCR[OCR Service]
    end

    CLI --> ANALYZER_API
    CLI --> ANONYMIZER_API
    REST --> ANALYZER_API
    REST --> ANONYMIZER_API
    LIB --> ANALYZER
    LIB --> ANONYMIZER

    ANALYZER_API --> ANALYZER
    ANONYMIZER_API --> ANONYMIZER
    IMAGE_API --> IMAGE_REDACTOR

    ANALYZER --> REGISTRY
    ANALYZER --> NLP_EXT
    REGISTRY --> PATTERN
    REGISTRY --> NLP
    REGISTRY --> CUSTOM

    ANONYMIZER --> REPLACE
    ANONYMIZER --> REDACT
    ANONYMIZER --> MASK
    ANONYMIZER --> HASH
    ANONYMIZER --> ENCRYPT
    ANONYMIZER --> CUSTOM_OP

    IMAGE_REDACTOR --> ANALYZER
    IMAGE_REDACTOR --> ANONYMIZER
    IMAGE_REDACTOR --> OCR

    STRUCTURED --> ANALYZER
    STRUCTURED --> ANONYMIZER

    ANALYZER --> COMMON
    ANONYMIZER --> COMMON
    IMAGE_REDACTOR --> COMMON
    STRUCTURED --> COMMON
    REGISTRY --> COMMON
```

## Component Architecture

### 1. presidio-common

The foundation library providing shared types, traits, and utilities.

```mermaid
classDiagram
    class RecognizerResult {
        +EntityType entity_type
        +usize start
        +usize end
        +f32 score
        +Option~AnalysisExplanation~ analysis_explanation
        +overlaps_with(other) bool
        +contains(other) bool
    }

    class EntityType {
        <<enumeration>>
        Email
        Phone
        CreditCard
        UsSsn
        ...
        +as_str() str
        +from_str(s) EntityType
    }

    class Language {
        <<enumeration>>
        En
        Es
        Fr
        De
        ...
        +as_str() str
        +from_str(s) Option~Language~
    }

    class EntityRecognizer {
        <<trait>>
        +name() str
        +supported_entities() EntityType[]
        +supported_languages() Language[]
        +analyze(text, entities, nlp, context) Result~RecognizerResult[]~
        +validate_result(text) bool
    }

    class NlpEngine {
        <<trait>>
        +process(text, language) Result~NlpArtifacts~
        +is_available(language) bool
        +name() str
    }

    class PresidioError {
        <<enumeration>>
        InvalidPattern
        RecognizerNotFound
        UnsupportedLanguage
        ...
    }

    RecognizerResult --> EntityType
    EntityRecognizer --> EntityType
    EntityRecognizer --> Language
```

### 2. presidio-analyzer

The PII detection engine.

```mermaid
classDiagram
    class AnalyzerEngine {
        -RecognizerRegistry registry
        -Option~Arc~NlpEngine~~ nlp_engine
        +new(registry) AnalyzerEngine
        +analyze(text, language, entities, ...) Result~RecognizerResult[]~
        +registry() RecognizerRegistry
    }

    class RecognizerRegistry {
        -HashMap~String, Arc~EntityRecognizer~~ recognizers
        +new() RecognizerRegistry
        +with_defaults() RecognizerRegistry
        +add_recognizer(recognizer)
        +remove_recognizer(name) Result
        +get_recognizers(language, entities) Vec~Arc~EntityRecognizer~~
    }

    class PatternRecognizer {
        -String name
        -Vec~EntityType~ supported_entities
        -Vec~Language~ supported_languages
        -Vec~Pattern~ patterns
        +new(...) PatternRecognizer
        +with_context(context) PatternRecognizer
        +with_validation(validate) PatternRecognizer
    }

    class EmailRecognizer {
        +new() EmailRecognizer
    }

    class CreditCardRecognizer {
        +new() CreditCardRecognizer
        -validate_luhn(number) bool
    }

    class PhoneRecognizer {
        +new() PhoneRecognizer
    }

    AnalyzerEngine --> RecognizerRegistry
    RecognizerRegistry --> EmailRecognizer
    RecognizerRegistry --> CreditCardRecognizer
    RecognizerRegistry --> PhoneRecognizer
    EmailRecognizer ..|> EntityRecognizer
    CreditCardRecognizer ..|> EntityRecognizer
    PhoneRecognizer ..|> EntityRecognizer
    PatternRecognizer ..|> EntityRecognizer
```

### 3. presidio-anonymizer

The PII transformation engine.

```mermaid
classDiagram
    class AnonymizerEngine {
        -HashMap~String, Arc~Operator~~ operators
        +new() AnonymizerEngine
        +anonymize(text, results, operators, strategy) Result~EngineResult~
        +add_operator(operator)
    }

    class Operator {
        <<trait>>
        +operate(text, params) Result~String~
        +validate(params) Result
        +operator_name() str
    }

    class ReplaceOperator {
        +operate(text, params) Result~String~
    }

    class RedactOperator {
        +operate(text, params) Result~String~
    }

    class MaskOperator {
        +operate(text, params) Result~String~
    }

    class HashOperator {
        +operate(text, params) Result~String~
    }

    class EncryptOperator {
        +operate(text, params) Result~String~
    }

    class ConflictResolutionStrategy {
        <<enumeration>>
        HighestScore
        Longest
        First
        Last
    }

    AnonymizerEngine --> Operator
    ReplaceOperator ..|> Operator
    RedactOperator ..|> Operator
    MaskOperator ..|> Operator
    HashOperator ..|> Operator
    EncryptOperator ..|> Operator
    AnonymizerEngine --> ConflictResolutionStrategy
```

## Data Flow

### Analysis Flow

```mermaid
sequenceDiagram
    participant Client
    participant Engine as AnalyzerEngine
    participant Registry as RecognizerRegistry
    participant NLP as NLP Engine
    participant Recognizer
    participant Utils

    Client->>Engine: analyze(text, language)
    Engine->>NLP: process(text, language)
    NLP-->>Engine: NlpArtifacts

    Engine->>Registry: get_recognizers(language, entities)
    Registry-->>Engine: Vec<Recognizer>

    loop for each recognizer
        Engine->>Recognizer: analyze(text, entities, nlp_artifacts)
        Recognizer->>Recognizer: match patterns
        Recognizer->>Recognizer: enhance_with_context
        Recognizer->>Recognizer: validate (if enabled)
        Recognizer-->>Engine: Vec<RecognizerResult>
    end

    Engine->>Utils: remove_duplicates(all_results)
    Utils-->>Engine: deduplicated results

    Engine->>Engine: filter by score_threshold
    Engine->>Engine: sort by position
    Engine-->>Client: Vec<RecognizerResult>
```

### Anonymization Flow

```mermaid
sequenceDiagram
    participant Client
    participant Engine as AnonymizerEngine
    participant Utils
    participant Operator

    Client->>Engine: anonymize(text, analyzer_results, operators)
    Engine->>Utils: merge_overlapping(results, strategy)
    Utils-->>Engine: resolved results

    loop for each result (reverse order)
        Engine->>Engine: lookup operator for entity_type
        Engine->>Operator: operate(text_segment, params)
        Operator->>Operator: perform transformation
        Operator-->>Engine: transformed_text
        Engine->>Engine: replace in original text
    end

    Engine-->>Client: EngineResult
```

### End-to-End PII Detection and Anonymization

```mermaid
flowchart TD
    Start([Input Text]) --> A[Analyzer Engine]
    A --> B{NLP Available?}
    B -->|Yes| C[Process with NLP]
    B -->|No| D[Skip NLP]
    C --> E[Get Relevant Recognizers]
    D --> E

    E --> F[Run Pattern Recognizers]
    E --> G[Run NLP Recognizers]
    E --> H[Run Custom Recognizers]

    F --> I[Collect All Results]
    G --> I
    H --> I

    I --> J[Remove Duplicates]
    J --> K[Filter by Score Threshold]
    K --> L[Sort by Position]

    L --> M{Anonymize?}
    M -->|No| N([Return Results])
    M -->|Yes| O[Anonymizer Engine]

    O --> P[Resolve Conflicts]
    P --> Q[Apply Operators]
    Q --> R{Operator Type}

    R -->|Replace| S[Replace with Value]
    R -->|Redact| T[Remove Text]
    R -->|Mask| U[Mask Characters]
    R -->|Hash| V[Hash Value]
    R -->|Encrypt| W[Encrypt Value]

    S --> X([Return Anonymized Text])
    T --> X
    U --> X
    V --> X
    W --> X
```

## Concurrency Model

```mermaid
graph TB
    subgraph "Thread-Safe Components"
        E[AnalyzerEngine<br/>Arc + Send + Sync]
        R[RecognizerRegistry<br/>Arc + Send + Sync]
        RE[Recognizers<br/>Arc + dyn EntityRecognizer]
    end

    subgraph "Parallel Processing"
        T1[Text Chunk 1]
        T2[Text Chunk 2]
        T3[Text Chunk 3]
        T4[Text Chunk 4]
    end

    subgraph "Results Aggregation"
        M[Merge & Deduplicate]
    end

    E --> R
    R --> RE

    T1 --> E
    T2 --> E
    T3 --> E
    T4 --> E

    E --> M
```

## Design Patterns

### 1. **Registry Pattern**
- `RecognizerRegistry` manages all available recognizers
- Allows dynamic addition/removal of recognizers
- Supports filtering by language and entity type

### 2. **Strategy Pattern**
- `ConflictResolutionStrategy` for handling overlapping entities
- Multiple operator types for different anonymization strategies

### 3. **Trait Objects (Polymorphism)**
- `EntityRecognizer` trait allows for different recognition implementations
- `Operator` trait for pluggable anonymization operations
- `NlpEngine` trait for external NLP service integration

### 4. **Builder Pattern**
- `PatternRecognizer::new().with_context().with_validation()`
- `AnalyzerEngine::new().with_nlp_engine()`

### 5. **Lazy Evaluation**
- Regex compilation using `lazy_static!`
- On-demand NLP processing

## Performance Optimizations

1. **Zero-Cost Abstractions**
   - Trait objects with dynamic dispatch only where needed
   - Generic types with monomorphization

2. **Memory Efficiency**
   - String slices (`&str`) instead of owned strings where possible
   - Arc for shared ownership without copying

3. **Regex Optimization**
   - Pre-compiled regexes using `lazy_static`
   - Regex caching to avoid recompilation

4. **Parallel Processing**
   - Thread-safe design allows parallel text processing
   - Rayon for data parallelism in batch operations

5. **Conflict Resolution**
   - Efficient overlap detection
   - Single-pass deduplication

## Error Handling

```mermaid
graph TD
    Op[Operation] --> Result{Result<T>}
    Result -->|Ok| Success[Continue Processing]
    Result -->|Err| Error[PresidioError]

    Error --> InvalidPattern
    Error --> RecognizerNotFound
    Error --> UnsupportedLanguage
    Error --> NlpEngineError
    Error --> ValidationError

    InvalidPattern --> Handle[Error Propagation<br/>or Recovery]
    RecognizerNotFound --> Handle
    UnsupportedLanguage --> Handle
    NlpEngineError --> Handle
    ValidationError --> Handle

    Handle --> Log[Log Error]
    Handle --> Retry[Retry Logic]
    Handle --> Fallback[Fallback Behavior]
    Handle --> UserError[Return to User]
```

## Extension Points

### Adding a New Recognizer

```rust
use presidio_common::{EntityRecognizer, EntityType, Language, RecognizerResult, Result};

pub struct MyCustomRecognizer;

impl EntityRecognizer for MyCustomRecognizer {
    fn name(&self) -> &str {
        "MyCustomRecognizer"
    }

    fn supported_entities(&self) -> &[EntityType] {
        &[EntityType::Custom("MY_TYPE".to_string())]
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
        // Custom detection logic
        Ok(vec![])
    }
}

// Usage
let mut registry = RecognizerRegistry::new();
registry.add_recognizer(Arc::new(MyCustomRecognizer));
```

### Adding a New Operator

```rust
use presidio_common::{Operator, OperatorConfig, Result};

pub struct MyCustomOperator;

impl Operator for MyCustomOperator {
    fn operate(&self, text: &str, params: &OperatorConfig) -> Result<String> {
        // Custom transformation logic
        Ok(text.to_uppercase())
    }

    fn validate(&self, _params: &OperatorConfig) -> Result<()> {
        Ok(())
    }

    fn operator_name(&self) -> &str {
        "my_custom"
    }
}
```

## Testing Strategy

### Unit Tests
- Each recognizer has dedicated tests
- Operator validation tests
- Utility function tests

### Integration Tests
- End-to-end analysis pipeline
- Combined analyzer + anonymizer flow
- Multi-language testing

### Performance Tests
- Benchmark regex matching
- Batch processing throughput
- Memory usage profiling

## Security Considerations

1. **Input Validation**
   - Regex pattern validation
   - Parameter sanitization
   - Size limits for text input

2. **Cryptography**
   - Industry-standard algorithms (AES-GCM, SHA-256)
   - Secure random number generation
   - Key management best practices

3. **Data Protection**
   - No persistent storage of PII
   - Memory cleanup for sensitive data
   - Audit logging support

## Future Enhancements

1. **Additional Recognizers**
   - More country-specific identifiers
   - Industry-specific patterns (medical, financial)
   - Custom ML-based recognizers

2. **Performance**
   - GPU acceleration for large-scale processing
   - Streaming API for large texts
   - Incremental processing

3. **Features**
   - Multi-language NLP support
   - Advanced context-aware scoring
   - Reversible anonymization with token vault
   - Real-time API rate limiting

4. **Integrations**
   - Kafka/RabbitMQ connectors
   - Cloud provider integrations (AWS, Azure, GCP)
   - Database anonymization plugins
