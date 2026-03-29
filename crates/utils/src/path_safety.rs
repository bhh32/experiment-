use std::path::{Path, PathBuf};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum PathError {
    #[error("path contains forbidden component: {0}")]
    ForbiddenComponent(String),
    #[error("path escapes base directory")]
    Escape,
    #[error("absolute paths are not allowed")]
    Absolute,
}

/// Resolve a user-provided relative path against a base directory safely.
/// Rejects absolute paths, `..`, and ensures the result stays under `base`.
pub fn safe_resolve(base: &Path, relative: &str) -> Result<PathBuf, PathError> {
    let rel = Path::new(relative);

    if rel.is_absolute() {
        return Err(PathError::Absolute);
    }

    for component in rel.components() {
        match component {
            std::path::Component::ParentDir => {
                return Err(PathError::ForbiddenComponent("..".into()));
            }
            std::path::Component::Normal(s) => {
                let s = s.to_string_lossy();
                if s.starts_with('.') && s.len() > 1 && !s.starts_with("..") {
                    // Allow dotfiles like .gitignore but not ..
                }
            }
            _ => {}
        }
    }

    let resolved = base.join(rel);

    // Canonicalize base (it must exist) and check the resolved path starts with it
    let canon_base = base
        .canonicalize()
        .map_err(|_| PathError::Escape)?;

    // The resolved path might not exist yet (creating a new file), so we
    // canonicalize the parent directory and check containment
    if let Some(parent) = resolved.parent() {
        if parent.exists() {
            let canon_parent = parent
                .canonicalize()
                .map_err(|_| PathError::Escape)?;
            if !canon_parent.starts_with(&canon_base) {
                return Err(PathError::Escape);
            }
        }
    }

    Ok(resolved)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn resolves_simple_path() {
        let tmp = tempfile::tempdir().unwrap();
        let result = safe_resolve(tmp.path(), "hello.md").unwrap();
        assert_eq!(result, tmp.path().join("hello.md"));
    }

    #[test]
    fn rejects_parent_traversal() {
        let tmp = tempfile::tempdir().unwrap();
        let result = safe_resolve(tmp.path(), "../etc/passwd");
        assert!(result.is_err());
    }

    #[test]
    fn rejects_absolute_path() {
        let tmp = tempfile::tempdir().unwrap();
        let result = safe_resolve(tmp.path(), "/etc/passwd");
        assert!(result.is_err());
    }

    #[test]
    fn allows_nested_path() {
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("subdir")).unwrap();
        let result = safe_resolve(tmp.path(), "subdir/file.md").unwrap();
        assert_eq!(result, tmp.path().join("subdir/file.md"));
    }

    #[test]
    fn rejects_dotdot_in_middle() {
        let tmp = tempfile::tempdir().unwrap();
        let result = safe_resolve(tmp.path(), "subdir/../../etc/passwd");
        assert!(result.is_err());
    }
}
