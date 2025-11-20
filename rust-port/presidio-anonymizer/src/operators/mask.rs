//! Mask operator - partially masks PII with characters.

use super::Operator;
use presidio_common::Result;
use serde_json::Value;

/// Operator that masks PII with characters.
pub struct MaskOperator;

impl Operator for MaskOperator {
    fn name(&self) -> &str {
        "mask"
    }

    fn operate(&self, text: &str, params: &Value) -> Result<String> {
        let mask_char = params
            .get("masking_char")
            .and_then(|v| v.as_str())
            .unwrap_or("*")
            .chars()
            .next()
            .unwrap_or('*');

        let chars_to_mask = params
            .get("chars_to_mask")
            .and_then(|v| v.as_i64())
            .unwrap_or(text.len() as i64) as usize;

        let from_end = params
            .get("from_end")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let text_len = text.len();
        let mask_count = chars_to_mask.min(text_len);

        let result = if from_end {
            // Mask from the end
            let keep_count = text_len.saturating_sub(mask_count);
            format!(
                "{}{}",
                &text[..keep_count],
                mask_char.to_string().repeat(mask_count)
            )
        } else {
            // Mask from the beginning
            format!(
                "{}{}",
                mask_char.to_string().repeat(mask_count),
                &text[mask_count..]
            )
        };

        Ok(result)
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
    fn test_mask_full() {
        let operator = MaskOperator;
        let params = json!({});
        let result = operator.operate("1234567890", &params).unwrap();
        assert_eq!(result, "**********");
    }

    #[test]
    fn test_mask_partial() {
        let operator = MaskOperator;
        let params = json!({"chars_to_mask": 6});
        let result = operator.operate("1234567890", &params).unwrap();
        assert_eq!(result, "******7890");
    }

    #[test]
    fn test_mask_from_end() {
        let operator = MaskOperator;
        let params = json!({"chars_to_mask": 4, "from_end": true});
        let result = operator.operate("1234567890", &params).unwrap();
        assert_eq!(result, "123456****");
    }

    #[test]
    fn test_mask_custom_char() {
        let operator = MaskOperator;
        let params = json!({"masking_char": "#", "chars_to_mask": 5});
        let result = operator.operate("1234567890", &params).unwrap();
        assert_eq!(result, "#####67890");
    }
}
