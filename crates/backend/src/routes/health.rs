use axum::{Router, routing::get, Json};
use serde_json::{json, Value};

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/api/health", get(health_check))
}

async fn health_check() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}
