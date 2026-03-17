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

// ---------------------------------------------------------------------------
// Resolved property types
// ---------------------------------------------------------------------------

/// Paragraph properties with all inheritance resolved to concrete values.
#[derive(Debug, Clone)]
pub struct ResolvedParagraphProperties {
    pub font_family: String,
    pub font_size_pts: f64,
    pub bold: bool,
    pub italic: bool,
    pub color: String,
    pub alignment: String,
    pub indent_left_pts: f64,
    pub indent_right_pts: f64,
    pub indent_first_line_pts: f64,
    pub space_before_pts: f64,
    pub space_after_pts: f64,
    pub line_spacing_multiplier: f64,
    pub keep_with_next: bool,
    pub keep_together: bool,
    pub page_break_before: bool,
    pub outline_level: u8,
    pub background_color: Option<String>,
}

impl Default for ResolvedParagraphProperties {
    fn default() -> Self {
        Self {
            font_family: "Calibri".to_string(),
            font_size_pts: 12.0,
            bold: false,
            italic: false,
            color: "#000000".to_string(),
            alignment: "left".to_string(),
            indent_left_pts: 0.0,
            indent_right_pts: 0.0,
            indent_first_line_pts: 0.0,
            space_before_pts: 0.0,
            space_after_pts: 8.0,
            line_spacing_multiplier: 1.15,
            keep_with_next: false,
            keep_together: false,
            page_break_before: false,
            outline_level: 0,
            background_color: None,
        }
    }
}

/// Character properties with all inheritance resolved to concrete values.
#[derive(Debug, Clone)]
pub struct ResolvedCharacterProperties {
    pub font_family: String,
    pub font_size_pts: f64,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub color: String,
    pub highlight: Option<String>,
    pub superscript: bool,
    pub subscript: bool,
    pub small_caps: bool,
    pub all_caps: bool,
    pub hidden: bool,
    pub language: Option<String>,
}

impl Default for ResolvedCharacterProperties {
    fn default() -> Self {
        Self {
            font_family: "Calibri".to_string(),
            font_size_pts: 12.0,
            bold: false,
            italic: false,
            underline: false,
            strikethrough: false,
            color: "#000000".to_string(),
            highlight: None,
            superscript: false,
            subscript: false,
            small_caps: false,
            all_caps: false,
            hidden: false,
            language: None,
        }
    }
}

// ---------------------------------------------------------------------------
// StyleCatalog
// ---------------------------------------------------------------------------

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
    /// Names of styles shown in the Quick Styles gallery
    #[serde(default)]
    pub quick_style_names: Vec<String>,
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

            let font_size = match level {
                1 => 32, // 16pt
                2 => 26, // 13pt
                3 => 24, // 12pt
                4 => 22, // 11pt
                _ => 20, // 10pt
            };
            style.char_properties.font_size_half_points = Some(font_size);

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
        self.paragraph_styles
            .insert("Subtitle".to_string(), subtitle);

        // Body Text style
        let mut body = ParagraphStyle::new("Body Text");
        body.base.built_in = true;
        body.base.display_name = Some("Body Text".to_string());
        body.base.parent = Some("Normal".to_string());
        body.base.next_style = Some("Body Text".to_string());
        body.properties.space_after_twips = Some(160);
        body.properties.line_spacing_type = Some("multiple".to_string());
        body.properties.line_spacing_value = Some(1.15);
        self.paragraph_styles
            .insert("Body Text".to_string(), body);

        // Quote style
        let mut quote = ParagraphStyle::new("Quote");
        quote.base.built_in = true;
        quote.base.display_name = Some("Quote".to_string());
        quote.base.parent = Some("Normal".to_string());
        quote.char_properties.italic = Some(true);
        quote.properties.indent_left_twips = Some(720);  // 0.5"
        quote.properties.indent_right_twips = Some(720);
        self.paragraph_styles.insert("Quote".to_string(), quote);

        // Caption style
        let mut caption = ParagraphStyle::new("Caption");
        caption.base.built_in = true;
        caption.base.display_name = Some("Caption".to_string());
        caption.base.parent = Some("Normal".to_string());
        caption.char_properties.italic = Some(true);
        caption.char_properties.font_size_half_points = Some(20); // 10pt
        caption.properties.space_before_twips = Some(40);
        caption.properties.space_after_twips = Some(100);
        self.paragraph_styles
            .insert("Caption".to_string(), caption);

        // List Paragraph style
        let mut list_para = ParagraphStyle::new("List Paragraph");
        list_para.base.built_in = true;
        list_para.base.display_name = Some("List Paragraph".to_string());
        list_para.base.parent = Some("Normal".to_string());
        list_para.base.next_style = Some("List Paragraph".to_string());
        list_para.properties.indent_left_twips = Some(720); // 0.5"
        self.paragraph_styles
            .insert("List Paragraph".to_string(), list_para);

        // Default character style
        let mut default_char = CharacterStyle::new("Default Paragraph Font");
        default_char.base.built_in = true;
        self.character_styles
            .insert("Default Paragraph Font".to_string(), default_char);

        // Strong character style
        let mut strong = CharacterStyle::new("Strong");
        strong.base.built_in = true;
        strong.base.display_name = Some("Strong".to_string());
        strong.properties.bold = Some(true);
        self.character_styles
            .insert("Strong".to_string(), strong);

        // Emphasis character style
        let mut emphasis = CharacterStyle::new("Emphasis");
        emphasis.base.built_in = true;
        emphasis.base.display_name = Some("Emphasis".to_string());
        emphasis.properties.italic = Some(true);
        self.character_styles
            .insert("Emphasis".to_string(), emphasis);

        // Hyperlink character style
        let mut hyperlink = CharacterStyle::new("Hyperlink");
        hyperlink.base.built_in = true;
        hyperlink.base.display_name = Some("Hyperlink".to_string());
        hyperlink.properties.color = Some("#0563C1".to_string());
        hyperlink.properties.underline = Some("single".to_string());
        self.character_styles
            .insert("Hyperlink".to_string(), hyperlink);

        // Code character style
        let mut code = CharacterStyle::new("Code");
        code.base.built_in = true;
        code.base.display_name = Some("Code".to_string());
        code.properties.font_family = Some("Consolas".to_string());
        self.character_styles.insert("Code".to_string(), code);

        // Default page style
        let mut default_page = PageStyle::new("Default");
        default_page.base.built_in = true;
        default_page.base.display_name = Some("Default".to_string());
        default_page.properties.width_twips = Some(12240);  // 8.5"
        default_page.properties.height_twips = Some(15840); // 11"
        default_page.properties.margin_top_twips = Some(1440);
        default_page.properties.margin_bottom_twips = Some(1440);
        default_page.properties.margin_left_twips = Some(1440);
        default_page.properties.margin_right_twips = Some(1440);
        default_page.properties.header_enabled = Some(false);
        default_page.properties.footer_enabled = Some(false);
        self.page_styles
            .insert("Default".to_string(), default_page);

        // First Page style
        let mut first_page = PageStyle::new("First Page");
        first_page.base.built_in = true;
        first_page.base.display_name = Some("First Page".to_string());
        first_page.base.next_style = Some("Default".to_string());
        self.page_styles
            .insert("First Page".to_string(), first_page);

        // Envelope page style
        let mut envelope = PageStyle::new("Envelope");
        envelope.base.built_in = true;
        envelope.base.display_name = Some("Envelope".to_string());
        envelope.properties.width_twips = Some(15840);  // 11"
        envelope.properties.height_twips = Some(5760);  // 4"
        self.page_styles.insert("Envelope".to_string(), envelope);

        // Default table style
        let mut default_table = TableStyle::new("Table Normal");
        default_table.base.built_in = true;
        default_table.properties.cell_padding_twips = Some(72);
        self.table_styles
            .insert("Table Normal".to_string(), default_table);

        // Table Grid style
        let mut table_grid = TableStyle::new("Table Grid");
        table_grid.base.built_in = true;
        table_grid.base.display_name = Some("Table Grid".to_string());
        table_grid.base.parent = Some("Table Normal".to_string());
        table_grid.properties.cell_padding_twips = Some(72);
        table_grid.properties.border_style = Some("single".to_string());
        self.table_styles
            .insert("Table Grid".to_string(), table_grid);

        // Quick styles gallery
        self.quick_style_names = vec![
            "Normal".to_string(),
            "Heading 1".to_string(),
            "Heading 2".to_string(),
            "Heading 3".to_string(),
            "Title".to_string(),
            "Subtitle".to_string(),
            "Body Text".to_string(),
            "Quote".to_string(),
            "Caption".to_string(),
            "List Paragraph".to_string(),
        ];
    }

    // -----------------------------------------------------------------------
    // Lookup helpers
    // -----------------------------------------------------------------------

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
        let mut names: Vec<&str> =
            self.paragraph_styles.keys().map(|s| s.as_str()).collect();
        names.sort();
        names
    }

    /// Get all character style names, sorted alphabetically.
    pub fn character_style_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> =
            self.character_styles.keys().map(|s| s.as_str()).collect();
        names.sort();
        names
    }

    // -----------------------------------------------------------------------
    // Style resolution
    // -----------------------------------------------------------------------

    /// Resolve a paragraph style with full inheritance chain.
    ///
    /// Walks parent styles from the named style up to the root, merging
    /// properties at each level (child overrides parent).
    pub fn resolve_paragraph_style(&self, style_name: &str) -> ResolvedParagraphProperties {
        // Collect the inheritance chain (root first, leaf last)
        let chain = self.collect_para_chain(style_name);

        let mut resolved = ResolvedParagraphProperties::default();

        for style in chain {
            // Merge character properties
            if let Some(v) = &style.char_properties.font_family {
                resolved.font_family = v.clone();
            }
            if let Some(hp) = style.char_properties.font_size_half_points {
                resolved.font_size_pts = hp as f64 / 2.0;
            }
            if let Some(v) = style.char_properties.bold {
                resolved.bold = v;
            }
            if let Some(v) = style.char_properties.italic {
                resolved.italic = v;
            }
            if let Some(v) = &style.char_properties.color {
                resolved.color = v.clone();
            }
            // Merge paragraph layout properties
            if let Some(v) = &style.properties.alignment {
                resolved.alignment = v.clone();
            }
            if let Some(v) = style.properties.indent_left_twips {
                resolved.indent_left_pts = v as f64 / 20.0;
            }
            if let Some(v) = style.properties.indent_right_twips {
                resolved.indent_right_pts = v as f64 / 20.0;
            }
            if let Some(v) = style.properties.indent_first_line_twips {
                resolved.indent_first_line_pts = v as f64 / 20.0;
            }
            if let Some(v) = style.properties.space_before_twips {
                resolved.space_before_pts = v as f64 / 20.0;
            }
            if let Some(v) = style.properties.space_after_twips {
                resolved.space_after_pts = v as f64 / 20.0;
            }
            if let Some(v) = style.properties.line_spacing_value {
                resolved.line_spacing_multiplier = v;
            }
            if let Some(v) = style.properties.keep_with_next {
                resolved.keep_with_next = v;
            }
            if let Some(v) = style.properties.keep_together {
                resolved.keep_together = v;
            }
            if let Some(v) = style.properties.page_break_before {
                resolved.page_break_before = v;
            }
            if let Some(v) = style.properties.outline_level {
                resolved.outline_level = v;
            }
            if let Some(v) = &style.properties.background_color {
                resolved.background_color = Some(v.clone());
            }
        }

        resolved
    }

    /// Resolve a character style with full inheritance chain.
    pub fn resolve_character_style(&self, style_name: &str) -> ResolvedCharacterProperties {
        let chain = self.collect_char_chain(style_name);

        let mut resolved = ResolvedCharacterProperties::default();

        for style in chain {
            if let Some(v) = &style.properties.font_family {
                resolved.font_family = v.clone();
            }
            if let Some(hp) = style.properties.font_size_half_points {
                resolved.font_size_pts = hp as f64 / 2.0;
            }
            if let Some(v) = style.properties.bold {
                resolved.bold = v;
            }
            if let Some(v) = style.properties.italic {
                resolved.italic = v;
            }
            if let Some(v) = &style.properties.underline {
                resolved.underline = v != "none";
            }
            if let Some(v) = &style.properties.strikethrough {
                resolved.strikethrough = v != "none";
            }
            if let Some(v) = &style.properties.color {
                resolved.color = v.clone();
            }
            if let Some(v) = &style.properties.highlight {
                resolved.highlight = Some(v.clone());
            }
            if let Some(v) = style.properties.superscript {
                resolved.superscript = v;
            }
            if let Some(v) = style.properties.subscript {
                resolved.subscript = v;
            }
            if let Some(v) = style.properties.small_caps {
                resolved.small_caps = v;
            }
            if let Some(v) = style.properties.all_caps {
                resolved.all_caps = v;
            }
            if let Some(v) = style.properties.hidden {
                resolved.hidden = v;
            }
            if let Some(v) = &style.properties.language {
                resolved.language = Some(v.clone());
            }
        }

        resolved
    }

    /// Return the names of styles that belong to the quick style gallery.
    pub fn get_quick_styles(&self) -> Vec<&str> {
        self.quick_style_names
            .iter()
            .filter(|n| self.paragraph_styles.contains_key(n.as_str()))
            .map(|n| n.as_str())
            .collect()
    }

    /// Update theme-dependent colors in all styles that use theme color slots.
    pub fn apply_theme(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        // Update hyperlink color in character style
        if let Some(hl) = self.character_styles.get_mut("Hyperlink") {
            hl.properties.color = Some(theme.colors.hyperlink.clone());
        }
        // Update accent color on heading styles
        if let Some(h1) = self.paragraph_styles.get_mut("Heading 1") {
            h1.char_properties.color = Some(theme.colors.accent1.clone());
        }
        if let Some(h2) = self.paragraph_styles.get_mut("Heading 2") {
            h2.char_properties.color = Some(theme.colors.accent2.clone());
        }
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    /// Build the parent-chain for a paragraph style, root first.
    fn collect_para_chain<'a>(&'a self, name: &str) -> Vec<&'a ParagraphStyle> {
        let mut chain: Vec<&ParagraphStyle> = Vec::new();
        let mut current_name = name.to_string();
        let mut visited = std::collections::HashSet::new();

        loop {
            if visited.contains(&current_name) {
                break; // Cycle guard
            }
            visited.insert(current_name.clone());

            match self.paragraph_styles.get(&current_name) {
                None => break,
                Some(style) => {
                    chain.push(style);
                    match &style.base.parent {
                        None => break,
                        Some(p) => current_name = p.clone(),
                    }
                }
            }
        }

        chain.reverse(); // Root first
        chain
    }

    /// Build the parent-chain for a character style, root first.
    fn collect_char_chain<'a>(&'a self, name: &str) -> Vec<&'a CharacterStyle> {
        let mut chain: Vec<&CharacterStyle> = Vec::new();
        let mut current_name = name.to_string();
        let mut visited = std::collections::HashSet::new();

        loop {
            if visited.contains(&current_name) {
                break;
            }
            visited.insert(current_name.clone());

            match self.character_styles.get(&current_name) {
                None => break,
                Some(style) => {
                    chain.push(style);
                    match &style.base.parent {
                        None => break,
                        Some(p) => current_name = p.clone(),
                    }
                }
            }
        }

        chain.reverse();
        chain
    }
}
