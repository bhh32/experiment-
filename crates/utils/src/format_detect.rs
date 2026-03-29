use std::path::Path;

use shared::FileFormat;

/// Detect file format from a path's extension.
pub fn detect_format(path: &Path) -> Option<FileFormat> {
    FileFormat::from_path(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_markdown() {
        assert_eq!(detect_format(Path::new("readme.md")), Some(FileFormat::Markdown));
        assert_eq!(detect_format(Path::new("notes.markdown")), Some(FileFormat::Markdown));
    }

    #[test]
    fn detects_docx() {
        assert_eq!(detect_format(Path::new("paper.docx")), Some(FileFormat::Docx));
    }

    #[test]
    fn detects_odt() {
        assert_eq!(detect_format(Path::new("paper.odt")), Some(FileFormat::Odt));
    }

    #[test]
    fn returns_none_for_unknown() {
        assert_eq!(detect_format(Path::new("image.png")), None);
        assert_eq!(detect_format(Path::new("noext")), None);
    }
}
