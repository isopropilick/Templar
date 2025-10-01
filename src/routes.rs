//! Route handlers: defines `/send` endpoint and a thin auth check.

use std::{collections::HashMap, sync::Arc};

use axum::{extract::State, http::StatusCode, Json};
use serde::Deserialize;

use crate::email::{render_and_send, EmailError, EmailState};

/// JSON payload for `/send`
#[derive(Deserialize)]
pub struct SendRequest {
    /// Comma-separated list or single recipient
    pub(crate) to: String,
    pub(crate) subject: String,
    /// Template filename (without `.hbs`)
    pub(crate) template: String,
    /// Arbitrary key/value vars for Handlebars
    #[serde(default)]
    pub(crate) vars: HashMap<String, serde_json::Value>,
}

/// Naive API key auth for demo.
/// - Expects `API_KEY` set in env.
/// - Compares against a pseudo header provided via env `API_KEY_CURRENT_REQUEST`.
/// - If no `API_KEY` is set, auth is disabled (dev convenience).
fn is_authorized() -> bool {
    match std::env::var("API_KEY") {
        Ok(key) if !key.is_empty() => {
            let provided = std::env::var("API_KEY_CURRENT_REQUEST").unwrap_or_default();
            key == provided
        }
        Ok(_) => false,
        Err(_) => true,
    }
}

/// POST `/send`
/// - Requires a valid `SendRequest` JSON body
/// - Returns `{"status":"ok","id":..}` or `{"error":..}`
pub async fn send_email(
    State(state): State<Arc<EmailState>>,
    Json(payload): Json<SendRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    // 1) Auth
    if !is_authorized() {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "unauthorized" })),
        ));
    }

    // 2) Try to render + send
    match render_and_send(state.as_ref(), payload).await {
        Ok(message_id) => Ok(Json(serde_json::json!({
            "status": "ok",
            "id": message_id,
        }))),
        Err(e) => {
            // Map domain error → status code
            let (code, msg) = match e {
                EmailError::TemplateNotFound(_) => (StatusCode::NOT_FOUND, e.to_string()),
                EmailError::RenderError(_) => (StatusCode::UNPROCESSABLE_ENTITY, e.to_string()),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            };
            Err((code, Json(serde_json::json!({ "error": msg }))))
        }
    }
}
