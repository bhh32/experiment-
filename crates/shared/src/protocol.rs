use serde::{Deserialize, Serialize};

use crate::FileFormat;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub format: FileFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertRequest {
    pub content: String,
    pub from: FileFormat,
    pub to: FileFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertResponse {
    pub content: String,
    pub format: FileFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewRequest {
    pub content: String,
    pub mode: FileFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewResponse {
    pub html: String,
    pub mode: FileFormat,
}
