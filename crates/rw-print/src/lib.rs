//! # rw-print
//!
//! Printing and PDF generation for Rust Writer.
//!
//! ## PDF Export
//! - High-quality PDF generation with embedded fonts
//! - PDF/A compliance option
//! - Hyperlink preservation
//! - Table of contents bookmark generation
//! - Image optimization (JPEG compression, downsampling)
//! - Page range selection
//! - Password protection
//!
//! ## System Printing
//! - CUPS integration on Linux
//! - System print dialog integration
//! - Page range, copies, duplex, paper tray selection
//! - Print preview

/// PDF export options.
#[derive(Debug, Clone)]
pub struct PdfExportOptions {
    /// Page range (None = all pages)
    pub page_range: Option<PageRange>,
    /// Embed fonts in the PDF
    pub embed_fonts: bool,
    /// PDF/A compliance
    pub pdf_a: bool,
    /// JPEG quality for images (0-100)
    pub image_quality: u8,
    /// Maximum image DPI (images above this are downsampled)
    pub max_image_dpi: u32,
    /// Include hyperlinks
    pub include_hyperlinks: bool,
    /// Include bookmarks (from headings)
    pub include_bookmarks: bool,
    /// Document title in PDF metadata
    pub title: Option<String>,
    /// Author in PDF metadata
    pub author: Option<String>,
    /// Password protection
    pub password: Option<String>,
}

impl Default for PdfExportOptions {
    fn default() -> Self {
        Self {
            page_range: None,
            embed_fonts: true,
            pdf_a: false,
            image_quality: 90,
            max_image_dpi: 300,
            include_hyperlinks: true,
            include_bookmarks: true,
            title: None,
            author: None,
            password: None,
        }
    }
}

/// Print options for system printing.
#[derive(Debug, Clone)]
pub struct PrintOptions {
    /// Page range
    pub page_range: Option<PageRange>,
    /// Number of copies
    pub copies: u32,
    /// Collate copies
    pub collate: bool,
    /// Duplex printing
    pub duplex: DuplexMode,
    /// Paper size
    pub paper_size: Option<String>,
    /// Paper tray/source
    pub paper_tray: Option<String>,
    /// Color mode
    pub color_mode: ColorMode,
    /// Pages per sheet
    pub pages_per_sheet: u32,
    /// Print in reverse order
    pub reverse: bool,
}

impl Default for PrintOptions {
    fn default() -> Self {
        Self {
            page_range: None,
            copies: 1,
            collate: true,
            duplex: DuplexMode::None,
            paper_size: None,
            paper_tray: None,
            color_mode: ColorMode::Color,
            pages_per_sheet: 1,
            reverse: false,
        }
    }
}

/// A page range specification.
#[derive(Debug, Clone)]
pub enum PageRange {
    /// All pages
    All,
    /// Specific pages (e.g., [1, 3, 5])
    Pages(Vec<usize>),
    /// A range of pages (start..=end)
    Range(usize, usize),
    /// Current page only
    Current,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DuplexMode {
    None,
    LongEdge,
    ShortEdge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    Color,
    Grayscale,
    BlackAndWhite,
}
