//! Replace operator - substitutes PII with a custom value.

use super::Operator;
use presidio_common::Result;
use serde_json::Value;

/// Operator that replaces PII with a custom value.
pub struct ReplaceOperator;

impl Operator for ReplaceOperator {
    fn name(&self) -> &str {
        "replace"
    }

    fn operate(&self, _text: &str, params: &Value) -> Result<String> {
        if let Some(new_value) = params.get("new_value").and_then(|v| v.as_str()) {
            Ok(new_value.to_string())
        } else {
            // Default: replace with entity type placeholder
            Ok("<ANONYMIZED>".to_string())
        }
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
    fn test_replace_with_custom_value() {
        let operator = ReplaceOperator;
        let params = json!({"new_value": "<EMAIL>"});
        let result = operator.operate("john@example.com", &params).unwrap();
        assert_eq!(result, "<EMAIL>");
    }

    #[test]
    fn test_replace_default() {
        let operator = ReplaceOperator;
        let params = json!({});
        let result = operator.operate("john@example.com", &params).unwrap();
        assert_eq!(result, "<ANONYMIZED>");
    }
}
