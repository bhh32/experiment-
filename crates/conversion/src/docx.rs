use crate::ConversionError;
use docx_rs::*;

/// Convert markdown content to a DOCX byte buffer.
pub fn markdown_to_docx(markdown: &str) -> Result<Vec<u8>, ConversionError> {
    let mut doc = Docx::new();

    for line in markdown.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            doc = doc.add_paragraph(Paragraph::new());
            continue;
        }

        // Headings
        if let Some(text) = trimmed.strip_prefix("### ") {
            doc = doc.add_paragraph(
                Paragraph::new()
                    .add_run(Run::new().add_text(text).bold())
                    .style("Heading3"),
            );
        } else if let Some(text) = trimmed.strip_prefix("## ") {
            doc = doc.add_paragraph(
                Paragraph::new()
                    .add_run(Run::new().add_text(text).bold())
                    .style("Heading2"),
            );
        } else if let Some(text) = trimmed.strip_prefix("# ") {
            doc = doc.add_paragraph(
                Paragraph::new()
                    .add_run(Run::new().add_text(text).bold())
                    .style("Heading1"),
            );
        } else if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            let text = &trimmed[2..];
            doc = doc.add_paragraph(
                Paragraph::new()
                    .add_run(Run::new().add_text(text))
                    .numbering(NumberingId::new(1), IndentLevel::new(0)),
            );
        } else if trimmed.starts_with("---") || trimmed.starts_with("***") {
            // Horizontal rule as empty paragraph with bottom border
            doc = doc.add_paragraph(Paragraph::new());
        } else {
            // Regular paragraph - handle inline formatting
            let runs = parse_inline_formatting(trimmed);
            let mut para = Paragraph::new();
            for (text, bold, italic) in runs {
                let mut run = Run::new().add_text(&text);
                if bold {
                    run = run.bold();
                }
                if italic {
                    run = run.italic();
                }
                para = para.add_run(run);
            }
            doc = doc.add_paragraph(para);
        }
    }

    let mut buf = Vec::new();
    doc.build()
        .pack(&mut std::io::Cursor::new(&mut buf))
        .map_err(|e| ConversionError::DocxError(e.to_string()))?;

    Ok(buf)
}

/// Parse inline markdown formatting into (text, bold, italic) spans.
fn parse_inline_formatting(text: &str) -> Vec<(String, bool, bool)> {
    let mut spans = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    let mut bold = false;
    let mut italic = false;

    while i < chars.len() {
        if i + 1 < chars.len() && chars[i] == '*' && chars[i + 1] == '*' {
            if !current.is_empty() {
                spans.push((current.clone(), bold, italic));
                current.clear();
            }
            bold = !bold;
            i += 2;
        } else if chars[i] == '*' || chars[i] == '_' {
            if !current.is_empty() {
                spans.push((current.clone(), bold, italic));
                current.clear();
            }
            italic = !italic;
            i += 1;
        } else {
            current.push(chars[i]);
            i += 1;
        }
    }

    if !current.is_empty() {
        spans.push((current, bold, italic));
    }

    if spans.is_empty() {
        spans.push((text.to_string(), false, false));
    }

    spans
}

/// Extract plain text from DOCX bytes (basic extraction).
pub fn docx_to_markdown(bytes: &[u8]) -> Result<String, ConversionError> {
    let doc = read_docx(bytes)
        .map_err(|e| ConversionError::DocxError(format!("{e:?}")))?;

    let mut lines = Vec::new();

    for child in doc.document.children {
        if let DocumentChild::Paragraph(para) = child {
            let mut line = String::new();
            let mut is_heading = false;
            let mut heading_level = 0u8;

            // Check paragraph style for headings
            if let Some(ref style) = para.property.style {
                let style_id = &style.val;
                if style_id.contains("Heading1") || style_id == "1" {
                    is_heading = true;
                    heading_level = 1;
                } else if style_id.contains("Heading2") || style_id == "2" {
                    is_heading = true;
                    heading_level = 2;
                } else if style_id.contains("Heading3") || style_id == "3" {
                    is_heading = true;
                    heading_level = 3;
                }
            }

            for child in &para.children {
                if let ParagraphChild::Run(run) = child {
                    let is_bold = run.run_property.bold.is_some();
                    let is_italic = run.run_property.italic.is_some();

                    for run_child in &run.children {
                        if let RunChild::Text(text) = run_child {
                            let t = &text.text;
                            if is_bold && !is_heading {
                                line.push_str(&format!("**{t}**"));
                            } else if is_italic {
                                line.push_str(&format!("*{t}*"));
                            } else {
                                line.push_str(t);
                            }
                        }
                    }
                }
            }

            if is_heading {
                let prefix = "#".repeat(heading_level as usize);
                lines.push(format!("{prefix} {line}"));
            } else {
                lines.push(line);
            }
        }
    }

    Ok(lines.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdown_to_docx_produces_valid_zip() {
        let bytes = markdown_to_docx("# Hello\n\nA paragraph.").unwrap();
        // DOCX is a ZIP file, starts with PK
        assert_eq!(&bytes[..2], b"PK");
        assert!(bytes.len() > 100);
    }

    #[test]
    fn roundtrip_preserves_heading() {
        let md = "# Test Heading";
        let docx_bytes = markdown_to_docx(md).unwrap();
        let back = docx_to_markdown(&docx_bytes).unwrap();
        assert!(back.contains("# Test Heading"));
    }

    #[test]
    fn roundtrip_preserves_paragraph() {
        let md = "Just a paragraph.";
        let docx_bytes = markdown_to_docx(md).unwrap();
        let back = docx_to_markdown(&docx_bytes).unwrap();
        assert!(back.contains("Just a paragraph."));
    }

    #[test]
    fn handles_bold_text() {
        let md = "This has **bold** words.";
        let docx_bytes = markdown_to_docx(md).unwrap();
        assert!(docx_bytes.len() > 100);
    }

    #[test]
    fn handles_empty_input() {
        let bytes = markdown_to_docx("").unwrap();
        assert_eq!(&bytes[..2], b"PK");
    }
}
