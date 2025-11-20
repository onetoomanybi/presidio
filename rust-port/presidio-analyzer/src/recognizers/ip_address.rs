//! IP address recognizer (IPv4 and IPv6).

use lazy_static::lazy_static;
use presidio_common::{
    EntityRecognizer, EntityType, Language, NlpArtifacts, RecognizerResult, Result,
};
use regex::Regex;

lazy_static! {
    static ref IPV4_REGEX: Regex = Regex::new(
        r"\b(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\b"
    )
    .unwrap();

    static ref IPV6_REGEX: Regex = Regex::new(
        r"(?i)\b(?:[0-9a-f]{1,4}:){7}[0-9a-f]{1,4}\b"
    )
    .unwrap();
}

/// Recognizer for IP addresses (IPv4 and IPv6).
pub struct IpAddressRecognizer;

impl IpAddressRecognizer {
    /// Creates a new IP address recognizer.
    pub fn new() -> Self {
        Self
    }
}

impl Default for IpAddressRecognizer {
    fn default() -> Self {
        Self::new()
    }
}

impl EntityRecognizer for IpAddressRecognizer {
    fn name(&self) -> &str {
        "IpAddressRecognizer"
    }

    fn supported_entities(&self) -> &[EntityType] {
        &[EntityType::IpAddress]
    }

    fn supported_languages(&self) -> &[Language] {
        &[
            Language::En,
            Language::Es,
            Language::Fr,
            Language::De,
            Language::It,
            Language::Pt,
            Language::Nl,
            Language::Pl,
            Language::Fi,
        ]
    }

    fn analyze(
        &self,
        text: &str,
        _entities: Option<&[EntityType]>,
        _nlp_artifacts: Option<&NlpArtifacts>,
        _context: Option<&[String]>,
    ) -> Result<Vec<RecognizerResult>> {
        let mut results = Vec::new();

        // Check for IPv4
        for mat in IPV4_REGEX.find_iter(text) {
            results.push(RecognizerResult::new(
                EntityType::IpAddress,
                mat.start(),
                mat.end(),
                0.6,
            ));
        }

        // Check for IPv6
        for mat in IPV6_REGEX.find_iter(text) {
            results.push(RecognizerResult::new(
                EntityType::IpAddress,
                mat.start(),
                mat.end(),
                0.6,
            ));
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipv4_detection() {
        let recognizer = IpAddressRecognizer::new();
        let text = "Server IP: 192.168.1.1";
        let results = recognizer.analyze(text, None, None, None).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].entity_type, EntityType::IpAddress);
        assert_eq!(&text[results[0].start..results[0].end], "192.168.1.1");
    }

    #[test]
    fn test_ipv6_detection() {
        let recognizer = IpAddressRecognizer::new();
        let text = "IPv6: 2001:0db8:85a3:0000:0000:8a2e:0370:7334";
        let results = recognizer.analyze(text, None, None, None).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].entity_type, EntityType::IpAddress);
    }

    #[test]
    fn test_multiple_ips() {
        let recognizer = IpAddressRecognizer::new();
        let text = "Connect to 10.0.0.1 or 192.168.1.100";
        let results = recognizer.analyze(text, None, None, None).unwrap();

        assert_eq!(results.len(), 2);
    }
}
