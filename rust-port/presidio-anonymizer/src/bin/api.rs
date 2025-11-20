use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use presidio_anonymizer::AnonymizerEngine;
use presidio_common::{ConflictResolutionStrategy, EntityType, RecognizerResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{info, Level};

#[derive(Clone)]
struct AppState {
    anonymizer: Arc<AnonymizerEngine>,
}

#[derive(Debug, Deserialize)]
struct AnonymizeRequest {
    /// Original text to anonymize
    text: String,

    /// Analyzer results with PII locations
    analyzer_results: Vec<AnalyzerResultDto>,

    /// Operators to apply per entity type
    #[serde(default)]
    anonymizers: HashMap<String, OperatorConfigDto>,

    /// Conflict resolution strategy (default: "merge_first_before_second")
    #[serde(default = "default_conflict_resolution")]
    conflict_resolution: String,
}

fn default_conflict_resolution() -> String {
    "merge_first_before_second".to_string()
}

#[derive(Debug, Deserialize)]
struct AnalyzerResultDto {
    entity_type: String,
    start: usize,
    end: usize,
    score: f32,
}

#[derive(Debug, Deserialize)]
struct OperatorConfigDto {
    /// Type of operator (replace, redact, mask, hash, encrypt, keep)
    #[serde(rename = "type")]
    operator_type: String,

    /// Parameters for the operator
    #[serde(default)]
    params: Value,
}

#[derive(Debug, Serialize)]
struct AnonymizeResponse {
    /// Anonymized text
    text: String,

    /// Details of anonymization operations performed
    items: Vec<OperationItemDto>,
}

#[derive(Debug, Serialize)]
struct OperationItemDto {
    /// Original entity type
    entity_type: String,

    /// Start position in original text
    start: usize,

    /// End position in original text
    end: usize,

    /// Operator applied
    operator: String,

    /// Original text (before anonymization)
    text: String,

    /// Anonymized result
    result: String,
}

#[derive(Debug, Serialize)]
struct OperatorsResponse {
    operators: Vec<String>,
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

async fn anonymize_handler(
    State(state): State<AppState>,
    Json(request): Json<AnonymizeRequest>,
) -> Result<Json<AnonymizeResponse>, (StatusCode, String)> {
    info!(
        "Anonymizing text of length {} with {} entities",
        request.text.len(),
        request.analyzer_results.len()
    );

    // Convert DTOs to internal types
    let mut analyzer_results = Vec::new();
    for dto in request.analyzer_results {
        let entity_type = EntityType::from_str(&dto.entity_type)
            .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid entity type: {}", e)))?;

        analyzer_results.push(RecognizerResult {
            entity_type,
            start: dto.start,
            end: dto.end,
            score: dto.score,
            analysis_explanation: None,
            recognition_metadata: HashMap::new(),
        });
    }

    // Convert operator configs
    let mut operators = HashMap::new();
    for (entity_str, op_config) in request.anonymizers {
        let entity_type = EntityType::from_str(&entity_str)
            .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid entity type: {}", e)))?;
        operators.insert(entity_type, (op_config.operator_type, op_config.params));
    }

    // Parse conflict resolution strategy
    let conflict_resolution = match request.conflict_resolution.as_str() {
        "merge_first_before_second" => ConflictResolutionStrategy::MergeFirstBeforeSecond,
        "merge_similar_or_contained" => ConflictResolutionStrategy::MergeSimilarOrContained,
        "remove_intersections" => ConflictResolutionStrategy::RemoveIntersections,
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!(
                    "Invalid conflict resolution strategy: {}",
                    request.conflict_resolution
                ),
            ))
        }
    };

    // Anonymize
    let result = state
        .anonymizer
        .anonymize(&request.text, &analyzer_results, &operators, conflict_resolution)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Convert to response
    let items = result
        .items
        .into_iter()
        .map(|item| OperationItemDto {
            entity_type: item.entity_type,
            start: item.start,
            end: item.end,
            operator: item.operator,
            text: item.text,
            result: item.result,
        })
        .collect();

    info!("Anonymization complete, {} operations performed", items.len());

    Ok(Json(AnonymizeResponse {
        text: result.text,
        items,
    }))
}

async fn operators_handler() -> Json<OperatorsResponse> {
    let operators = vec![
        "replace".to_string(),
        "redact".to_string(),
        "mask".to_string(),
        "hash".to_string(),
        "encrypt".to_string(),
        "keep".to_string(),
    ];

    Json(OperatorsResponse { operators })
}

async fn deanonymize_handler() -> Result<Json<Value>, (StatusCode, String)> {
    // Deanonymization is only supported for reversible operations like encrypt
    // This would require storing the mapping or encryption keys
    Err((
        StatusCode::NOT_IMPLEMENTED,
        "Deanonymization requires separate key management and is not yet implemented".to_string(),
    ))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();

    info!("Initializing Presidio Anonymizer API");

    // Create anonymizer engine with default operators
    let anonymizer = AnonymizerEngine::with_defaults();

    // Create shared state
    let state = AppState {
        anonymizer: Arc::new(anonymizer),
    };

    // Build router
    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/anonymize", post(anonymize_handler))
        .route("/deanonymize", post(deanonymize_handler))
        .route("/anonymizers", get(operators_handler))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Start server
    let addr = std::env::var("PRESIDIO_ANONYMIZER_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:3001".to_string());
    info!("Starting Presidio Anonymizer API on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
