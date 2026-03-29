use std::path::Path;

use crate::FileOpsError;
use utils::safe_resolve;

/// Write contents to a file given a base directory and relative path.
/// Creates parent directories if needed.
pub async fn write_file(base: &Path, relative: &str, contents: &[u8]) -> Result<(), FileOpsError> {
    let path = safe_resolve(base, relative)?;
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::fs::write(&path, contents).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[tokio::test]
    async fn writes_new_file() {
        let tmp = tempfile::tempdir().unwrap();
        write_file(tmp.path(), "new.md", b"content").await.unwrap();
        assert_eq!(fs::read_to_string(tmp.path().join("new.md")).unwrap(), "content");
    }

    #[tokio::test]
    async fn overwrites_existing() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("exist.md"), "old").unwrap();
        write_file(tmp.path(), "exist.md", b"new").await.unwrap();
        assert_eq!(fs::read_to_string(tmp.path().join("exist.md")).unwrap(), "new");
    }

    #[tokio::test]
    async fn rejects_traversal() {
        let tmp = tempfile::tempdir().unwrap();
        let result = write_file(tmp.path(), "../escape.md", b"bad").await;
        assert!(result.is_err());
    }
}
