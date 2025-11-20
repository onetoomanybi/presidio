//! Custom recognizer example
//!
//! This example demonstrates how to create a custom entity recognizer.
//!
//! Run with: cargo run --example custom_recognizer

use presidio_analyzer::{AnalyzerEngine, RecognizerRegistry};
use presidio_common::{
    AnalysisExplanation, EntityRecognizer, EntityType, Language, NlpArtifacts, RecognizerResult,
    Result,
};
use regex::Regex;
use std::sync::Arc;

/// Custom recognizer for employee IDs in format EMP-XXXXX
struct EmployeeIdRecognizer {
    pattern: Regex,
}

impl EmployeeIdRecognizer {
    fn new() -> Self {
        Self {
            pattern: Regex::new(r"\bEMP-\d{5}\b").unwrap(),
        }
    }
}

impl EntityRecognizer for EmployeeIdRecognizer {
    fn name(&self) -> &str {
        "EmployeeIdRecognizer"
    }

    fn supported_entities(&self) -> &[EntityType] {
        // Use a custom entity type
        &[EntityType::Custom("EMPLOYEE_ID".to_string())]
    }

    fn supported_languages(&self) -> &[Language] {
        // Support all languages
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
        entities: Option<&[EntityType]>,
        _nlp_artifacts: Option<&NlpArtifacts>,
        _context: Option<&[String]>,
    ) -> Result<Vec<RecognizerResult>> {
        // Filter by entity types if specified
        if let Some(entity_filter) = entities {
            let employee_id = EntityType::Custom("EMPLOYEE_ID".to_string());
            if !entity_filter.contains(&employee_id) {
                return Ok(vec![]);
            }
        }

        let mut results = Vec::new();

        for capture in self.pattern.find_iter(text) {
            results.push(RecognizerResult {
                entity_type: EntityType::Custom("EMPLOYEE_ID".to_string()),
                start: capture.start(),
                end: capture.end(),
                score: 0.9, // High confidence for exact pattern match
                analysis_explanation: Some(AnalysisExplanation {
                    recognizer: self.name().to_string(),
                    pattern_name: Some("employee_id_pattern".to_string()),
                    pattern: Some(r"\bEMP-\d{5}\b".to_string()),
                    original_score: 0.9,
                    score: 0.9,
                    textual_explanation: Some(
                        "Matched employee ID pattern EMP-XXXXX".to_string(),
                    ),
                    score_context_improvement: 0.0,
                    supportive_context_word: None,
                    validation_result: None,
                }),
                recognition_metadata: Default::default(),
            });
        }

        Ok(results)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== CUSTOM RECOGNIZER EXAMPLE ===\n");

    // Create a registry and add custom recognizer
    let mut registry = RecognizerRegistry::new();

    // Add default recognizers
    registry.add_recognizer(Arc::new(
        presidio_analyzer::recognizers::EmailRecognizer::new(),
    ));

    // Add our custom recognizer
    registry.add_recognizer(Arc::new(EmployeeIdRecognizer::new()));

    // Create analyzer with custom registry
    let analyzer = AnalyzerEngine::new(registry);

    // Test text
    let text = "Employee EMP-12345 sent an email to john@example.com. \
                Another employee EMP-67890 was CC'd.";

    println!("Analyzing text:\n{}\n", text);

    // Analyze
    let results = analyzer.analyze(text, Language::En, None, None, 0.0, true)?;

    println!("Found {} entities:\n", results.len());
    for result in results {
        println!("Entity: {}", result.entity_type);
        println!("  Text: {}", &text[result.start..result.end]);
        println!("  Score: {:.2}", result.score);
        println!("  Recognizer: {}", result.analysis_explanation.as_ref().map(|e| e.recognizer.as_str()).unwrap_or("unknown"));
        if let Some(explanation) = &result.analysis_explanation {
            if let Some(pattern) = &explanation.pattern {
                println!("  Pattern: {}", pattern);
            }
        }
        println!();
    }

    Ok(())
}
