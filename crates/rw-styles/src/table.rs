use crate::{StyleBase, StyleCategory};
use serde::{Deserialize, Serialize};

/// A table style definition.
///
/// Table styles control the visual appearance of tables, including
/// borders, cell shading, and text formatting. They support banded
/// rows/columns and special formatting for header/footer rows and
/// first/last columns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableStyle {
    pub base: StyleBase,
    pub properties: TableStyleProperties,
    /// Conditional formatting for specific table regions
    pub conditional: Vec<ConditionalTableFormat>,
}

impl TableStyle {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            base: StyleBase::new(name, StyleCategory::Table),
            properties: TableStyleProperties::default(),
            conditional: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TableStyleProperties {
    /// Table alignment: "left", "center", "right"
    pub alignment: Option<String>,
    /// Cell padding in twips
    pub cell_padding_twips: Option<i32>,
    /// Default cell background
    pub cell_background: Option<String>,
    /// Border color
    pub border_color: Option<String>,
    /// Border width in twips
    pub border_width_twips: Option<i32>,
    /// Border style
    pub border_style: Option<String>,
    /// Banded rows
    pub banded_rows: Option<bool>,
    /// Banded columns
    pub banded_columns: Option<bool>,
}

/// Conditional formatting applied to specific regions of a table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalTableFormat {
    pub region: TableRegion,
    pub background: Option<String>,
    pub border_color: Option<String>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub font_color: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TableRegion {
    WholeTable,
    HeaderRow,
    FooterRow,
    FirstColumn,
    LastColumn,
    OddRows,
    EvenRows,
    OddColumns,
    EvenColumns,
    TopLeftCell,
    TopRightCell,
    BottomLeftCell,
    BottomRightCell,
}
