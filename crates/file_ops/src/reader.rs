use std::path::Path;

use crate::FileOpsError;
use utils::safe_resolve;

/// Read a file's contents given a base directory and relative path.
pub async fn read_file(base: &Path, relative: &str) -> Result<Vec<u8>, FileOpsError> {
    let path = safe_resolve(base, relative)?;
    let contents = tokio::fs::read(&path).await?;
    Ok(contents)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[tokio::test]
    async fn reads_existing_file() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("test.md"), "# Hello").unwrap();
        let contents = read_file(tmp.path(), "test.md").await.unwrap();
        assert_eq!(contents, b"# Hello");
    }

    #[tokio::test]
    async fn errors_on_missing_file() {
        let tmp = tempfile::tempdir().unwrap();
        let result = read_file(tmp.path(), "missing.md").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn rejects_traversal() {
        let tmp = tempfile::tempdir().unwrap();
        let result = read_file(tmp.path(), "../etc/passwd").await;
        assert!(result.is_err());
    }
}
