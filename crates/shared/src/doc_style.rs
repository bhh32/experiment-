/// Document-level style settings shared between DOCX export and preview rendering.
/// Both pipelines use these exact same calculations so the preview matches the export.
#[derive(Debug, Clone)]
pub struct DocStyle {
    pub body_font: String,
    pub body_size_pt: f32,
    pub line_spacing: f32, // multiplier: 1.0 = single, 1.5, 2.0 = double
    pub code_font: String,
}

impl Default for DocStyle {
    fn default() -> Self {
        Self {
            body_font: "Calibri".into(),
            body_size_pt: 11.0,
            line_spacing: 1.15,
            code_font: "Consolas".into(),
        }
    }
}

impl DocStyle {
    pub fn new(font: &str, line_spacing: f32) -> Self {
        Self {
            body_font: font.into(),
            body_size_pt: 11.0,
            line_spacing,
            code_font: "Consolas".into(),
        }
    }

    // --- Sizes in half-points (for docx-rs) ---

    pub fn body_size_half_pts(&self) -> usize {
        (self.body_size_pt * 2.0) as usize
    }

    pub fn heading1_half_pts(&self) -> usize {
        ((self.body_size_pt * 2.36) * 2.0) as usize
    }

    pub fn heading2_half_pts(&self) -> usize {
        ((self.body_size_pt * 1.82) * 2.0) as usize
    }

    pub fn heading3_half_pts(&self) -> usize {
        ((self.body_size_pt * 1.27) * 2.0) as usize
    }

    pub fn heading4_half_pts(&self) -> usize {
        self.body_size_half_pts()
    }

    pub fn code_size_half_pts(&self) -> usize {
        ((self.body_size_pt * 0.91) * 2.0) as usize
    }

    pub fn line_spacing_twips(&self) -> i32 {
        (self.line_spacing * 240.0) as i32
    }

    // --- Sizes in pt (for CSS preview) ---

    pub fn heading1_pt(&self) -> f32 {
        self.body_size_pt * 2.36
    }

    pub fn heading2_pt(&self) -> f32 {
        self.body_size_pt * 1.82
    }

    pub fn heading3_pt(&self) -> f32 {
        self.body_size_pt * 1.27
    }

    pub fn heading4_pt(&self) -> f32 {
        self.body_size_pt
    }

    pub fn code_size_pt(&self) -> f32 {
        self.body_size_pt * 0.91
    }

    // --- Spacing in pt (for CSS) ---

    /// Paragraph spacing after, in pt (8pt default for 11pt body)
    pub fn para_after_pt(&self) -> f32 {
        8.0
    }

    /// Heading spacing before, in pt
    pub fn heading_before_pt(&self) -> f32 {
        12.0
    }

    /// Heading spacing after, in pt
    pub fn heading_after_pt(&self) -> f32 {
        6.0
    }
}
