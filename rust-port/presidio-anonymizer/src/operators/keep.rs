//! Keep operator - leaves PII unchanged.

use super::Operator;
use presidio_common::Result;
use serde_json::Value;

/// Operator that keeps PII unchanged.
pub struct KeepOperator;

impl Operator for KeepOperator {
    fn name(&self) -> &str {
        "keep"
    }

    fn operate(&self, text: &str, _params: &Value) -> Result<String> {
        Ok(text.to_string())
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
    fn test_keep() {
        let operator = KeepOperator;
        let params = json!({});
        let result = operator.operate("john@example.com", &params).unwrap();
        assert_eq!(result, "john@example.com");
    }
}
