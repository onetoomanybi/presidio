//! # Presidio Common
//!
//! Core types, traits, and utilities shared across all Presidio components.
//!
//! This crate provides the foundational data structures and abstractions used throughout
//! the Presidio PII detection and anonymization system, including:
//!
//! - Entity recognition results and patterns
//! - Language and entity type enumerations
//! - Core traits for recognizers and NLP engines
//! - Error types
//! - Common utilities
//!
//! ## Example
//!
//! ```rust
//! use presidio_common::{RecognizerResult, EntityType, Language};
//!
//! let result = RecognizerResult::new(
//!     EntityType::Email,
//!     10,
//!     30,
//!     0.95,
//! );
//!
//! assert_eq!(result.entity_type, EntityType::Email);
//! assert_eq!(result.score, 0.95);
//! ```

pub mod error;
pub mod types;
pub mod traits;
pub mod utils;

// Re-export commonly used items
pub use error::{PresidioError, Result};
pub use types::*;
pub use traits::*;