//! Anonymization operators for transforming PII.

pub mod replace;
pub mod redact;
pub mod mask;
pub mod hash;
pub mod encrypt;
pub mod keep;

pub use replace::ReplaceOperator;
pub use redact::RedactOperator;
pub use mask::MaskOperator;
pub use hash::HashOperator;
pub use encrypt::EncryptOperator;
pub use keep::KeepOperator;

use presidio_common::Result;
use serde_json::Value;

/// Trait for anonymization operators.
///
/// All operators must implement this trait to be used by the anonymizer engine.
pub trait Operator: Send + Sync {
    /// Returns the name of this operator.
    fn name(&self) -> &str;

    /// Applies the operator to transform text.
    ///
    /// # Arguments
    ///
    /// * `text` - The text segment to transform
    /// * `params` - JSON parameters for the operator
    ///
    /// # Returns
    ///
    /// The transformed text.
    fn operate(&self, text: &str, params: &Value) -> Result<String>;

    /// Validates the operator parameters.
    fn validate(&self, params: &Value) -> Result<()>;
}
