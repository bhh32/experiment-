mod reader;
mod writer;
mod listing;

pub use reader::read_file;
pub use writer::write_file;
pub use listing::list_files;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum FileOpsError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("path error: {0}")]
    Path(#[from] utils::PathError),
    #[error("unsupported format: {0}")]
    UnsupportedFormat(String),
}
