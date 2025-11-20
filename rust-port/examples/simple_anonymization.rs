//! Simple text anonymization example
//!
//! This example demonstrates basic PII anonymization.
//!
//! Run with: cargo run --example simple_anonymization

use presidio_anonymizer::AnonymizerEngine;
use presidio_common::{ConflictResolutionStrategy, EntityType, RecognizerResult};
use serde_json::json;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create anonymizer with default operators
    let anonymizer = AnonymizerEngine::with_defaults();

    // Original text
    let text = "My email is john.doe@example.com and my phone is 555-123-4567";

    println!("Original text:\n{}\n", text);

    // Simulated analyzer results (normally from AnalyzerEngine)
    let analyzer_results = vec![
        RecognizerResult::new(EntityType::Email, 12, 33, 0.9),
        RecognizerResult::new(EntityType::PhoneNumber, 50, 62, 0.8),
    ];

    // Configure operators for each entity type
    let mut operators = HashMap::new();

    // Replace email with placeholder
    operators.insert(
        EntityType::Email,
        ("replace".to_string(), json!({"new_value": "<EMAIL>"})),
    );

    // Mask phone number
    operators.insert(
        EntityType::PhoneNumber,
        (
            "mask".to_string(),
            json!({
                "masking_char": "*",
                "chars_to_mask": 7,
                "from_end": true
            }),
        ),
    );

    // Anonymize
    let result = anonymizer.anonymize(
        text,
        &analyzer_results,
        &operators,
        ConflictResolutionStrategy::MergeFirstBeforeSecond,
    )?;

    // Print results
    println!("Anonymized text:\n{}\n", result.text);

    println!("Operations performed:");
    for item in result.items {
        println!("  {} at {}..{}", item.operator, item.start, item.end);
        println!("    Original: {}", item.text);
        println!("    Result: {}", item.result);
    }

    Ok(())
}
