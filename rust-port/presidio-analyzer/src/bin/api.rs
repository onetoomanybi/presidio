use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use presidio_analyzer::AnalyzerEngine;
use presidio_common::{EntityType, Language, RecognizerResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{info, Level};

#[derive(Clone)]
struct AppState {
    analyzer: Arc<AnalyzerEngine>,
}

#[derive(Debug, Deserialize)]
struct AnalyzeRequest {
    /// Text to analyze for PII entities
    text: String,

    /// Language of the text (default: "en")
    #[serde(default = "default_language")]
    language: String,

    /// Specific entities to detect (optional, detects all by default)
    entities: Option<Vec<String>>,

    /// Minimum score threshold (default: 0.0)
    #[serde(default)]
    score_threshold: f32,

    /// Return detailed decision process
    #[serde(default)]
    return_decision_process: bool,

    /// Correlation ID for tracking
    correlation_id: Option<String>,
}

fn default_language() -> String {
    "en".to_string()
}

#[derive(Debug, Serialize)]
struct AnalyzeResponse {
    results: Vec<RecognizerResultDto>,
}

#[derive(Debug, Serialize)]
struct RecognizerResultDto {
    entity_type: String,
    start: usize,
    end: usize,
    score: f32,
    analysis_explanation: Option<AnalysisExplanationDto>,
}

#[derive(Debug, Serialize)]
struct AnalysisExplanationDto {
    recognizer: String,
    pattern_name: Option<String>,
    original_score: f32,
    score: f32,
}

#[derive(Debug, Serialize)]
struct RecognizersResponse {
    recognizers: Vec<String>,
}

#[derive(Debug, Serialize)]
struct EntitiesResponse {
    supported_entities: Vec<String>,
}

#[derive(Debug, Serialize)]
struct LanguagesResponse {
    supported_languages: Vec<String>,
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: String,
    version: String,
}

async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn analyze_handler(
    State(state): State<AppState>,
    Json(request): Json<AnalyzeRequest>,
) -> Result<Json<AnalyzeResponse>, (StatusCode, String)> {
    info!(
        "Analyzing text of length {} in language {}",
        request.text.len(),
        request.language
    );

    // Parse language
    let language = Language::from_str(&request.language)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid language: {}", e)))?;

    // Parse entity types if provided
    let entities = if let Some(entity_strs) = request.entities {
        let mut parsed_entities = Vec::new();
        for entity_str in entity_strs {
            let entity = EntityType::from_str(&entity_str)
                .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid entity: {}", e)))?;
            parsed_entities.push(entity);
        }
        Some(parsed_entities)
    } else {
        None
    };

    // Analyze text
    let results = state
        .analyzer
        .analyze(
            &request.text,
            language,
            entities.as_deref(),
            request.correlation_id.as_deref(),
            if request.score_threshold > 0.0 {
                Some(request.score_threshold)
            } else {
                None
            },
            request.return_decision_process,
        )
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Convert to DTOs
    let dto_results: Vec<RecognizerResultDto> = results
        .into_iter()
        .map(|r| RecognizerResultDto {
            entity_type: r.entity_type.to_string(),
            start: r.start,
            end: r.end,
            score: r.score,
            analysis_explanation: r.analysis_explanation.map(|e| AnalysisExplanationDto {
                recognizer: e.recognizer,
                pattern_name: e.pattern_name,
                original_score: e.original_score,
                score: e.score,
            }),
        })
        .collect();

    info!("Found {} PII entities", dto_results.len());

    Ok(Json(AnalyzeResponse {
        results: dto_results,
    }))
}

async fn recognizers_handler(State(state): State<AppState>) -> Json<RecognizersResponse> {
    let recognizers = state.analyzer.get_recognizers();
    Json(RecognizersResponse { recognizers })
}

async fn supported_entities_handler() -> Json<EntitiesResponse> {
    let entities = vec![
        "EMAIL".to_string(),
        "PHONE_NUMBER".to_string(),
        "CREDIT_CARD".to_string(),
        "IP_ADDRESS".to_string(),
        "URL".to_string(),
        "US_SSN".to_string(),
        "UK_NHS".to_string(),
        "PERSON".to_string(),
        "LOCATION".to_string(),
        "DATE_TIME".to_string(),
        "IBAN_CODE".to_string(),
        "NRP".to_string(),
        "MEDICAL_LICENSE".to_string(),
        "US_DRIVER_LICENSE".to_string(),
        "US_PASSPORT".to_string(),
        "US_BANK_NUMBER".to_string(),
        "CRYPTO".to_string(),
        "AU_ABN".to_string(),
        "AU_ACN".to_string(),
        "AU_TFN".to_string(),
        "AU_MEDICARE".to_string(),
        "IN_PAN".to_string(),
        "IN_AADHAAR".to_string(),
        "SG_NRIC_FIN".to_string(),
        "ES_NIF".to_string(),
        "IT_FISCAL_CODE".to_string(),
        "IT_DRIVER_LICENSE".to_string(),
        "IT_VAT_CODE".to_string(),
        "IT_PASSPORT".to_string(),
        "IT_IDENTITY_CARD".to_string(),
    ];

    Json(EntitiesResponse {
        supported_entities: entities,
    })
}

async fn supported_languages_handler() -> Json<LanguagesResponse> {
    let languages = vec![
        "en".to_string(),
        "es".to_string(),
        "fr".to_string(),
        "de".to_string(),
        "it".to_string(),
        "pt".to_string(),
        "nl".to_string(),
        "pl".to_string(),
        "fi".to_string(),
        "ko".to_string(),
        "th".to_string(),
    ];

    Json(LanguagesResponse {
        supported_languages: languages,
    })
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();

    info!("Initializing Presidio Analyzer API");

    // Create analyzer engine with default recognizers
    let analyzer = AnalyzerEngine::with_defaults();

    // Create shared state
    let state = AppState {
        analyzer: Arc::new(analyzer),
    };

    // Build router
    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/analyze", post(analyze_handler))
        .route("/recognizers", get(recognizers_handler))
        .route("/supportedentities", get(supported_entities_handler))
        .route("/supportedlanguages", get(supported_languages_handler))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Start server
    let addr = std::env::var("PRESIDIO_ANALYZER_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".to_string());
    info!("Starting Presidio Analyzer API on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
