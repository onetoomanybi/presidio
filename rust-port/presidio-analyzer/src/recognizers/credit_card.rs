//! Credit card number recognizer with Luhn validation.

use lazy_static::lazy_static;
use presidio_common::{
    EntityRecognizer, EntityType, Language, NlpArtifacts, RecognizerResult, Result,
};
use regex::Regex;

lazy_static! {
    static ref CREDIT_CARD_REGEX: Regex = Regex::new(
        r"\b[0-9]{4}[\s-]?[0-9]{4}[\s-]?[0-9]{4}[\s-]?[0-9]{4}\b"
    )
    .unwrap();
}

/// Recognizer for credit card numbers.
pub struct CreditCardRecognizer;

impl CreditCardRecognizer {
    /// Creates a new credit card recognizer.
    pub fn new() -> Self {
        Self
    }

    /// Validates a credit card number using the Luhn algorithm.
    fn validate_luhn(&self, number: &str) -> bool {
        presidio_common::luhn_checksum(number)
    }
}

impl Default for CreditCardRecognizer {
    fn default() -> Self {
        Self::new()
    }
}

impl EntityRecognizer for CreditCardRecognizer {
    fn name(&self) -> &str {
        "CreditCardRecognizer"
    }

    fn supported_entities(&self) -> &[EntityType] {
        &[EntityType::CreditCard]
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
        context: Option<&[String]>,
    ) -> Result<Vec<RecognizerResult>> {
        let mut results = Vec::new();

        for mat in CREDIT_CARD_REGEX.find_iter(text) {
            let matched_text = mat.as_str();

            // Validate using Luhn checksum
            if !self.validate_luhn(matched_text) {
                continue;
            }

            let mut score: f32 = 0.8; // High confidence due to Luhn validation

            // Check for context words
            if let Some(context_words) = context {
                let context_window = 100;
                let context_start = mat.start().saturating_sub(context_window);
                let context_end = (mat.start() + context_window).min(text.len());
                let context_text = &text[context_start..context_end].to_lowercase();

                for word in context_words {
                    if context_text.contains(&word.to_lowercase()) {
                        score = (score + 0.2).min(1.0);
                        break;
                    }
                }
            }

            results.push(RecognizerResult::new(
                EntityType::CreditCard,
                mat.start(),
                mat.end(),
                score,
            ));
        }

        Ok(results)
    }

    fn validate_result(&self, text: &str) -> bool {
        self.validate_luhn(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credit_card_detection() {
        let recognizer = CreditCardRecognizer::new();
        // Valid Visa test number
        let text = "Card number: 4532 0151 1283 0366";
        let results = recognizer.analyze(text, None, None, None).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].entity_type, EntityType::CreditCard);
        assert!(results[0].score >= 0.8);
    }

    #[test]
    fn test_invalid_credit_card() {
        let recognizer = CreditCardRecognizer::new();
        // Invalid number (doesn't pass Luhn)
        let text = "Card number: 1234 5678 9012 3456";
        let results = recognizer.analyze(text, None, None, None).unwrap();

        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_credit_card_with_context() {
        let recognizer = CreditCardRecognizer::new();
        let text = "My credit card is 4532 0151 1283 0366";
        let context = vec!["card".to_string(), "credit".to_string()];
        let results = recognizer
            .analyze(text, None, None, Some(&context))
            .unwrap();

        assert_eq!(results.len(), 1);
        assert!(results[0].score > 0.8);
    }
}
