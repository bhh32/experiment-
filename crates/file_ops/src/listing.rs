use std::path::Path;

use shared::{FileFormat, SUPPORTED_EXTENSIONS};
use crate::FileOpsError;

/// List files in a directory and subdirectories, filtering to supported document formats.
pub async fn list_files(dir: &Path) -> Result<Vec<shared::FileEntry>, FileOpsError> {
    let mut entries = Vec::new();
    list_files_recursive(dir, dir, &mut entries).await?;
    entries.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(entries)
}

async fn list_files_recursive(
    base: &Path,
    dir: &Path,
    entries: &mut Vec<shared::FileEntry>,
) -> Result<(), FileOpsError> {
    let mut read_dir = tokio::fs::read_dir(dir).await?;

    while let Some(entry) = read_dir.next_entry().await? {
        let path = entry.path();
        let file_type = entry.file_type().await?;

        if file_type.is_dir() {
            // Skip hidden directories
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !name.starts_with('.') {
                Box::pin(list_files_recursive(base, &path, entries)).await?;
            }
            continue;
        }

        if !file_type.is_file() {
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

        // Relative path from the base directory
        let rel_path = path
            .strip_prefix(base)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();

        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        if let Some(format) = FileFormat::from_extension(ext) {
            entries.push(shared::FileEntry {
                name,
                path: rel_path,
                size_bytes: metadata.len(),
                format,
            });
        }
    }

    Ok(())
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

    #[tokio::test]
    async fn lists_subdirectories() {
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("subdir")).unwrap();
        fs::write(tmp.path().join("root.md"), "root").unwrap();
        fs::write(tmp.path().join("subdir/nested.md"), "nested").unwrap();

        let files = list_files(tmp.path()).await.unwrap();
        let paths: Vec<&str> = files.iter().map(|f| f.path.as_str()).collect();
        assert!(paths.contains(&"root.md"));
        assert!(paths.contains(&"subdir/nested.md"));
    }

    #[tokio::test]
    async fn skips_hidden_dirs() {
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir_all(tmp.path().join(".hidden")).unwrap();
        fs::write(tmp.path().join(".hidden/secret.md"), "secret").unwrap();
        fs::write(tmp.path().join("visible.md"), "visible").unwrap();

        let files = list_files(tmp.path()).await.unwrap();
        let names: Vec<&str> = files.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(names, vec!["visible.md"]);
    }
}
