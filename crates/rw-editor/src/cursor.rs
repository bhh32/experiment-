use serde::{Deserialize, Serialize};
use unicode_segmentation::UnicodeSegmentation;
use rw_document::{Document, Block};

/// A position within the document, identified by section, block, and
/// inline offsets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CursorPosition {
    /// Section index (0-based)
    pub section: usize,
    /// Block index within the section (0-based)
    pub block: usize,
    /// For paragraphs: the inline element index
    pub inline: usize,
    /// Character offset within the text run (byte offset)
    pub offset: usize,
}

impl CursorPosition {
    pub fn start() -> Self {
        Self {
            section: 0,
            block: 0,
            inline: 0,
            offset: 0,
        }
    }
}

impl Default for CursorPosition {
    fn default() -> Self {
        Self::start()
    }
}

/// The document cursor.
///
/// Tracks the current editing position and provides movement operations.
/// The cursor also remembers a "preferred column" for vertical movement
/// so that moving up/down through lines of varying length stays in
/// roughly the same horizontal position.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cursor {
    /// Current position
    pub position: CursorPosition,
    /// Preferred horizontal offset (in twips) for vertical movement
    pub preferred_x: Option<i32>,
    /// Whether the cursor is visible (for blink state)
    pub visible: bool,
}

impl Cursor {
    pub fn new() -> Self {
        Self {
            position: CursorPosition::start(),
            preferred_x: None,
            visible: true,
        }
    }

    /// Resolve the cursor position to (section_idx, block_idx, char_offset_in_plain_text).
    ///
    /// Returns `None` if the cursor position is out of bounds.
    pub fn resolve_offset(&self, doc: &Document) -> Option<(usize, usize, usize)> {
        let sec_idx = self.position.section;
        let blk_idx = self.position.block;

        let section = doc.sections.get(sec_idx)?;
        let block = section.content.get(blk_idx)?;

        if let Block::Paragraph(para) = block {
            // Walk text runs to compute the byte offset within the paragraph's plain text
            let mut char_offset = 0usize;
            for (i, inline) in para.content.iter().enumerate() {
                if i == self.position.inline {
                    // Add the offset within this run
                    char_offset += self.position.offset;
                    break;
                }
                match inline {
                    rw_document::Inline::Text(run) => char_offset += run.text.len(),
                    rw_document::Inline::Tab => char_offset += 1,
                    rw_document::Inline::NonBreakingSpace => {
                        char_offset += '\u{00A0}'.len_utf8();
                    }
                    rw_document::Inline::Break(rw_document::inline::BreakType::Line) => {
                        char_offset += 1;
                    }
                    _ => {}
                }
            }
            Some((sec_idx, blk_idx, char_offset))
        } else {
            None
        }
    }

    /// Set position from (section, block, char_offset_in_plain_text).
    ///
    /// Walks the paragraph inline list to find the correct run and offset.
    fn set_from_plain_offset(&mut self, doc: &Document, sec: usize, blk: usize, target: usize) {
        if let Some(section) = doc.sections.get(sec) {
            if let Some(Block::Paragraph(para)) = section.content.get(blk) {
                let mut remaining = target;
                for (i, inline) in para.content.iter().enumerate() {
                    let run_len = match inline {
                        rw_document::Inline::Text(run) => run.text.len(),
                        rw_document::Inline::Tab => 1,
                        rw_document::Inline::NonBreakingSpace => '\u{00A0}'.len_utf8(),
                        rw_document::Inline::Break(rw_document::inline::BreakType::Line) => 1,
                        _ => 0,
                    };
                    if remaining <= run_len {
                        self.position = CursorPosition {
                            section: sec,
                            block: blk,
                            inline: i,
                            offset: remaining,
                        };
                        return;
                    }
                    remaining -= run_len;
                }
                // End of paragraph
                let last_inline = para.content.len().saturating_sub(1);
                let last_offset = match para.content.last() {
                    Some(rw_document::Inline::Text(run)) => run.text.len(),
                    Some(rw_document::Inline::Tab) => 1,
                    _ => 0,
                };
                self.position = CursorPosition {
                    section: sec,
                    block: blk,
                    inline: last_inline,
                    offset: last_offset,
                };
                return;
            }
        }
        // Fallback — clamp to start
        self.position = CursorPosition::start();
    }

    /// Move cursor one grapheme to the right.
    pub fn move_right(&mut self, doc: &Document) {
        let sec = self.position.section;
        let blk = self.position.block;

        if let Some(section) = doc.sections.get(sec) {
            if let Some(Block::Paragraph(para)) = section.content.get(blk) {
                let plain = para.plain_text();
                if let Some((sec_i, blk_i, offset)) = self.resolve_offset(doc) {
                    // Find the next grapheme boundary after `offset`
                    let mut graphemes = plain.grapheme_indices(true);
                    // Advance to (or past) the current offset
                    let mut next_offset: Option<usize> = None;
                    let mut found_current = false;
                    for (byte_pos, g) in graphemes.by_ref() {
                        if byte_pos == offset && !found_current {
                            found_current = true;
                            next_offset = Some(byte_pos + g.len());
                            break;
                        } else if byte_pos > offset && !found_current {
                            // offset is inside a multi-byte grapheme — advance to end of it
                            next_offset = Some(byte_pos);
                            break;
                        }
                    }
                    if let Some(new_offset) = next_offset {
                        if new_offset <= plain.len() {
                            self.set_from_plain_offset(doc, sec_i, blk_i, new_offset);
                            self.preferred_x = None;
                            return;
                        }
                    }
                }
            }
        }

        // Move to start of next block / section
        self.advance_block(doc);
    }

    /// Move cursor one grapheme to the left.
    pub fn move_left(&mut self, doc: &Document) {
        let sec = self.position.section;
        let blk = self.position.block;

        if let Some(section) = doc.sections.get(sec) {
            if let Some(Block::Paragraph(para)) = section.content.get(blk) {
                let plain = para.plain_text();
                if let Some((sec_i, blk_i, offset)) = self.resolve_offset(doc) {
                    if offset > 0 {
                        // Find the grapheme boundary before `offset`
                        let mut last_boundary = 0usize;
                        for (byte_pos, _) in plain.grapheme_indices(true) {
                            if byte_pos >= offset {
                                break;
                            }
                            last_boundary = byte_pos;
                        }
                        self.set_from_plain_offset(doc, sec_i, blk_i, last_boundary);
                        self.preferred_x = None;
                        return;
                    }
                }
            }
        }

        // Move to end of previous block / section
        self.retreat_block(doc);
    }

    /// Move to the next/previous word boundary.
    pub fn move_to_word_boundary(&mut self, doc: &Document, direction: MoveDirection) {
        if let Some((sec, blk, offset)) = self.resolve_offset(doc) {
            if let Some(section) = doc.sections.get(sec) {
                if let Some(Block::Paragraph(para)) = section.content.get(blk) {
                    let plain = para.plain_text();
                    let new_offset = match direction {
                        MoveDirection::Right => find_next_word_boundary(&plain, offset),
                        MoveDirection::Left => find_prev_word_boundary(&plain, offset),
                        _ => offset,
                    };
                    self.set_from_plain_offset(doc, sec, blk, new_offset);
                    self.preferred_x = None;
                    return;
                }
            }
        }
        // fallback: single char move
        match direction {
            MoveDirection::Right => self.move_right(doc),
            MoveDirection::Left => self.move_left(doc),
            _ => {}
        }
    }

    /// Move to the start or end of the current paragraph (line boundary).
    pub fn move_to_line_boundary(&mut self, doc: &Document, direction: MoveDirection) {
        if let Some((sec, blk, _offset)) = self.resolve_offset(doc) {
            match direction {
                MoveDirection::Left => {
                    self.set_from_plain_offset(doc, sec, blk, 0);
                }
                MoveDirection::Right => {
                    if let Some(section) = doc.sections.get(sec) {
                        if let Some(Block::Paragraph(para)) = section.content.get(blk) {
                            let len = para.plain_text().len();
                            self.set_from_plain_offset(doc, sec, blk, len);
                        }
                    }
                }
                _ => {}
            }
            self.preferred_x = None;
        }
    }

    /// Move to the start or end of the document.
    pub fn move_to_document_boundary(&mut self, doc: &Document, direction: MoveDirection) {
        match direction {
            MoveDirection::Left => {
                self.position = CursorPosition::start();
                self.preferred_x = None;
            }
            MoveDirection::Right => {
                // Find the last section, last block, end of its plain text
                if let Some((last_sec, section)) = doc.sections.iter().enumerate().last() {
                    if let Some((last_blk, block)) = section.content.iter().enumerate().last() {
                        let end_offset = match block {
                            Block::Paragraph(para) => para.plain_text().len(),
                            _ => 0,
                        };
                        self.set_from_plain_offset(doc, last_sec, last_blk, end_offset);
                    }
                }
                self.preferred_x = None;
            }
            _ => {}
        }
    }

    // ---- internal helpers ----

    /// Advance to the start of the next block (across sections if needed).
    fn advance_block(&mut self, doc: &Document) {
        let sec = self.position.section;
        let blk = self.position.block;
        if let Some(section) = doc.sections.get(sec) {
            if blk + 1 < section.content.len() {
                self.position = CursorPosition {
                    section: sec,
                    block: blk + 1,
                    inline: 0,
                    offset: 0,
                };
                return;
            }
        }
        // Try next section
        if sec + 1 < doc.sections.len() {
            self.position = CursorPosition {
                section: sec + 1,
                block: 0,
                inline: 0,
                offset: 0,
            };
        }
        // else: already at document end, stay put
    }

    /// Retreat to the end of the previous block (across sections if needed).
    fn retreat_block(&mut self, doc: &Document) {
        let sec = self.position.section;
        let blk = self.position.block;
        if blk > 0 {
            let prev_blk = blk - 1;
            let end_offset = if let Some(section) = doc.sections.get(sec) {
                if let Some(Block::Paragraph(para)) = section.content.get(prev_blk) {
                    para.plain_text().len()
                } else {
                    0
                }
            } else {
                0
            };
            self.set_from_plain_offset(doc, sec, prev_blk, end_offset);
            self.preferred_x = None;
            return;
        }
        if sec > 0 {
            let prev_sec = sec - 1;
            if let Some(section) = doc.sections.get(prev_sec) {
                if let Some(last_blk) = section.content.len().checked_sub(1) {
                    let end_offset = if let Some(Block::Paragraph(para)) =
                        section.content.get(last_blk)
                    {
                        para.plain_text().len()
                    } else {
                        0
                    };
                    self.set_from_plain_offset(doc, prev_sec, last_blk, end_offset);
                    self.preferred_x = None;
                    return;
                }
            }
        }
        // Already at start
    }
}

impl Default for Cursor {
    fn default() -> Self {
        Self::new()
    }
}

/// Direction of cursor movement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveDirection {
    Left,
    Right,
    Up,
    Down,
}

/// Granularity of cursor movement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveUnit {
    /// Single grapheme cluster
    Character,
    /// Word boundary
    Word,
    /// Start/end of line (visual line)
    Line,
    /// Start/end of paragraph
    Paragraph,
    /// Start/end of page
    Page,
    /// Start/end of document
    Document,
}

// ---- word boundary helpers ----

/// Find the byte offset of the next word boundary to the right of `offset` in `text`.
fn find_next_word_boundary(text: &str, offset: usize) -> usize {
    let bytes = text.as_bytes();
    let len = bytes.len();
    if offset >= len {
        return len;
    }

    // Skip any whitespace at or after offset
    let mut pos = offset;
    while pos < len && (bytes[pos] as char).is_whitespace() {
        pos += 1;
    }
    // Then skip the word
    while pos < len && !(bytes[pos] as char).is_whitespace() {
        pos += 1;
    }
    pos
}

/// Find the byte offset of the previous word boundary to the left of `offset` in `text`.
fn find_prev_word_boundary(text: &str, offset: usize) -> usize {
    if offset == 0 {
        return 0;
    }
    let bytes = text.as_bytes();
    let mut pos = offset;

    // Step back one char first
    if pos > 0 {
        pos -= 1;
    }
    // Skip whitespace going left
    while pos > 0 && (bytes[pos] as char).is_whitespace() {
        pos -= 1;
    }
    // Skip word chars going left
    while pos > 0 && !(bytes[pos - 1] as char).is_whitespace() {
        pos -= 1;
    }
    pos
}
