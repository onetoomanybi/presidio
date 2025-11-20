//! Utility functions for Presidio.

use crate::RecognizerResult;

/// Removes duplicate and overlapping results, keeping the highest scored ones.
///
/// # Arguments
///
/// * `results` - Vector of recognizer results
///
/// # Returns
///
/// Deduplicated results
pub fn remove_duplicates(mut results: Vec<RecognizerResult>) -> Vec<RecognizerResult> {
    if results.is_empty() {
        return results;
    }

    // Sort by start position, then by score (descending)
    results.sort_by(|a, b| {
        a.start
            .cmp(&b.start)
            .then_with(|| b.score.partial_cmp(&a.score).unwrap())
    });

    let mut deduplicated: Vec<RecognizerResult> = Vec::new();
    let mut i = 0;

    while i < results.len() {
        let current = &results[i];
        let mut keep = true;

        // Check if this result is contained by or identical to any already kept result
        for kept in &deduplicated {
            if kept.contains(current) || (kept.start == current.start && kept.end == current.end) {
                keep = false;
                break;
            }
        }

        if keep {
            deduplicated.push(current.clone());
        }

        i += 1;
    }

    deduplicated
}

/// Merges overlapping results based on a strategy.
///
/// # Arguments
///
/// * `results` - Vector of recognizer results
/// * `strategy` - Conflict resolution strategy
///
/// # Returns
///
/// Merged results
pub fn merge_overlapping(
    results: Vec<RecognizerResult>,
    strategy: ConflictResolutionStrategy,
) -> Vec<RecognizerResult> {
    if results.is_empty() {
        return results;
    }

    let mut sorted = results;
    sorted.sort_by_key(|r| r.start);

    let mut merged = Vec::new();
    let mut current: Option<RecognizerResult> = None;

    for result in sorted {
        match current.take() {
            None => current = Some(result),
            Some(curr) if !curr.overlaps_with(&result) => {
                merged.push(curr);
                current = Some(result);
            }
            Some(curr) => {
                current = Some(resolve_conflict(&curr, &result, strategy));
            }
        }
    }

    if let Some(curr) = current {
        merged.push(curr);
    }

    merged
}

/// Strategies for resolving conflicts between overlapping entities.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictResolutionStrategy {
    /// Keep the entity with the highest score
    HighestScore,
    /// Keep the longest entity
    Longest,
    /// Keep the first entity
    First,
    /// Keep the last entity
    Last,
}

fn resolve_conflict(
    a: &RecognizerResult,
    b: &RecognizerResult,
    strategy: ConflictResolutionStrategy,
) -> RecognizerResult {
    match strategy {
        ConflictResolutionStrategy::HighestScore => {
            if a.score >= b.score {
                a.clone()
            } else {
                b.clone()
            }
        }
        ConflictResolutionStrategy::Longest => {
            if a.len() >= b.len() {
                a.clone()
            } else {
                b.clone()
            }
        }
        ConflictResolutionStrategy::First => a.clone(),
        ConflictResolutionStrategy::Last => b.clone(),
    }
}

/// Validates a checksum using the Luhn algorithm (used for credit cards, etc.).
///
/// # Arguments
///
/// * `number` - The number to validate
///
/// # Returns
///
/// `true` if the checksum is valid
pub fn luhn_checksum(number: &str) -> bool {
    let digits: Vec<u32> = number
        .chars()
        .filter(|c| c.is_ascii_digit())
        .filter_map(|c| c.to_digit(10))
        .collect();

    if digits.is_empty() {
        return false;
    }

    let mut sum = 0;
    let mut double = false;

    for &digit in digits.iter().rev() {
        let mut value = digit;
        if double {
            value *= 2;
            if value > 9 {
                value -= 9;
            }
        }
        sum += value;
        double = !double;
    }

    sum % 10 == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EntityType;

    #[test]
    fn test_remove_duplicates() {
        let results = vec![
            RecognizerResult::new(EntityType::Email, 0, 10, 0.9),
            RecognizerResult::new(EntityType::Email, 0, 10, 0.8),
            RecognizerResult::new(EntityType::Email, 20, 30, 0.9),
        ];

        let dedup = remove_duplicates(results);
        assert_eq!(dedup.len(), 2);
        assert_eq!(dedup[0].score, 0.9);
    }

    #[test]
    fn test_merge_overlapping() {
        let results = vec![
            RecognizerResult::new(EntityType::Email, 0, 10, 0.9),
            RecognizerResult::new(EntityType::Email, 5, 15, 0.8),
            RecognizerResult::new(EntityType::Email, 20, 30, 0.9),
        ];

        let merged = merge_overlapping(results, ConflictResolutionStrategy::HighestScore);
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0].score, 0.9);
    }

    #[test]
    fn test_luhn_checksum() {
        assert!(luhn_checksum("4532015112830366")); // Valid Visa
        assert!(luhn_checksum("5425233430109903")); // Valid Mastercard
        assert!(!luhn_checksum("1234567890123456")); // Invalid
        assert!(!luhn_checksum("")); // Empty
    }
}
