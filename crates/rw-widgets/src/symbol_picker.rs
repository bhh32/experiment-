//! Symbol/special character picker dialog.

/// A special character or symbol.
#[derive(Debug, Clone)]
pub struct Symbol {
    /// Unicode codepoint
    pub codepoint: u32,
    /// Display character
    pub character: char,
    /// Unicode name / description
    pub name: String,
    /// Category this symbol belongs to
    pub category: SymbolCategory,
}

impl Symbol {
    pub fn new(codepoint: u32, name: impl Into<String>, category: SymbolCategory) -> Self {
        let character = char::from_u32(codepoint).unwrap_or('\u{FFFD}');
        Self {
            codepoint,
            character,
            name: name.into(),
            category,
        }
    }
}

/// Categories of symbols.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolCategory {
    /// Common punctuation and typographic symbols
    General,
    /// Currency symbols
    Currency,
    /// Mathematical operators and symbols
    Mathematics,
    /// Arrow symbols
    Arrows,
    /// Geometric shapes
    Shapes,
    /// Letterlike symbols (©, ™, etc.)
    Letterlike,
    /// Latin extended characters
    LatinExtended,
    /// Greek and Coptic
    Greek,
    /// Emoji and miscellaneous
    Emoji,
}

impl SymbolCategory {
    pub fn label(&self) -> &'static str {
        match self {
            Self::General => "General",
            Self::Currency => "Currency",
            Self::Mathematics => "Mathematics",
            Self::Arrows => "Arrows",
            Self::Shapes => "Shapes",
            Self::Letterlike => "Letterlike",
            Self::LatinExtended => "Latin Extended",
            Self::Greek => "Greek",
            Self::Emoji => "Emoji",
        }
    }

    pub fn all() -> &'static [SymbolCategory] {
        &[
            Self::General,
            Self::Currency,
            Self::Mathematics,
            Self::Arrows,
            Self::Shapes,
            Self::Letterlike,
            Self::LatinExtended,
            Self::Greek,
            Self::Emoji,
        ]
    }
}

/// Symbol picker state.
#[derive(Debug, Clone)]
pub struct SymbolPickerState {
    /// All available symbols
    pub symbols: Vec<Symbol>,
    /// Recent symbols (most recently inserted first)
    pub recent: Vec<Symbol>,
    /// Maximum recent symbols to display
    pub max_recent: usize,
    /// Currently selected category
    pub category: SymbolCategory,
    /// Currently selected / hovered symbol codepoint
    pub selected: Option<u32>,
    /// Search filter string
    pub filter: String,
    /// Font to use for symbol display (usually "Segoe UI Symbol" or similar)
    pub font: String,
}

impl Default for SymbolPickerState {
    fn default() -> Self {
        Self {
            symbols: default_symbols(),
            recent: Vec::new(),
            max_recent: 16,
            category: SymbolCategory::General,
            selected: None,
            filter: String::new(),
            font: "DejaVu Sans".to_string(),
        }
    }
}

impl SymbolPickerState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Switch to a symbol category.
    pub fn set_category(&mut self, cat: SymbolCategory) {
        self.category = cat;
        self.selected = None;
    }

    /// Select a symbol by codepoint and add to recent.
    pub fn select(&mut self, codepoint: u32) {
        self.selected = Some(codepoint);
    }

    /// Insert the currently selected symbol, recording it as recently used.
    /// Returns the character if one was selected.
    pub fn insert_selected(&mut self) -> Option<char> {
        let cp = self.selected?;
        let ch = char::from_u32(cp)?;

        // Find the symbol and push to recent
        if let Some(sym) = self.symbols.iter().find(|s| s.codepoint == cp).cloned() {
            self.recent.retain(|r| r.codepoint != cp);
            self.recent.insert(0, sym);
            if self.recent.len() > self.max_recent {
                self.recent.truncate(self.max_recent);
            }
        }

        Some(ch)
    }

    /// Return symbols for the current category matching the filter.
    pub fn visible_symbols(&self) -> Vec<&Symbol> {
        self.symbols
            .iter()
            .filter(|s| s.category == self.category)
            .filter(|s| {
                self.filter.is_empty()
                    || s.name.to_lowercase().contains(&self.filter.to_lowercase())
            })
            .collect()
    }
}

/// Build the default symbol list for common categories.
pub fn default_symbols() -> Vec<Symbol> {
    let mut syms: Vec<Symbol> = Vec::new();

    // General punctuation & typographic
    let general: &[(u32, &str)] = &[
        (0x00A9, "Copyright sign"),
        (0x00AE, "Registered sign"),
        (0x2122, "Trade mark sign"),
        (0x00B0, "Degree sign"),
        (0x00B1, "Plus-minus sign"),
        (0x00B5, "Micro sign"),
        (0x00B6, "Pilcrow sign"),
        (0x00B7, "Middle dot"),
        (0x00BD, "Vulgar fraction one half"),
        (0x00BC, "Vulgar fraction one quarter"),
        (0x00BE, "Vulgar fraction three quarters"),
        (0x2013, "En dash"),
        (0x2014, "Em dash"),
        (0x2018, "Left single quotation mark"),
        (0x2019, "Right single quotation mark"),
        (0x201C, "Left double quotation mark"),
        (0x201D, "Right double quotation mark"),
        (0x2026, "Horizontal ellipsis"),
        (0x00A0, "No-break space"),
        (0x00AD, "Soft hyphen"),
    ];
    for &(cp, name) in general {
        syms.push(Symbol::new(cp, name, SymbolCategory::General));
    }

    // Currency
    let currency: &[(u32, &str)] = &[
        (0x0024, "Dollar sign"),
        (0x00A2, "Cent sign"),
        (0x00A3, "Pound sign"),
        (0x00A4, "Currency sign"),
        (0x00A5, "Yen sign"),
        (0x20AC, "Euro sign"),
        (0x20A3, "French franc sign"),
        (0x20A4, "Lira sign"),
        (0x20B9, "Indian rupee sign"),
        (0x20BF, "Bitcoin sign"),
    ];
    for &(cp, name) in currency {
        syms.push(Symbol::new(cp, name, SymbolCategory::Currency));
    }

    // Mathematics
    let maths: &[(u32, &str)] = &[
        (0x00D7, "Multiplication sign"),
        (0x00F7, "Division sign"),
        (0x2200, "For all"),
        (0x2202, "Partial differential"),
        (0x2203, "There exists"),
        (0x2205, "Empty set"),
        (0x2207, "Nabla"),
        (0x2208, "Element of"),
        (0x2211, "N-ary summation"),
        (0x221A, "Square root"),
        (0x221E, "Infinity"),
        (0x222B, "Integral"),
        (0x2248, "Almost equal to"),
        (0x2260, "Not equal to"),
        (0x2264, "Less-than or equal to"),
        (0x2265, "Greater-than or equal to"),
        (0x03C0, "Greek small letter pi"),
    ];
    for &(cp, name) in maths {
        syms.push(Symbol::new(cp, name, SymbolCategory::Mathematics));
    }

    // Arrows
    let arrows: &[(u32, &str)] = &[
        (0x2190, "Leftwards arrow"),
        (0x2191, "Upwards arrow"),
        (0x2192, "Rightwards arrow"),
        (0x2193, "Downwards arrow"),
        (0x2194, "Left right arrow"),
        (0x21D0, "Leftwards double arrow"),
        (0x21D2, "Rightwards double arrow"),
        (0x21D4, "Left right double arrow"),
        (0x2713, "Check mark"),
        (0x2714, "Heavy check mark"),
        (0x2717, "Ballot x"),
        (0x2718, "Heavy ballot x"),
    ];
    for &(cp, name) in arrows {
        syms.push(Symbol::new(cp, name, SymbolCategory::Arrows));
    }

    // Greek
    let greek: &[(u32, &str)] = &[
        (0x0391, "Greek capital alpha"),
        (0x0392, "Greek capital beta"),
        (0x0393, "Greek capital gamma"),
        (0x0394, "Greek capital delta"),
        (0x03B1, "Greek small alpha"),
        (0x03B2, "Greek small beta"),
        (0x03B3, "Greek small gamma"),
        (0x03B4, "Greek small delta"),
        (0x03B5, "Greek small epsilon"),
        (0x03B8, "Greek small theta"),
        (0x03BB, "Greek small lambda"),
        (0x03BC, "Greek small mu"),
        (0x03C3, "Greek small sigma"),
        (0x03C9, "Greek small omega"),
    ];
    for &(cp, name) in greek {
        syms.push(Symbol::new(cp, name, SymbolCategory::Greek));
    }

    syms
}
