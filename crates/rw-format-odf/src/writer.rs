//! ODF document writer.
//!
//! Serializes a Document model into an .odt ZIP archive.

use crate::OdfError;
use rw_document::{
    Block, Document, Inline,
    block::TableBlock,
    inline::BreakType,
};
use std::io::{Write, Seek};
use zip::{ZipWriter, write::SimpleFileOptions};

/// MIME type for ODF text documents.
pub const ODT_MIME_TYPE: &str = "application/vnd.oasis.opendocument.text";

/// Write a Document to an ODF .odt file.
pub fn write_odf(doc: &Document, path: &std::path::Path) -> Result<(), OdfError> {
    let file = std::fs::File::create(path)?;
    let buf_writer = std::io::BufWriter::new(file);
    write_odf_to_writer(doc, buf_writer)
}

/// Write a Document to any Write + Seek writer as an ODF archive.
pub fn write_odf_to_writer<W: Write + Seek>(
    doc: &Document,
    writer: W,
) -> Result<(), OdfError> {
    let mut zip = ZipWriter::new(writer);

    // mimetype MUST be first and uncompressed
    let stored = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Stored);
    let deflated = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    // mimetype (uncompressed, no extra data)
    zip.start_file("mimetype", stored)?;
    zip.write_all(ODT_MIME_TYPE.as_bytes())?;

    // META-INF/manifest.xml
    zip.start_file("META-INF/manifest.xml", deflated)?;
    zip.write_all(MANIFEST_XML.as_bytes())?;

    // meta.xml
    zip.start_file("meta.xml", deflated)?;
    let meta_xml = generate_meta_xml(doc);
    zip.write_all(meta_xml.as_bytes())?;

    // styles.xml
    zip.start_file("styles.xml", deflated)?;
    zip.write_all(STYLES_XML.as_bytes())?;

    // content.xml
    zip.start_file("content.xml", deflated)?;
    let content_xml = generate_content_xml(doc);
    zip.write_all(content_xml.as_bytes())?;

    // settings.xml
    zip.start_file("settings.xml", deflated)?;
    zip.write_all(SETTINGS_XML.as_bytes())?;

    zip.finish()?;
    Ok(())
}

fn generate_meta_xml(doc: &Document) -> String {
    let mut xml = String::new();
    xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    xml.push('\n');
    xml.push_str(r#"<office:document-meta xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0" xmlns:meta="urn:oasis:names:tc:opendocument:xmlns:meta:1.0" xmlns:dc="http://purl.org/dc/elements/1.1/" office:version="1.3">"#);
    xml.push('\n');
    xml.push_str("<office:meta>\n");

    let meta = &doc.metadata;
    if let Some(ref title) = meta.title {
        xml.push_str(&format!("<dc:title>{}</dc:title>\n", xml_escape(title)));
    }
    if let Some(ref creator) = meta.author {
        xml.push_str(&format!("<dc:creator>{}</dc:creator>\n", xml_escape(creator)));
        xml.push_str(&format!(
            "<meta:initial-creator>{}</meta:initial-creator>\n",
            xml_escape(creator)
        ));
    }
    if let Some(ref subject) = meta.subject {
        xml.push_str(&format!("<dc:subject>{}</dc:subject>\n", xml_escape(subject)));
    }
    xml.push_str(&format!(
        "<meta:creation-date>{}</meta:creation-date>\n",
        meta.created.format("%Y-%m-%dT%H:%M:%S")
    ));

    xml.push_str("</office:meta>\n");
    xml.push_str("</office:document-meta>\n");
    xml
}

fn generate_content_xml(doc: &Document) -> String {
    let mut xml = String::new();
    xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    xml.push('\n');
    xml.push_str(r#"<office:document-content xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0" xmlns:style="urn:oasis:names:tc:opendocument:xmlns:style:1.0" xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0" xmlns:table="urn:oasis:names:tc:opendocument:xmlns:table:1.0" xmlns:draw="urn:oasis:names:tc:opendocument:xmlns:drawing:1.0" xmlns:fo="urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:meta="urn:oasis:names:tc:opendocument:xmlns:meta:1.0" xmlns:number="urn:oasis:names:tc:opendocument:xmlns:datastyle:1.0" xmlns:svg="urn:oasis:names:tc:opendocument:xmlns:svg-compatible:1.0" xmlns:chart="urn:oasis:names:tc:opendocument:xmlns:chart:1.0" xmlns:dr3d="urn:oasis:names:tc:opendocument:xmlns:dr3d:1.0" xmlns:math="http://www.w3.org/1998/Math/MathML" xmlns:form="urn:oasis:names:tc:opendocument:xmlns:form:1.0" xmlns:script="urn:oasis:names:tc:opendocument:xmlns:script:1.0" xmlns:ooo="http://openoffice.org/2004/office" xmlns:ooow="http://openoffice.org/2004/writer" xmlns:oooc="http://openoffice.org/2004/calc" xmlns:dom="http://www.w3.org/2001/xml-events" xmlns:xforms="http://www.w3.org/2002/xforms" xmlns:xsd="http://www.w3.org/2001/XMLSchema" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:rpt="http://openoffice.org/2005/report" xmlns:of="urn:oasis:names:tc:opendocument:xmlns:of:1.2" xmlns:xhtml="http://www.w3.org/1999/xhtml" xmlns:grddl="http://www.w3.org/2003/g/data-view#" xmlns:tableooo="http://openoffice.org/2009/table" xmlns:field="urn:openoffice:names:experimental:ooo-ms-interop:xmlns:field:1.0" xmlns:formx="urn:openoffice:names:experimental:ooxml-odf-interop:xmlns:form:1.0" xmlns:css3t="http://www.w3.org/TR/css3-text/" office:version="1.3">"#);
    xml.push('\n');

    // Automatic styles
    xml.push_str("<office:automatic-styles>\n");
    collect_automatic_styles(doc, &mut xml);
    xml.push_str("</office:automatic-styles>\n");

    // Body
    xml.push_str("<office:body>\n");
    xml.push_str("<office:text>\n");

    for section in &doc.sections {
        for block in &section.content {
            write_block_odf(block, &mut xml);
        }
    }

    xml.push_str("</office:text>\n");
    xml.push_str("</office:body>\n");
    xml.push_str("</office:document-content>\n");
    xml
}

fn collect_automatic_styles(_doc: &Document, xml: &mut String) {
    // We generate inline styles for bold/italic/etc.
    // For simplicity, generate named styles for each combination encountered.
    // In a real implementation this would deduplicate styles.
    // Here we emit a few common styles.
    xml.push_str(r#"<style:style style:name="bold" style:family="text"><style:text-properties fo:font-weight="bold"/></style:style>"#);
    xml.push('\n');
    xml.push_str(r#"<style:style style:name="italic" style:family="text"><style:text-properties fo:font-style="italic"/></style:style>"#);
    xml.push('\n');
    xml.push_str(r#"<style:style style:name="bolditalic" style:family="text"><style:text-properties fo:font-weight="bold" fo:font-style="italic"/></style:style>"#);
    xml.push('\n');
    xml.push_str(r#"<style:style style:name="underline" style:family="text"><style:text-properties style:text-underline-style="solid" style:text-underline-width="auto" style:text-underline-color="font-color"/></style:style>"#);
    xml.push('\n');
}

fn write_block_odf(block: &Block, xml: &mut String) {
    match block {
        Block::Paragraph(para) => {
            let _style = para.properties.paragraph_style.as_deref().unwrap_or("Text_20_Body");
            let outline_level = para.properties.outline_level;

            if let Some(level) = outline_level {
                if (1..=6).contains(&level) {
                    xml.push_str(&format!(
                        "<text:h text:outline-level=\"{}\" text:style-name=\"Heading_20_{}\">",
                        level, level
                    ));
                    for inline in &para.content {
                        write_inline_odf(inline, xml);
                    }
                    xml.push_str("</text:h>\n");
                    return;
                }
            }

            // Build paragraph style attrs
            let mut para_style = "Text_20_Body".to_string();
            if let Some(ref s) = para.properties.paragraph_style {
                if s.starts_with("Heading") {
                    if let Some(level_char) = s.chars().last() {
                        if let Some(level) = level_char.to_digit(10) {
                            xml.push_str(&format!(
                                "<text:h text:outline-level=\"{}\" text:style-name=\"Heading_20_{}\">",
                                level, level
                            ));
                            for inline in &para.content {
                                write_inline_odf(inline, xml);
                            }
                            xml.push_str("</text:h>\n");
                            return;
                        }
                    }
                }
                para_style = s.replace(' ', "_20_");
            }

            xml.push_str(&format!(
                "<text:p text:style-name=\"{}\">",
                xml_escape(&para_style)
            ));

            for inline in &para.content {
                write_inline_odf(inline, xml);
            }

            xml.push_str("</text:p>\n");
        }
        Block::Table(tbl) => {
            write_table_odf(tbl, xml);
        }
        Block::HorizontalRule(_) => {
            // Empty paragraph as separator
            xml.push_str("<text:p text:style-name=\"Horizontal_20_Line\"/>\n");
        }
        Block::SubSection(sub) => {
            for block in &sub.content {
                write_block_odf(block, xml);
            }
        }
        _ => {}
    }
}

fn write_inline_odf(inline: &Inline, xml: &mut String) {
    match inline {
        Inline::Text(run) => {
            let props = &run.properties;
            let is_bold = props.bold == Some(true);
            let is_italic = props.italic == Some(true);
            let has_underline = props.underline.is_some();
            let has_color = props.color.is_some();
            let has_size = props.font_size.is_some();
            let has_font = props.font_family.is_some();

            if is_bold || is_italic || has_underline || has_color || has_size || has_font {
                // Build inline style
                let mut style_parts = Vec::new();
                if is_bold {
                    style_parts.push("fo:font-weight=\"bold\"".to_string());
                }
                if is_italic {
                    style_parts.push("fo:font-style=\"italic\"".to_string());
                }
                if has_underline {
                    style_parts.push("style:text-underline-style=\"solid\"".to_string());
                    style_parts.push("style:text-underline-color=\"font-color\"".to_string());
                }
                if let Some(ref color) = props.color {
                    style_parts.push(format!(
                        "fo:color=\"#{:02X}{:02X}{:02X}\"",
                        color.r, color.g, color.b
                    ));
                }
                if let Some(size) = props.font_size {
                    let pt = size as f64 / 2.0;
                    style_parts.push(format!("fo:font-size=\"{}pt\"", pt));
                }
                if let Some(ref font) = props.font_family {
                    style_parts.push(format!("fo:font-family=\"{}\"", xml_escape(font)));
                }

                // We use a direct style:style element approach with text:span
                // For simplicity, output styling as inline span
                let style_name = format!("rw_inline_{}", compute_style_hash(props));
                xml.push_str(&format!(
                    "<text:span text:style-name=\"{}\">{}</text:span>",
                    xml_escape(&style_name),
                    xml_escape(&run.text)
                ));
            } else {
                xml.push_str(&xml_escape(&run.text));
            }
        }
        Inline::Break(break_type) => match break_type {
            BreakType::Line => xml.push_str("<text:line-break/>"),
            BreakType::Page => {
                xml.push_str("</text:p><text:p text:style-name=\"Text_20_Body\">");
            }
            BreakType::Column => xml.push_str("<text:line-break/>"),
        },
        Inline::Tab => xml.push_str("<text:tab/>"),
        Inline::NonBreakingSpace => xml.push_str("<text:s/>"),
        Inline::Hyperlink(link) => {
            let target = match &link.target {
                rw_document::inline::HyperlinkTarget::Url(url) => url.clone(),
                rw_document::inline::HyperlinkTarget::Bookmark(bm) => format!("#{}", bm),
                rw_document::inline::HyperlinkTarget::Email { address, .. } => {
                    format!("mailto:{}", address)
                }
            };
            xml.push_str(&format!(
                "<text:a xlink:type=\"simple\" xlink:href=\"{}\">",
                xml_escape(&target)
            ));
            for inner in &link.content {
                write_inline_odf(inner, xml);
            }
            xml.push_str("</text:a>");
        }
        _ => {}
    }
}

fn compute_style_hash(props: &rw_document::properties::CharacterProperties) -> u64 {
    use std::hash::{Hash, Hasher};
    use std::collections::hash_map::DefaultHasher;
    let mut hasher = DefaultHasher::new();
    props.bold.hash(&mut hasher);
    props.italic.hash(&mut hasher);
    // Hash underline as discriminant only (variant index)
    let underline_discriminant: u8 = match props.underline {
        None => 0,
        Some(rw_document::properties::UnderlineStyle::Single) => 1,
        Some(rw_document::properties::UnderlineStyle::Double) => 2,
        Some(rw_document::properties::UnderlineStyle::Dotted) => 3,
        Some(rw_document::properties::UnderlineStyle::Dashed) => 4,
        Some(rw_document::properties::UnderlineStyle::DashDot) => 5,
        Some(rw_document::properties::UnderlineStyle::DashDotDot) => 6,
        Some(rw_document::properties::UnderlineStyle::Wave) => 7,
        Some(rw_document::properties::UnderlineStyle::Thick) => 8,
        Some(rw_document::properties::UnderlineStyle::Words) => 9,
    };
    underline_discriminant.hash(&mut hasher);
    if let Some(ref color) = props.color {
        color.r.hash(&mut hasher);
        color.g.hash(&mut hasher);
        color.b.hash(&mut hasher);
    }
    props.font_size.hash(&mut hasher);
    if let Some(ref font) = props.font_family {
        font.hash(&mut hasher);
    }
    hasher.finish()
}

fn write_table_odf(tbl: &TableBlock, xml: &mut String) {
    xml.push_str("<table:table>\n");

    let cols = tbl.cols as usize;
    for i in 0..cols {
        xml.push_str(&format!(
            "<table:table-column table:style-name=\"col{}\"/>\n", i
        ));
    }

    let rows = tbl.rows as usize;
    for row_idx in 0..rows {
        xml.push_str("<table:table-row>\n");
        for col_idx in 0..cols {
            let cell_idx = row_idx * cols + col_idx;
            xml.push_str("<table:table-cell>\n");
            if let Some(cell) = tbl.cells.get(cell_idx) {
                for block in &cell.content {
                    write_block_odf(block, xml);
                }
            } else {
                xml.push_str("<text:p/>\n");
            }
            xml.push_str("</table:table-cell>\n");
        }
        xml.push_str("</table:table-row>\n");
    }

    xml.push_str("</table:table>\n");
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

const MANIFEST_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<manifest:manifest xmlns:manifest="urn:oasis:names:tc:opendocument:xmlns:manifest:1.0" manifest:version="1.3">
  <manifest:file-entry manifest:full-path="/" manifest:version="1.3" manifest:media-type="application/vnd.oasis.opendocument.text"/>
  <manifest:file-entry manifest:full-path="content.xml" manifest:media-type="text/xml"/>
  <manifest:file-entry manifest:full-path="styles.xml" manifest:media-type="text/xml"/>
  <manifest:file-entry manifest:full-path="meta.xml" manifest:media-type="text/xml"/>
  <manifest:file-entry manifest:full-path="settings.xml" manifest:media-type="text/xml"/>
</manifest:manifest>
"#;

const STYLES_XML: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<office:document-styles xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0" xmlns:style="urn:oasis:names:tc:opendocument:xmlns:style:1.0" xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0" xmlns:fo="urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0" office:version="1.3">
<office:styles>
  <style:style style:name="Default_20_Paragraph_20_Style" style:family="paragraph" style:class="text">
    <style:paragraph-properties fo:margin-top="0cm" fo:margin-bottom="0.212cm"/>
    <style:text-properties style:font-name="Liberation Serif" fo:font-size="12pt" fo:color="#000000"/>
  </style:style>
  <style:style style:name="Text_20_Body" style:display-name="Text Body" style:family="paragraph" style:parent-style-name="Default_20_Paragraph_20_Style" style:class="text"/>
  <style:style style:name="Heading_20_1" style:display-name="Heading 1" style:family="paragraph" style:class="text">
    <style:text-properties fo:font-size="24pt" fo:font-weight="bold"/>
  </style:style>
  <style:style style:name="Heading_20_2" style:display-name="Heading 2" style:family="paragraph" style:class="text">
    <style:text-properties fo:font-size="18pt" fo:font-weight="bold"/>
  </style:style>
  <style:style style:name="Heading_20_3" style:display-name="Heading 3" style:family="paragraph" style:class="text">
    <style:text-properties fo:font-size="14pt" fo:font-weight="bold"/>
  </style:style>
  <style:style style:name="Heading_20_4" style:display-name="Heading 4" style:family="paragraph" style:class="text">
    <style:text-properties fo:font-size="12pt" fo:font-weight="bold"/>
  </style:style>
</office:styles>
</office:document-styles>
"##;

const SETTINGS_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-settings xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0" office:version="1.3">
<office:settings/>
</office:document-settings>
"#;
