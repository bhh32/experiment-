use std::process::Command;

use crate::ConversionError;

fn pandoc_available() -> bool {
    Command::new("pandoc")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success())
}

/// Convert markdown to ODT bytes using pandoc.
pub fn markdown_to_odt(markdown: &str) -> Result<Vec<u8>, ConversionError> {
    if !pandoc_available() {
        return Err(ConversionError::PandocUnavailable(
            "pandoc is not installed".into(),
        ));
    }

    let tmp_dir = tempfile::tempdir()
        .map_err(|e| ConversionError::Io(e))?;
    let input_path = tmp_dir.path().join("input.md");
    let output_path = tmp_dir.path().join("output.odt");

    std::fs::write(&input_path, markdown)?;

    let output = Command::new("pandoc")
        .args([
            input_path.to_str().unwrap(),
            "-o",
            output_path.to_str().unwrap(),
        ])
        .output()?;

    if !output.status.success() {
        return Err(ConversionError::PandocFailed(
            String::from_utf8_lossy(&output.stderr).into_owned(),
        ));
    }

    let bytes = std::fs::read(&output_path)?;
    Ok(bytes)
}

/// Convert ODT bytes to markdown using pandoc.
pub fn odt_to_markdown(bytes: &[u8]) -> Result<String, ConversionError> {
    if !pandoc_available() {
        return Err(ConversionError::PandocUnavailable(
            "pandoc is not installed".into(),
        ));
    }

    let tmp_dir = tempfile::tempdir()?;
    let input_path = tmp_dir.path().join("input.odt");
    let output_path = tmp_dir.path().join("output.md");

    std::fs::write(&input_path, bytes)?;

    let output = Command::new("pandoc")
        .args([
            input_path.to_str().unwrap(),
            "-t",
            "gfm",
            "-o",
            output_path.to_str().unwrap(),
        ])
        .output()?;

    if !output.status.success() {
        return Err(ConversionError::PandocFailed(
            String::from_utf8_lossy(&output.stderr).into_owned(),
        ));
    }

    let md = std::fs::read_to_string(&output_path)?;
    Ok(md)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn odt_roundtrip() {
        if !pandoc_available() {
            eprintln!("skipping: pandoc not installed");
            return;
        }
        let md = "# Test\n\nA paragraph with **bold** text.\n";
        let odt_bytes = markdown_to_odt(md).unwrap();
        // ODT is a ZIP file
        assert_eq!(&odt_bytes[..2], b"PK");

        let back = odt_to_markdown(&odt_bytes).unwrap();
        assert!(back.contains("Test"));
        assert!(back.contains("paragraph"));
    }

    #[test]
    fn odt_produces_valid_zip() {
        if !pandoc_available() {
            eprintln!("skipping: pandoc not installed");
            return;
        }
        let bytes = markdown_to_odt("Hello world").unwrap();
        assert_eq!(&bytes[..2], b"PK");
        assert!(bytes.len() > 100);
    }
}
