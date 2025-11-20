//! Simple text analysis example
//!
//! This example demonstrates basic PII detection in text.
//!
//! Run with: cargo run --example simple_analysis

use presidio_analyzer::AnalyzerEngine;
use presidio_common::Language;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create analyzer with default recognizers
    let analyzer = AnalyzerEngine::with_defaults();

    // Text to analyze
    let text = "Hello, my name is John Doe. My email is john.doe@example.com and my phone is 555-123-4567. \
                My credit card is 4532-0151-1283-0366.";

    println!("Analyzing text:\n{}\n", text);

    // Analyze for all PII entities
    let results = analyzer.analyze(
        text,
        Language::En,
        None,  // Detect all entity types
        None,  // No correlation ID
        0.5,   // Minimum score threshold
        false, // Don't return decision process
    )?;

    // Print results
    println!("Found {} PII entities:\n", results.len());
    for result in results {
        println!("Entity: {}", result.entity_type);
        println!("  Location: {}..{}", result.start, result.end);
        println!("  Text: {}", &text[result.start..result.end]);
        println!("  Score: {:.2}", result.score);
        println!();
    }

    Ok(())
}
