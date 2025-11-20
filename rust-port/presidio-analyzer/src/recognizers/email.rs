//! Email address recognizer.

use lazy_static::lazy_static;
use presidio_common::{
    EntityRecognizer, EntityType, Language, NlpArtifacts, Pattern, RecognizerResult, Result,
};
use regex::Regex;

lazy_static! {
    static ref EMAIL_REGEX: Regex = Regex::new(
        r"(?i)\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b"
    )
    .unwrap();
}

/// Recognizer for email addresses.
pub struct EmailRecognizer {
    pattern: Pattern,
}

impl EmailRecognizer {
    /// Creates a new email recognizer.
    pub fn new() -> Self {
        Self {
            pattern: Pattern::new("email_pattern", EMAIL_REGEX.clone(), 0.5)
                .with_context(vec![
                    "email".to_string(),
                    "mail".to_string(),
                    "e-mail".to_string(),
                    "contact".to_string(),
                ]),
        }
    }

    fn enhance_with_context(&self, score: f32, text: &str, start: usize) -> f32 {
        let context_window = 100;
        let context_start = start.saturating_sub(context_window);
        let context_end = (start + context_window).min(text.len());
        let context_text = &text[context_start..context_end].to_lowercase();

        if let Some(context_words) = &self.pattern.context {
            for word in context_words {
                if context_text.contains(&word.to_lowercase()) {
                    return (score + 0.4).min(1.0);
                }
            }
        }

        score
    }
}

impl Default for EmailRecognizer {
    fn default() -> Self {
        Self::new()
    }
}

impl EntityRecognizer for EmailRecognizer {
    fn name(&self) -> &str {
        "EmailRecognizer"
    }

    fn supported_entities(&self) -> &[EntityType] {
        &[EntityType::Email]
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
            Language::Ko,
            Language::Th,
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

        for mat in EMAIL_REGEX.find_iter(text) {
            let score = self.enhance_with_context(0.5, text, mat.start());

            results.push(RecognizerResult::new(
                EntityType::Email,
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
    fn test_email_detection() {
        let recognizer = EmailRecognizer::new();
        let text = "Contact me at john.doe@example.com for more info";
        let results = recognizer.analyze(text, None, None, None).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].entity_type, EntityType::Email);
        assert_eq!(&text[results[0].start..results[0].end], "john.doe@example.com");
    }

    #[test]
    fn test_email_with_context() {
        let recognizer = EmailRecognizer::new();
        let text = "Please email me at john@example.com";
        let results = recognizer.analyze(text, None, None, None).unwrap();

        assert_eq!(results.len(), 1);
        assert!(results[0].score > 0.5); // Should be enhanced due to "email" context
    }

    #[test]
    fn test_multiple_emails() {
        let recognizer = EmailRecognizer::new();
        let text = "Contact john@example.com or jane@example.org";
        let results = recognizer.analyze(text, None, None, None).unwrap();

        assert_eq!(results.len(), 2);
    }
}
