use std::path::PathBuf;

use tower_http::services::{ServeDir, ServeFile};

#[tokio::main]
async fn main() {
    let data_dir = std::env::var("DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("./data"));

    let public_dir = std::env::var("PUBLIC_DIR")
        .unwrap_or_else(|_| "./public".to_string());

    // Create data directory if it doesn't exist
    tokio::fs::create_dir_all(&data_dir).await.unwrap();

    let state = backend::AppState::new(data_dir);

    let serve_dir = ServeDir::new(&public_dir)
        .fallback(ServeFile::new(format!("{public_dir}/index.html")));

    let app = backend::api_router(state)
        .fallback_service(serve_dir);

    let port = shared::DEFAULT_PORT;
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .unwrap();

    eprintln!("Listening on http://localhost:{port}");
    axum::serve(listener, app).await.unwrap();
}
