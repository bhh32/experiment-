use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, put},
    Json,
};

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/files", get(list_files))
        .route("/api/files/{*path}", get(read_file).put(write_file).delete(delete_file))
}

async fn list_files(State(state): State<AppState>) -> impl IntoResponse {
    match file_ops::list_files(&state.data_dir).await {
        Ok(files) => (StatusCode::OK, Json(serde_json::to_value(files).unwrap())).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn read_file(
    State(state): State<AppState>,
    Path(path): Path<String>,
) -> impl IntoResponse {
    match file_ops::read_file(&state.data_dir, &path).await {
        Ok(contents) => {
            // If it's a text file, return as string. Otherwise binary.
            match String::from_utf8(contents.clone()) {
                Ok(text) => (StatusCode::OK, text).into_response(),
                Err(_) => (StatusCode::OK, contents).into_response(),
            }
        }
        Err(e) => {
            let status = if e.to_string().contains("path") {
                StatusCode::BAD_REQUEST
            } else {
                StatusCode::NOT_FOUND
            };
            (status, e.to_string()).into_response()
        }
    }
}

async fn write_file(
    State(state): State<AppState>,
    Path(path): Path<String>,
    body: String,
) -> impl IntoResponse {
    match file_ops::write_file(&state.data_dir, &path, body.as_bytes()).await {
        Ok(()) => StatusCode::OK,
        Err(e) => {
            if e.to_string().contains("path") {
                StatusCode::BAD_REQUEST
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }
}

async fn delete_file(
    State(state): State<AppState>,
    Path(path): Path<String>,
) -> impl IntoResponse {
    let resolved = match utils::safe_resolve(&state.data_dir, &path) {
        Ok(p) => p,
        Err(_) => return StatusCode::BAD_REQUEST,
    };

    match tokio::fs::remove_file(&resolved).await {
        Ok(()) => StatusCode::OK,
        Err(_) => StatusCode::NOT_FOUND,
    }
}
