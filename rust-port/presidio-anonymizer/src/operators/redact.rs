//! Redact operator - completely removes PII.

use super::Operator;
use presidio_common::Result;
use serde_json::Value;

/// Operator that completely removes PII.
pub struct RedactOperator;

impl Operator for RedactOperator {
    fn name(&self) -> &str {
        "redact"
    }

    fn operate(&self, _text: &str, _params: &Value) -> Result<String> {
        Ok(String::new())
    }

    fn validate(&self, _params: &Value) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_redact() {
        let operator = RedactOperator;
        let params = json!({});
        let result = operator.operate("john@example.com", &params).unwrap();
        assert_eq!(result, "");
    }
}
