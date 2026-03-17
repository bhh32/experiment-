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

use std::path::Path;
use rw_document::Document;
use rw_layout::LayoutResult;
use thiserror::Error;

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

/// Settings for a single print job.
#[derive(Debug, Clone)]
pub struct PrintSettings {
    /// Number of copies to print
    pub copies: u32,
    /// Page range (None = all)
    pub page_range: Option<PageRange>,
    /// Page orientation
    pub orientation: Orientation,
    /// Paper size name (e.g. "A4", "Letter")
    pub paper_size: String,
}

impl Default for PrintSettings {
    fn default() -> Self {
        Self {
            copies: 1,
            page_range: None,
            orientation: Orientation::Portrait,
            paper_size: "A4".to_string(),
        }
    }
}

/// Page orientation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    Portrait,
    Landscape,
}

/// Error type for print operations.
#[derive(Debug, Error)]
pub enum PrintError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("PDF generation error: {0}")]
    PdfGeneration(String),
    #[error("No pages to print")]
    NoPages,
    #[error("Invalid page range")]
    InvalidPageRange,
}

/// The main print manager.
#[derive(Debug, Default)]
pub struct PrintManager;

impl PrintManager {
    pub fn new() -> Self {
        Self
    }

    /// Generate a basic PDF file from the document and its layout.
    ///
    /// This implementation writes a minimal valid PDF skeleton. Full
    /// rendering of text and images requires a complete PDF rendering
    /// pipeline that is beyond the scope of this stub.
    pub fn print_to_pdf(
        &self,
        doc: &Document,
        layout: &LayoutResult,
        path: &Path,
    ) -> Result<(), PrintError> {
        use std::io::Write;

        if layout.page_count == 0 {
            return Err(PrintError::NoPages);
        }

        let title = doc.metadata.title.as_deref().unwrap_or("Document");
        let author = doc.metadata.author.as_deref().unwrap_or("");
        let page_count = layout.page_count;

        // Build a minimal PDF structure
        let mut pdf = Vec::new();

        // Header
        writeln!(pdf, "%PDF-1.4")?;

        // Object offsets for the cross-reference table
        let mut offsets: Vec<u64> = Vec::new();

        // Object 1: Catalog
        offsets.push(pdf.len() as u64);
        writeln!(pdf, "1 0 obj")?;
        writeln!(pdf, "<< /Type /Catalog /Pages 2 0 R >>")?;
        writeln!(pdf, "endobj")?;

        // Object 2: Pages
        offsets.push(pdf.len() as u64);
        writeln!(pdf, "2 0 obj")?;
        write!(pdf, "<< /Type /Pages /Kids [")?;
        for i in 0..page_count {
            write!(pdf, "{} 0 R ", i + 3)?;
        }
        writeln!(pdf, "] /Count {} >>", page_count)?;
        writeln!(pdf, "endobj")?;

        // Objects 3..N: One Page object per page
        for _i in 0..page_count {
            offsets.push(pdf.len() as u64);
            let obj_num = offsets.len();
            writeln!(pdf, "{} 0 obj", obj_num)?;
            writeln!(
                pdf,
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] >>"
            )?;
            writeln!(pdf, "endobj")?;
        }

        // Info dictionary
        offsets.push(pdf.len() as u64);
        let info_obj = offsets.len();
        writeln!(pdf, "{} 0 obj", info_obj)?;
        writeln!(
            pdf,
            "<< /Title ({}) /Author ({}) >>",
            pdf_escape(title),
            pdf_escape(author)
        )?;
        writeln!(pdf, "endobj")?;

        // Cross-reference table
        let xref_offset = pdf.len() as u64;
        writeln!(pdf, "xref")?;
        writeln!(pdf, "0 {}", offsets.len() + 1)?;
        writeln!(pdf, "0000000000 65535 f ")?;
        for offset in &offsets {
            writeln!(pdf, "{:010} 00000 n ", offset)?;
        }

        // Trailer
        writeln!(pdf, "trailer")?;
        writeln!(
            pdf,
            "<< /Size {} /Root 1 0 R /Info {} 0 R >>",
            offsets.len() + 1,
            info_obj
        )?;
        writeln!(pdf, "startxref")?;
        writeln!(pdf, "{}", xref_offset)?;
        writeln!(pdf, "%%EOF")?;

        // Write to file
        let mut file = std::fs::File::create(path)?;
        file.write_all(&pdf)?;

        Ok(())
    }
}

/// Escape a string for PDF string literal syntax.
fn pdf_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('(', "\\(")
        .replace(')', "\\)")
}
