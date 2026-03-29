use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("Failed to parse response: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("Authentication required")]
    Unauthorized,
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("API error ({status}): {message}")]
    Api { status: u16, message: String },
}

// Tauri commands need serializable errors
impl serde::Serialize for ApiError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
