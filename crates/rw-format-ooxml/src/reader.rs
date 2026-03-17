//! OOXML document reader.
//!
//! Parses a .docx ZIP archive and constructs a Document model.
//! Handles the OOXML WordprocessingML schema.

use crate::OoxmlError;
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
use std::io::{Read, Seek};
use zip::ZipArchive;

/// Read a .docx document from any reader that supports Read + Seek.
pub fn read_docx_from_reader<R: Read + Seek>(reader: R) -> Result<Document, OoxmlError> {
    let mut archive = ZipArchive::new(reader)?;
    let mut doc = Document::new();
    doc.sections.clear();

    // Parse metadata from docProps/core.xml
    if let Ok(meta) = parse_core_props(&mut archive) {
        doc.metadata = meta;
    }

    // Parse main document body
    let body_xml = read_zip_entry_to_string(&mut archive, "word/document.xml")?;
    let mut section = Section::new();
    parse_document_body(&body_xml, &mut section.content)?;
    if section.content.is_empty() {
        section.content.push(Block::Paragraph(Paragraph::new()));
    }
    doc.sections.push(section);

    Ok(doc)
}

fn read_zip_entry_to_string<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    name: &str,
) -> Result<String, OoxmlError> {
    let mut entry = archive
        .by_name(name)
        .map_err(|_| OoxmlError::InvalidStructure(format!("Missing entry: {}", name)))?;
    let mut s = String::new();
    entry.read_to_string(&mut s)?;
    Ok(s)
}

fn parse_core_props<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
) -> Result<Metadata, OoxmlError> {
    let xml = match read_zip_entry_to_string(archive, "docProps/core.xml") {
        Ok(s) => s,
        Err(_) => return Ok(Metadata::default()),
    };

    let mut meta = Metadata::default();
    let mut reader = Reader::from_str(&xml);
    reader.config_mut().trim_text(true);

    let mut current_element = String::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                current_element = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
            }
            Ok(Event::Text(e)) => {
                let text = e.unescape().unwrap_or_default().to_string();
                match current_element.as_str() {
                    "title" => meta.title = Some(text),
                    "creator" => meta.author = Some(text),
                    "subject" => meta.subject = Some(text),
                    "description" => meta.description = Some(text),
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(OoxmlError::Xml(e)),
            _ => {}
        }
        buf.clear();
    }

    Ok(meta)
}

fn parse_document_body(
    xml: &str,
    content: &mut Vec<Block>,
) -> Result<(), OoxmlError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);

    let mut buf = Vec::new();

    // State for building paragraphs
    let mut current_para: Option<Paragraph> = None;
    let mut current_run_props = CharacterProperties::default();
    let mut current_para_props = ParagraphProperties::default();
    let mut current_run_text = String::new();
    let mut in_para = false;
    let mut in_run = false;
    let mut in_rpr = false;
    let mut in_ppr = false;
    let mut in_text = false;
    let mut preserve_space = false;

    // Table state
    let mut table_stack: Vec<TableBlock> = Vec::new();
    let mut row_cells_stack: Vec<Vec<TableCell>> = Vec::new();
    let mut cell_content_stack: Vec<Vec<Block>> = Vec::new();
    let mut in_table = false;
    let mut in_row = false;
    let mut in_cell = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();

                match name.as_str() {
                    "p" => {
                        in_para = true;
                        current_para = Some(Paragraph::new());
                        current_para_props = ParagraphProperties::default();
                    }
                    "r" => {
                        if in_para {
                            in_run = true;
                            current_run_text = String::new();
                            current_run_props = CharacterProperties::default();
                        }
                    }
                    "rPr" => {
                        if in_run {
                            in_rpr = true;
                        }
                    }
                    "pPr" => {
                        if in_para {
                            in_ppr = true;
                        }
                    }
                    "t" => {
                        if in_run {
                            in_text = true;
                            preserve_space = false;
                            for attr in e.attributes().flatten() {
                                let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                                if key.ends_with("space") || key == "xml:space" {
                                    let val = String::from_utf8_lossy(&attr.value).to_string();
                                    preserve_space = val == "preserve";
                                }
                            }
                        }
                    }
                    "b" if in_rpr => {
                        current_run_props.bold = Some(true);
                    }
                    "i" if in_rpr => {
                        current_run_props.italic = Some(true);
                    }
                    "u" if in_rpr => {
                        let mut underline = UnderlineStyle::Single;
                        let mut has_none = false;
                        for attr in e.attributes().flatten() {
                            let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                            if key == "w:val" || key == "val" {
                                let val = String::from_utf8_lossy(&attr.value).to_string();
                                has_none = val == "none";
                            }
                        }
                        if !has_none {
                            current_run_props.underline = Some(underline);
                        }
                    }
                    "color" if in_rpr => {
                        for attr in e.attributes().flatten() {
                            let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                            if key == "w:val" || key == "val" {
                                let val = String::from_utf8_lossy(&attr.value).to_string();
                                if val != "auto" {
                                    if let Some(color) = Color::from_hex(&val) {
                                        current_run_props.color = Some(color);
                                    }
                                }
                            }
                        }
                    }
                    "sz" if in_rpr => {
                        for attr in e.attributes().flatten() {
                            let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                            if key == "w:val" || key == "val" {
                                let val = String::from_utf8_lossy(&attr.value).to_string();
                                if let Ok(size) = val.parse::<u32>() {
                                    current_run_props.font_size = Some(size);
                                }
                            }
                        }
                    }
                    "rFonts" if in_rpr => {
                        for attr in e.attributes().flatten() {
                            let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                            if key == "w:ascii" || key == "ascii" {
                                let val = String::from_utf8_lossy(&attr.value).to_string();
                                current_run_props.font_family = Some(val);
                            }
                        }
                    }
                    "jc" if in_ppr => {
                        for attr in e.attributes().flatten() {
                            let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                            if key == "w:val" || key == "val" {
                                let val = String::from_utf8_lossy(&attr.value).to_string();
                                current_para_props.alignment = Some(match val.as_str() {
                                    "center" => Alignment::Center,
                                    "right" => Alignment::Right,
                                    "both" | "justify" => Alignment::Justify,
                                    _ => Alignment::Left,
                                });
                            }
                        }
                    }
                    "pStyle" if in_ppr => {
                        for attr in e.attributes().flatten() {
                            let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                            if key == "w:val" || key == "val" {
                                let val = String::from_utf8_lossy(&attr.value).to_string();
                                current_para_props.paragraph_style = Some(val);
                            }
                        }
                    }
                    "tbl" => {
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
                    "tr" if in_table => {
                        in_row = true;
                    }
                    "tc" if in_row => {
                        in_cell = true;
                        cell_content_stack.push(Vec::new());
                    }
                    _ => {}
                }
            }
            Ok(Event::Empty(ref e)) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                match name.as_str() {
                    "b" if in_rpr => {
                        current_run_props.bold = Some(true);
                    }
                    "i" if in_rpr => {
                        current_run_props.italic = Some(true);
                    }
                    "br" if in_run => {
                        let mut break_type = BreakType::Line;
                        for attr in e.attributes().flatten() {
                            let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                            if key == "w:type" || key == "type" {
                                let val = String::from_utf8_lossy(&attr.value).to_string();
                                break_type = match val.as_str() {
                                    "page" => BreakType::Page,
                                    "column" => BreakType::Column,
                                    _ => BreakType::Line,
                                };
                            }
                        }
                        if let Some(ref mut para) = current_para {
                            para.content.push(Inline::Break(break_type));
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(e)) => {
                if in_text {
                    let text = e.unescape().unwrap_or_default().to_string();
                    current_run_text.push_str(&text);
                }
            }
            Ok(Event::End(ref e)) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();

                match name.as_str() {
                    "t" => {
                        in_text = false;
                        preserve_space = false;
                    }
                    "rPr" => {
                        in_rpr = false;
                    }
                    "pPr" => {
                        in_ppr = false;
                        if let Some(ref mut para) = current_para {
                            para.properties = current_para_props.clone();
                        }
                    }
                    "r" if in_run => {
                        in_run = false;
                        if !current_run_text.is_empty() {
                            let mut run = TextRun::new(current_run_text.clone());
                            run.properties = current_run_props.clone();
                            if let Some(ref mut para) = current_para {
                                para.content.push(Inline::Text(run));
                            }
                        }
                        current_run_text.clear();
                    }
                    "p" if in_para => {
                        in_para = false;
                        if let Some(para) = current_para.take() {
                            if in_cell {
                                if let Some(cell_content) = cell_content_stack.last_mut() {
                                    cell_content.push(Block::Paragraph(para));
                                }
                            } else {
                                content.push(Block::Paragraph(para));
                            }
                        }
                    }
                    "tc" if in_cell => {
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
                    "tr" if in_row => {
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
                    "tbl" if in_table => {
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
            Err(e) => return Err(OoxmlError::Xml(e)),
            _ => {}
        }
        buf.clear();
    }

    Ok(())
}
