/// Provides functions for creating common default style sets.
///
/// This module contains factory functions that produce pre-configured
/// style catalogs for different use cases (e.g., a minimal set, a
/// Word-compatible set, an ODF-compatible set).

use crate::catalog::StyleCatalog;
use crate::list::{ListLevelStyle, ListStyle};
use crate::paragraph::ParagraphStyle;
use crate::table::TableStyle;

/// Create a minimal style catalog with just the essential styles.
pub fn minimal_catalog() -> StyleCatalog {
    let mut catalog = StyleCatalog::with_defaults();
    register_list_styles(&mut catalog);
    register_extended_table_styles(&mut catalog);
    register_additional_paragraph_styles(&mut catalog);
    catalog
}

/// Register built-in list styles.
pub fn register_list_styles(catalog: &mut StyleCatalog) {
    // --- Bullet List ---
    let mut bullet = ListStyle::new("Bullet List");
    bullet.base.built_in = true;
    bullet.base.display_name = Some("Bullet List".to_string());

    let bullet_chars = ["•", "◦", "▪", "▸", "–", "○", "□", "‣", "⁃", "◉"];
    for (i, &ch) in bullet_chars.iter().enumerate() {
        bullet.levels.push(ListLevelStyle {
            level: i as u8,
            format: "bullet".to_string(),
            level_text: ch.to_string(),
            start: 1,
            bullet_char: Some(ch.to_string()),
            bullet_font: Some("Symbol".to_string()),
            indent_twips: (i as i32 + 1) * 720, // 0.5" per level
            text_indent_twips: -360,             // -0.25" hanging
            suffix: "tab".to_string(),
            char_style: None,
        });
    }
    catalog
        .list_styles
        .insert("Bullet List".to_string(), bullet);

    // --- Numbered List ---
    let mut numbered = ListStyle::new("Numbered List");
    numbered.base.built_in = true;
    numbered.base.display_name = Some("Numbered List".to_string());

    let formats = [
        ("decimal", "%1."),
        ("lower-alpha", "%1."),
        ("lower-roman", "%1."),
        ("decimal", "(%1)"),
        ("lower-alpha", "(%1)"),
        ("lower-roman", "(%1)"),
        ("decimal", "%1."),
        ("lower-alpha", "%1."),
        ("lower-roman", "%1."),
        ("decimal", "%1."),
    ];
    for (i, &(fmt, tmpl)) in formats.iter().enumerate() {
        numbered.levels.push(ListLevelStyle {
            level: i as u8,
            format: fmt.to_string(),
            level_text: tmpl.to_string(),
            start: 1,
            bullet_char: None,
            bullet_font: None,
            indent_twips: (i as i32 + 1) * 720,
            text_indent_twips: -360,
            suffix: "tab".to_string(),
            char_style: None,
        });
    }
    catalog
        .list_styles
        .insert("Numbered List".to_string(), numbered);

    // --- Outline Numbered List ---
    let mut outline = ListStyle::new("Outline Numbered");
    outline.base.built_in = true;
    outline.base.display_name = Some("Outline Numbered".to_string());

    for i in 0..10u8 {
        // Build hierarchical level text: "1.", "1.1.", "1.1.1." etc.
        let level_text = (0..=i)
            .map(|l| format!("%{}", l + 1))
            .collect::<Vec<_>>()
            .join(".")
            + ".";

        outline.levels.push(ListLevelStyle {
            level: i,
            format: "decimal".to_string(),
            level_text,
            start: 1,
            bullet_char: None,
            bullet_font: None,
            indent_twips: (i as i32 + 1) * 720,
            text_indent_twips: -360,
            suffix: "tab".to_string(),
            char_style: None,
        });
    }
    catalog
        .list_styles
        .insert("Outline Numbered".to_string(), outline);
}

/// Register extended table styles.
pub fn register_extended_table_styles(catalog: &mut StyleCatalog) {
    // Table Simple — minimal borders, no shading
    let mut simple = TableStyle::new("Table Simple");
    simple.base.built_in = true;
    simple.base.display_name = Some("Table Simple".to_string());
    simple.base.parent = Some("Table Normal".to_string());
    simple.properties.cell_padding_twips = Some(72);
    simple.properties.border_style = Some("single".to_string());
    simple.properties.border_width_twips = Some(15);
    simple.properties.border_color = Some("#000000".to_string());
    catalog
        .table_styles
        .insert("Table Simple".to_string(), simple);

    // Table Elegant — header row highlighted, alternating row shading
    let mut elegant = TableStyle::new("Table Elegant");
    elegant.base.built_in = true;
    elegant.base.display_name = Some("Table Elegant".to_string());
    elegant.base.parent = Some("Table Normal".to_string());
    elegant.properties.cell_padding_twips = Some(100);
    elegant.properties.border_style = Some("single".to_string());
    elegant.properties.banded_rows = Some(true);
    use crate::table::{ConditionalTableFormat, TableRegion};
    elegant.conditional.push(ConditionalTableFormat {
        region: TableRegion::HeaderRow,
        background: Some("#4472C4".to_string()),
        border_color: None,
        bold: Some(true),
        italic: None,
        font_color: Some("#FFFFFF".to_string()),
    });
    elegant.conditional.push(ConditionalTableFormat {
        region: TableRegion::OddRows,
        background: Some("#D9E2F3".to_string()),
        border_color: None,
        bold: None,
        italic: None,
        font_color: None,
    });
    catalog
        .table_styles
        .insert("Table Elegant".to_string(), elegant);

    // Table Colorful — bold header, accent banding
    let mut colorful = TableStyle::new("Table Colorful");
    colorful.base.built_in = true;
    colorful.base.display_name = Some("Table Colorful".to_string());
    colorful.base.parent = Some("Table Normal".to_string());
    colorful.properties.cell_padding_twips = Some(80);
    colorful.properties.banded_rows = Some(true);
    colorful.conditional.push(ConditionalTableFormat {
        region: TableRegion::HeaderRow,
        background: Some("#70AD47".to_string()),
        border_color: None,
        bold: Some(true),
        italic: None,
        font_color: Some("#FFFFFF".to_string()),
    });
    colorful.conditional.push(ConditionalTableFormat {
        region: TableRegion::OddRows,
        background: Some("#E2EFDA".to_string()),
        border_color: None,
        bold: None,
        italic: None,
        font_color: None,
    });
    catalog
        .table_styles
        .insert("Table Colorful".to_string(), colorful);
}

/// Register additional paragraph styles.
pub fn register_additional_paragraph_styles(catalog: &mut StyleCatalog) {
    // Code Block
    let mut code_block = ParagraphStyle::new("Code Block");
    code_block.base.built_in = true;
    code_block.base.display_name = Some("Code Block".to_string());
    code_block.base.parent = Some("Normal".to_string());
    code_block.char_properties.font_family = Some("Consolas".to_string());
    code_block.char_properties.font_size_half_points = Some(20); // 10pt
    code_block.properties.space_before_twips = Some(80);
    code_block.properties.space_after_twips = Some(80);
    code_block.properties.background_color = Some("#F0F0F0".to_string());
    code_block.properties.indent_left_twips = Some(360);
    code_block.properties.indent_right_twips = Some(360);
    catalog
        .paragraph_styles
        .insert("Code Block".to_string(), code_block);

    // Preformatted text
    let mut preformat = ParagraphStyle::new("Preformatted");
    preformat.base.built_in = true;
    preformat.base.display_name = Some("Preformatted".to_string());
    preformat.base.parent = Some("Normal".to_string());
    preformat.char_properties.font_family = Some("Courier New".to_string());
    preformat.char_properties.font_size_half_points = Some(20);
    preformat.properties.space_after_twips = Some(0);
    preformat.properties.suppress_hyphenation = Some(true);
    catalog
        .paragraph_styles
        .insert("Preformatted".to_string(), preformat);

    // Intense Quote
    let mut intense_quote = ParagraphStyle::new("Intense Quote");
    intense_quote.base.built_in = true;
    intense_quote.base.display_name = Some("Intense Quote".to_string());
    intense_quote.base.parent = Some("Normal".to_string());
    intense_quote.char_properties.italic = Some(true);
    intense_quote.char_properties.color = Some("#4472C4".to_string());
    intense_quote.properties.alignment = Some("center".to_string());
    intense_quote.properties.indent_left_twips = Some(1080);
    intense_quote.properties.indent_right_twips = Some(1080);
    intense_quote.properties.space_before_twips = Some(100);
    intense_quote.properties.space_after_twips = Some(100);
    catalog
        .paragraph_styles
        .insert("Intense Quote".to_string(), intense_quote);

    // Footnote Text
    let mut footnote = ParagraphStyle::new("Footnote Text");
    footnote.base.built_in = true;
    footnote.base.display_name = Some("Footnote Text".to_string());
    footnote.base.parent = Some("Normal".to_string());
    footnote.char_properties.font_size_half_points = Some(20); // 10pt
    footnote.properties.space_after_twips = Some(0);
    catalog
        .paragraph_styles
        .insert("Footnote Text".to_string(), footnote);

    // Header / Footer paragraph style
    let mut header_style = ParagraphStyle::new("Header");
    header_style.base.built_in = true;
    header_style.base.display_name = Some("Header".to_string());
    header_style.base.parent = Some("Normal".to_string());
    header_style.properties.space_after_twips = Some(0);
    catalog
        .paragraph_styles
        .insert("Header".to_string(), header_style);

    let mut footer_style = ParagraphStyle::new("Footer");
    footer_style.base.built_in = true;
    footer_style.base.display_name = Some("Footer".to_string());
    footer_style.base.parent = Some("Normal".to_string());
    footer_style.properties.space_after_twips = Some(0);
    catalog
        .paragraph_styles
        .insert("Footer".to_string(), footer_style);
}
