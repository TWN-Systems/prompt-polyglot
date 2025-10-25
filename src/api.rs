use crate::models::{OptimizationRequest, OptimizationResult, ReviewDecision, ReviewSession};
use crate::optimizer::Optimizer;
use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// Shared application state
pub struct AppState {
    pub optimizer: Arc<Mutex<Optimizer>>,
    pub review_sessions: Arc<Mutex<std::collections::HashMap<String, ReviewSession>>>,
}

/// Health check endpoint
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "prompt-compress",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// Optimize a prompt
pub async fn optimize_prompt(
    data: web::Data<AppState>,
    request: web::Json<OptimizationRequest>,
) -> impl Responder {
    let mut optimizer = match data.optimizer.lock() {
        Ok(opt) => opt,
        Err(_) => return HttpResponse::InternalServerError().json(ErrorResponse {
            error: "Failed to acquire optimizer lock".to_string(),
        }),
    };

    match optimizer.optimize(&request) {
        Ok(result) => {
            // If there are items requiring review, create a session
            if !result.requires_review.is_empty() {
                let session_id = uuid::Uuid::new_v4().to_string();
                let session = ReviewSession {
                    session_id: session_id.clone(),
                    pending_optimizations: result.requires_review.clone(),
                    decisions: std::collections::HashMap::new(),
                };

                if let Ok(mut sessions) = data.review_sessions.lock() {
                    sessions.insert(session_id.clone(), session);
                }

                HttpResponse::Ok().json(OptimizationResponse {
                    result,
                    review_session_id: Some(session_id),
                })
            } else {
                HttpResponse::Ok().json(OptimizationResponse {
                    result,
                    review_session_id: None,
                })
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse {
            error: format!("Optimization failed: {}", e),
        }),
    }
}

/// Webhook endpoint for automated parsing
/// Receives a webhook request, optimizes the prompt, and returns the result
#[derive(Debug, Deserialize)]
pub struct WebhookRequest {
    pub prompt: String,
    #[serde(default)]
    pub output_language: Option<String>,
    #[serde(default)]
    pub confidence_threshold: Option<f64>,
    #[serde(default)]
    pub aggressive_mode: Option<bool>,
    #[serde(default)]
    pub callback_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct WebhookResponse {
    pub request_id: String,
    pub optimized_prompt: String,
    pub original_tokens: usize,
    pub optimized_tokens: usize,
    pub token_savings: i64,
    pub savings_percentage: f64,
    pub status: String,
}

pub async fn webhook_optimize(
    data: web::Data<AppState>,
    request: web::Json<WebhookRequest>,
) -> impl Responder {
    let mut optimizer = match data.optimizer.lock() {
        Ok(opt) => opt,
        Err(_) => return HttpResponse::InternalServerError().json(ErrorResponse {
            error: "Failed to acquire optimizer lock".to_string(),
        }),
    };

    // Convert webhook request to optimization request
    let output_language = match request.output_language.as_deref() {
        Some("mandarin") | Some("zh") => crate::models::Language::Mandarin,
        _ => crate::models::Language::English,
    };

    let opt_request = OptimizationRequest {
        prompt: request.prompt.clone(),
        output_language,
        confidence_threshold: request.confidence_threshold.unwrap_or(0.85),
        aggressive_mode: request.aggressive_mode.unwrap_or(false),
        directive_format: crate::models::DirectiveFormat::Bracketed,
    };

    match optimizer.optimize(&opt_request) {
        Ok(result) => {
            let request_id = uuid::Uuid::new_v4().to_string();

            // If callback URL is provided, send result asynchronously
            if let Some(callback_url) = &request.callback_url {
                let response = WebhookResponse {
                    request_id: request_id.clone(),
                    optimized_prompt: result.optimized_prompt.clone(),
                    original_tokens: result.original_tokens,
                    optimized_tokens: result.optimized_tokens,
                    token_savings: result.token_savings,
                    savings_percentage: result.savings_percentage,
                    status: "completed".to_string(),
                };

                // Spawn async task to send callback (non-blocking)
                let callback_url = callback_url.clone();
                actix_rt::spawn(async move {
                    let client = reqwest::Client::new();
                    let _ = client.post(&callback_url).json(&response).send().await;
                });
            }

            HttpResponse::Ok().json(WebhookResponse {
                request_id,
                optimized_prompt: result.optimized_prompt,
                original_tokens: result.original_tokens,
                optimized_tokens: result.optimized_tokens,
                token_savings: result.token_savings,
                savings_percentage: result.savings_percentage,
                status: "completed".to_string(),
            })
        }
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse {
            error: format!("Optimization failed: {}", e),
        }),
    }
}

/// Get review session
pub async fn get_review_session(
    data: web::Data<AppState>,
    session_id: web::Path<String>,
) -> impl Responder {
    let sessions = match data.review_sessions.lock() {
        Ok(s) => s,
        Err(_) => return HttpResponse::InternalServerError().json(ErrorResponse {
            error: "Failed to acquire sessions lock".to_string(),
        }),
    };

    match sessions.get(session_id.as_str()) {
        Some(session) => HttpResponse::Ok().json(session),
        None => HttpResponse::NotFound().json(ErrorResponse {
            error: "Review session not found".to_string(),
        }),
    }
}

/// Submit review decisions
#[derive(Debug, Deserialize)]
pub struct ReviewSubmission {
    pub decisions: std::collections::HashMap<String, ReviewDecision>,
}

pub async fn submit_review(
    data: web::Data<AppState>,
    session_id: web::Path<String>,
    submission: web::Json<ReviewSubmission>,
) -> impl Responder {
    let mut sessions = match data.review_sessions.lock() {
        Ok(s) => s,
        Err(_) => return HttpResponse::InternalServerError().json(ErrorResponse {
            error: "Failed to acquire sessions lock".to_string(),
        }),
    };

    match sessions.get_mut(session_id.as_str()) {
        Some(session) => {
            session.decisions.extend(submission.decisions.clone());

            // Update corpus with feedback
            let mut optimizer = match data.optimizer.lock() {
                Ok(opt) => opt,
                Err(_) => return HttpResponse::InternalServerError().json(ErrorResponse {
                    error: "Failed to acquire optimizer lock".to_string(),
                }),
            };

            for (opt_id, decision) in &submission.decisions {
                if let Some(opt) = session
                    .pending_optimizations
                    .iter()
                    .find(|o| &o.id == opt_id)
                {
                    let accepted = matches!(decision, ReviewDecision::Accept);
                    optimizer.calculator_mut().update_corpus(
                        &opt.original_text,
                        accepted,
                        opt.token_savings,
                    );
                }
            }

            HttpResponse::Ok().json(serde_json::json!({
                "status": "success",
                "message": "Review decisions recorded"
            }))
        }
        None => HttpResponse::NotFound().json(ErrorResponse {
            error: "Review session not found".to_string(),
        }),
    }
}

/// Analyze prompt without optimizing
pub async fn analyze_prompt(
    data: web::Data<AppState>,
    request: web::Json<OptimizationRequest>,
) -> impl Responder {
    let mut optimizer = match data.optimizer.lock() {
        Ok(opt) => opt,
        Err(_) => return HttpResponse::InternalServerError().json(ErrorResponse {
            error: "Failed to acquire optimizer lock".to_string(),
        }),
    };

    match optimizer.optimize(&request) {
        Ok(result) => {
            // Return analysis without applying optimizations
            let analysis = AnalysisResponse {
                original_tokens: result.original_tokens,
                potential_savings: result.token_savings,
                savings_percentage: result.savings_percentage,
                detected_optimizations: result.optimizations.len() + result.requires_review.len(),
                auto_apply_count: result.optimizations.len(),
                review_required_count: result.requires_review.len(),
                optimizations: result.optimizations,
                requires_review: result.requires_review,
            };

            HttpResponse::Ok().json(analysis)
        }
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse {
            error: format!("Analysis failed: {}", e),
        }),
    }
}

// Response structures

#[derive(Debug, Serialize)]
pub struct OptimizationResponse {
    pub result: OptimizationResult,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_session_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AnalysisResponse {
    pub original_tokens: usize,
    pub potential_savings: i64,
    pub savings_percentage: f64,
    pub detected_optimizations: usize,
    pub auto_apply_count: usize,
    pub review_required_count: usize,
    pub optimizations: Vec<crate::models::Optimization>,
    pub requires_review: Vec<crate::models::Optimization>,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// Configure API routes
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .route("/health", web::get().to(health_check))
            .route("/optimize", web::post().to(optimize_prompt))
            .route("/analyze", web::post().to(analyze_prompt))
            .route("/webhook/optimize", web::post().to(webhook_optimize))
            .route("/review/{session_id}", web::get().to(get_review_session))
            .route("/review/{session_id}", web::post().to(submit_review)),
    );
}
