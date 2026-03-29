use std::path::Path;

use shared::{FileFormat, SUPPORTED_EXTENSIONS};
use crate::FileOpsError;

/// List files in a directory, filtering to supported document formats.
pub async fn list_files(dir: &Path) -> Result<Vec<shared::FileEntry>, FileOpsError> {
    let mut entries = Vec::new();
    let mut read_dir = tokio::fs::read_dir(dir).await?;

    while let Some(entry) = read_dir.next_entry().await? {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        if !SUPPORTED_EXTENSIONS.contains(&ext) {
            continue;
        }

        let metadata = entry.metadata().await?;
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        if let Some(format) = FileFormat::from_extension(ext) {
            entries.push(shared::FileEntry {
                name: name.clone(),
                path: name,
                size_bytes: metadata.len(),
                format,
            });
        }
    }

    entries.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[tokio::test]
    async fn lists_supported_files_only() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("doc.md"), "markdown").unwrap();
        fs::write(tmp.path().join("paper.docx"), "docx").unwrap();
        fs::write(tmp.path().join("image.png"), "png").unwrap();
        fs::write(tmp.path().join("readme.txt"), "txt").unwrap();

        let files = list_files(tmp.path()).await.unwrap();
        let names: Vec<&str> = files.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(names, vec!["doc.md", "paper.docx"]);
    }

    #[tokio::test]
    async fn empty_directory() {
        let tmp = tempfile::tempdir().unwrap();
        let files = list_files(tmp.path()).await.unwrap();
        assert!(files.is_empty());
    }

    #[tokio::test]
    async fn includes_file_size() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("test.md"), "hello world").unwrap();
        let files = list_files(tmp.path()).await.unwrap();
        assert_eq!(files[0].size_bytes, 11);
    }
}
