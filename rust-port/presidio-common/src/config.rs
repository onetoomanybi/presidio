//! Configuration support for Presidio recognizers.
//!
//! This module provides YAML-based configuration for defining custom recognizers
//! without writing Rust code.

use crate::{EntityType, Language};
use serde::{Deserialize, Serialize};

/// YAML configuration for a pattern-based recognizer.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RecognizerConfig {
    /// Name of the recognizer
    pub name: String,

    /// Supported entity types
    pub supported_entities: Vec<String>,

    /// Supported languages
    pub supported_languages: Vec<String>,

    /// Regex patterns for recognition
    pub patterns: Vec<PatternConfig>,

    /// Optional context words that increase confidence
    #[serde(default)]
    pub context: Vec<String>,

    /// Optional deny list (patterns to exclude)
    #[serde(default)]
    pub deny_list: Vec<String>,

    /// Optional validation function name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation: Option<String>,
}

/// Configuration for a single pattern.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PatternConfig {
    /// Name of the pattern
    pub name: String,

    /// Regex pattern string
    pub regex: String,

    /// Base score for matches (0.0 to 1.0)
    pub score: f32,
}

/// Complete configuration file structure.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PresidioConfig {
    /// List of recognizer configurations
    pub recognizers: Vec<RecognizerConfig>,

    /// Global settings
    #[serde(default)]
    pub settings: GlobalSettings,
}

/// Global settings for Presidio.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GlobalSettings {
    /// Default language
    #[serde(default = "default_language")]
    pub default_language: String,

    /// Default score threshold
    #[serde(default = "default_threshold")]
    pub default_threshold: f32,

    /// Enable NLP processing
    #[serde(default)]
    pub enable_nlp: bool,
}

fn default_language() -> String {
    "en".to_string()
}

fn default_threshold() -> f32 {
    0.5
}

impl Default for GlobalSettings {
    fn default() -> Self {
        Self {
            default_language: default_language(),
            default_threshold: default_threshold(),
            enable_nlp: false,
        }
    }
}

impl RecognizerConfig {
    /// Loads recognizer configuration from a YAML string.
    pub fn from_yaml(yaml: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(yaml)
    }

    /// Converts the configuration to YAML string.
    pub fn to_yaml(&self) -> Result<String, serde_yaml::Error> {
        serde_yaml::to_string(self)
    }

    /// Parses entity types from string names.
    pub fn get_entity_types(&self) -> Vec<EntityType> {
        self.supported_entities
            .iter()
            .map(|s| EntityType::from_str(s))
            .collect()
    }

    /// Parses languages from string names.
    pub fn get_languages(&self) -> Vec<Language> {
        self.supported_languages
            .iter()
            .filter_map(|s| Language::from_str(s))
            .collect()
    }
}

impl PresidioConfig {
    /// Loads configuration from a YAML string.
    pub fn from_yaml(yaml: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(yaml)
    }

    /// Converts the configuration to YAML string.
    pub fn to_yaml(&self) -> Result<String, serde_yaml::Error> {
        serde_yaml::to_string(self)
    }

    /// Creates an example configuration.
    pub fn example() -> Self {
        Self {
            recognizers: vec![
                RecognizerConfig {
                    name: "custom_id_recognizer".to_string(),
                    supported_entities: vec!["CUSTOM_ID".to_string()],
                    supported_languages: vec!["en".to_string()],
                    patterns: vec![PatternConfig {
                        name: "custom_id_pattern".to_string(),
                        regex: r"\b[A-Z]{3}-\d{6}\b".to_string(),
                        score: 0.85,
                    }],
                    context: vec!["ID".to_string(), "identifier".to_string()],
                    deny_list: vec![],
                    validation: None,
                },
            ],
            settings: GlobalSettings::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recognizer_config_from_yaml() {
        let yaml = r#"
name: test_recognizer
supported_entities:
  - EMAIL
supported_languages:
  - en
patterns:
  - name: email_pattern
    regex: '\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b'
    score: 0.9
context:
  - email
  - e-mail
deny_list: []
        "#;

        let config = RecognizerConfig::from_yaml(yaml).unwrap();
        assert_eq!(config.name, "test_recognizer");
        assert_eq!(config.supported_entities, vec!["EMAIL"]);
        assert_eq!(config.patterns.len(), 1);
        assert_eq!(config.patterns[0].score, 0.9);
    }

    #[test]
    fn test_presidio_config_example() {
        let config = PresidioConfig::example();
        assert_eq!(config.recognizers.len(), 1);
        assert_eq!(config.settings.default_language, "en");
        assert_eq!(config.settings.default_threshold, 0.5);

        // Should be able to serialize and deserialize
        let yaml = config.to_yaml().unwrap();
        let parsed = PresidioConfig::from_yaml(&yaml).unwrap();
        assert_eq!(parsed.recognizers.len(), 1);
    }

    #[test]
    fn test_get_entity_types() {
        let config = RecognizerConfig {
            name: "test".to_string(),
            supported_entities: vec!["EMAIL".to_string(), "PHONE_NUMBER".to_string()],
            supported_languages: vec!["en".to_string()],
            patterns: vec![],
            context: vec![],
            deny_list: vec![],
            validation: None,
        };

        let entity_types = config.get_entity_types();
        assert_eq!(entity_types.len(), 2);
    }

    #[test]
    fn test_get_languages() {
        let config = RecognizerConfig {
            name: "test".to_string(),
            supported_entities: vec![],
            supported_languages: vec!["en".to_string(), "es".to_string(), "invalid".to_string()],
            patterns: vec![],
            context: vec![],
            deny_list: vec![],
            validation: None,
        };

        let languages = config.get_languages();
        assert_eq!(languages.len(), 2); // "invalid" should be filtered out
    }
}
