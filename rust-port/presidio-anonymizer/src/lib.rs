//! # Presidio Anonymizer
//!
//! PII transformation engine for anonymizing detected entities.
//!
//! This crate provides the anonymizer component of Presidio, which transforms PII entities using various operators:
//! - Replace: Substitute with a custom value
//! - Redact: Complete removal
//! - Mask: Partial masking with characters
//! - Hash: SHA-256/SHA-512 hashing
//! - Encrypt: AES-GCM encryption
//!
//! ## Example
//!
//! ```rust,no_run
//! use presidio_anonymizer::{AnonymizerEngine, operators::ReplaceOperator};
//! use presidio_common::{RecognizerResult, EntityType, ConflictResolutionStrategy};
//! use std::sync::Arc;
//! use std::collections::HashMap;
//!
//! let mut engine = AnonymizerEngine::new();
//! engine.add_operator(Arc::new(ReplaceOperator));
//!
//! let text = "My email is john@example.com";
//! let results = vec![
//!     RecognizerResult::new(EntityType::Email, 12, 29, 0.9),
//! ];
//!
//! let mut operators = HashMap::new();
//! operators.insert(
//!     EntityType::Email,
//!     ("replace".to_string(), serde_json::json!({"new_value": "<EMAIL>"})),
//! );
//!
//! let anonymized = engine.anonymize(
//!     text,
//!     &results,
//!     &operators,
//!     ConflictResolutionStrategy::HighestScore,
//! ).unwrap();
//!
//! println!("Anonymized: {}", anonymized.text);
//! ```

pub mod anonymizer_engine;
pub mod operators;

// Re-export main types
pub use anonymizer_engine::AnonymizerEngine;
pub use presidio_common::*;