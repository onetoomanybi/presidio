//! Core data types for Presidio.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a detected PII entity in text.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecognizerResult {
    /// The type of PII entity detected (e.g., EMAIL, CREDIT_CARD, SSN)
    pub entity_type: EntityType,

    /// Start position in the text (inclusive)
    pub start: usize,

    /// End position in the text (exclusive)
    pub end: usize,

    /// Confidence score (0.0 to 1.0)
    pub score: f32,

    /// Optional explanation of how this entity was detected
    pub analysis_explanation: Option<AnalysisExplanation>,

    /// Additional metadata about the recognition
    #[serde(default)]
    pub recognition_metadata: HashMap<String, serde_json::Value>,
}

impl RecognizerResult {
    /// Creates a new recognizer result.
    pub fn new(entity_type: EntityType, start: usize, end: usize, score: f32) -> Self {
        Self {
            entity_type,
            start,
            end,
            score,
            analysis_explanation: None,
            recognition_metadata: HashMap::new(),
        }
    }

    /// Creates a new recognizer result with explanation.
    pub fn with_explanation(
        entity_type: EntityType,
        start: usize,
        end: usize,
        score: f32,
        explanation: AnalysisExplanation,
    ) -> Self {
        Self {
            entity_type,
            start,
            end,
            score,
            analysis_explanation: Some(explanation),
            recognition_metadata: HashMap::new(),
        }
    }

    /// Checks if this result overlaps with another.
    pub fn overlaps_with(&self, other: &RecognizerResult) -> bool {
        self.start < other.end && other.start < self.end
    }

    /// Checks if this result contains another.
    pub fn contains(&self, other: &RecognizerResult) -> bool {
        self.start <= other.start && self.end >= other.end
    }

    /// Returns the length of the detected entity.
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    /// Checks if the entity is empty.
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

/// Explanation of how a PII entity was detected.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalysisExplanation {
    /// Name of the recognizer that detected this entity
    pub recognizer: String,

    /// Name of the pattern that matched (if pattern-based)
    pub pattern_name: Option<String>,

    /// The actual pattern/regex used (if applicable)
    pub pattern: Option<String>,

    /// Original score before context enhancement
    pub original_score: f32,

    /// Final score after context enhancement
    pub score: f32,

    /// Textual explanation of the detection
    pub textual_explanation: Option<String>,

    /// Score improvement from context
    pub score_context_improvement: f32,

    /// Context word that supported the detection
    pub supportive_context_word: Option<String>,

    /// Result of validation (if validation was performed)
    pub validation_result: Option<ValidationResult>,
}

/// Result of a validation check (e.g., Luhn checksum, IBAN validation).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether validation passed
    pub is_valid: bool,

    /// Validation method used
    pub validation_method: String,

    /// Additional validation details
    pub details: Option<String>,
}

/// A regex pattern used for PII detection.
#[derive(Debug, Clone)]
pub struct Pattern {
    /// Name of the pattern
    pub name: String,

    /// Compiled regex pattern
    #[allow(dead_code)]
    pub regex: Regex,

    /// Base score for matches of this pattern
    pub score: f32,

    /// Optional context words that boost confidence
    pub context: Option<Vec<String>>,

    /// Optional deny list for false positives
    pub deny_list: Option<Vec<String>>,
}

impl Pattern {
    /// Creates a new pattern.
    pub fn new(name: impl Into<String>, regex: Regex, score: f32) -> Self {
        Self {
            name: name.into(),
            regex,
            score,
            context: None,
            deny_list: None,
        }
    }

    /// Sets context words for this pattern.
    pub fn with_context(mut self, context: Vec<String>) -> Self {
        self.context = Some(context);
        self
    }

    /// Sets deny list for this pattern.
    pub fn with_deny_list(mut self, deny_list: Vec<String>) -> Self {
        self.deny_list = Some(deny_list);
        self
    }
}

/// Supported languages for PII detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    /// English
    #[serde(rename = "en")]
    En,
    /// Spanish
    #[serde(rename = "es")]
    Es,
    /// French
    #[serde(rename = "fr")]
    Fr,
    /// German
    #[serde(rename = "de")]
    De,
    /// Italian
    #[serde(rename = "it")]
    It,
    /// Portuguese
    #[serde(rename = "pt")]
    Pt,
    /// Dutch
    #[serde(rename = "nl")]
    Nl,
    /// Polish
    #[serde(rename = "pl")]
    Pl,
    /// Finnish
    #[serde(rename = "fi")]
    Fi,
    /// Korean
    #[serde(rename = "ko")]
    Ko,
    /// Thai
    #[serde(rename = "th")]
    Th,
}

impl Language {
    /// Returns the language code as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Language::En => "en",
            Language::Es => "es",
            Language::Fr => "fr",
            Language::De => "de",
            Language::It => "it",
            Language::Pt => "pt",
            Language::Nl => "nl",
            Language::Pl => "pl",
            Language::Fi => "fi",
            Language::Ko => "ko",
            Language::Th => "th",
        }
    }

    /// Parses a language from a string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "en" | "english" => Some(Language::En),
            "es" | "spanish" => Some(Language::Es),
            "fr" | "french" => Some(Language::Fr),
            "de" | "german" => Some(Language::De),
            "it" | "italian" => Some(Language::It),
            "pt" | "portuguese" => Some(Language::Pt),
            "nl" | "dutch" => Some(Language::Nl),
            "pl" | "polish" => Some(Language::Pl),
            "fi" | "finnish" => Some(Language::Fi),
            "ko" | "korean" => Some(Language::Ko),
            "th" | "thai" => Some(Language::Th),
            _ => None,
        }
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Types of PII entities that can be detected.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EntityType {
    // Generic entities
    Email,
    Phone,
    Url,
    IpAddress,
    CreditCard,
    Iban,
    Crypto,
    Date,
    Time,
    Person,
    Location,
    Organization,

    // US-specific
    UsSsn,
    UsDriverLicense,
    UsPassport,
    UsItin,
    UsBankAccount,
    UsMedicalLicense,

    // UK-specific
    UkNhs,
    UkNino,

    // Australia-specific
    AuAbn,
    AuAcn,
    AuTfn,
    AuMedicare,

    // India-specific
    InAadhaar,
    InPan,
    InPassport,
    InVehicleRegistration,
    InVoterId,
    InGstin,

    // Italy-specific
    ItFiscalCode,
    ItIdentityCard,
    ItDriverLicense,
    ItPassport,
    ItVat,

    // Spain-specific
    EsNif,
    EsNie,

    // Singapore-specific
    SgFin,
    SgUen,

    // Poland-specific
    PlPesel,

    // Finland-specific
    FiPersonalIdentityCode,

    // Korea-specific
    KrRrn,

    // Thailand-specific
    ThTnin,

    // Custom entity type
    Custom(String),
}

impl EntityType {
    /// Returns the entity type as a string.
    pub fn as_str(&self) -> &str {
        match self {
            EntityType::Email => "EMAIL",
            EntityType::Phone => "PHONE",
            EntityType::Url => "URL",
            EntityType::IpAddress => "IP_ADDRESS",
            EntityType::CreditCard => "CREDIT_CARD",
            EntityType::Iban => "IBAN",
            EntityType::Crypto => "CRYPTO",
            EntityType::Date => "DATE",
            EntityType::Time => "TIME",
            EntityType::Person => "PERSON",
            EntityType::Location => "LOCATION",
            EntityType::Organization => "ORGANIZATION",
            EntityType::UsSsn => "US_SSN",
            EntityType::UsDriverLicense => "US_DRIVER_LICENSE",
            EntityType::UsPassport => "US_PASSPORT",
            EntityType::UsItin => "US_ITIN",
            EntityType::UsBankAccount => "US_BANK_ACCOUNT",
            EntityType::UsMedicalLicense => "US_MEDICAL_LICENSE",
            EntityType::UkNhs => "UK_NHS",
            EntityType::UkNino => "UK_NINO",
            EntityType::AuAbn => "AU_ABN",
            EntityType::AuAcn => "AU_ACN",
            EntityType::AuTfn => "AU_TFN",
            EntityType::AuMedicare => "AU_MEDICARE",
            EntityType::InAadhaar => "IN_AADHAAR",
            EntityType::InPan => "IN_PAN",
            EntityType::InPassport => "IN_PASSPORT",
            EntityType::InVehicleRegistration => "IN_VEHICLE_REGISTRATION",
            EntityType::InVoterId => "IN_VOTER_ID",
            EntityType::InGstin => "IN_GSTIN",
            EntityType::ItFiscalCode => "IT_FISCAL_CODE",
            EntityType::ItIdentityCard => "IT_IDENTITY_CARD",
            EntityType::ItDriverLicense => "IT_DRIVER_LICENSE",
            EntityType::ItPassport => "IT_PASSPORT",
            EntityType::ItVat => "IT_VAT",
            EntityType::EsNif => "ES_NIF",
            EntityType::EsNie => "ES_NIE",
            EntityType::SgFin => "SG_FIN",
            EntityType::SgUen => "SG_UEN",
            EntityType::PlPesel => "PL_PESEL",
            EntityType::FiPersonalIdentityCode => "FI_PERSONAL_IDENTITY_CODE",
            EntityType::KrRrn => "KR_RRN",
            EntityType::ThTnin => "TH_TNIN",
            EntityType::Custom(s) => s,
        }
    }

    /// Parses an entity type from a string.
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "EMAIL" => EntityType::Email,
            "PHONE" => EntityType::Phone,
            "URL" => EntityType::Url,
            "IP_ADDRESS" => EntityType::IpAddress,
            "CREDIT_CARD" => EntityType::CreditCard,
            "IBAN" => EntityType::Iban,
            "CRYPTO" => EntityType::Crypto,
            "DATE" => EntityType::Date,
            "TIME" => EntityType::Time,
            "PERSON" => EntityType::Person,
            "LOCATION" => EntityType::Location,
            "ORGANIZATION" => EntityType::Organization,
            "US_SSN" => EntityType::UsSsn,
            "US_DRIVER_LICENSE" => EntityType::UsDriverLicense,
            "US_PASSPORT" => EntityType::UsPassport,
            "US_ITIN" => EntityType::UsItin,
            "US_BANK_ACCOUNT" => EntityType::UsBankAccount,
            "US_MEDICAL_LICENSE" => EntityType::UsMedicalLicense,
            "UK_NHS" => EntityType::UkNhs,
            "UK_NINO" => EntityType::UkNino,
            "AU_ABN" => EntityType::AuAbn,
            "AU_ACN" => EntityType::AuAcn,
            "AU_TFN" => EntityType::AuTfn,
            "AU_MEDICARE" => EntityType::AuMedicare,
            "IN_AADHAAR" => EntityType::InAadhaar,
            "IN_PAN" => EntityType::InPan,
            "IN_PASSPORT" => EntityType::InPassport,
            "IN_VEHICLE_REGISTRATION" => EntityType::InVehicleRegistration,
            "IN_VOTER_ID" => EntityType::InVoterId,
            "IN_GSTIN" => EntityType::InGstin,
            "IT_FISCAL_CODE" => EntityType::ItFiscalCode,
            "IT_IDENTITY_CARD" => EntityType::ItIdentityCard,
            "IT_DRIVER_LICENSE" => EntityType::ItDriverLicense,
            "IT_PASSPORT" => EntityType::ItPassport,
            "IT_VAT" => EntityType::ItVat,
            "ES_NIF" => EntityType::EsNif,
            "ES_NIE" => EntityType::EsNie,
            "SG_FIN" => EntityType::SgFin,
            "SG_UEN" => EntityType::SgUen,
            "PL_PESEL" => EntityType::PlPesel,
            "FI_PERSONAL_IDENTITY_CODE" => EntityType::FiPersonalIdentityCode,
            "KR_RRN" => EntityType::KrRrn,
            "TH_TNIN" => EntityType::ThTnin,
            _ => EntityType::Custom(s.to_string()),
        }
    }
}

impl std::fmt::Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// NLP artifacts generated by processing text.
#[derive(Debug, Clone)]
pub struct NlpArtifacts {
    /// Tokens extracted from text
    pub tokens: Vec<String>,

    /// Lemmatized forms of tokens
    pub lemmas: Vec<String>,

    /// Part-of-speech tags
    pub pos_tags: Vec<String>,

    /// Named entities detected by NLP
    pub entities: Vec<NlpEntity>,

    /// Language of the text
    pub language: Language,
}

/// A named entity detected by an NLP engine.
#[derive(Debug, Clone)]
pub struct NlpEntity {
    /// Entity type (e.g., PERSON, LOCATION, ORG)
    pub entity_type: String,

    /// Start position in text
    pub start: usize,

    /// End position in text
    pub end: usize,

    /// Confidence score
    pub score: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recognizer_result_overlap() {
        let r1 = RecognizerResult::new(EntityType::Email, 0, 10, 0.9);
        let r2 = RecognizerResult::new(EntityType::Email, 5, 15, 0.9);
        let r3 = RecognizerResult::new(EntityType::Email, 20, 30, 0.9);

        assert!(r1.overlaps_with(&r2));
        assert!(!r1.overlaps_with(&r3));
    }

    #[test]
    fn test_recognizer_result_contains() {
        let r1 = RecognizerResult::new(EntityType::Email, 0, 20, 0.9);
        let r2 = RecognizerResult::new(EntityType::Email, 5, 15, 0.9);

        assert!(r1.contains(&r2));
        assert!(!r2.contains(&r1));
    }

    #[test]
    fn test_language_from_str() {
        assert_eq!(Language::from_str("en"), Some(Language::En));
        assert_eq!(Language::from_str("English"), Some(Language::En));
        assert_eq!(Language::from_str("es"), Some(Language::Es));
        assert_eq!(Language::from_str("invalid"), None);
    }

    #[test]
    fn test_entity_type_from_str() {
        assert_eq!(EntityType::from_str("EMAIL"), EntityType::Email);
        assert_eq!(EntityType::from_str("US_SSN"), EntityType::UsSsn);
        assert_eq!(
            EntityType::from_str("CUSTOM_TYPE"),
            EntityType::Custom("CUSTOM_TYPE".to_string())
        );
    }
}
