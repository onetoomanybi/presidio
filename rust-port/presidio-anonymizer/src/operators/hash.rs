//! Hash operator - hashes PII using SHA-256 or SHA-512.

use super::Operator;
use presidio_common::{PresidioError, Result};
use serde_json::Value;
use sha2::{Digest, Sha256, Sha512};

/// Operator that hashes PII.
pub struct HashOperator;

impl Operator for HashOperator {
    fn name(&self) -> &str {
        "hash"
    }

    fn operate(&self, text: &str, params: &Value) -> Result<String> {
        let hash_type = params
            .get("hash_type")
            .and_then(|v| v.as_str())
            .unwrap_or("sha256");

        let hash = match hash_type {
            "sha256" => {
                let mut hasher = Sha256::new();
                hasher.update(text.as_bytes());
                format!("{:x}", hasher.finalize())
            }
            "sha512" => {
                let mut hasher = Sha512::new();
                hasher.update(text.as_bytes());
                format!("{:x}", hasher.finalize())
            }
            _ => {
                return Err(PresidioError::InvalidOperatorConfig(format!(
                    "Unsupported hash type: {}",
                    hash_type
                )))
            }
        };

        Ok(hash)
    }

    fn validate(&self, params: &Value) -> Result<()> {
        if let Some(hash_type) = params.get("hash_type").and_then(|v| v.as_str()) {
            if hash_type != "sha256" && hash_type != "sha512" {
                return Err(PresidioError::InvalidOperatorConfig(format!(
                    "Unsupported hash type: {}. Use 'sha256' or 'sha512'",
                    hash_type
                )));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_hash_sha256() {
        let operator = HashOperator;
        let params = json!({"hash_type": "sha256"});
        let result = operator.operate("test", &params).unwrap();
        // SHA-256 hash of "test"
        assert_eq!(
            result,
            "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"
        );
    }

    #[test]
    fn test_hash_sha512() {
        let operator = HashOperator;
        let params = json!({"hash_type": "sha512"});
        let result = operator.operate("test", &params).unwrap();
        assert_eq!(result.len(), 128); // SHA-512 produces 128 hex characters
    }

    #[test]
    fn test_hash_default() {
        let operator = HashOperator;
        let params = json!({});
        let result = operator.operate("test", &params).unwrap();
        assert_eq!(
            result,
            "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"
        );
    }

    #[test]
    fn test_hash_invalid_type() {
        let operator = HashOperator;
        let params = json!({"hash_type": "md5"});
        assert!(operator.validate(&params).is_err());
    }
}
