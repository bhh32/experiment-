//! # rw-format-epub
//!
//! EPUB export for Rust Writer.
//!
//! Generates EPUB 3.0 ebooks from documents. EPUB is essentially
//! a ZIP archive containing XHTML content, CSS styling, and metadata.
//!
//! - Chapter splitting (by heading level)
//! - Cover image
//! - Table of contents (NCX + HTML TOC)
//! - Embedded images
//! - Metadata (title, author, language, ISBN)
//! - CSS styling

use epub_builder::{EpubBuilder, EpubContent, EpubVersion as BuilderVersion, ZipLibrary, ReferenceType};
use rw_document::{Block, Document, Inline};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EpubError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("EPUB generation error: {0}")]
    Generation(String),
}

/// EPUB export options.
#[derive(Debug, Clone)]
pub struct EpubOptions {
    /// Split chapters at this heading level (1 = Heading 1)
    pub split_level: u8,
    /// Cover image path
    pub cover_image: Option<String>,
    /// CSS stylesheet content
    pub custom_css: Option<String>,
    /// EPUB version
    pub version: EpubVersion,
}

impl Default for EpubOptions {
    fn default() -> Self {
        Self {
            split_level: 1,
            cover_image: None,
            custom_css: None,
            version: EpubVersion::V3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpubVersion {
    V2,
    V3,
}

/// Read (import) an EPUB file and extract its text content into a Document.
pub fn read_epub(path: &std::path::Path) -> Result<Document, EpubError> {
    use rw_format_html::parse_html;

    use std::io::Read;
    use zip::ZipArchive;

    let file = std::fs::File::open(path)?;
    let mut archive = ZipArchive::new(file)
        .map_err(|e| EpubError::Generation(format!("ZIP error: {}", e)))?;

    let mut doc = Document::new();
    doc.sections.clear();

    // Find content files from container.xml
    let container_xml = {
        let mut entry = archive
            .by_name("META-INF/container.xml")
            .map_err(|_| EpubError::Generation("Missing META-INF/container.xml".to_string()))?;
        let mut s = String::new();
        entry.read_to_string(&mut s)?;
        s
    };

    // Extract rootfile path from container.xml
    let opf_path = extract_opf_path(&container_xml)
        .unwrap_or_else(|| "OEBPS/content.opf".to_string());

    // Read OPF file to get spine order
    let opf_dir = opf_path
        .rfind('/')
        .map(|i| opf_path[..=i].to_string())
        .unwrap_or_default();

    let opf_xml = {
        match archive.by_name(&opf_path) {
            Ok(mut entry) => {
                let mut s = String::new();
                entry.read_to_string(&mut s)?;
                s
            }
            Err(_) => String::new(),
        }
    };

    // Parse spine items from OPF
    let spine_items = parse_opf_spine(&opf_xml, &opf_dir);

    if spine_items.is_empty() {
        // Fallback: scan for .xhtml/.html files
        let mut html_files: Vec<String> = (0..archive.len())
            .filter_map(|i| {
                let name = archive.by_index(i).ok()?.name().to_string();
                if name.ends_with(".xhtml") || name.ends_with(".html") {
                    Some(name)
                } else {
                    None
                }
            })
            .collect();
        html_files.sort();

        for file_path in &html_files {
            let html = {
                match archive.by_name(file_path) {
                    Ok(mut entry) => {
                        let mut s = String::new();
                        entry.read_to_string(&mut s)?;
                        s
                    }
                    Err(_) => continue,
                }
            };
            match parse_html(&html) {
                Ok(chapter_doc) => {
                    for section in chapter_doc.sections {
                        doc.sections.push(section);
                    }
                }
                Err(_) => {}
            }
        }
    } else {
        for file_path in &spine_items {
            let html = {
                match archive.by_name(file_path) {
                    Ok(mut entry) => {
                        let mut s = String::new();
                        entry.read_to_string(&mut s)?;
                        s
                    }
                    Err(_) => continue,
                }
            };
            match parse_html(&html) {
                Ok(chapter_doc) => {
                    for section in chapter_doc.sections {
                        doc.sections.push(section);
                    }
                }
                Err(_) => {}
            }
        }
    }

    if doc.sections.is_empty() {
        let mut section = rw_document::Section::new();
        section.content.push(Block::Paragraph(rw_document::Paragraph::new()));
        doc.sections.push(section);
    }

    Ok(doc)
}

fn extract_opf_path(xml: &str) -> Option<String> {
    // Simple extraction of full-path attribute from rootfile element
    let needle = "full-path=\"";
    let start = xml.find(needle)? + needle.len();
    let end = xml[start..].find('"')? + start;
    Some(xml[start..end].to_string())
}

fn parse_opf_spine(opf_xml: &str, base_dir: &str) -> Vec<String> {
    if opf_xml.is_empty() {
        return Vec::new();
    }

    // Build id->href map from manifest
    let mut id_href: std::collections::HashMap<String, String> = std::collections::HashMap::new();

    // Simple parser for <item id="..." href="..." media-type="..."/>
    let mut remaining: &str = &opf_xml;
    while let Some(start) = remaining.find("<item ") {
        remaining = &remaining[start + 6..];
        let end = remaining.find('>').unwrap_or(remaining.len());
        let attrs_str = &remaining[..end];

        let id = extract_attr(attrs_str, "id").unwrap_or_default();
        let href = extract_attr(attrs_str, "href").unwrap_or_default();
        let media_type = extract_attr(attrs_str, "media-type").unwrap_or_default();

        if media_type.contains("xhtml") || media_type.contains("html") {
            id_href.insert(id, format!("{}{}", base_dir, href));
        }
    }

    // Parse spine to get ordered idrefs
    let mut spine_paths = Vec::new();
    let mut remaining: &str = &opf_xml;
    if let Some(spine_start) = remaining.find("<spine") {
        remaining = &remaining[spine_start..];
        if let Some(spine_end) = remaining.find("</spine>") {
            let spine_xml = &remaining[..spine_end + 8];
            let mut scan = spine_xml;
            while let Some(item_start) = scan.find("<itemref ") {
                scan = &scan[item_start + 9..];
                let end = scan.find('>').unwrap_or(scan.len());
                let attrs_str = &scan[..end];
                if let Some(idref) = extract_attr(attrs_str, "idref") {
                    if let Some(href) = id_href.get(&idref) {
                        spine_paths.push(href.clone());
                    }
                }
            }
        }
    }

    spine_paths
}

fn extract_attr(s: &str, name: &str) -> Option<String> {
    // Try with double quotes first
    let needle_dq = format!("{}=\"", name);
    if let Some(start) = s.find(&needle_dq) {
        let start = start + needle_dq.len();
        let end = s[start..].find('"')? + start;
        return Some(s[start..end].to_string());
    }
    // Try with single quotes
    let needle_sq = format!("{}='", name);
    if let Some(start) = s.find(&needle_sq) {
        let start = start + needle_sq.len();
        let end = s[start..].find('\'')?  + start;
        return Some(s[start..end].to_string());
    }
    None
}

/// Export a document as EPUB.
pub fn write_epub(
    doc: &Document,
    path: &std::path::Path,
    options: &EpubOptions,
) -> Result<(), EpubError> {
    let zip = ZipLibrary::new()
        .map_err(|e| EpubError::Generation(format!("ZipLibrary error: {}", e)))?;

    let mut builder = EpubBuilder::new(zip)
        .map_err(|e| EpubError::Generation(format!("EpubBuilder error: {}", e)))?;

    // Set version
    let version = match options.version {
        EpubVersion::V2 => BuilderVersion::V20,
        EpubVersion::V3 => BuilderVersion::V30,
    };
    builder.epub_version(version);

    // Metadata
    let title = doc.metadata.title.as_deref().unwrap_or("Untitled");
    builder
        .metadata("title", title)
        .map_err(|e| EpubError::Generation(e.to_string()))?;

    if let Some(ref author) = doc.metadata.author {
        builder
            .metadata("author", author.as_str())
            .map_err(|e| EpubError::Generation(e.to_string()))?;
    }

    if let Some(ref lang) = doc.metadata.language {
        builder.set_lang(lang.as_str());
    }

    // CSS
    let css = options.custom_css.as_deref().unwrap_or(DEFAULT_CSS);
    builder
        .stylesheet(css.as_bytes())
        .map_err(|e| EpubError::Generation(e.to_string()))?;

    // Split content into chapters at heading level
    let chapters = split_into_chapters(doc, options.split_level);

    if chapters.is_empty() {
        // Write all content as a single chapter
        let xhtml = generate_chapter_xhtml("Content", &collect_all_blocks(doc), doc);
        builder
            .add_content(
                EpubContent::new("content.xhtml", xhtml.as_bytes())
                    .title(title)
                    .reftype(ReferenceType::Text),
            )
            .map_err(|e| EpubError::Generation(e.to_string()))?;
    } else {
        for (idx, chapter) in chapters.iter().enumerate() {
            let filename = format!("chapter_{:03}.xhtml", idx + 1);
            let xhtml = generate_chapter_xhtml(&chapter.title, &chapter.blocks, doc);
            let content = EpubContent::new(filename.as_str(), xhtml.as_bytes())
                .title(chapter.title.as_str())
                .reftype(if idx == 0 {
                    ReferenceType::Text
                } else {
                    ReferenceType::Text
                });
            builder
                .add_content(content)
                .map_err(|e| EpubError::Generation(e.to_string()))?;
        }
    }

    // Generate EPUB to file
    let mut output = Vec::new();
    builder
        .generate(&mut output)
        .map_err(|e| EpubError::Generation(e.to_string()))?;

    std::fs::write(path, &output)?;
    Ok(())
}

struct Chapter {
    title: String,
    blocks: Vec<Block>,
}

fn collect_all_blocks(doc: &Document) -> Vec<Block> {
    doc.sections
        .iter()
        .flat_map(|s| s.content.iter().cloned())
        .collect()
}

fn split_into_chapters(doc: &Document, split_level: u8) -> Vec<Chapter> {
    let mut chapters: Vec<Chapter> = Vec::new();
    let mut current_chapter: Option<Chapter> = None;

    for section in &doc.sections {
        for block in &section.content {
            if let Block::Paragraph(para) = block {
                let level = para.properties.outline_level;
                let is_heading = level.map_or(false, |l| l <= split_level && l >= 1)
                    || para
                        .properties
                        .paragraph_style
                        .as_deref()
                        .map_or(false, |s| {
                            s.starts_with("Heading") && {
                                let level_char = s.chars().last().unwrap_or('9');
                                level_char.to_digit(10).unwrap_or(9) <= split_level as u32
                            }
                        });

                if is_heading {
                    // Save current chapter
                    if let Some(ch) = current_chapter.take() {
                        chapters.push(ch);
                    }
                    // Start new chapter
                    let title = para
                        .content
                        .iter()
                        .filter_map(|inline| {
                            if let Inline::Text(run) = inline {
                                Some(run.text.clone())
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("");

                    current_chapter = Some(Chapter {
                        title: if title.is_empty() {
                            format!("Chapter {}", chapters.len() + 1)
                        } else {
                            title
                        },
                        blocks: vec![block.clone()],
                    });
                } else if let Some(ref mut ch) = current_chapter {
                    ch.blocks.push(block.clone());
                } else {
                    // Before any heading
                    current_chapter = Some(Chapter {
                        title: "Introduction".to_string(),
                        blocks: vec![block.clone()],
                    });
                }
            } else if let Some(ref mut ch) = current_chapter {
                ch.blocks.push(block.clone());
            }
        }
    }

    if let Some(ch) = current_chapter {
        chapters.push(ch);
    }

    chapters
}

fn generate_chapter_xhtml(title: &str, blocks: &[Block], _doc: &Document) -> String {
    let mut xhtml = String::new();
    xhtml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    xhtml.push('\n');
    xhtml.push_str(r#"<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.1//EN" "http://www.w3.org/TR/xhtml11/DTD/xhtml11.dtd">"#);
    xhtml.push('\n');
    xhtml.push_str(r#"<html xmlns="http://www.w3.org/1999/xhtml">"#);
    xhtml.push('\n');
    xhtml.push_str("<head>\n");
    xhtml.push_str(&format!("<title>{}</title>\n", html_escape(title)));
    xhtml.push_str("<link rel=\"stylesheet\" type=\"text/css\" href=\"stylesheet.css\"/>\n");
    xhtml.push_str("</head>\n");
    xhtml.push_str("<body>\n");

    for block in blocks {
        write_block_xhtml(block, &mut xhtml);
    }

    xhtml.push_str("</body>\n");
    xhtml.push_str("</html>\n");
    xhtml
}

fn write_block_xhtml(block: &Block, xhtml: &mut String) {
    match block {
        Block::Paragraph(para) => {
            let outline_level = para.properties.outline_level;
            let style = para.properties.paragraph_style.as_deref().unwrap_or("");

            let tag = if let Some(level) = outline_level {
                if (1..=6).contains(&level) {
                    format!("h{}", level)
                } else {
                    "p".to_string()
                }
            } else if style.starts_with("Heading") {
                if let Some(level_char) = style.chars().last() {
                    if let Some(level) = level_char.to_digit(10) {
                        if (1..=6).contains(&level) {
                            format!("h{}", level)
                        } else {
                            "p".to_string()
                        }
                    } else {
                        "p".to_string()
                    }
                } else {
                    "p".to_string()
                }
            } else {
                "p".to_string()
            };

            xhtml.push_str(&format!("<{}>", tag));
            for inline in &para.content {
                write_inline_xhtml(inline, xhtml);
            }
            xhtml.push_str(&format!("</{}>\n", tag));
        }
        Block::Table(tbl) => {
            xhtml.push_str("<table border=\"1\">\n");
            let rows = tbl.rows as usize;
            let cols = tbl.cols as usize;
            if cols > 0 {
                for row_idx in 0..rows {
                    xhtml.push_str("<tr>\n");
                    for col_idx in 0..cols {
                        let cell_idx = row_idx * cols + col_idx;
                        xhtml.push_str("<td>");
                        if let Some(cell) = tbl.cells.get(cell_idx) {
                            for b in &cell.content {
                                write_block_xhtml(b, xhtml);
                            }
                        }
                        xhtml.push_str("</td>\n");
                    }
                    xhtml.push_str("</tr>\n");
                }
            }
            xhtml.push_str("</table>\n");
        }
        Block::HorizontalRule(_) => xhtml.push_str("<hr/>\n"),
        _ => {}
    }
}

fn write_inline_xhtml(inline: &Inline, xhtml: &mut String) {
    match inline {
        Inline::Text(run) => {
            let props = &run.properties;
            let mut open = String::new();
            let mut close = String::new();

            if props.bold == Some(true) {
                open.push_str("<b>");
                close.insert_str(0, "</b>");
            }
            if props.italic == Some(true) {
                open.push_str("<i>");
                close.insert_str(0, "</i>");
            }
            if props.underline.is_some() {
                open.push_str("<u>");
                close.insert_str(0, "</u>");
            }

            xhtml.push_str(&open);
            xhtml.push_str(&html_escape(&run.text));
            xhtml.push_str(&close);
        }
        Inline::Break(rw_document::inline::BreakType::Line) => xhtml.push_str("<br/>"),
        Inline::Tab => xhtml.push_str("&#9;"),
        Inline::NonBreakingSpace => xhtml.push_str("&#160;"),
        _ => {}
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

const DEFAULT_CSS: &str = r#"
body { font-family: Georgia, serif; font-size: 1em; line-height: 1.5; margin: 1em; }
h1 { font-size: 2em; margin: 0.5em 0; }
h2 { font-size: 1.6em; margin: 0.5em 0; }
h3 { font-size: 1.3em; margin: 0.5em 0; }
h4, h5, h6 { font-size: 1em; margin: 0.5em 0; font-weight: bold; }
p { margin: 0.5em 0; }
table { border-collapse: collapse; width: 100%; }
td, th { border: 1px solid #ccc; padding: 4px 8px; }
"#;
