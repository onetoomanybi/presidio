//! Core traits for Presidio components.

use crate::{EntityType, Language, NlpArtifacts, RecognizerResult, Result};

/// Trait for PII entity recognizers.
///
/// All recognizers must implement this trait to be used by the analyzer engine.
/// Recognizers can be pattern-based, NLP-based, or use any other detection method.
pub trait EntityRecognizer: Send + Sync {
    /// Returns the name of this recognizer.
    fn name(&self) -> &str;

    /// Returns the entity types this recognizer can detect.
    fn supported_entities(&self) -> &[EntityType];

    /// Returns the languages this recognizer supports.
    fn supported_languages(&self) -> &[Language];

    /// Analyzes text and returns detected entities.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to analyze
    /// * `entities` - Optional filter for specific entity types
    /// * `nlp_artifacts` - Optional NLP processing results
    /// * `context` - Optional context words for score enhancement
    ///
    /// # Returns
    ///
    /// A vector of detected PII entities.
    fn analyze(
        &self,
        text: &str,
        entities: Option<&[EntityType]>,
        nlp_artifacts: Option<&NlpArtifacts>,
        context: Option<&[String]>,
    ) -> Result<Vec<RecognizerResult>>;

    /// Validates a detected match.
    ///
    /// This can be used for checksums, format validation, etc.
    /// Default implementation always returns true.
    fn validate_result(&self, _text: &str) -> bool {
        true
    }

    /// Returns whether this recognizer supports the given language.
    fn supports_language(&self, language: Language) -> bool {
        self.supported_languages().contains(&language)
    }

    /// Returns whether this recognizer supports the given entity type.
    fn supports_entity(&self, entity_type: &EntityType) -> bool {
        self.supported_entities().contains(entity_type)
    }
}

/// Trait for NLP engines that process text.
///
/// NLP engines extract linguistic features like tokens, lemmas, POS tags,
/// and named entities that can be used by recognizers.
pub trait NlpEngine: Send + Sync {
    /// Processes text and returns NLP artifacts.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to process
    /// * `language` - The language of the text
    ///
    /// # Returns
    ///
    /// NLP artifacts including tokens, lemmas, POS tags, and entities.
    fn process(&self, text: &str, language: Language) -> Result<NlpArtifacts>;

    /// Returns whether this NLP engine supports the given language.
    fn is_available(&self, language: Language) -> bool;

    /// Returns the name of this NLP engine.
    fn name(&self) -> &str;
}

/// Trait for context-aware score enhancement.
///
/// Context-aware enhancers boost confidence scores based on surrounding words or linguistic context.
pub trait ContextAwareEnhancer: Send + Sync {
    /// Enhances recognizer results using context.
    ///
    /// # Arguments
    ///
    /// * `results` - The initial recognizer results
    /// * `text` - The original text
    /// * `nlp_artifacts` - Optional NLP processing results
    ///
    /// # Returns
    ///
    /// Enhanced results with potentially higher scores.
    fn enhance_using_context(
        &self,
        results: &[RecognizerResult],
        text: &str,
        nlp_artifacts: Option<&NlpArtifacts>,
    ) -> Vec<RecognizerResult>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockRecognizer;

    impl EntityRecognizer for MockRecognizer {
        fn name(&self) -> &str {
            "mock"
        }

        fn supported_entities(&self) -> &[EntityType] {
            &[EntityType::Email]
        }

        fn supported_languages(&self) -> &[Language] {
            &[Language::En]
        }

        fn analyze(
            &self,
            _text: &str,
            _entities: Option<&[EntityType]>,
            _nlp_artifacts: Option<&NlpArtifacts>,
            _context: Option<&[String]>,
        ) -> Result<Vec<RecognizerResult>> {
            Ok(vec![])
        }
    }

    #[test]
    fn test_recognizer_trait() {
        let recognizer = MockRecognizer;
        assert_eq!(recognizer.name(), "mock");
        assert!(recognizer.supports_language(Language::En));
        assert!(!recognizer.supports_language(Language::Es));
        assert!(recognizer.supports_entity(&EntityType::Email));
        assert!(!recognizer.supports_entity(&EntityType::Phone));
    }
}
