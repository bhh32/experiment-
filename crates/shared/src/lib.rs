mod document;
mod protocol;
mod doc_style;

pub use document::{DocumentMeta, FileFormat};
pub use protocol::{ConvertRequest, ConvertResponse, FileEntry, PreviewRequest, PreviewResponse};
pub use doc_style::DocStyle;

pub const API_PREFIX: &str = "/api";
pub const DEFAULT_PORT: u16 = 8080;
pub const SUPPORTED_EXTENSIONS: &[&str] = &["md", "markdown", "docx", "odt"];
