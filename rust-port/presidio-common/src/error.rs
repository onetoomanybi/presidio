//! Error types for Presidio operations.

use thiserror::Error;

/// The main error type for Presidio operations.
#[derive(Error, Debug)]
pub enum PresidioError {
    /// Invalid regex pattern
    #[error("Invalid pattern: {0}")]
    InvalidPattern(String),

    /// Recognizer not found in registry
    #[error("Recognizer not found: {0}")]
    RecognizerNotFound(String),

    /// Unsupported language
    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),

    /// NLP engine error
    #[error("NLP engine error: {0}")]
    NlpEngineError(String),

    /// Operator not found
    #[error("Operator not found: {0}")]
    OperatorNotFound(String),

    /// Invalid operator configuration
    #[error("Invalid operator configuration: {0}")]
    InvalidOperatorConfig(String),

    /// Validation error
    #[error("Validation failed: {0}")]
    ValidationError(String),

    /// Regex compilation error
    #[error("Regex error: {0}")]
    RegexError(#[from] regex::Error),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Entity type not supported
    #[error("Entity type not supported: {0}")]
    UnsupportedEntityType(String),

    /// Generic error with context
    #[error("{0}")]
    Other(String),
}

impl From<serde_json::Error> for PresidioError {
    fn from(err: serde_json::Error) -> Self {
        PresidioError::SerializationError(err.to_string())
    }
}

/// Result type alias for Presidio operations.
pub type Result<T> = std::result::Result<T, PresidioError>;
