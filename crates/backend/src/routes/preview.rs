use axum::{
    Router,
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json,
};
use shared::{FileFormat, PreviewRequest, PreviewResponse};

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/api/preview", post(preview))
}

async fn preview(Json(req): Json<PreviewRequest>) -> impl IntoResponse {
    let html = match req.mode {
        FileFormat::Markdown => markdown_preview::render_preview(&req.content),
        FileFormat::Docx => markdown_preview::render_docx_preview(&req.content),
        FileFormat::Odt => markdown_preview::render_odt_preview(&req.content),
    };

    (
        StatusCode::OK,
        Json(PreviewResponse {
            html,
            mode: req.mode,
        }),
    )
        .into_response()
}
