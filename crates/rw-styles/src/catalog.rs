use crate::character::CharacterStyle;
use crate::frame::FrameStyle;
use crate::list::ListStyle;
use crate::page::PageStyle;
use crate::paragraph::ParagraphStyle;
use crate::table::TableStyle;
use crate::theme::Theme;
use crate::StyleName;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The complete collection of styles for a document.
///
/// The catalog owns all style definitions and provides methods
/// to look up, add, remove, and resolve styles with inheritance.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StyleCatalog {
    pub paragraph_styles: HashMap<StyleName, ParagraphStyle>,
    pub character_styles: HashMap<StyleName, CharacterStyle>,
    pub page_styles: HashMap<StyleName, PageStyle>,
    pub list_styles: HashMap<StyleName, ListStyle>,
    pub table_styles: HashMap<StyleName, TableStyle>,
    pub frame_styles: HashMap<StyleName, FrameStyle>,
    pub theme: Theme,
}

impl StyleCatalog {
    /// Create a new catalog pre-populated with default styles.
    pub fn with_defaults() -> Self {
        let mut catalog = Self::default();
        catalog.register_default_styles();
        catalog
    }

    /// Register the built-in default styles.
    fn register_default_styles(&mut self) {
        // Normal paragraph style (the root of the paragraph style hierarchy)
        let mut normal = ParagraphStyle::new("Normal");
        normal.base.built_in = true;
        normal.base.display_name = Some("Normal".to_string());
        normal.char_properties.font_family = Some("Calibri".to_string());
        normal.char_properties.font_size_half_points = Some(24); // 12pt
        normal.properties.line_spacing_type = Some("multiple".to_string());
        normal.properties.line_spacing_value = Some(1.15);
        normal.properties.space_after_twips = Some(160); // 8pt
        self.paragraph_styles.insert("Normal".to_string(), normal);

        // Heading styles
        for level in 1..=9u8 {
            let name = format!("Heading {}", level);
            let mut style = ParagraphStyle::new(&name);
            style.base.built_in = true;
            style.base.display_name = Some(name.clone());
            style.base.parent = Some("Normal".to_string());
            style.base.next_style = Some("Normal".to_string());
            style.properties.outline_level = Some(level);
            style.properties.keep_with_next = Some(true);
            style.char_properties.bold = Some(true);

            // Scale font sizes for heading levels
            let font_size = match level {
                1 => 32, // 16pt
                2 => 26, // 13pt
                3 => 24, // 12pt
                4 => 22, // 11pt
                _ => 20, // 10pt
            };
            style.char_properties.font_size_half_points = Some(font_size);

            // Spacing
            let space_before = match level {
                1 => 480, // 24pt
                2 => 200, // 10pt
                _ => 160, // 8pt
            };
            style.properties.space_before_twips = Some(space_before);
            style.properties.space_after_twips = Some(0);

            self.paragraph_styles.insert(name, style);
        }

        // Title style
        let mut title = ParagraphStyle::new("Title");
        title.base.built_in = true;
        title.base.display_name = Some("Title".to_string());
        title.base.parent = Some("Normal".to_string());
        title.base.next_style = Some("Normal".to_string());
        title.char_properties.font_size_half_points = Some(56); // 28pt
        title.properties.alignment = Some("center".to_string());
        title.properties.space_after_twips = Some(60);
        self.paragraph_styles.insert("Title".to_string(), title);

        // Subtitle style
        let mut subtitle = ParagraphStyle::new("Subtitle");
        subtitle.base.built_in = true;
        subtitle.base.display_name = Some("Subtitle".to_string());
        subtitle.base.parent = Some("Normal".to_string());
        subtitle.base.next_style = Some("Normal".to_string());
        subtitle.char_properties.font_size_half_points = Some(28); // 14pt
        subtitle.char_properties.color = Some("#5A5A5A".to_string());
        subtitle.properties.alignment = Some("center".to_string());
        self.paragraph_styles.insert("Subtitle".to_string(), subtitle);

        // Default character style
        let mut default_char = CharacterStyle::new("Default Paragraph Font");
        default_char.base.built_in = true;
        self.character_styles.insert("Default Paragraph Font".to_string(), default_char);

        // Strong character style
        let mut strong = CharacterStyle::new("Strong");
        strong.base.built_in = true;
        strong.base.display_name = Some("Strong".to_string());
        strong.properties.bold = Some(true);
        self.character_styles.insert("Strong".to_string(), strong);

        // Emphasis character style
        let mut emphasis = CharacterStyle::new("Emphasis");
        emphasis.base.built_in = true;
        emphasis.base.display_name = Some("Emphasis".to_string());
        emphasis.properties.italic = Some(true);
        self.character_styles.insert("Emphasis".to_string(), emphasis);

        // Default page style
        let mut default_page = PageStyle::new("Default");
        default_page.base.built_in = true;
        default_page.base.display_name = Some("Default".to_string());
        default_page.properties.width_twips = Some(12240);  // 8.5"
        default_page.properties.height_twips = Some(15840); // 11"
        default_page.properties.margin_top_twips = Some(1440);    // 1"
        default_page.properties.margin_bottom_twips = Some(1440); // 1"
        default_page.properties.margin_left_twips = Some(1440);   // 1"
        default_page.properties.margin_right_twips = Some(1440);  // 1"
        default_page.properties.header_enabled = Some(false);
        default_page.properties.footer_enabled = Some(false);
        self.page_styles.insert("Default".to_string(), default_page);

        // First Page style
        let mut first_page = PageStyle::new("First Page");
        first_page.base.built_in = true;
        first_page.base.display_name = Some("First Page".to_string());
        first_page.base.next_style = Some("Default".to_string());
        self.page_styles.insert("First Page".to_string(), first_page);

        // Default table style
        let mut default_table = TableStyle::new("Table Normal");
        default_table.base.built_in = true;
        default_table.properties.cell_padding_twips = Some(72); // ~1mm
        self.table_styles.insert("Table Normal".to_string(), default_table);
    }

    /// Look up a paragraph style by name.
    pub fn get_paragraph_style(&self, name: &str) -> Option<&ParagraphStyle> {
        self.paragraph_styles.get(name)
    }

    /// Look up a character style by name.
    pub fn get_character_style(&self, name: &str) -> Option<&CharacterStyle> {
        self.character_styles.get(name)
    }

    /// Look up a page style by name.
    pub fn get_page_style(&self, name: &str) -> Option<&PageStyle> {
        self.page_styles.get(name)
    }

    /// Get all paragraph style names, sorted alphabetically.
    pub fn paragraph_style_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.paragraph_styles.keys().map(|s| s.as_str()).collect();
        names.sort();
        names
    }

    /// Get all character style names, sorted alphabetically.
    pub fn character_style_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.character_styles.keys().map(|s| s.as_str()).collect();
        names.sort();
        names
    }
}
