//! Structured data support for Presidio.
//!
//! This module provides functionality for analyzing and anonymizing structured data
//! formats like JSON, with path-based field targeting.

use presidio_analyzer::AnalyzerEngine;
use presidio_anonymizer::AnonymizerEngine;
use presidio_common::{ConflictResolutionStrategy, EntityType, Language, RecognizerResult};
use serde_json::Value;
use std::collections::HashMap;

/// Engine for handling structured data (JSON) PII detection and anonymization.
pub struct StructuredEngine {
    analyzer: AnalyzerEngine,
    anonymizer: AnonymizerEngine,
}

/// Configuration for analyzing/anonymizing specific JSON paths.
#[derive(Debug, Clone)]
pub struct PathConfig {
    /// JSON path (e.g., "user.email", "contacts[*].phone")
    pub path: String,
    /// Entity types to detect at this path
    pub entity_types: Option<Vec<EntityType>>,
    /// Operator to use for anonymization
    pub operator: Option<(String, Value)>,
}

/// Result of structured data analysis.
#[derive(Debug)]
pub struct StructuredAnalysisResult {
    /// Path where PII was found
    pub path: String,
    /// The original value
    pub value: String,
    /// Detection results
    pub results: Vec<RecognizerResult>,
}

/// Result of structured data anonymization.
#[derive(Debug)]
pub struct StructuredAnonymizationResult {
    /// The anonymized JSON
    pub data: Value,
    /// Details of anonymization operations
    pub operations: Vec<AnonymizationOperation>,
}

#[derive(Debug)]
pub struct AnonymizationOperation {
    pub path: String,
    pub original_value: String,
    pub anonymized_value: String,
    pub operator: String,
}

impl StructuredEngine {
    /// Creates a new structured engine with default analyzers and anonymizers.
    pub fn new() -> Self {
        Self {
            analyzer: AnalyzerEngine::with_defaults(),
            anonymizer: AnonymizerEngine::with_defaults(),
        }
    }

    /// Creates a structured engine with custom analyzer and anonymizer.
    pub fn with_engines(analyzer: AnalyzerEngine, anonymizer: AnonymizerEngine) -> Self {
        Self {
            analyzer,
            anonymizer,
        }
    }

    /// Analyzes JSON data for PII at specified paths.
    ///
    /// # Arguments
    ///
    /// * `data` - JSON data to analyze
    /// * `configs` - Path configurations specifying where to look for PII
    /// * `language` - Language of the text content
    /// * `score_threshold` - Minimum confidence score
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use presidio_structured::{StructuredEngine, PathConfig};
    /// use presidio_common::Language;
    /// use serde_json::json;
    ///
    /// let engine = StructuredEngine::new();
    /// let data = json!({
    ///     "user": {
    ///         "email": "john@example.com",
    ///         "phone": "555-1234"
    ///     }
    /// });
    ///
    /// let configs = vec![
    ///     PathConfig {
    ///         path: "user.email".to_string(),
    ///         entity_types: None,
    ///         operator: None,
    ///     },
    /// ];
    ///
    /// let results = engine.analyze_json(&data, &configs, Language::En, 0.5).unwrap();
    /// ```
    pub fn analyze_json(
        &self,
        data: &Value,
        configs: &[PathConfig],
        language: Language,
        score_threshold: f32,
    ) -> anyhow::Result<Vec<StructuredAnalysisResult>> {
        let mut results = Vec::new();

        for config in configs {
            let values = self.extract_values(data, &config.path);

            for (path, value) in values {
                if let Value::String(text) = value {
                    let analysis_results = self.analyzer.analyze(
                        text,
                        language,
                        config.entity_types.as_deref(),
                        None,
                        score_threshold,
                        false,
                    )?;

                    if !analysis_results.is_empty() {
                        results.push(StructuredAnalysisResult {
                            path,
                            value: text.clone(),
                            results: analysis_results,
                        });
                    }
                }
            }
        }

        Ok(results)
    }

    /// Anonymizes JSON data based on analysis results and configurations.
    ///
    /// # Arguments
    ///
    /// * `data` - JSON data to anonymize
    /// * `configs` - Path configurations with operators
    /// * `language` - Language of the text content
    /// * `score_threshold` - Minimum confidence score
    /// * `conflict_resolution` - Strategy for handling overlapping entities
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use presidio_structured::{StructuredEngine, PathConfig};
    /// use presidio_common::{Language, ConflictResolutionStrategy};
    /// use serde_json::json;
    ///
    /// let engine = StructuredEngine::new();
    /// let data = json!({
    ///     "user": {
    ///         "email": "john@example.com"
    ///     }
    /// });
    ///
    /// let configs = vec![
    ///     PathConfig {
    ///         path: "user.email".to_string(),
    ///         entity_types: None,
    ///         operator: Some(("replace".to_string(), json!({"new_value": "<EMAIL>"}))),
    ///     },
    /// ];
    ///
    /// let result = engine.anonymize_json(
    ///     &data,
    ///     &configs,
    ///     Language::En,
    ///     0.5,
    ///     ConflictResolutionStrategy::HighestScore
    /// ).unwrap();
    /// ```
    pub fn anonymize_json(
        &self,
        data: &Value,
        configs: &[PathConfig],
        language: Language,
        score_threshold: f32,
        conflict_resolution: ConflictResolutionStrategy,
    ) -> anyhow::Result<StructuredAnonymizationResult> {
        let mut anonymized_data = data.clone();
        let mut operations = Vec::new();

        for config in configs {
            // Collect path-value pairs first to avoid borrow checker issues
            let values: Vec<(String, String)> = self.extract_values(&anonymized_data, &config.path)
                .into_iter()
                .filter_map(|(path, value)| {
                    if let Value::String(text) = value {
                        Some((path, text.clone()))
                    } else {
                        None
                    }
                })
                .collect();

            for (path, text) in values {
                // Analyze for PII
                let analysis_results = self.analyzer.analyze(
                    &text,
                    language,
                    config.entity_types.as_deref(),
                    None,
                    score_threshold,
                    false,
                )?;

                if !analysis_results.is_empty() && config.operator.is_some() {
                    // Prepare operators map
                    let mut operators = HashMap::new();
                    let (operator_name, params) = config.operator.as_ref().unwrap();

                    for result in &analysis_results {
                        operators.insert(
                            result.entity_type.clone(),
                            (operator_name.clone(), params.clone()),
                        );
                    }

                    // Anonymize
                    let anon_result = self.anonymizer.anonymize(
                        &text,
                        &analysis_results,
                        &operators,
                        conflict_resolution,
                    )?;

                    // Update the JSON
                    self.set_value(&mut anonymized_data, &path, Value::String(anon_result.text.clone()));

                    operations.push(AnonymizationOperation {
                        path,
                        original_value: text,
                        anonymized_value: anon_result.text,
                        operator: operator_name.clone(),
                    });
                }
            }
        }

        Ok(StructuredAnonymizationResult {
            data: anonymized_data,
            operations,
        })
    }

    /// Extracts values at a given JSON path.
    fn extract_values<'a>(&self, data: &'a Value, path: &str) -> Vec<(String, &'a Value)> {
        let mut results = Vec::new();
        let parts: Vec<&str> = path.split('.').collect();
        self.extract_values_recursive(data, &parts, 0, String::new(), &mut results);
        results
    }

    #[allow(clippy::only_used_in_recursion)]
    fn extract_values_recursive<'a>(
        &self,
        current: &'a Value,
        parts: &[&str],
        index: usize,
        current_path: String,
        results: &mut Vec<(String, &'a Value)>,
    ) {
        if index >= parts.len() {
            results.push((current_path, current));
            return;
        }

        let part = parts[index];
        let new_path = if current_path.is_empty() {
            part.to_string()
        } else {
            format!("{}.{}", current_path, part)
        };

        match current {
            Value::Object(map) => {
                if let Some(value) = map.get(part) {
                    self.extract_values_recursive(value, parts, index + 1, new_path, results);
                }
            }
            Value::Array(arr) => {
                if part == "*" {
                    for (i, item) in arr.iter().enumerate() {
                        let array_path = format!("{}[{}]", current_path, i);
                        self.extract_values_recursive(item, parts, index + 1, array_path, results);
                    }
                }
            }
            _ => {}
        }
    }

    /// Sets a value at a given JSON path.
    fn set_value(&self, data: &mut Value, path: &str, new_value: Value) {
        let parts: Vec<&str> = path.split('.').collect();
        self.set_value_recursive(data, &parts, 0, new_value);
    }

    #[allow(clippy::only_used_in_recursion)]
    fn set_value_recursive(&self, current: &mut Value, parts: &[&str], index: usize, new_value: Value) {
        if index >= parts.len() - 1 {
            if let Value::Object(map) = current {
                map.insert(parts[index].to_string(), new_value);
            }
            return;
        }

        let part = parts[index];

        if let Value::Object(map) = current {
            if let Some(next) = map.get_mut(part) {
                self.set_value_recursive(next, parts, index + 1, new_value);
            }
        }
    }
}

impl Default for StructuredEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_extract_simple_path() {
        let engine = StructuredEngine::new();
        let data = json!({
            "user": {
                "email": "test@example.com"
            }
        });

        let values = engine.extract_values(&data, "user.email");
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].0, "user.email");
        assert_eq!(values[0].1, &Value::String("test@example.com".to_string()));
    }

    #[test]
    fn test_analyze_json() {
        let engine = StructuredEngine::new();
        let data = json!({
            "user": {
                "email": "john@example.com",
                "name": "John Doe"
            }
        });

        let configs = vec![PathConfig {
            path: "user.email".to_string(),
            entity_types: None,
            operator: None,
        }];

        let results = engine
            .analyze_json(&data, &configs, Language::En, 0.0)
            .unwrap();

        assert!(!results.is_empty());
        assert_eq!(results[0].path, "user.email");
    }

    #[test]
    fn test_anonymize_json() {
        let engine = StructuredEngine::new();
        let data = json!({
            "user": {
                "email": "john@example.com"
            }
        });

        let configs = vec![PathConfig {
            path: "user.email".to_string(),
            entity_types: None,
            operator: Some(("replace".to_string(), json!({"new_value": "<EMAIL>"}))),
        }];

        let result = engine
            .anonymize_json(
                &data,
                &configs,
                Language::En,
                0.0,
                ConflictResolutionStrategy::HighestScore,
            )
            .unwrap();

        assert!(!result.operations.is_empty());
        assert_eq!(
            result.data["user"]["email"],
            Value::String("<EMAIL>".to_string())
        );
    }

    #[test]
    fn test_nested_path() {
        let engine = StructuredEngine::new();
        let data = json!({
            "company": {
                "employees": {
                    "manager": {
                        "email": "manager@example.com"
                    }
                }
            }
        });

        let values = engine.extract_values(&data, "company.employees.manager.email");
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].0, "company.employees.manager.email");
    }
}
