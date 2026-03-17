use crate::line::{GlyphRun, LayoutLine, PositionedGlyph};
use crate::page::LayoutPage;
use crate::result::LayoutResult;
use crate::LayoutRect;
use rw_document::document::DocumentDefaults;
use rw_document::inline::Inline;
use rw_document::paragraph::Paragraph;
use rw_document::{Block, Document, ElementId};
use rw_styles::catalog::StyleCatalog;

/// The main layout engine.
///
/// Takes a document and computes the complete page layout.
/// The layout engine is designed to be incremental — when the
/// document changes, only affected pages are re-laid-out.
pub struct LayoutEngine {
    /// DPI for pixel conversion
    pub dpi: f64,
    /// Whether to enable hyphenation
    pub hyphenation: bool,
}

// ---------------------------------------------------------------------------
// Internal word-chunk type used during line breaking
// ---------------------------------------------------------------------------
struct WordChunk {
    text: String,
    run_id: ElementId,
    font_size: f64,
    bold: bool,
    italic: bool,
    color: String,
}

impl LayoutEngine {
    pub fn new() -> Self {
        Self {
            dpi: 96.0,
            hyphenation: true,
        }
    }

    /// Perform a full layout of the document.
    pub fn layout(&self, document: &Document) -> LayoutResult {
        self.layout_with_styles(document, &StyleCatalog::with_defaults())
    }

    /// Perform a full layout of the document with a given style catalog.
    pub fn layout_with_styles(
        &self,
        document: &Document,
        _catalog: &StyleCatalog,
    ) -> LayoutResult {
        let defaults = &document.defaults;
        let mut pages: Vec<LayoutPage> = Vec::new();

        for section in &document.sections {
            let page_props = section.page_properties.clone();
            let mut current_page = LayoutPage::new(pages.len(), page_props.clone());
            let mut cursor_y = current_page.content_area.y;
            let available_width = current_page.content_area.width;
            let content_x = current_page.content_area.x;
            let content_bottom =
                current_page.content_area.y + current_page.content_area.height;

            for block in &section.content {
                match block {
                    Block::Paragraph(para) => {
                        // Forced page break before paragraph
                        if para.properties.page_break_before == Some(true)
                            && !pages.is_empty()
                        {
                            pages.push(current_page);
                            current_page =
                                LayoutPage::new(pages.len(), page_props.clone());
                            cursor_y = current_page.content_area.y;
                        }

                        // Check for explicit page-break inline
                        let has_page_break = para.content.iter().any(|i| {
                            matches!(
                                i,
                                Inline::Break(rw_document::inline::BreakType::Page)
                            )
                        });

                        let lines =
                            self.layout_paragraph(para, available_width, defaults);

                        let para_height: f64 =
                            lines.iter().map(|l| l.bounds.height).sum();

                        let space_before = para
                            .properties
                            .space_before
                            .map(|t| t.to_points())
                            .unwrap_or(0.0);
                        let space_after = para
                            .properties
                            .space_after
                            .map(|t| t.to_points())
                            .or_else(|| {
                                defaults
                                    .default_para_properties
                                    .space_after
                                    .map(|t| t.to_points())
                            })
                            .unwrap_or(8.0);

                        cursor_y += space_before;

                        // Page break if the whole paragraph doesn't fit
                        if cursor_y + para_height > content_bottom
                            && !current_page.lines.is_empty()
                        {
                            pages.push(current_page);
                            current_page =
                                LayoutPage::new(pages.len(), page_props.clone());
                            cursor_y = current_page.content_area.y;
                        }

                        // Place each line, potentially spanning pages
                        for mut line in lines {
                            line.bounds.x = content_x
                                + para
                                    .properties
                                    .indent_left
                                    .map(|t| t.to_points())
                                    .unwrap_or(0.0);
                            line.bounds.y = cursor_y;

                            // Reposition runs within the line
                            let mut run_x = line.bounds.x;
                            for run in &mut line.runs {
                                run.bounds.x = run_x;
                                run.bounds.y = cursor_y;
                                run_x += run.bounds.width;
                            }

                            cursor_y += line.bounds.height;
                            current_page.lines.push(line);

                            // Overflow: push page and start new one
                            if cursor_y > content_bottom {
                                pages.push(current_page);
                                current_page = LayoutPage::new(
                                    pages.len(),
                                    page_props.clone(),
                                );
                                cursor_y = current_page.content_area.y;
                            }
                        }

                        cursor_y += space_after;

                        if has_page_break {
                            pages.push(current_page);
                            current_page =
                                LayoutPage::new(pages.len(), page_props.clone());
                            cursor_y = current_page.content_area.y;
                        }
                    }
                    // Tables, frames, horizontal rules, etc. are skipped for now
                    _ => {}
                }
            }

            pages.push(current_page);
        }

        let page_count = pages.len();
        LayoutResult { pages, page_count }
    }

    /// Lay out a single paragraph into a sequence of `LayoutLine`s.
    ///
    /// Measures each text run, breaks at word boundaries that exceed
    /// `available_width`, and produces positioned lines (y=0 relative;
    /// the caller repositions them).
    pub fn layout_paragraph(
        &self,
        para: &Paragraph,
        available_width: f64,
        defaults: &DocumentDefaults,
    ) -> Vec<LayoutLine> {
        // Resolve effective font size (in points)
        let default_font_size = defaults
            .default_char_properties
            .font_size
            .map(|hp| hp as f64 / 2.0)
            .unwrap_or(12.0);

        let first_line_extra = para
            .properties
            .indent_first_line
            .map(|t| t.to_points())
            .unwrap_or(0.0);

        // Collect word chunks from inline content
        let mut chunks: Vec<WordChunk> = Vec::new();
        for inline in &para.content {
            match inline {
                Inline::Text(run) => {
                    let size = run
                        .properties
                        .font_size
                        .map(|hp| hp as f64 / 2.0)
                        .unwrap_or(default_font_size);
                    let bold = run.properties.bold.unwrap_or(false);
                    let italic = run.properties.italic.unwrap_or(false);
                    let color = run
                        .properties
                        .color
                        .map(|c| format!("#{:02X}{:02X}{:02X}", c.r, c.g, c.b))
                        .unwrap_or_else(|| "#000000".to_string());
                    for word in split_words(&run.text) {
                        if !word.is_empty() {
                            chunks.push(WordChunk {
                                text: word,
                                run_id: run.id,
                                font_size: size,
                                bold,
                                italic,
                                color: color.clone(),
                            });
                        }
                    }
                }
                Inline::Tab => {
                    chunks.push(WordChunk {
                        text: "\t".to_string(),
                        run_id: ElementId::new(),
                        font_size: default_font_size,
                        bold: false,
                        italic: false,
                        color: "#000000".to_string(),
                    });
                }
                _ => {}
            }
        }

        if chunks.is_empty() {
            // Empty paragraph: one blank line
            let line_height = default_font_size * 1.2;
            return vec![LayoutLine {
                bounds: LayoutRect::new(0.0, 0.0, available_width, line_height),
                runs: Vec::new(),
                baseline: default_font_size * 0.8,
                paragraph_id: para.id,
                line_in_paragraph: 0,
            }];
        }

        // Greedy line-breaking
        let mut lines: Vec<LayoutLine> = Vec::new();
        let mut line_start: usize = 0;
        let mut line_width = 0.0f64;
        let mut line_index: usize = 0;

        for (i, chunk) in chunks.iter().enumerate() {
            let word_width = Self::estimate_text_width(&chunk.text, chunk.font_size);
            let indent = if i == 0 { first_line_extra } else { 0.0 };
            let effective_width = available_width - indent;

            if line_width + word_width > effective_width && i > line_start {
                // Flush current line
                let line = build_line(
                    line_index,
                    para.id,
                    &chunks[line_start..i],
                    available_width,
                    para.properties
                        .default_char_props
                        .as_ref()
                        .and_then(|cp| cp.font_size)
                        .map(|hp| hp as f64 / 2.0)
                        .unwrap_or(default_font_size),
                );
                lines.push(line);
                line_start = i;
                line_width = 0.0;
                line_index += 1;
            }

            line_width += word_width;
        }

        // Flush trailing chunks
        if line_start < chunks.len() {
            let line = build_line(
                line_index,
                para.id,
                &chunks[line_start..],
                available_width,
                default_font_size,
            );
            lines.push(line);
        }

        lines
    }

    /// Estimate the point width of a text string.
    ///
    /// Simple approximation: `0.6 × font_size` per character.
    pub fn estimate_text_width(text: &str, font_size: f64) -> f64 {
        text.chars().count() as f64 * font_size * 0.6
    }

    /// Break a text string into lines that fit within `max_width`.
    ///
    /// Uses greedy word-boundary breaking.
    pub fn break_text_into_lines(text: &str, max_width: f64, font_size: f64) -> Vec<String> {
        let words = split_words(text);
        let mut lines: Vec<String> = Vec::new();
        let mut current = String::new();
        let mut current_width = 0.0f64;

        for word in words {
            let w = Self::estimate_text_width(&word, font_size);
            if current_width + w > max_width && !current.is_empty() {
                lines.push(current.trim_end().to_string());
                current = word;
                current_width = w;
            } else {
                current.push_str(&word);
                current_width += w;
            }
        }

        if !current.trim().is_empty() {
            lines.push(current.trim_end().to_string());
        }

        if lines.is_empty() {
            lines.push(String::new());
        }

        lines
    }
}

impl Default for LayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Split `text` into word tokens, preserving trailing spaces as part of
/// each token so that widths are measured correctly.
fn split_words(text: &str) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    let mut current = String::new();

    for ch in text.chars() {
        match ch {
            ' ' | '\t' => {
                current.push(ch);
                result.push(std::mem::take(&mut current));
            }
            '\n' => {
                if !current.is_empty() {
                    result.push(std::mem::take(&mut current));
                }
                result.push("\n".to_string());
            }
            _ => current.push(ch),
        }
    }

    if !current.is_empty() {
        result.push(current);
    }

    result
}

/// Build a `LayoutLine` from a slice of word chunks.
///
/// Line is positioned at (0, 0); the caller moves it to the correct page
/// position later.
fn build_line(
    line_index: usize,
    para_id: ElementId,
    chunks: &[WordChunk],
    available_width: f64,
    default_font_size: f64,
) -> LayoutLine {
    // Derive line metrics from the largest font in the line
    let max_font = chunks
        .iter()
        .map(|c| c.font_size)
        .fold(default_font_size, f64::max);
    let line_height = max_font * 1.2;
    let baseline = max_font * 0.8;

    let mut runs: Vec<GlyphRun> = Vec::new();
    let mut x_offset = 0.0f64;

    for chunk in chunks {
        let width = LayoutEngine::estimate_text_width(&chunk.text, chunk.font_size);
        let char_count = chunk.text.chars().count();
        let char_advance = if char_count > 0 {
            width / char_count as f64
        } else {
            0.0
        };

        let glyphs: Vec<PositionedGlyph> = chunk
            .text
            .char_indices()
            .enumerate()
            .map(|(i, (byte_off, _))| PositionedGlyph {
                glyph_id: 0,
                x: i as f64 * char_advance,
                y: 0.0,
                advance: char_advance,
                byte_offset: byte_off,
            })
            .collect();

        runs.push(GlyphRun {
            bounds: LayoutRect::new(x_offset, 0.0, width, line_height),
            glyphs,
            font_family: "Liberation Serif".to_string(),
            font_size: chunk.font_size,
            bold: chunk.bold,
            italic: chunk.italic,
            color: chunk.color.clone(),
            text_run_id: chunk.run_id,
            text: chunk.text.clone(),
        });

        x_offset += width;
    }

    let total_width = x_offset.min(available_width);

    LayoutLine {
        bounds: LayoutRect::new(0.0, 0.0, total_width, line_height),
        runs,
        baseline,
        paragraph_id: para_id,
        line_in_paragraph: line_index,
    }
}
