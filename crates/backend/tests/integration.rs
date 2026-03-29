use std::path::PathBuf;

use reqwest::Client;
use serde_json::Value;

async fn spawn_server() -> (String, PathBuf, tempfile::TempDir) {
    let tmp = tempfile::tempdir().unwrap();
    let data_dir = tmp.path().to_path_buf();
    let state = backend::AppState::new(data_dir.clone());

    let app = backend::api_router(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let base_url = format!("http://127.0.0.1:{port}");
    (base_url, data_dir, tmp)
}

#[tokio::test]
async fn health_returns_ok() {
    let (url, _, _tmp) = spawn_server().await;
    let resp = Client::new()
        .get(format!("{url}/api/health"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn list_files_empty() {
    let (url, _, _tmp) = spawn_server().await;
    let resp = Client::new()
        .get(format!("{url}/api/files"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert!(body.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn file_crud_roundtrip() {
    let (url, _, _tmp) = spawn_server().await;
    let client = Client::new();

    // Create
    let resp = client
        .put(format!("{url}/api/files/test.md"))
        .body("# Hello World")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // Read
    let resp = client
        .get(format!("{url}/api/files/test.md"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(resp.text().await.unwrap(), "# Hello World");

    // List
    let resp = client
        .get(format!("{url}/api/files"))
        .send()
        .await
        .unwrap();
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body.as_array().unwrap().len(), 1);
    assert_eq!(body[0]["name"], "test.md");

    // Delete
    let resp = client
        .delete(format!("{url}/api/files/test.md"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // Verify deleted
    let resp = client
        .get(format!("{url}/api/files/test.md"))
        .send()
        .await
        .unwrap();
    assert_ne!(resp.status(), 200);
}

#[tokio::test]
async fn list_filters_unsupported() {
    let (url, data_dir, _tmp) = spawn_server().await;

    std::fs::write(data_dir.join("doc.md"), "markdown").unwrap();
    std::fs::write(data_dir.join("image.png"), "png").unwrap();

    let resp = Client::new()
        .get(format!("{url}/api/files"))
        .send()
        .await
        .unwrap();
    let body: Value = resp.json().await.unwrap();
    let names: Vec<&str> = body
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"doc.md"));
    assert!(!names.contains(&"image.png"));
}

#[tokio::test]
async fn convert_md_to_docx() {
    let (url, _, _tmp) = spawn_server().await;
    let resp = Client::new()
        .post(format!("{url}/api/convert"))
        .json(&serde_json::json!({
            "content": "# Test\n\nHello world.",
            "from": "markdown",
            "to": "docx"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["format"], "docx");
    // Content is base64 encoded
    assert!(!body["content"].as_str().unwrap().is_empty());
}

#[tokio::test]
async fn preview_markdown_mode() {
    let (url, _, _tmp) = spawn_server().await;
    let resp = Client::new()
        .post(format!("{url}/api/preview"))
        .json(&serde_json::json!({
            "content": "**bold**",
            "mode": "markdown"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert!(body["html"].as_str().unwrap().contains("<strong>bold</strong>"));
}

#[tokio::test]
async fn preview_docx_mode() {
    let (url, _, _tmp) = spawn_server().await;
    let resp = Client::new()
        .post(format!("{url}/api/preview"))
        .json(&serde_json::json!({
            "content": "test",
            "mode": "docx"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert!(body["html"].as_str().unwrap().contains("Calibri"));
}

#[tokio::test]
async fn preview_odt_mode() {
    let (url, _, _tmp) = spawn_server().await;
    let resp = Client::new()
        .post(format!("{url}/api/preview"))
        .json(&serde_json::json!({
            "content": "test",
            "mode": "odt"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert!(body["html"].as_str().unwrap().contains("Liberation Serif"));
}

#[tokio::test]
async fn path_traversal_blocked() {
    let (url, _, _tmp) = spawn_server().await;
    // Various traversal attempts
    for path in &["../etc/passwd", "..%2F..%2Fetc%2Fpasswd"] {
        let resp = Client::new()
            .get(format!("{url}/api/files/{path}"))
            .send()
            .await
            .unwrap();
        // Either 400 (our handler) or 404 (router normalization) — both block access
        assert!(
            resp.status() == 400 || resp.status() == 404,
            "expected 400 or 404 for {path}, got {}",
            resp.status()
        );
    }
}
