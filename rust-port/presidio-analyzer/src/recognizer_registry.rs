//! Recognizer registry for managing PII recognizers.

use presidio_common::{EntityRecognizer, EntityType, Language, PresidioError, Result};
use std::collections::HashMap;
use std::sync::Arc;

/// Registry for managing entity recognizers.
///
/// The registry holds all available recognizers and provides methods to
/// query recognizers by language and entity type.
#[derive(Clone)]
pub struct RecognizerRegistry {
    recognizers: HashMap<String, Arc<dyn EntityRecognizer>>,
}

impl RecognizerRegistry {
    /// Creates a new empty registry.
    pub fn new() -> Self {
        Self {
            recognizers: HashMap::new(),
        }
    }

    /// Creates a new registry with default recognizers loaded.
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();

        // Add default generic recognizers
        registry.add_recognizer(Arc::new(
            crate::recognizers::EmailRecognizer::new()
        ));
        registry.add_recognizer(Arc::new(
            crate::recognizers::UrlRecognizer::new()
        ));
        registry.add_recognizer(Arc::new(
            crate::recognizers::PhoneRecognizer::new()
        ));
        registry.add_recognizer(Arc::new(
            crate::recognizers::CreditCardRecognizer::new()
        ));
        registry.add_recognizer(Arc::new(
            crate::recognizers::IpAddressRecognizer::new()
        ));

        registry
    }

    /// Adds a recognizer to the registry.
    pub fn add_recognizer(&mut self, recognizer: Arc<dyn EntityRecognizer>) {
        self.recognizers.insert(recognizer.name().to_string(), recognizer);
    }

    /// Removes a recognizer from the registry.
    pub fn remove_recognizer(&mut self, name: &str) -> Result<()> {
        self.recognizers
            .remove(name)
            .ok_or_else(|| PresidioError::RecognizerNotFound(name.to_string()))?;
        Ok(())
    }

    /// Gets a recognizer by name.
    pub fn get_recognizer(&self, name: &str) -> Option<Arc<dyn EntityRecognizer>> {
        self.recognizers.get(name).cloned()
    }

    /// Gets all recognizers that support a given language and optionally filter by entity types.
    pub fn get_recognizers(
        &self,
        language: Language,
        entities: Option<&[EntityType]>,
    ) -> Vec<Arc<dyn EntityRecognizer>> {
        self.recognizers
            .values()
            .filter(|r| r.supports_language(language))
            .filter(|r| {
                if let Some(entity_filter) = entities {
                    r.supported_entities()
                        .iter()
                        .any(|e| entity_filter.contains(e))
                } else {
                    true
                }
            })
            .cloned()
            .collect()
    }

    /// Gets all recognizer names.
    pub fn get_recognizer_names(&self) -> Vec<String> {
        self.recognizers.keys().cloned().collect()
    }

    /// Gets all supported entity types across all recognizers.
    pub fn get_supported_entities(&self) -> Vec<EntityType> {
        let mut entities: Vec<EntityType> = self
            .recognizers
            .values()
            .flat_map(|r| r.supported_entities().iter().cloned())
            .collect();
        entities.sort_by_key(|e| e.as_str().to_string());
        entities.dedup();
        entities
    }

    /// Gets all supported languages across all recognizers.
    pub fn get_supported_languages(&self) -> Vec<Language> {
        let mut languages: Vec<Language> = self
            .recognizers
            .values()
            .flat_map(|r| r.supported_languages().iter().copied())
            .collect();
        languages.sort_by_key(|l| l.as_str());
        languages.dedup();
        languages
    }

    /// Returns the number of recognizers in the registry.
    pub fn len(&self) -> usize {
        self.recognizers.len()
    }

    /// Returns whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.recognizers.is_empty()
    }
}

impl Default for RecognizerRegistry {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_new() {
        let registry = RecognizerRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_registry_with_defaults() {
        let registry = RecognizerRegistry::with_defaults();
        assert!(!registry.is_empty());
        assert!(registry.len() >= 5); // At least 5 default recognizers
    }

    #[test]
    fn test_get_recognizers_by_language() {
        let registry = RecognizerRegistry::with_defaults();
        let recognizers = registry.get_recognizers(Language::En, None);
        assert!(!recognizers.is_empty());
    }

    #[test]
    fn test_get_supported_entities() {
        let registry = RecognizerRegistry::with_defaults();
        let entities = registry.get_supported_entities();
        assert!(entities.contains(&EntityType::Email));
        assert!(entities.contains(&EntityType::CreditCard));
    }
}
