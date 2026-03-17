//! OOXML document writer.
//!
//! Serializes a Document model into a .docx ZIP archive.
//! Generates valid WordprocessingML XML conforming to ECMA-376.

use crate::OoxmlError;
use rw_document::{
    Block, Document, Inline,
    block::TableBlock,
    inline::BreakType,
    properties::{Alignment, UnderlineStyle},
};
use std::io::{Cursor, Write, Seek};
use zip::{ZipWriter, write::SimpleFileOptions};

/// Write a document to a .docx ZIP archive using the provided writer.
pub fn write_docx_to_writer<W: Write + Seek>(
    doc: &Document,
    writer: W,
) -> Result<(), OoxmlError> {
    let mut zip = ZipWriter::new(writer);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    // [Content_Types].xml
    zip.start_file("[Content_Types].xml", options)?;
    zip.write_all(CONTENT_TYPES_XML.as_bytes())?;

    // _rels/.rels
    zip.start_file("_rels/.rels", options)?;
    zip.write_all(RELS_XML.as_bytes())?;

    // word/_rels/document.xml.rels
    zip.start_file("word/_rels/document.xml.rels", options)?;
    zip.write_all(DOCUMENT_RELS_XML.as_bytes())?;

    // word/document.xml
    zip.start_file("word/document.xml", options)?;
    let doc_xml = generate_document_xml(doc);
    zip.write_all(doc_xml.as_bytes())?;

    // word/styles.xml
    zip.start_file("word/styles.xml", options)?;
    zip.write_all(STYLES_XML.as_bytes())?;

    // docProps/core.xml
    zip.start_file("docProps/core.xml", options)?;
    let core_xml = generate_core_props_xml(doc);
    zip.write_all(core_xml.as_bytes())?;

    // Write embedded images
    for (data_id, resource) in &doc.resources.resources {
        let filename = resource
            .filename
            .as_deref()
            .unwrap_or("image.bin");
        let path = format!("word/media/{}", filename);
        zip.start_file(&path, options)?;
        zip.write_all(&resource.data)?;
    }

    zip.finish()?;
    Ok(())
}

fn generate_document_xml(doc: &Document) -> String {
    let mut xml = String::new();
    xml.push_str(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#);
    xml.push('\n');
    xml.push_str(r#"<w:document xmlns:wpc="http://schemas.microsoft.com/office/word/2010/wordprocessingCanvas" xmlns:cx="http://schemas.microsoft.com/office/drawing/2014/chartex" xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006" xmlns:aink="http://schemas.microsoft.com/office/drawing/2016/ink" xmlns:am3d="http://schemas.microsoft.com/office/drawing/2017/model3d" xmlns:o="urn:schemas-microsoft-com:office:office" xmlns:oel="http://schemas.microsoft.com/office/2019/extlst" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math" xmlns:v="urn:schemas-microsoft-com:vml" xmlns:wp14="http://schemas.microsoft.com/office/word/2010/wordprocessingDrawing" xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" xmlns:w10="urn:schemas-microsoft-com:office:word" xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml" xmlns:w15="http://schemas.microsoft.com/office/word/2012/wordml" xmlns:w16cex="http://schemas.microsoft.com/office/word/2018/wordml/cex" xmlns:w16cid="http://schemas.microsoft.com/office/word/2016/wordml/cid" xmlns:w16="http://schemas.microsoft.com/office/word/2018/wordml" xmlns:w16sdtdh="http://schemas.microsoft.com/office/word/2020/wordml/sdtdatahash" xmlns:w16se="http://schemas.microsoft.com/office/word/2015/wordml/symex" xmlns:wpg="http://schemas.microsoft.com/office/word/2010/wordprocessingGroup" xmlns:wpi="http://schemas.microsoft.com/office/word/2010/wordprocessingInk" xmlns:wne="http://schemas.microsoft.com/office/word/2006/wordml" xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape" mc:Ignorable="w14 w15 w16se w16cid w16 w16cex w16sdtdh wp14">"#);
    xml.push('\n');
    xml.push_str("<w:body>\n");

    for section in &doc.sections {
        for block in &section.content {
            write_block_xml(block, &mut xml);
        }
    }

    // Section properties (required at end of body)
    xml.push_str("<w:sectPr>\n");
    xml.push_str("<w:pgSz w:w=\"12240\" w:h=\"15840\"/>\n");
    xml.push_str("<w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\" w:header=\"720\" w:footer=\"720\" w:gutter=\"0\"/>\n");
    xml.push_str("</w:sectPr>\n");

    xml.push_str("</w:body>\n");
    xml.push_str("</w:document>\n");
    xml
}

fn write_block_xml(block: &Block, xml: &mut String) {
    match block {
        Block::Paragraph(para) => {
            xml.push_str("<w:p>\n");

            // Paragraph properties
            let props = &para.properties;
            let has_props = props.paragraph_style.is_some()
                || props.alignment.is_some();
            if has_props {
                xml.push_str("<w:pPr>\n");
                if let Some(ref style) = props.paragraph_style {
                    xml.push_str(&format!(
                        "<w:pStyle w:val=\"{}\"/>\n",
                        xml_escape(style)
                    ));
                }
                if let Some(align) = props.alignment {
                    let val = match align {
                        Alignment::Left => "left",
                        Alignment::Center => "center",
                        Alignment::Right => "right",
                        Alignment::Justify => "both",
                        Alignment::Distribute => "distribute",
                    };
                    xml.push_str(&format!("<w:jc w:val=\"{}\"/>\n", val));
                }
                xml.push_str("</w:pPr>\n");
            }

            // Inline content
            for inline in &para.content {
                write_inline_xml(inline, xml);
            }

            xml.push_str("</w:p>\n");
        }
        Block::Table(tbl) => {
            write_table_xml(tbl, xml);
        }
        Block::HorizontalRule(_) => {
            // Write as a paragraph with a bottom border
            xml.push_str("<w:p><w:pPr><w:pBdr><w:bottom w:val=\"single\" w:sz=\"6\" w:space=\"1\" w:color=\"auto\"/></w:pBdr></w:pPr></w:p>\n");
        }
        _ => {}
    }
}

fn write_inline_xml(inline: &Inline, xml: &mut String) {
    match inline {
        Inline::Text(run) => {
            xml.push_str("<w:r>\n");

            // Run properties
            let props = &run.properties;
            let has_rpr = props.bold.is_some()
                || props.italic.is_some()
                || props.underline.is_some()
                || props.color.is_some()
                || props.font_size.is_some()
                || props.font_family.is_some();

            if has_rpr {
                xml.push_str("<w:rPr>\n");
                if props.bold == Some(true) {
                    xml.push_str("<w:b/>\n");
                }
                if props.italic == Some(true) {
                    xml.push_str("<w:i/>\n");
                }
                if let Some(underline) = props.underline {
                    let val = match underline {
                        UnderlineStyle::Single => "single",
                        UnderlineStyle::Double => "double",
                        UnderlineStyle::Dotted => "dotted",
                        UnderlineStyle::Dashed => "dash",
                        UnderlineStyle::Wave => "wave",
                        _ => "single",
                    };
                    xml.push_str(&format!("<w:u w:val=\"{}\"/>\n", val));
                }
                if let Some(ref color) = props.color {
                    xml.push_str(&format!(
                        "<w:color w:val=\"{:02X}{:02X}{:02X}\"/>\n",
                        color.r, color.g, color.b
                    ));
                }
                if let Some(size) = props.font_size {
                    xml.push_str(&format!("<w:sz w:val=\"{}\"/>\n", size));
                }
                if let Some(ref font) = props.font_family {
                    xml.push_str(&format!(
                        "<w:rFonts w:ascii=\"{}\" w:hAnsi=\"{}\"/>\n",
                        xml_escape(font),
                        xml_escape(font)
                    ));
                }
                xml.push_str("</w:rPr>\n");
            }

            // Text content - use preserve space if needed
            let text = &run.text;
            let needs_preserve = text.starts_with(' ') || text.ends_with(' ');
            if needs_preserve {
                xml.push_str(&format!(
                    "<w:t xml:space=\"preserve\">{}</w:t>\n",
                    xml_escape(text)
                ));
            } else {
                xml.push_str(&format!("<w:t>{}</w:t>\n", xml_escape(text)));
            }

            xml.push_str("</w:r>\n");
        }
        Inline::Break(break_type) => {
            xml.push_str("<w:r><w:br");
            match break_type {
                BreakType::Page => xml.push_str(" w:type=\"page\""),
                BreakType::Column => xml.push_str(" w:type=\"column\""),
                BreakType::Line => {} // default line break
            }
            xml.push_str("/></w:r>\n");
        }
        Inline::Tab => {
            xml.push_str("<w:r><w:tab/></w:r>\n");
        }
        Inline::Hyperlink(link) => {
            for inner in &link.content {
                write_inline_xml(inner, xml);
            }
        }
        _ => {}
    }
}

fn write_table_xml(tbl: &TableBlock, xml: &mut String) {
    xml.push_str("<w:tbl>\n");
    xml.push_str("<w:tblPr><w:tblStyle w:val=\"TableGrid\"/></w:tblPr>\n");

    let cols = tbl.cols as usize;
    if cols > 0 {
        xml.push_str("<w:tblGrid>\n");
        for _ in 0..cols {
            xml.push_str("<w:gridCol w:w=\"1440\"/>\n");
        }
        xml.push_str("</w:tblGrid>\n");
    }

    let rows = tbl.rows as usize;
    let cols = tbl.cols as usize;
    for row_idx in 0..rows {
        xml.push_str("<w:tr>\n");
        for col_idx in 0..cols {
            let cell_idx = row_idx * cols + col_idx;
            if let Some(cell) = tbl.cells.get(cell_idx) {
                xml.push_str("<w:tc>\n");
                xml.push_str("<w:tcPr><w:tcW w:w=\"1440\" w:type=\"dxa\"/></w:tcPr>\n");
                for block in &cell.content {
                    write_block_xml(block, xml);
                }
                if cell.content.is_empty() {
                    xml.push_str("<w:p/>\n");
                }
                xml.push_str("</w:tc>\n");
            }
        }
        xml.push_str("</w:tr>\n");
    }

    xml.push_str("</w:tbl>\n");
}

fn generate_core_props_xml(doc: &Document) -> String {
    let mut xml = String::new();
    xml.push_str(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#);
    xml.push('\n');
    xml.push_str(r#"<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:dcterms="http://purl.org/dc/terms/" xmlns:dcmitype="http://purl.org/dc/dcmitype/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">"#);
    xml.push('\n');

    let meta = &doc.metadata;
    if let Some(ref title) = meta.title {
        xml.push_str(&format!("<dc:title>{}</dc:title>\n", xml_escape(title)));
    }
    if let Some(ref author) = meta.author {
        xml.push_str(&format!("<dc:creator>{}</dc:creator>\n", xml_escape(author)));
    }
    if let Some(ref subject) = meta.subject {
        xml.push_str(&format!("<dc:subject>{}</dc:subject>\n", xml_escape(subject)));
    }
    if let Some(ref description) = meta.description {
        xml.push_str(&format!(
            "<dc:description>{}</dc:description>\n",
            xml_escape(description)
        ));
    }
    xml.push_str(&format!(
        "<dcterms:created xsi:type=\"dcterms:W3CDTF\">{}</dcterms:created>\n",
        meta.created.format("%Y-%m-%dT%H:%M:%SZ")
    ));
    xml.push_str(&format!(
        "<dcterms:modified xsi:type=\"dcterms:W3CDTF\">{}</dcterms:modified>\n",
        meta.modified.format("%Y-%m-%dT%H:%M:%SZ")
    ));

    xml.push_str("</cp:coreProperties>\n");
    xml
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

const CONTENT_TYPES_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
  <Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/>
  <Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/>
</Types>
"#;

const RELS_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/>
</Relationships>
"#;

const DOCUMENT_RELS_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
</Relationships>
"#;

const STYLES_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:style w:type="paragraph" w:styleId="Normal" w:default="1">
    <w:name w:val="Normal"/>
    <w:pPr><w:spacing w:after="160" w:line="259" w:lineRule="auto"/></w:pPr>
    <w:rPr><w:sz w:val="24"/></w:rPr>
  </w:style>
  <w:style w:type="paragraph" w:styleId="Heading1">
    <w:name w:val="heading 1"/>
    <w:basedOn w:val="Normal"/>
    <w:rPr><w:b/><w:sz w:val="48"/></w:rPr>
  </w:style>
  <w:style w:type="paragraph" w:styleId="Heading2">
    <w:name w:val="heading 2"/>
    <w:basedOn w:val="Normal"/>
    <w:rPr><w:b/><w:sz w:val="36"/></w:rPr>
  </w:style>
  <w:style w:type="paragraph" w:styleId="Heading3">
    <w:name w:val="heading 3"/>
    <w:basedOn w:val="Normal"/>
    <w:rPr><w:b/><w:sz w:val="28"/></w:rPr>
  </w:style>
  <w:style w:type="table" w:styleId="TableGrid">
    <w:name w:val="Table Grid"/>
  </w:style>
</w:styles>
"#;
