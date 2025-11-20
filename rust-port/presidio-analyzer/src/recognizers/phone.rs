//! Phone number recognizer.

use lazy_static::lazy_static;
use presidio_common::{
    EntityRecognizer, EntityType, Language, NlpArtifacts, RecognizerResult, Result,
};
use regex::Regex;

lazy_static! {
    static ref PHONE_REGEX: Regex = Regex::new(
        r"(?:\+?1[-.\s]?)?\(?([0-9]{3})\)?[-.\s]?([0-9]{3})[-.\s]?([0-9]{4})"
    )
    .unwrap();
}

/// Recognizer for phone numbers.
pub struct PhoneRecognizer;

impl PhoneRecognizer {
    /// Creates a new phone recognizer.
    pub fn new() -> Self {
        Self
    }
}

impl Default for PhoneRecognizer {
    fn default() -> Self {
        Self::new()
    }
}

impl EntityRecognizer for PhoneRecognizer {
    fn name(&self) -> &str {
        "PhoneRecognizer"
    }

    fn supported_entities(&self) -> &[EntityType] {
        &[EntityType::Phone]
    }

    fn supported_languages(&self) -> &[Language] {
        &[Language::En]
    }

    fn analyze(
        &self,
        text: &str,
        _entities: Option<&[EntityType]>,
        _nlp_artifacts: Option<&NlpArtifacts>,
        context: Option<&[String]>,
    ) -> Result<Vec<RecognizerResult>> {
        let mut results = Vec::new();

        for mat in PHONE_REGEX.find_iter(text) {
            let mut score: f32 = 0.4;

            // Check for context words
            if let Some(context_words) = context {
                let context_window = 100;
                let context_start = mat.start().saturating_sub(context_window);
                let context_end = (mat.start() + context_window).min(text.len());
                let context_text = &text[context_start..context_end].to_lowercase();

                for word in context_words {
                    if context_text.contains(&word.to_lowercase()) {
                        score = (score + 0.4).min(1.0);
                        break;
                    }
                }
            }

            results.push(RecognizerResult::new(
                EntityType::Phone,
                mat.start(),
                mat.end(),
                score,
            ));
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phone_detection() {
        let recognizer = PhoneRecognizer::new();
        let text = "Call me at 555-123-4567";
        let results = recognizer.analyze(text, None, None, None).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].entity_type, EntityType::Phone);
    }

    #[test]
    fn test_phone_with_context() {
        let recognizer = PhoneRecognizer::new();
        let text = "My phone is 555-123-4567";
        let context = vec!["phone".to_string()];
        let results = recognizer
            .analyze(text, None, None, Some(&context))
            .unwrap();

        assert_eq!(results.len(), 1);
        assert!(results[0].score > 0.4);
    }
}
