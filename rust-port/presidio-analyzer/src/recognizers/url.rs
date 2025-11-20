//! URL recognizer.

use lazy_static::lazy_static;
use presidio_common::{
    EntityRecognizer, EntityType, Language, NlpArtifacts, RecognizerResult, Result,
};
use regex::Regex;

lazy_static! {
    static ref URL_REGEX: Regex = Regex::new(
        r"(?i)\b(https?://|www\.)[a-zA-Z0-9-._~:/?#\[\]@!$&'()*+,;=]+"
    )
    .unwrap();
}

/// Recognizer for URLs.
pub struct UrlRecognizer;

impl UrlRecognizer {
    /// Creates a new URL recognizer.
    pub fn new() -> Self {
        Self
    }
}

impl Default for UrlRecognizer {
    fn default() -> Self {
        Self::new()
    }
}

impl EntityRecognizer for UrlRecognizer {
    fn name(&self) -> &str {
        "UrlRecognizer"
    }

    fn supported_entities(&self) -> &[EntityType] {
        &[EntityType::Url]
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

        for mat in URL_REGEX.find_iter(text) {
            results.push(RecognizerResult::new(
                EntityType::Url,
                mat.start(),
                mat.end(),
                0.5,
            ));
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_detection() {
        let recognizer = UrlRecognizer::new();
        let text = "Visit https://example.com for more information";
        let results = recognizer.analyze(text, None, None, None).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].entity_type, EntityType::Url);
        assert_eq!(&text[results[0].start..results[0].end], "https://example.com");
    }

    #[test]
    fn test_www_url() {
        let recognizer = UrlRecognizer::new();
        let text = "Check out www.example.com/page";
        let results = recognizer.analyze(text, None, None, None).unwrap();

        assert_eq!(results.len(), 1);
    }
}
