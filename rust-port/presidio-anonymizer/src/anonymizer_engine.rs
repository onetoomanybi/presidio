//! Main anonymizer engine for PII transformation.

use crate::operators::Operator;
use presidio_common::{
    merge_overlapping, ConflictResolutionStrategy, EntityType, PresidioError, RecognizerResult,
    Result,
};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Result of an anonymization operation.
#[derive(Debug, Clone)]
pub struct EngineResult {
    /// The anonymized text
    pub text: String,
    /// Details of each transformation
    pub items: Vec<OperatorResult>,
}

/// Result of applying an operator to a single entity.
#[derive(Debug, Clone)]
pub struct OperatorResult {
    /// Start position in original text
    pub start: usize,
    /// End position in original text
    pub end: usize,
    /// Entity type that was anonymized
    pub entity_type: EntityType,
    /// Original text
    pub text: String,
    /// Operator used
    pub operator: String,
}

/// The main anonymizer engine for transforming PII.
pub struct AnonymizerEngine {
    operators: HashMap<String, Arc<dyn Operator>>,
}

impl AnonymizerEngine {
    /// Creates a new anonymizer engine.
    pub fn new() -> Self {
        Self {
            operators: HashMap::new(),
        }
    }

    /// Creates an anonymizer engine with default operators.
    pub fn with_defaults() -> Self {
        let mut engine = Self::new();

        engine.add_operator(Arc::new(crate::operators::ReplaceOperator));
        engine.add_operator(Arc::new(crate::operators::RedactOperator));
        engine.add_operator(Arc::new(crate::operators::MaskOperator));
        engine.add_operator(Arc::new(crate::operators::HashOperator));
        engine.add_operator(Arc::new(crate::operators::EncryptOperator));
        engine.add_operator(Arc::new(crate::operators::KeepOperator));

        engine
    }

    /// Adds an operator to the engine.
    pub fn add_operator(&mut self, operator: Arc<dyn Operator>) {
        self.operators.insert(operator.name().to_string(), operator);
    }

    /// Removes an operator from the engine.
    pub fn remove_operator(&mut self, name: &str) -> Result<()> {
        self.operators
            .remove(name)
            .ok_or_else(|| PresidioError::OperatorNotFound(name.to_string()))?;
        Ok(())
    }

    /// Anonymizes text based on analyzer results.
    ///
    /// # Arguments
    ///
    /// * `text` - The original text
    /// * `analyzer_results` - PII entities detected by the analyzer
    /// * `operators` - Map of entity types to (operator_name, params)
    /// * `conflict_resolution` - Strategy for handling overlapping entities
    ///
    /// # Returns
    ///
    /// The anonymized result with transformed text and operation details.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use presidio_anonymizer::{AnonymizerEngine};
    /// use presidio_common::{RecognizerResult, EntityType, ConflictResolutionStrategy};
    /// use std::collections::HashMap;
    /// use serde_json::json;
    ///
    /// let engine = AnonymizerEngine::with_defaults();
    ///
    /// let text = "My email is john@example.com";
    /// let results = vec![
    ///     RecognizerResult::new(EntityType::Email, 12, 29, 0.9),
    /// ];
    ///
    /// let mut operators = HashMap::new();
    /// operators.insert(
    ///     EntityType::Email,
    ///     ("replace".to_string(), json!({"new_value": "<EMAIL>"})),
    /// );
    ///
    /// let anonymized = engine.anonymize(
    ///     text,
    ///     &results,
    ///     &operators,
    ///     ConflictResolutionStrategy::HighestScore,
    /// ).unwrap();
    ///
    /// assert_eq!(anonymized.text, "My email is <EMAIL>");
    /// ```
    pub fn anonymize(
        &self,
        text: &str,
        analyzer_results: &[RecognizerResult],
        operators: &HashMap<EntityType, (String, Value)>,
        conflict_resolution: ConflictResolutionStrategy,
    ) -> Result<EngineResult> {
        if analyzer_results.is_empty() {
            return Ok(EngineResult {
                text: text.to_string(),
                items: vec![],
            });
        }

        // Resolve conflicts
        let mut results = merge_overlapping(analyzer_results.to_vec(), conflict_resolution);

        // Sort by position (descending) to process from end to start
        results.sort_by(|a, b| b.start.cmp(&a.start));

        let mut anonymized_text = text.to_string();
        let mut items = Vec::new();

        for result in results {
            // Get operator for this entity type
            let (operator_name, params) = operators
                .get(&result.entity_type)
                .cloned()
                .unwrap_or_else(|| {
                    // Default: replace with entity type
                    (
                        "replace".to_string(),
                        serde_json::json!({"new_value": format!("<{}>", result.entity_type.as_str())}),
                    )
                });

            // Get the operator
            let operator = self.operators.get(&operator_name).ok_or_else(|| {
                PresidioError::OperatorNotFound(format!(
                    "Operator '{}' not found",
                    operator_name
                ))
            })?;

            // Validate parameters
            operator.validate(&params)?;

            // Get the original text segment (from the original text, not anonymized)
            let original_segment = &text[result.start..result.end];

            // Apply the operator
            let transformed = operator.operate(original_segment, &params)?;

            // Replace in the anonymized text (from end to start to preserve positions)
            anonymized_text.replace_range(result.start..result.end, &transformed);

            // Record the operation
            items.push(OperatorResult {
                start: result.start,
                end: result.end,
                entity_type: result.entity_type.clone(),
                text: original_segment.to_string(),
                operator: operator_name,
            });
        }

        // Reverse items to maintain original order
        items.reverse();

        Ok(EngineResult {
            text: anonymized_text,
            items,
        })
    }

    /// Returns the list of available operators.
    pub fn get_operators(&self) -> Vec<String> {
        self.operators.keys().cloned().collect()
    }
}

impl Default for AnonymizerEngine {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use presidio_common::EntityType;
    use serde_json::json;

    #[test]
    fn test_anonymizer_basic() {
        let engine = AnonymizerEngine::with_defaults();

        let text = "My email is john@example.com";
        let results = vec![RecognizerResult::new(EntityType::Email, 12, 28, 0.9)];

        let mut operators = HashMap::new();
        operators.insert(
            EntityType::Email,
            ("replace".to_string(), json!({"new_value": "<EMAIL>"})),
        );

        let anonymized = engine
            .anonymize(text, &results, &operators, ConflictResolutionStrategy::HighestScore)
            .unwrap();

        assert_eq!(anonymized.text, "My email is <EMAIL>");
        assert_eq!(anonymized.items.len(), 1);
    }

    #[test]
    fn test_anonymizer_redact() {
        let engine = AnonymizerEngine::with_defaults();

        let text = "My email is john@example.com";
        let results = vec![RecognizerResult::new(EntityType::Email, 12, 28, 0.9)];

        let mut operators = HashMap::new();
        operators.insert(EntityType::Email, ("redact".to_string(), json!({})));

        let anonymized = engine
            .anonymize(text, &results, &operators, ConflictResolutionStrategy::HighestScore)
            .unwrap();

        assert_eq!(anonymized.text, "My email is ");
    }

    #[test]
    fn test_anonymizer_mask() {
        let engine = AnonymizerEngine::with_defaults();

        let text = "Card: 1234567890123456";
        let results = vec![RecognizerResult::new(EntityType::CreditCard, 6, 22, 0.9)];

        let mut operators = HashMap::new();
        operators.insert(
            EntityType::CreditCard,
            ("mask".to_string(), json!({"chars_to_mask": 12})),
        );

        let anonymized = engine
            .anonymize(text, &results, &operators, ConflictResolutionStrategy::HighestScore)
            .unwrap();

        assert_eq!(anonymized.text, "Card: ************3456");
    }

    #[test]
    fn test_anonymizer_hash() {
        let engine = AnonymizerEngine::with_defaults();

        let text = "My email is john@example.com";
        let results = vec![RecognizerResult::new(EntityType::Email, 12, 28, 0.9)];

        let mut operators = HashMap::new();
        operators.insert(
            EntityType::Email,
            ("hash".to_string(), json!({"hash_type": "sha256"})),
        );

        let anonymized = engine
            .anonymize(text, &results, &operators, ConflictResolutionStrategy::HighestScore)
            .unwrap();

        assert!(anonymized.text.contains("My email is "));
        assert!(anonymized.text.len() > 50); // Hash is much longer
    }

    #[test]
    fn test_anonymizer_multiple_entities() {
        let engine = AnonymizerEngine::with_defaults();

        let text = "Email: john@example.com, Phone: 555-1234";
        let results = vec![
            RecognizerResult::new(EntityType::Email, 7, 23, 0.9),
            RecognizerResult::new(EntityType::Phone, 32, 40, 0.8),
        ];

        let mut operators = HashMap::new();
        operators.insert(
            EntityType::Email,
            ("replace".to_string(), json!({"new_value": "<EMAIL>"})),
        );
        operators.insert(
            EntityType::Phone,
            ("replace".to_string(), json!({"new_value": "<PHONE>"})),
        );

        let anonymized = engine
            .anonymize(text, &results, &operators, ConflictResolutionStrategy::HighestScore)
            .unwrap();

        assert_eq!(anonymized.text, "Email: <EMAIL>, Phone: <PHONE>");
        assert_eq!(anonymized.items.len(), 2);
    }

    #[test]
    fn test_anonymizer_default_operator() {
        let engine = AnonymizerEngine::with_defaults();

        let text = "My email is john@example.com";
        let results = vec![RecognizerResult::new(EntityType::Email, 12, 28, 0.9)];

        // No operators specified - should use default
        let operators = HashMap::new();

        let anonymized = engine
            .anonymize(text, &results, &operators, ConflictResolutionStrategy::HighestScore)
            .unwrap();

        assert_eq!(anonymized.text, "My email is <EMAIL>");
    }

    #[test]
    fn test_anonymizer_empty_results() {
        let engine = AnonymizerEngine::with_defaults();

        let text = "My email is john@example.com";
        let results = vec![];
        let operators = HashMap::new();

        let anonymized = engine
            .anonymize(text, &results, &operators, ConflictResolutionStrategy::HighestScore)
            .unwrap();

        assert_eq!(anonymized.text, text);
        assert_eq!(anonymized.items.len(), 0);
    }
}
