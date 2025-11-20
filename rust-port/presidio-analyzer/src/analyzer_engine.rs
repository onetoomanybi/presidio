//! Main analyzer engine for PII detection.

use presidio_common::{EntityType, Language, NlpEngine, RecognizerResult, Result};
use std::sync::Arc;

use crate::RecognizerRegistry;

/// The main analyzer engine for detecting PII in text.
///
/// The analyzer engine coordinates the recognizers and applies
/// context-aware enhancements to improve detection accuracy.
pub struct AnalyzerEngine {
    registry: RecognizerRegistry,
    nlp_engine: Option<Arc<dyn NlpEngine>>,
}

impl AnalyzerEngine {
    /// Creates a new analyzer engine with the given registry.
    pub fn new(registry: RecognizerRegistry) -> Self {
        Self {
            registry,
            nlp_engine: None,
        }
    }

    /// Creates a new analyzer engine with default recognizers.
    pub fn with_defaults() -> Self {
        Self::new(RecognizerRegistry::with_defaults())
    }

    /// Sets the NLP engine to use for processing.
    pub fn with_nlp_engine(mut self, nlp_engine: Arc<dyn NlpEngine>) -> Self {
        self.nlp_engine = Some(nlp_engine);
        self
    }

    /// Analyzes text and returns detected PII entities.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to analyze
    /// * `language` - The language of the text
    /// * `entities` - Optional filter for specific entity types
    /// * `correlation_id` - Optional correlation ID for tracking
    /// * `score_threshold` - Minimum confidence score (0.0 to 1.0)
    /// * `return_decision_process` - Whether to include analysis explanation
    ///
    /// # Returns
    ///
    /// A vector of detected PII entities.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use presidio_analyzer::{AnalyzerEngine, RecognizerRegistry};
    /// use presidio_common::Language;
    ///
    /// let registry = RecognizerRegistry::with_defaults();
    /// let engine = AnalyzerEngine::new(registry);
    ///
    /// let results = engine.analyze(
    ///     "My email is john@example.com",
    ///     Language::En,
    ///     None,
    ///     None,
    ///     0.5,
    ///     false,
    /// ).unwrap();
    ///
    /// println!("Found {} entities", results.len());
    /// ```
    pub fn analyze(
        &self,
        text: &str,
        language: Language,
        entities: Option<&[EntityType]>,
        _correlation_id: Option<&str>,
        score_threshold: f32,
        _return_decision_process: bool,
    ) -> Result<Vec<RecognizerResult>> {
        // Process text with NLP engine if available
        let nlp_artifacts = if let Some(nlp) = &self.nlp_engine {
            Some(nlp.process(text, language)?)
        } else {
            None
        };

        // Get relevant recognizers
        let recognizers = self.registry.get_recognizers(language, entities);

        // Run all recognizers
        let mut all_results = Vec::new();
        for recognizer in recognizers {
            let results = recognizer.analyze(
                text,
                entities,
                nlp_artifacts.as_ref(),
                None,
            )?;
            all_results.extend(results);
        }

        // Remove duplicates
        all_results = presidio_common::remove_duplicates(all_results);

        // Filter by score threshold
        all_results.retain(|r| r.score >= score_threshold);

        // Sort by start position
        all_results.sort_by_key(|r| r.start);

        Ok(all_results)
    }

    /// Returns the recognizer registry.
    pub fn registry(&self) -> &RecognizerRegistry {
        &self.registry
    }

    /// Returns a mutable reference to the recognizer registry.
    pub fn registry_mut(&mut self) -> &mut RecognizerRegistry {
        &mut self.registry
    }
}

impl Default for AnalyzerEngine {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyzer_engine_basic() {
        let registry = RecognizerRegistry::with_defaults();
        let engine = AnalyzerEngine::new(registry);

        let text = "My email is john@example.com and my card is 4532 0151 1283 0366";
        let results = engine
            .analyze(text, Language::En, None, None, 0.0, false)
            .unwrap();

        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.entity_type == EntityType::Email));
        assert!(results.iter().any(|r| r.entity_type == EntityType::CreditCard));
    }

    #[test]
    fn test_analyzer_engine_with_score_threshold() {
        let registry = RecognizerRegistry::with_defaults();
        let engine = AnalyzerEngine::new(registry);

        let text = "My email is john@example.com";
        let results = engine
            .analyze(text, Language::En, None, None, 0.9, false)
            .unwrap();

        // Email recognizer has score 0.5, so with threshold 0.9 it should be filtered out
        // unless context boost applies
        assert!(results.is_empty() || results[0].score >= 0.9);
    }

    #[test]
    fn test_analyzer_engine_entity_filter() {
        let registry = RecognizerRegistry::with_defaults();
        let engine = AnalyzerEngine::new(registry);

        let text = "Email: john@example.com, IP: 192.168.1.1";
        let results = engine
            .analyze(
                text,
                Language::En,
                Some(&[EntityType::Email]),
                None,
                0.0,
                false,
            )
            .unwrap();

        assert!(results.iter().all(|r| r.entity_type == EntityType::Email));
    }

    #[test]
    fn test_multiple_entities() {
        let registry = RecognizerRegistry::with_defaults();
        let engine = AnalyzerEngine::new(registry);

        let text = "Contact: john@example.com, jane@example.org, Phone: 555-123-4567";
        let results = engine
            .analyze(text, Language::En, None, None, 0.0, false)
            .unwrap();

        assert!(results.len() >= 2); // At least 2 emails
    }
}
