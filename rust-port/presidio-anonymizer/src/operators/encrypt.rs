//! Encrypt operator - encrypts PII using AES-GCM.

use super::Operator;
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use presidio_common::{PresidioError, Result};
use serde_json::Value;

/// Operator that encrypts PII using AES-256-GCM.
pub struct EncryptOperator;

impl Operator for EncryptOperator {
    fn name(&self) -> &str {
        "encrypt"
    }

    fn operate(&self, text: &str, params: &Value) -> Result<String> {
        let key_str = params
            .get("key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                PresidioError::InvalidOperatorConfig("Encryption key required".to_string())
            })?;

        // Key must be 32 bytes for AES-256
        let key_bytes = if key_str.len() >= 32 {
            key_str.as_bytes()[..32].to_vec()
        } else {
            // Pad key if too short
            let mut padded = key_str.as_bytes().to_vec();
            padded.resize(32, 0);
            padded
        };

        let cipher = Aes256Gcm::new_from_slice(&key_bytes).map_err(|e| {
            PresidioError::InvalidOperatorConfig(format!("Invalid encryption key: {}", e))
        })?;

        // Generate a random nonce (12 bytes for AES-GCM)
        let nonce_bytes: [u8; 12] = {
            use aes_gcm::aead::rand_core::RngCore;
            let mut bytes = [0u8; 12];
            OsRng.fill_bytes(&mut bytes);
            bytes
        };
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher.encrypt(nonce, text.as_bytes()).map_err(|e| {
            PresidioError::Other(format!("Encryption failed: {}", e))
        })?;

        // Combine nonce and ciphertext for storage
        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&ciphertext);

        // Return as base64
        Ok(base64_encode(&result))
    }

    fn validate(&self, params: &Value) -> Result<()> {
        if params.get("key").and_then(|v| v.as_str()).is_none() {
            return Err(PresidioError::InvalidOperatorConfig(
                "Encryption key required".to_string(),
            ));
        }
        Ok(())
    }
}

// Simple base64 encoding without external dependency
fn base64_encode(bytes: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();

    for chunk in bytes.chunks(3) {
        let b1 = chunk[0];
        let b2 = chunk.get(1).copied().unwrap_or(0);
        let b3 = chunk.get(2).copied().unwrap_or(0);

        result.push(CHARS[(b1 >> 2) as usize] as char);
        result.push(CHARS[(((b1 & 0x03) << 4) | (b2 >> 4)) as usize] as char);
        result.push(if chunk.len() > 1 {
            CHARS[(((b2 & 0x0f) << 2) | (b3 >> 6)) as usize] as char
        } else {
            '='
        });
        result.push(if chunk.len() > 2 {
            CHARS[(b3 & 0x3f) as usize] as char
        } else {
            '='
        });
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_encrypt() {
        let operator = EncryptOperator;
        let params = json!({"key": "my-secret-encryption-key-32bytes"});
        let result = operator.operate("sensitive data", &params).unwrap();

        // Should return base64-encoded data
        assert!(!result.is_empty());
        assert!(result.chars().all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '='));
    }

    #[test]
    fn test_encrypt_no_key() {
        let operator = EncryptOperator;
        let params = json!({});
        assert!(operator.validate(&params).is_err());
    }

    #[test]
    fn test_encrypt_different_results() {
        let operator = EncryptOperator;
        let params = json!({"key": "my-secret-encryption-key-32bytes"});
        let result1 = operator.operate("test", &params).unwrap();
        let result2 = operator.operate("test", &params).unwrap();

        // Due to random nonce, results should be different
        assert_ne!(result1, result2);
    }
}
