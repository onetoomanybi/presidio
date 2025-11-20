//! Complete analysis and anonymization pipeline
//!
//! This example demonstrates the full workflow: analyze -> anonymize
//!
//! Run with: cargo run --example complete_pipeline

use presidio_analyzer::AnalyzerEngine;
use presidio_anonymizer::AnonymizerEngine;
use presidio_common::{ConflictResolutionStrategy, EntityType, Language};
use serde_json::json;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create engines
    let analyzer = AnalyzerEngine::with_defaults();
    let anonymizer = AnonymizerEngine::with_defaults();

    // Original text with PII
    let text = "Hello! My name is John Doe. \
                You can reach me at john.doe@example.com or call 555-123-4567. \
                My credit card number is 4532-0151-1283-0366.";

    println!("=== ORIGINAL TEXT ===");
    println!("{}\n", text);

    // Step 1: Analyze for PII
    println!("=== STEP 1: ANALYSIS ===");
    let results = analyzer.analyze(text, Language::En, None, None, 0.5, false)?;

    println!("Found {} PII entities:", results.len());
    for result in &results {
        println!(
            "  - {} at {}..{} (score: {:.2}): {}",
            result.entity_type,
            result.start,
            result.end,
            result.score,
            &text[result.start..result.end]
        );
    }
    println!();

    // Step 2: Configure anonymization operators
    println!("=== STEP 2: CONFIGURE OPERATORS ===");
    let mut operators = HashMap::new();

    // Different strategies for different entity types
    operators.insert(
        EntityType::Email,
        ("replace".to_string(), json!({"new_value": "<EMAIL_ADDRESS>"})),
    );

    operators.insert(
        EntityType::PhoneNumber,
        (
            "mask".to_string(),
            json!({
                "masking_char": "X",
                "chars_to_mask": 8,
                "from_end": true
            }),
        ),
    );

    operators.insert(
        EntityType::CreditCard,
        (
            "mask".to_string(),
            json!({
                "masking_char": "*",
                "chars_to_mask": 12,
                "from_end": false
            }),
        ),
    );

    println!("Configured operators:");
    for (entity, (op, _)) in &operators {
        println!("  - {}: {}", entity, op);
    }
    println!();

    // Step 3: Anonymize
    println!("=== STEP 3: ANONYMIZATION ===");
    let anonymized = anonymizer.anonymize(
        text,
        &results,
        &operators,
        ConflictResolutionStrategy::MergeFirstBeforeSecond,
    )?;

    println!("Anonymized text:");
    println!("{}\n", anonymized.text);

    println!("Operations performed ({} total):", anonymized.items.len());
    for item in anonymized.items {
        println!(
            "  - {} '{}' -> '{}' using {}",
            item.entity_type, item.text, item.result, item.operator
        );
    }

    Ok(())
}
