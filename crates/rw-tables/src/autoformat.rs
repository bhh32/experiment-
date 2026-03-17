//! Table autoformat — predefined table styles.

use rw_document::block::{TableBlock, TableBorders};
use rw_document::properties::{BorderLine, BorderStyle};
use rw_document::{Color, Twips};

/// A predefined table autoformat.
#[derive(Debug, Clone)]
pub struct TableAutoFormat {
    pub name: String,
    pub description: String,
    /// Whether to apply font formatting
    pub apply_font: bool,
    /// Whether to apply border formatting
    pub apply_borders: bool,
    /// Whether to apply shading
    pub apply_shading: bool,
    /// Whether to apply alignment
    pub apply_alignment: bool,
    /// Whether the first row is special (header)
    pub header_row: bool,
    /// Whether the last row is special (total)
    pub footer_row: bool,
    /// Whether the first column is special
    pub first_column: bool,
    /// Whether the last column is special
    pub last_column: bool,
    /// Whether to use banded rows
    pub banded_rows: bool,
    /// Whether to use banded columns
    pub banded_columns: bool,
}

/// Predefined style presets for tables.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableStylePreset {
    /// No borders, no shading
    Plain,
    /// Full grid borders
    Grid,
    /// Light grid borders (thin lines)
    GridLight,
    /// Word-style List Table 1 (header row shaded)
    ListTable1,
    /// Word-style List Table 2 (banded rows)
    ListTable2,
}

/// Apply a style preset to a table (modifies cell properties and table borders).
pub fn apply_preset(table: &mut TableBlock, preset: TableStylePreset) {
    match preset {
        TableStylePreset::Plain => apply_plain(table),
        TableStylePreset::Grid => apply_grid(table, 20, Color::BLACK),
        TableStylePreset::GridLight => apply_grid(table, 10, Color::rgb(180, 180, 180)),
        TableStylePreset::ListTable1 => apply_list_table1(table),
        TableStylePreset::ListTable2 => apply_list_table2(table),
    }
}

fn thin_border(width: i32, color: Color) -> BorderLine {
    BorderLine {
        style: BorderStyle::Single,
        width: Twips(width),
        color,
    }
}

fn full_borders(width: i32, color: Color) -> TableBorders {
    let b = thin_border(width, color);
    TableBorders {
        top: Some(b),
        bottom: Some(b),
        left: Some(b),
        right: Some(b),
        inside_horizontal: Some(b),
        inside_vertical: Some(b),
    }
}

fn apply_plain(table: &mut TableBlock) {
    table.properties.borders = None;
    for cell in &mut table.cells {
        cell.properties.borders = None;
        cell.properties.background = None;
    }
}

fn apply_grid(table: &mut TableBlock, width: i32, color: Color) {
    let borders = full_borders(width, color);
    table.properties.borders = Some(borders.clone());
    for cell in &mut table.cells {
        cell.properties.borders = Some(borders.clone());
        cell.properties.background = None;
    }
}

fn apply_list_table1(table: &mut TableBlock) {
    let cols = table.cols as usize;
    let header_bg = Color::rgb(70, 130, 180);   // steel blue header
    let border = thin_border(15, Color::BLACK);
    let borders = TableBorders {
        top: Some(border),
        bottom: Some(border),
        left: None,
        right: None,
        inside_horizontal: Some(border),
        inside_vertical: None,
    };

    table.properties.borders = Some(borders.clone());

    for (idx, cell) in table.cells.iter_mut().enumerate() {
        let row = idx / cols.max(1);
        cell.properties.borders = Some(borders.clone());
        if row == 0 {
            cell.properties.background = Some(header_bg);
        } else {
            cell.properties.background = None;
        }
    }
}

fn apply_list_table2(table: &mut TableBlock) {
    let cols = table.cols as usize;
    let band_bg = Color::rgb(220, 235, 247);    // light blue banding
    let border = thin_border(10, Color::rgb(150, 150, 150));
    let borders = TableBorders {
        top: Some(border),
        bottom: Some(border),
        left: None,
        right: None,
        inside_horizontal: Some(border),
        inside_vertical: None,
    };

    table.properties.borders = Some(borders.clone());

    for (idx, cell) in table.cells.iter_mut().enumerate() {
        let row = idx / cols.max(1);
        cell.properties.borders = Some(borders.clone());
        if row % 2 == 1 {
            cell.properties.background = Some(band_bg);
        } else {
            cell.properties.background = None;
        }
    }
}
