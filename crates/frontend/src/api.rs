use gloo_net::http::Request;
use shared::{ConvertResponse, FileEntry, PreviewResponse};

const BASE: &str = "/api";

pub async fn health() -> Result<String, String> {
    let resp = Request::get(&format!("{BASE}/health"))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    resp.text().await.map_err(|e| e.to_string())
}

pub async fn list_files() -> Result<Vec<FileEntry>, String> {
    let resp = Request::get(&format!("{BASE}/files"))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    resp.json().await.map_err(|e| e.to_string())
}

pub async fn read_file(path: &str) -> Result<String, String> {
    let resp = Request::get(&format!("{BASE}/files/{path}"))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    resp.text().await.map_err(|e| e.to_string())
}

pub async fn save_file(path: &str, content: &str) -> Result<(), String> {
    Request::put(&format!("{BASE}/files/{path}"))
        .body(content)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn delete_file(path: &str) -> Result<(), String> {
    Request::delete(&format!("{BASE}/files/{path}"))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn convert(content: &str, from: &str, to: &str, font: Option<&str>, font_size: Option<f32>, line_height: Option<f32>) -> Result<ConvertResponse, String> {
    let resp = Request::post(&format!("{BASE}/convert"))
        .json(&serde_json::json!({
            "content": content,
            "from": from,
            "to": to,
            "font": font,
            "font_size": font_size,
            "line_height": line_height,
        }))
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    resp.json().await.map_err(|e| e.to_string())
}

pub async fn preview(content: &str, mode: &str) -> Result<PreviewResponse, String> {
    let resp = Request::post(&format!("{BASE}/preview"))
        .json(&serde_json::json!({
            "content": content,
            "mode": mode,
        }))
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    resp.json().await.map_err(|e| e.to_string())
}
