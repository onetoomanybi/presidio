//! Pattern-based PII recognizer using regex.

use presidio_common::{
    AnalysisExplanation, EntityRecognizer, EntityType, Language, NlpArtifacts, Pattern,
    RecognizerResult, Result, ValidationResult,
};

/// A pattern-based recognizer that uses regular expressions to detect PII.
pub struct PatternRecognizer {
    /// Name of the recognizer
    name: String,

    /// Entity types this recognizer detects
    supported_entities: Vec<EntityType>,

    /// Languages this recognizer supports
    supported_languages: Vec<Language>,

    /// Patterns to match
    patterns: Vec<Pattern>,

    /// Optional context words that boost confidence
    context: Option<Vec<String>>,

    /// Whether to perform validation
    validate: bool,
}

impl PatternRecognizer {
    /// Creates a new pattern recognizer.
    pub fn new(
        name: impl Into<String>,
        supported_entities: Vec<EntityType>,
        supported_languages: Vec<Language>,
        patterns: Vec<Pattern>,
    ) -> Self {
        Self {
            name: name.into(),
            supported_entities,
            supported_languages,
            patterns,
            context: None,
            validate: false,
        }
    }

    /// Sets context words for score enhancement.
    pub fn with_context(mut self, context: Vec<String>) -> Self {
        self.context = Some(context);
        self
    }

    /// Enables validation.
    pub fn with_validation(mut self, validate: bool) -> Self {
        self.validate = validate;
        self
    }

    /// Validates a matched string.
    ///
    /// Override this in specific recognizers for custom validation logic.
    pub fn validate_pattern(&self, _text: &str) -> bool {
        true
    }

    /// Enhances score based on context.
    fn enhance_with_context(
        &self,
        score: f32,
        text: &str,
        start: usize,
    ) -> (f32, Option<String>) {
        if let Some(context_words) = &self.context {
            // Look for context words in the surrounding text
            let context_window = 100;
            let context_start = start.saturating_sub(context_window);
            let context_end = (start + context_window).min(text.len());
            let context_text = &text[context_start..context_end];

            for word in context_words {
                if context_text.to_lowercase().contains(&word.to_lowercase()) {
                    // Boost score by 0.1 if context word is found
                    return (
                        (score + 0.1).min(1.0),
                        Some(word.clone()),
                    );
                }
            }
        }
        (score, None)
    }
}

impl EntityRecognizer for PatternRecognizer {
    fn name(&self) -> &str {
        &self.name
    }

    fn supported_entities(&self) -> &[EntityType] {
        &self.supported_entities
    }

    fn supported_languages(&self) -> &[Language] {
        &self.supported_languages
    }

    fn analyze(
        &self,
        text: &str,
        entities: Option<&[EntityType]>,
        _nlp_artifacts: Option<&NlpArtifacts>,
        _context: Option<&[String]>,
    ) -> Result<Vec<RecognizerResult>> {
        // Filter entities if specified
        if let Some(entity_filter) = entities {
            if !self
                .supported_entities
                .iter()
                .any(|e| entity_filter.contains(e))
            {
                return Ok(vec![]);
            }
        }

        let mut results = Vec::new();

        for pattern in &self.patterns {
            for mat in pattern.regex.find_iter(text) {
                let matched_text = mat.as_str();

                // Check deny list
                if let Some(deny_list) = &pattern.deny_list {
                    if deny_list
                        .iter()
                        .any(|d| matched_text.to_lowercase().contains(&d.to_lowercase()))
                    {
                        continue;
                    }
                }

                // Perform validation if enabled
                let is_valid = if self.validate {
                    self.validate_pattern(matched_text)
                } else {
                    true
                };

                if !is_valid {
                    continue;
                }

                // Get base score and enhance with context
                let (score, supportive_word) =
                    self.enhance_with_context(pattern.score, text, mat.start());

                // Create explanation
                let explanation = AnalysisExplanation {
                    recognizer: self.name.clone(),
                    pattern_name: Some(pattern.name.clone()),
                    pattern: Some(pattern.regex.as_str().to_string()),
                    original_score: pattern.score,
                    score,
                    textual_explanation: Some(format!(
                        "Detected using pattern: {}",
                        pattern.name
                    )),
                    score_context_improvement: score - pattern.score,
                    supportive_context_word: supportive_word,
                    validation_result: if self.validate {
                        Some(ValidationResult {
                            is_valid,
                            validation_method: "pattern_validation".to_string(),
                            details: None,
                        })
                    } else {
                        None
                    },
                };

                let result = RecognizerResult::with_explanation(
                    self.supported_entities[0].clone(),
                    mat.start(),
                    mat.end(),
                    score,
                    explanation,
                );

                results.push(result);
            }
        }

        Ok(results)
    }

    fn validate_result(&self, text: &str) -> bool {
        if self.validate {
            self.validate_pattern(text)
        } else {
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    #[test]
    fn test_pattern_recognizer_basic() {
        let pattern = Pattern::new(
            "test_pattern",
            Regex::new(r"\b\d{3}-\d{4}\b").unwrap(),
            0.9,
        );

        let recognizer = PatternRecognizer::new(
            "test",
            vec![EntityType::Phone],
            vec![Language::En],
            vec![pattern],
        );

        let text = "Call me at 555-1234";
        let results = recognizer.analyze(text, None, None, None).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].start, 11);
        assert_eq!(results[0].end, 19);
        assert_eq!(results[0].score, 0.9);
    }

    #[test]
    fn test_pattern_recognizer_with_context() {
        let pattern = Pattern::new(
            "test_pattern",
            Regex::new(r"\b\d{3}-\d{4}\b").unwrap(),
            0.7,
        );

        let recognizer = PatternRecognizer::new(
            "test",
            vec![EntityType::Phone],
            vec![Language::En],
            vec![pattern],
        )
        .with_context(vec!["phone".to_string(), "call".to_string()]);

        let text = "My phone number is 555-1234";
        let results = recognizer.analyze(text, None, None, None).unwrap();

        assert_eq!(results.len(), 1);
        assert!(results[0].score > 0.7); // Should be enhanced
    }

    #[test]
    fn test_pattern_recognizer_deny_list() {
        let pattern = Pattern::new(
            "test_pattern",
            Regex::new(r"\b\d{3}-\d{4}\b").unwrap(),
            0.9,
        )
        .with_deny_list(vec!["000-0000".to_string()]);

        let recognizer = PatternRecognizer::new(
            "test",
            vec![EntityType::Phone],
            vec![Language::En],
            vec![pattern],
        );

        let text = "Numbers: 555-1234 and 000-0000";
        let results = recognizer.analyze(text, None, None, None).unwrap();

        assert_eq!(results.len(), 1); // Only 555-1234, not 000-0000
        assert_eq!(results[0].start, 9);
    }
}
