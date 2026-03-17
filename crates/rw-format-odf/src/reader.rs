//! ODF document reader.
//!
//! Parses an .odt ZIP archive and constructs a Document model.

use crate::OdfError;
use quick_xml::events::Event;
use quick_xml::Reader;
use rw_document::{
    Block, Color, Document, Inline, Paragraph, Section, TextRun,
    block::{TableBlock, TableCell, TableCellProperties, TableProperties},
    inline::BreakType,
    metadata::Metadata,
    properties::{Alignment, CharacterProperties, ParagraphProperties, UnderlineStyle},
    ElementId,
};
use std::collections::HashMap;
use std::io::{Read, Seek};
use zip::ZipArchive;

/// ODF XML namespace constants.
pub mod ns {
    pub const OFFICE: &str = "urn:oasis:names:tc:opendocument:xmlns:office:1.0";
    pub const TEXT: &str = "urn:oasis:names:tc:opendocument:xmlns:text:1.0";
    pub const STYLE: &str = "urn:oasis:names:tc:opendocument:xmlns:style:1.0";
    pub const TABLE: &str = "urn:oasis:names:tc:opendocument:xmlns:table:1.0";
    pub const DRAW: &str = "urn:oasis:names:tc:opendocument:xmlns:drawing:1.0";
    pub const FO: &str = "urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0";
    pub const META: &str = "urn:oasis:names:tc:opendocument:xmlns:meta:1.0";
    pub const SVG: &str = "urn:oasis:names:tc:opendocument:xmlns:svg-compatible:1.0";
    pub const XLINK: &str = "http://www.w3.org/1999/xlink";
    pub const DC: &str = "http://purl.org/dc/elements/1.1/";
}

/// Read an ODF .odt file from the given path.
pub fn read_odf(path: &std::path::Path) -> Result<Document, OdfError> {
    let file = std::fs::File::open(path)?;
    let buf_reader = std::io::BufReader::new(file);
    read_odf_from_reader(buf_reader)
}

/// Read an ODF document from any Read + Seek reader.
pub fn read_odf_from_reader<R: Read + Seek>(reader: R) -> Result<Document, OdfError> {
    let mut archive = ZipArchive::new(reader)?;
    let mut doc = Document::new();
    doc.sections.clear();

    // Parse metadata from meta.xml
    if let Ok(meta) = parse_meta(&mut archive) {
        doc.metadata = meta;
    }

    // Parse main content from content.xml
    let content_xml = read_zip_entry_to_string(&mut archive, "content.xml")?;

    // Parse automatic styles
    let styles = parse_auto_styles(&content_xml);

    // Parse body content
    let mut section = Section::new();
    parse_content_xml(&content_xml, &styles, &mut section.content)?;

    if section.content.is_empty() {
        section.content.push(Block::Paragraph(Paragraph::new()));
    }
    doc.sections.push(section);

    Ok(doc)
}

fn read_zip_entry_to_string<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    name: &str,
) -> Result<String, OdfError> {
    let mut entry = archive
        .by_name(name)
        .map_err(|_| OdfError::InvalidStructure(format!("Missing entry: {}", name)))?;
    let mut s = String::new();
    entry.read_to_string(&mut s)?;
    Ok(s)
}

/// Simple character style info gathered from automatic styles
#[derive(Clone, Default)]
struct StyleInfo {
    bold: Option<bool>,
    italic: Option<bool>,
    underline: Option<bool>,
    font_size: Option<u32>,
    font_family: Option<String>,
    color: Option<Color>,
    alignment: Option<Alignment>,
    outline_level: Option<u8>,
    paragraph_style: Option<String>,
}

fn parse_auto_styles(xml: &str) -> HashMap<String, StyleInfo> {
    let mut styles: HashMap<String, StyleInfo> = HashMap::new();
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut current_style_name = String::new();
    let mut current_info = StyleInfo::default();
    let mut in_style = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                match name.as_str() {
                    "style" => {
                        // Check if it's a style:style element
                        current_style_name = String::new();
                        current_info = StyleInfo::default();
                        in_style = true;
                        for attr in e.attributes().flatten() {
                            let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                            match key.as_str() {
                                "style:name" | "name" => {
                                    current_style_name =
                                        String::from_utf8_lossy(&attr.value).to_string();
                                }
                                "style:family" | "family" => {}
                                "style:parent-style-name" | "parent-style-name" => {
                                    let parent =
                                        String::from_utf8_lossy(&attr.value).to_string();
                                    // Inherit heading level from parent style name
                                    if parent.starts_with("Heading") {
                                        if let Some(level_char) = parent.chars().last() {
                                            if let Some(level) = level_char.to_digit(10) {
                                                current_info.outline_level = Some(level as u8);
                                                current_info.paragraph_style =
                                                    Some(format!("Heading{}", level));
                                            }
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    "text-properties" if in_style => {
                        for attr in e.attributes().flatten() {
                            let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                            let val = String::from_utf8_lossy(&attr.value).to_string();
                            match key.as_str() {
                                "fo:font-weight" | "font-weight" => {
                                    current_info.bold = Some(val == "bold");
                                }
                                "fo:font-style" | "font-style" => {
                                    current_info.italic = Some(val == "italic");
                                }
                                "style:text-underline-style"
                                | "text-underline-style" => {
                                    current_info.underline = Some(val != "none");
                                }
                                "fo:font-size" | "font-size" => {
                                    // Parse pt value like "12pt"
                                    let size_str = val.trim_end_matches("pt");
                                    if let Ok(size_f) = size_str.parse::<f64>() {
                                        // half-points
                                        current_info.font_size = Some((size_f * 2.0) as u32);
                                    }
                                }
                                "fo:font-family" | "font-family" | "fo:font-name" => {
                                    current_info.font_family = Some(val.trim_matches('\'').to_string());
                                }
                                "fo:color" | "color" => {
                                    if let Some(c) = Color::from_hex(&val) {
                                        current_info.color = Some(c);
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    "paragraph-properties" if in_style => {
                        for attr in e.attributes().flatten() {
                            let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                            let val = String::from_utf8_lossy(&attr.value).to_string();
                            if key == "fo:text-align" || key == "text-align" {
                                current_info.alignment = Some(match val.as_str() {
                                    "center" => Alignment::Center,
                                    "end" | "right" => Alignment::Right,
                                    "justify" => Alignment::Justify,
                                    _ => Alignment::Left,
                                });
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                if name == "style" && in_style && !current_style_name.is_empty() {
                    styles.insert(current_style_name.clone(), current_info.clone());
                    in_style = false;
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    styles
}

fn parse_meta<R: Read + Seek>(archive: &mut ZipArchive<R>) -> Result<Metadata, OdfError> {
    let xml = match read_zip_entry_to_string(archive, "meta.xml") {
        Ok(s) => s,
        Err(_) => return Ok(Metadata::default()),
    };

    let mut meta = Metadata::default();
    let mut reader = Reader::from_str(&xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut current = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                current = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
            }
            Ok(Event::Text(e)) => {
                let text = e.unescape().unwrap_or_default().to_string();
                match current.as_str() {
                    "title" => meta.title = Some(text),
                    "creator" | "initial-creator" => {
                        if meta.author.is_none() {
                            meta.author = Some(text);
                        }
                    }
                    "subject" => meta.subject = Some(text),
                    "description" => meta.description = Some(text),
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(OdfError::Xml(e)),
            _ => {}
        }
        buf.clear();
    }

    Ok(meta)
}

fn parse_content_xml(
    xml: &str,
    styles: &HashMap<String, StyleInfo>,
    content: &mut Vec<Block>,
) -> Result<(), OdfError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);
    let mut buf = Vec::new();

    let mut in_body = false;
    let mut in_text = false;

    let mut current_para: Option<Paragraph> = None;
    let mut current_char_props = CharacterProperties::default();
    let mut current_para_props = ParagraphProperties::default();
    let mut current_run_text = String::new();
    let mut in_para = false;
    let mut in_span = false;
    let mut span_depth = 0u32;

    // Table state
    let mut table_stack: Vec<TableBlock> = Vec::new();
    let mut row_cells_stack: Vec<Vec<TableCell>> = Vec::new();
    let mut cell_content_stack: Vec<Vec<Block>> = Vec::new();
    let mut in_table = false;
    let mut in_row = false;
    let mut in_cell = false;

    let mut element_stack: Vec<String> = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let local_name =
                    String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                element_stack.push(local_name.clone());

                match local_name.as_str() {
                    "body" => in_body = true,
                    "text" if in_body => in_text = true,
                    "p" | "h" if in_text || in_cell => {
                        in_para = true;
                        current_para = Some(Paragraph::new());
                        current_para_props = ParagraphProperties::default();
                        current_char_props = CharacterProperties::default();

                        // Get style name
                        for attr in e.attributes().flatten() {
                            let key =
                                String::from_utf8_lossy(attr.key.as_ref()).to_string();
                            if key == "text:style-name" || key == "style-name" {
                                let style_name =
                                    String::from_utf8_lossy(&attr.value).to_string();
                                if let Some(style) = styles.get(&style_name) {
                                    if let Some(align) = style.alignment {
                                        current_para_props.alignment = Some(align);
                                    }
                                    if let Some(level) = style.outline_level {
                                        current_para_props.outline_level = Some(level);
                                    }
                                    if let Some(ref ps) = style.paragraph_style {
                                        current_para_props.paragraph_style = Some(ps.clone());
                                    }
                                }
                                // For heading elements
                                if local_name == "h" {
                                    for attr2 in e.attributes().flatten() {
                                        let k2 = String::from_utf8_lossy(attr2.key.as_ref()).to_string();
                                        if k2 == "text:outline-level" || k2 == "outline-level" {
                                            if let Ok(level) = String::from_utf8_lossy(&attr2.value)
                                                .parse::<u8>()
                                            {
                                                current_para_props.outline_level = Some(level);
                                                current_para_props.paragraph_style =
                                                    Some(format!("Heading{}", level));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        // For h elements, also check outline-level directly
                        if local_name == "h" {
                            for attr in e.attributes().flatten() {
                                let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                                if key == "text:outline-level" || key == "outline-level" {
                                    if let Ok(level) = String::from_utf8_lossy(&attr.value)
                                        .parse::<u8>()
                                    {
                                        current_para_props.outline_level = Some(level);
                                        current_para_props.paragraph_style =
                                            Some(format!("Heading{}", level));
                                    }
                                }
                            }
                        }
                    }
                    "span" if in_para => {
                        in_span = true;
                        span_depth += 1;

                        // Flush current text as a run
                        if !current_run_text.is_empty() {
                            let mut run = TextRun::new(current_run_text.clone());
                            run.properties = current_char_props.clone();
                            if let Some(ref mut para) = current_para {
                                para.content.push(Inline::Text(run));
                            }
                            current_run_text.clear();
                        }

                        // Get span style
                        for attr in e.attributes().flatten() {
                            let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                            if key == "text:style-name" || key == "style-name" {
                                let style_name =
                                    String::from_utf8_lossy(&attr.value).to_string();
                                if let Some(style) = styles.get(&style_name) {
                                    if style.bold == Some(true) {
                                        current_char_props.bold = Some(true);
                                    }
                                    if style.italic == Some(true) {
                                        current_char_props.italic = Some(true);
                                    }
                                    if style.underline == Some(true) {
                                        current_char_props.underline = Some(UnderlineStyle::Single);
                                    }
                                    if let Some(size) = style.font_size {
                                        current_char_props.font_size = Some(size);
                                    }
                                    if let Some(ref font) = style.font_family {
                                        current_char_props.font_family = Some(font.clone());
                                    }
                                    if let Some(color) = style.color {
                                        current_char_props.color = Some(color);
                                    }
                                }
                            }
                        }
                    }
                    "table" if in_text => {
                        in_table = true;
                        table_stack.push(TableBlock {
                            id: ElementId::new(),
                            rows: 0,
                            cols: 0,
                            cells: Vec::new(),
                            properties: TableProperties::default(),
                        });
                        row_cells_stack.push(Vec::new());
                        cell_content_stack.push(Vec::new());
                    }
                    "table-row" if in_table => {
                        in_row = true;
                    }
                    "table-cell" if in_row => {
                        in_cell = true;
                        cell_content_stack.push(Vec::new());
                    }
                    _ => {}
                }
            }
            Ok(Event::Empty(ref e)) => {
                let local_name =
                    String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                match local_name.as_str() {
                    "line-break" if in_para => {
                        if !current_run_text.is_empty() {
                            let mut run = TextRun::new(current_run_text.clone());
                            run.properties = current_char_props.clone();
                            if let Some(ref mut para) = current_para {
                                para.content.push(Inline::Text(run));
                            }
                            current_run_text.clear();
                        }
                        if let Some(ref mut para) = current_para {
                            para.content.push(Inline::Break(BreakType::Line));
                        }
                    }
                    "tab" if in_para => {
                        if !current_run_text.is_empty() {
                            let mut run = TextRun::new(current_run_text.clone());
                            run.properties = current_char_props.clone();
                            if let Some(ref mut para) = current_para {
                                para.content.push(Inline::Text(run));
                            }
                            current_run_text.clear();
                        }
                        if let Some(ref mut para) = current_para {
                            para.content.push(Inline::Tab);
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(e)) => {
                if in_para {
                    let text = e.unescape().unwrap_or_default().to_string();
                    current_run_text.push_str(&text);
                }
            }
            Ok(Event::End(ref e)) => {
                let local_name =
                    String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                element_stack.pop();

                match local_name.as_str() {
                    "span" if in_span => {
                        // Flush text
                        if !current_run_text.is_empty() {
                            let mut run = TextRun::new(current_run_text.clone());
                            run.properties = current_char_props.clone();
                            if let Some(ref mut para) = current_para {
                                para.content.push(Inline::Text(run));
                            }
                            current_run_text.clear();
                        }
                        if span_depth > 0 {
                            span_depth -= 1;
                        }
                        if span_depth == 0 {
                            in_span = false;
                            current_char_props = CharacterProperties::default();
                        }
                    }
                    "p" | "h" if in_para => {
                        // Flush remaining text
                        if !current_run_text.is_empty() {
                            let mut run = TextRun::new(current_run_text.clone());
                            run.properties = current_char_props.clone();
                            if let Some(ref mut para) = current_para {
                                para.content.push(Inline::Text(run));
                            }
                            current_run_text.clear();
                        }
                        in_para = false;
                        if let Some(mut para) = current_para.take() {
                            para.properties = current_para_props.clone();
                            if in_cell {
                                if let Some(cell_content) = cell_content_stack.last_mut() {
                                    cell_content.push(Block::Paragraph(para));
                                }
                            } else {
                                content.push(Block::Paragraph(para));
                            }
                        }
                    }
                    "table-cell" if in_cell => {
                        in_cell = false;
                        let cell_content = cell_content_stack.pop().unwrap_or_default();
                        let cell = TableCell {
                            id: ElementId::new(),
                            content: cell_content,
                            properties: TableCellProperties::default(),
                            col_span: 1,
                            row_span: 1,
                        };
                        if let Some(row_cells) = row_cells_stack.last_mut() {
                            row_cells.push(cell);
                        }
                    }
                    "table-row" if in_row => {
                        in_row = false;
                        if let Some(tbl) = table_stack.last_mut() {
                            tbl.rows += 1;
                            if let Some(row_cells) = row_cells_stack.last_mut() {
                                let cols = row_cells.len() as u32;
                                if cols > tbl.cols {
                                    tbl.cols = cols;
                                }
                                tbl.cells.append(row_cells);
                            }
                        }
                    }
                    "table" if in_table => {
                        let _ = row_cells_stack.pop();
                        let _ = cell_content_stack.pop();
                        if let Some(tbl) = table_stack.pop() {
                            content.push(Block::Table(tbl));
                        }
                        in_table = !table_stack.is_empty();
                        in_row = false;
                        in_cell = false;
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(OdfError::Xml(e)),
            _ => {}
        }
        buf.clear();
    }

    Ok(())
}
