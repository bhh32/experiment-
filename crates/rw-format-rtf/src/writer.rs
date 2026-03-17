//! RTF generator.
//!
//! Serializes a Document model into RTF format (RTF 1.x specification).

use crate::RtfError;
use rw_document::{
    Block, Color, Document, Inline,
    inline::BreakType,
    properties::{Alignment, UnderlineStyle},
};
use std::collections::HashMap;

/// Write a Document to an RTF file.
pub fn write_rtf(doc: &Document, path: &std::path::Path) -> Result<(), RtfError> {
    let rtf = generate_rtf(doc);
    std::fs::write(path, rtf)?;
    Ok(())
}

/// Generate an RTF string from a Document.
pub fn generate_rtf(doc: &Document) -> String {
    let mut out = String::new();

    // Collect all unique fonts and colors
    let mut fonts: Vec<String> = vec!["Times New Roman".to_string()];
    let mut colors: Vec<Color> = vec![Color::BLACK];

    // First pass: collect fonts and colors from the document
    for section in &doc.sections {
        for block in &section.content {
            collect_resources(block, &mut fonts, &mut colors);
        }
    }

    // RTF header
    out.push_str("{\\rtf1\\ansi\\deff0\n");

    // Font table
    out.push_str("{\\fonttbl\n");
    for (i, font) in fonts.iter().enumerate() {
        out.push_str(&format!(
            "{{\\f{}\\froman\\fcharset0 {};}}\n",
            i, rtf_escape(font)
        ));
    }
    out.push_str("}\n");

    // Color table
    out.push_str("{\\colortbl ;");
    for color in colors.iter().skip(1) {
        out.push_str(&format!(
            "\\red{}\\green{}\\blue{};",
            color.r, color.g, color.b
        ));
    }
    out.push_str("}\n");

    // Document metadata as info block
    out.push_str("{\\info\n");
    if let Some(ref title) = doc.metadata.title {
        out.push_str(&format!("{{\\title {}}}\n", rtf_escape(title)));
    }
    if let Some(ref author) = doc.metadata.author {
        out.push_str(&format!("{{\\author {}}}\n", rtf_escape(author)));
    }
    out.push_str("}\n");

    // Default formatting
    out.push_str("\\widowctrl\\wpaper12240\\wpaperh15840\\margl1440\\margr1440\\margt1440\\margb1440\\f0\\fs24\n");

    // Build font and color lookup maps
    let font_map: HashMap<String, usize> = fonts
        .iter()
        .enumerate()
        .map(|(i, f)| (f.clone(), i))
        .collect();
    let color_map: HashMap<(u8, u8, u8), usize> = colors
        .iter()
        .enumerate()
        .skip(1) // skip index 0 (auto)
        .map(|(i, c)| ((c.r, c.g, c.b), i))
        .collect();

    // Document body
    for section in &doc.sections {
        for block in &section.content {
            write_block_rtf(block, &font_map, &color_map, &mut out);
        }
    }

    out.push_str("}\n");
    out
}

fn write_block_rtf(
    block: &Block,
    font_map: &HashMap<String, usize>,
    color_map: &HashMap<(u8, u8, u8), usize>,
    out: &mut String,
) {
    match block {
        Block::Paragraph(para) => {
            // Paragraph properties
            out.push_str("\\pard");

            let props = &para.properties;
            if let Some(align) = props.alignment {
                match align {
                    Alignment::Center => out.push_str("\\qc"),
                    Alignment::Right => out.push_str("\\qr"),
                    Alignment::Justify => out.push_str("\\qj"),
                    Alignment::Left => out.push_str("\\ql"),
                    Alignment::Distribute => out.push_str("\\qd"),
                }
            }

            // Style name
            if let Some(ref style) = props.paragraph_style {
                if style.starts_with("Heading") || style.starts_with("heading") {
                    if let Some(level_char) = style.chars().last() {
                        if let Some(level) = level_char.to_digit(10) {
                            // Use heading outline level
                            out.push_str(&format!("\\outlinelevel{}", level - 1));
                            // Bigger font for headings
                            let size = match level {
                                1 => 48u32,
                                2 => 36,
                                3 => 28,
                                _ => 24,
                            };
                            out.push_str(&format!("\\fs{}\\b", size));
                        }
                    }
                }
            }

            out.push(' ');

            // Inline content
            for inline in &para.content {
                write_inline_rtf(inline, font_map, color_map, out);
            }

            // Paragraph break
            out.push_str("\\par\n");
        }
        Block::Table(tbl) => {
            // Simple table output
            let rows = tbl.rows as usize;
            let cols = tbl.cols as usize;
            if cols == 0 {
                return;
            }
            let col_width = 1440i32; // 1 inch per column

            for row_idx in 0..rows {
                // Row definition
                out.push_str("\\trowd\\trgaph108\\trleft-108\n");
                for col_idx in 0..cols {
                    let right = col_width * (col_idx as i32 + 1);
                    out.push_str(&format!("\\cellx{}\n", right));
                }

                for col_idx in 0..cols {
                    let cell_idx = row_idx * cols + col_idx;
                    if let Some(cell) = tbl.cells.get(cell_idx) {
                        out.push_str("\\pard\\intbl ");
                        for block in &cell.content {
                            write_block_rtf(block, font_map, color_map, out);
                        }
                        out.push_str("\\cell\n");
                    } else {
                        out.push_str("\\pard\\intbl\\cell\n");
                    }
                }
                out.push_str("\\row\n");
            }
        }
        Block::HorizontalRule(_) => {
            out.push_str("\\pard\\brdrb\\brdrs\\brdrw10\\brsp20\\par\n");
        }
        _ => {}
    }
}

fn write_inline_rtf(
    inline: &Inline,
    font_map: &HashMap<String, usize>,
    color_map: &HashMap<(u8, u8, u8), usize>,
    out: &mut String,
) {
    match inline {
        Inline::Text(run) => {
            let props = &run.properties;
            let mut fmt = String::new();
            let mut reset = String::new();

            if props.bold == Some(true) {
                fmt.push_str("\\b");
                reset.push_str("\\b0");
            }
            if props.italic == Some(true) {
                fmt.push_str("\\i");
                reset.push_str("\\i0");
            }
            if let Some(underline) = props.underline {
                match underline {
                    UnderlineStyle::Single => fmt.push_str("\\ul"),
                    UnderlineStyle::Double => fmt.push_str("\\uldb"),
                    UnderlineStyle::Dotted => fmt.push_str("\\uld"),
                    UnderlineStyle::Wave => fmt.push_str("\\ulwave"),
                    _ => fmt.push_str("\\ul"),
                }
                reset.push_str("\\ulnone");
            }
            if let Some(size) = props.font_size {
                fmt.push_str(&format!("\\fs{}", size));
                reset.push_str("\\fs24");
            }
            if let Some(ref font) = props.font_family {
                if let Some(&idx) = font_map.get(font) {
                    fmt.push_str(&format!("\\f{}", idx));
                    reset.push_str("\\f0");
                }
            }
            if let Some(ref color) = props.color {
                if let Some(&idx) = color_map.get(&(color.r, color.g, color.b)) {
                    fmt.push_str(&format!("\\cf{}", idx));
                    reset.push_str("\\cf0");
                }
            }
            if props.superscript == Some(true) {
                fmt.push_str("\\super");
                reset.push_str("\\nosupersub");
            }
            if props.subscript == Some(true) {
                fmt.push_str("\\sub");
                reset.push_str("\\nosupersub");
            }

            if !fmt.is_empty() {
                out.push_str(&fmt);
                out.push(' ');
            }

            out.push_str(&rtf_encode_text(&run.text));

            if !reset.is_empty() {
                out.push_str(&reset);
                out.push(' ');
            }
        }
        Inline::Break(break_type) => match break_type {
            BreakType::Line => out.push_str("\\line\n"),
            BreakType::Page => out.push_str("\\page\n"),
            BreakType::Column => out.push_str("\\column\n"),
        },
        Inline::Tab => out.push_str("\\tab "),
        Inline::NonBreakingSpace => out.push_str("\\~"),
        Inline::Hyperlink(link) => {
            for inner in &link.content {
                write_inline_rtf(inner, font_map, color_map, out);
            }
        }
        _ => {}
    }
}

/// Encode a text string for RTF, escaping special characters.
fn rtf_encode_text(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        match c {
            '{' => out.push_str("\\{"),
            '}' => out.push_str("\\}"),
            '\\' => out.push_str("\\\\"),
            c if c.is_ascii() => out.push(c),
            c => {
                // Encode as Unicode escape
                let code = c as u32;
                if code <= 0x7FFF {
                    out.push_str(&format!("\\u{}?", code));
                } else {
                    out.push_str(&format!("\\u{}?", code as i32 - 65536));
                }
            }
        }
    }
    out
}

fn rtf_escape(s: &str) -> String {
    s.replace('{', "\\{")
        .replace('}', "\\}")
        .replace('\\', "\\\\")
}

/// Collect all unique font names and colors from document blocks.
fn collect_resources(
    block: &Block,
    fonts: &mut Vec<String>,
    colors: &mut Vec<Color>,
) {
    match block {
        Block::Paragraph(para) => {
            for inline in &para.content {
                collect_inline_resources(inline, fonts, colors);
            }
        }
        Block::Table(tbl) => {
            for cell in &tbl.cells {
                for block in &cell.content {
                    collect_resources(block, fonts, colors);
                }
            }
        }
        _ => {}
    }
}

fn collect_inline_resources(inline: &Inline, fonts: &mut Vec<String>, colors: &mut Vec<Color>) {
    match inline {
        Inline::Text(run) => {
            if let Some(ref font) = run.properties.font_family {
                if !fonts.contains(font) {
                    fonts.push(font.clone());
                }
            }
            if let Some(color) = run.properties.color {
                if color != Color::BLACK && !colors.iter().any(|c| c.r == color.r && c.g == color.g && c.b == color.b) {
                    colors.push(color);
                }
            }
        }
        Inline::Hyperlink(link) => {
            for inner in &link.content {
                collect_inline_resources(inner, fonts, colors);
            }
        }
        _ => {}
    }
}
