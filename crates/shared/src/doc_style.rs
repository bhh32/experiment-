/// Document-level style settings shared between DOCX export and preview rendering.
/// Both pipelines use these exact same calculations so the preview matches the export.
#[derive(Debug, Clone)]
pub struct DocStyle {
    pub body_font: String,
    pub body_size_pt: f32,
    pub line_spacing: f32, // multiplier: 1.0 = single, 1.5, 2.0 = double
    pub code_font: String,
    /// Page width in inches (US Letter = 8.5)
    pub page_width_in: f32,
    /// Page height in inches (US Letter = 11.0)
    pub page_height_in: f32,
    /// Margin in inches (all sides)
    pub margin_in: f32,
}

impl Default for DocStyle {
    fn default() -> Self {
        Self::academic()
    }
}

impl DocStyle {
    /// Academic defaults: Times New Roman, 12pt, single-spaced, 1" margins, US Letter
    /// Line spacing changes are intentional from the user.
    pub fn academic() -> Self {
        Self {
            body_font: "Times New Roman".into(),
            body_size_pt: 12.0,
            line_spacing: 1.0,
            code_font: "Courier New".into(),
            page_width_in: 8.5,
            page_height_in: 11.0,
            margin_in: 1.0,
        }
    }

    /// Business/modern defaults: Calibri, 11pt, single spacing
    pub fn business() -> Self {
        Self {
            body_font: "Calibri".into(),
            body_size_pt: 11.0,
            line_spacing: 1.0,
            code_font: "Consolas".into(),
            page_width_in: 8.5,
            page_height_in: 11.0,
            margin_in: 1.0,
        }
    }

    pub fn new(font: &str, line_spacing: f32) -> Self {
        Self {
            body_font: font.into(),
            line_spacing,
            ..Self::academic()
        }
    }

    // --- Sizes in half-points (for docx-rs w:sz) ---

    pub fn body_size_half_pts(&self) -> usize {
        (self.body_size_pt * 2.0) as usize
    }

    pub fn heading1_half_pts(&self) -> usize {
        ((self.body_size_pt * 2.0) * 2.0) as usize // 2x body
    }

    pub fn heading2_half_pts(&self) -> usize {
        ((self.body_size_pt * 1.5) * 2.0) as usize // 1.5x body
    }

    pub fn heading3_half_pts(&self) -> usize {
        ((self.body_size_pt * 1.17) * 2.0) as usize // 1.17x body
    }

    pub fn heading4_half_pts(&self) -> usize {
        self.body_size_half_pts()
    }

    pub fn code_size_half_pts(&self) -> usize {
        ((self.body_size_pt * 0.83) * 2.0) as usize
    }

    /// Line spacing in twips (240 twips = 1 line at single spacing)
    pub fn line_spacing_twips(&self) -> i32 {
        (self.line_spacing * 240.0) as i32
    }

    // --- Sizes in twips (for docx-rs page layout: 1 inch = 1440 twips) ---

    pub fn page_width_twips(&self) -> u32 {
        (self.page_width_in * 1440.0) as u32
    }

    pub fn page_height_twips(&self) -> u32 {
        (self.page_height_in * 1440.0) as u32
    }

    pub fn margin_twips(&self) -> i32 {
        (self.margin_in * 1440.0) as i32
    }

    // --- Sizes in pt (for CSS preview) ---

    pub fn heading1_pt(&self) -> f32 {
        self.body_size_pt * 2.0
    }

    pub fn heading2_pt(&self) -> f32 {
        self.body_size_pt * 1.5
    }

    pub fn heading3_pt(&self) -> f32 {
        self.body_size_pt * 1.17
    }

    pub fn heading4_pt(&self) -> f32 {
        self.body_size_pt
    }

    pub fn code_size_pt(&self) -> f32 {
        self.body_size_pt * 0.83
    }

    /// Paragraph spacing after, in pt
    pub fn para_after_pt(&self) -> f32 {
        if self.line_spacing >= 2.0 {
            0.0 // Double-spaced papers don't add extra after-paragraph spacing
        } else {
            8.0
        }
    }

    /// Heading spacing before, in pt
    pub fn heading_before_pt(&self) -> f32 {
        12.0
    }

    /// Heading spacing after, in pt
    pub fn heading_after_pt(&self) -> f32 {
        if self.line_spacing >= 2.0 {
            0.0 // Double-spaced: headings use line spacing, not extra after
        } else {
            6.0
        }
    }
}
