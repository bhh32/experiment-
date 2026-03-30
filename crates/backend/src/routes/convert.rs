use axum::{
    Router,
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json,
};
use shared::{ConvertRequest, ConvertResponse, DocStyle, FileFormat};

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/api/convert", post(convert))
}

async fn convert(Json(req): Json<ConvertRequest>) -> impl IntoResponse {
    match (req.from, req.to) {
        (FileFormat::Markdown, FileFormat::Docx) => {
            let mut style = DocStyle::default();
            if let Some(ref f) = req.font {
                style.body_font = f.clone();
            }
            if let Some(fs) = req.font_size {
                style.body_size_pt = fs;
            }
            if let Some(lh) = req.line_height {
                style.line_spacing = lh;
            }

            // Parse markdown → Document IR → DOCX (single source of truth)
            let doc = document_ir::parse_markdown(&req.content);
            match document_ir::render_to_docx(&doc, &style) {
                Ok(bytes) => {
                    let encoded = base64_encode(&bytes);
                    let resp = ConvertResponse {
                        content: encoded,
                        format: FileFormat::Docx,
                    };
                    (StatusCode::OK, Json(resp)).into_response()
                }
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
            }
        }
        (FileFormat::Markdown, FileFormat::Odt) => {
            match conversion::markdown_to_odt(&req.content) {
                Ok(bytes) => {
                    let encoded = base64_encode(&bytes);
                    let resp = ConvertResponse {
                        content: encoded,
                        format: FileFormat::Odt,
                    };
                    (StatusCode::OK, Json(resp)).into_response()
                }
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
            }
        }
        _ => (
            StatusCode::BAD_REQUEST,
            "unsupported conversion".to_string(),
        )
            .into_response(),
    }
}

fn base64_encode(data: &[u8]) -> String {
    use std::fmt::Write;
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let triple = (b0 << 16) | (b1 << 8) | b2;

        let _ = write!(result, "{}", CHARS[((triple >> 18) & 0x3F) as usize] as char);
        let _ = write!(result, "{}", CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            let _ = write!(result, "{}", CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            let _ = write!(result, "{}", CHARS[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}
