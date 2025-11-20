//! # Presidio Analyzer
//!
//! Context-aware PII detection engine for identifying personally identifiable information in text.
//!
//! This crate provides the analyzer component of Presidio, which detects PII entities using:
//! - Pattern-based recognizers (regex)
//! - NLP-based recognizers
//! - Context-aware scoring
//! - Customizable recognizer registry
//!
//! ## Example
//!
//! ```rust,no_run
//! use presidio_analyzer::{AnalyzerEngine, RecognizerRegistry};
//! use presidio_common::Language;
//!
//! let registry = RecognizerRegistry::new();
//! let engine = AnalyzerEngine::new(registry);
//!
//! let results = engine.analyze(
//!     "My email is john@example.com and my phone is 555-1234",
//!     Language::En,
//!     None,
//!     None,
//!     0.0,
//!     false,
//! ).unwrap();
//!
//! println!("Found {} PII entities", results.len());
//! ```

pub mod analyzer_engine;
pub mod recognizer_registry;
pub mod pattern_recognizer;
pub mod recognizers;

// Re-export main types
pub use analyzer_engine::AnalyzerEngine;
pub use recognizer_registry::RecognizerRegistry;
pub use pattern_recognizer::PatternRecognizer;
pub use presidio_common::*;