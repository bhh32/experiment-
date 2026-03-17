//! RTF parser.
//!
//! Implements a state-machine parser for RTF control words and groups.
//! Supports RTF 1.x (as used by Microsoft Word and LibreOffice).

use crate::RtfError;
use rw_document::{
    Block, Color, Document, Inline, Paragraph, Section, TextRun,
    properties::{Alignment, CharacterProperties, ParagraphProperties, UnderlineStyle},
};

/// Read an RTF file and convert it to a Document.
pub fn read_rtf(path: &std::path::Path) -> Result<Document, RtfError> {
    let content = std::fs::read_to_string(path)?;
    parse_rtf(&content)
}

/// Parse an RTF string into a Document.
pub fn parse_rtf(rtf: &str) -> Result<Document, RtfError> {
    let mut doc = Document::new();
    doc.sections.clear();

    let bytes = rtf.as_bytes();
    let mut parser = RtfParser::new(bytes);
    parser.parse()?;

    let mut section = rw_document::Section::new();
    section.content = parser.blocks;
    if section.content.is_empty() {
        section.content.push(Block::Paragraph(Paragraph::new()));
    }
    doc.sections.push(section);

    Ok(doc)
}

/// RTF token types
#[derive(Debug, Clone)]
enum RtfToken {
    GroupOpen,
    GroupClose,
    ControlWord(String, Option<i32>),
    ControlSymbol(char),
    Text(String),
}

struct RtfParser<'a> {
    data: &'a [u8],
    pos: usize,
    pub blocks: Vec<Block>,

    // State
    state_stack: Vec<ParserState>,
    font_table: Vec<String>,
    color_table: Vec<Color>,
}

#[derive(Clone, Default)]
struct ParserState {
    char_props: CharacterProperties,
    para_props: ParagraphProperties,
    in_fonttbl: bool,
    in_colortbl: bool,
    in_info: bool,
    current_font_idx: usize,
    current_font_name: String,
    skip_destination: bool,
}

impl<'a> RtfParser<'a> {
    fn new(data: &'a [u8]) -> Self {
        let mut state_stack = Vec::new();
        state_stack.push(ParserState::default());
        Self {
            data,
            pos: 0,
            blocks: Vec::new(),
            state_stack,
            font_table: Vec::new(),
            color_table: vec![Color::BLACK], // Index 0 is auto/black
        }
    }

    fn state(&self) -> &ParserState {
        self.state_stack.last().unwrap()
    }

    fn state_mut(&mut self) -> &mut ParserState {
        self.state_stack.last_mut().unwrap()
    }

    fn parse(&mut self) -> Result<(), RtfError> {
        // Collect all tokens first
        let tokens = self.tokenize()?;
        self.process_tokens(&tokens)
    }

    fn tokenize(&mut self) -> Result<Vec<RtfToken>, RtfError> {
        let mut tokens = Vec::new();

        while self.pos < self.data.len() {
            let b = self.data[self.pos];
            match b {
                b'{' => {
                    tokens.push(RtfToken::GroupOpen);
                    self.pos += 1;
                }
                b'}' => {
                    tokens.push(RtfToken::GroupClose);
                    self.pos += 1;
                }
                b'\\' => {
                    self.pos += 1;
                    if self.pos >= self.data.len() {
                        break;
                    }
                    let next = self.data[self.pos];
                    if next.is_ascii_alphabetic() {
                        // Control word
                        let start = self.pos;
                        while self.pos < self.data.len() && self.data[self.pos].is_ascii_alphabetic() {
                            self.pos += 1;
                        }
                        let word = String::from_utf8_lossy(&self.data[start..self.pos]).to_string();

                        // Optional numeric parameter
                        let param = if self.pos < self.data.len() {
                            let sign = if self.data[self.pos] == b'-' {
                                self.pos += 1;
                                -1i32
                            } else {
                                1i32
                            };
                            if self.pos < self.data.len() && self.data[self.pos].is_ascii_digit() {
                                let num_start = self.pos;
                                while self.pos < self.data.len() && self.data[self.pos].is_ascii_digit() {
                                    self.pos += 1;
                                }
                                let num_str = String::from_utf8_lossy(&self.data[num_start..self.pos]).to_string();
                                let num: i32 = num_str.parse().unwrap_or(0) * sign;
                                // Consume one optional space delimiter
                                if self.pos < self.data.len() && self.data[self.pos] == b' ' {
                                    self.pos += 1;
                                }
                                Some(num)
                            } else {
                                // Consume one optional space delimiter
                                if self.pos < self.data.len() && self.data[self.pos] == b' ' {
                                    self.pos += 1;
                                }
                                None
                            }
                        } else {
                            None
                        };

                        tokens.push(RtfToken::ControlWord(word, param));
                    } else if next == b'\'' {
                        // Hex escape
                        self.pos += 1;
                        if self.pos + 1 < self.data.len() {
                            let hi = self.data[self.pos];
                            let lo = self.data[self.pos + 1];
                            self.pos += 2;
                            let hex = format!("{}{}", hi as char, lo as char);
                            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                                tokens.push(RtfToken::Text((byte as char).to_string()));
                            }
                        }
                    } else if next == b'*' {
                        // \* destination marker - mark as optional/ignorable
                        tokens.push(RtfToken::ControlSymbol('*'));
                        self.pos += 1;
                    } else if next == b'\n' || next == b'\r' {
                        // Escaped newline = paragraph break
                        tokens.push(RtfToken::ControlWord("par".to_string(), None));
                        self.pos += 1;
                    } else {
                        // Control symbol
                        tokens.push(RtfToken::ControlSymbol(next as char));
                        self.pos += 1;
                    }
                }
                b'\n' | b'\r' => {
                    // Literal newlines are ignored in RTF
                    self.pos += 1;
                }
                _ => {
                    // Text
                    let start = self.pos;
                    while self.pos < self.data.len() {
                        let cb = self.data[self.pos];
                        if cb == b'{' || cb == b'}' || cb == b'\\' || cb == b'\n' || cb == b'\r' {
                            break;
                        }
                        self.pos += 1;
                    }
                    let text = String::from_utf8_lossy(&self.data[start..self.pos]).to_string();
                    if !text.is_empty() {
                        tokens.push(RtfToken::Text(text));
                    }
                }
            }
        }

        Ok(tokens)
    }

    fn process_tokens(&mut self, tokens: &[RtfToken]) -> Result<(), RtfError> {
        let mut current_para = Paragraph::new();
        let mut current_char_props = CharacterProperties::default();
        let mut current_para_props = ParagraphProperties::default();
        let mut current_text = String::new();

        // Group depth tracking for destination skipping
        let mut skip_depth: Option<usize> = None;
        let mut depth: usize = 0;

        // Track special destinations
        let mut in_fonttbl = false;
        let mut fonttbl_depth: usize = 0;
        let mut current_font_idx: usize = 0;
        let mut current_font_name = String::new();

        let mut in_colortbl = false;
        let mut colortbl_depth: usize = 0;
        let mut color_r: Option<u8> = None;
        let mut color_g: Option<u8> = None;
        let mut color_b: Option<u8> = None;

        let mut ignore_next = false; // For \* destinations

        let mut group_char_props: Vec<CharacterProperties> = vec![CharacterProperties::default()];
        let mut group_para_props: Vec<ParagraphProperties> = vec![ParagraphProperties::default()];
        let mut group_in_fonttbl: Vec<bool> = vec![false];
        let mut group_in_colortbl: Vec<bool> = vec![false];

        for tok in tokens {
            match tok {
                RtfToken::GroupOpen => {
                    depth += 1;
                    group_char_props.push(current_char_props.clone());
                    group_para_props.push(current_para_props.clone());
                    group_in_fonttbl.push(in_fonttbl);
                    group_in_colortbl.push(in_colortbl);

                    if ignore_next {
                        skip_depth = Some(depth);
                        ignore_next = false;
                    }
                }
                RtfToken::GroupClose => {
                    if let Some(sd) = skip_depth {
                        if depth == sd {
                            skip_depth = None;
                        }
                    }

                    // Save font name if in font table
                    if in_fonttbl && depth == fonttbl_depth {
                        if !current_font_name.is_empty() {
                            let name = current_font_name
                                .trim_end_matches(';')
                                .trim()
                                .to_string();
                            while self.font_table.len() <= current_font_idx {
                                self.font_table.push(String::new());
                            }
                            self.font_table[current_font_idx] = name;
                            current_font_name.clear();
                        }
                    }

                    // Handle color table entry
                    if in_colortbl && depth == colortbl_depth {
                        in_colortbl = false;
                    }

                    if depth > 0 {
                        depth -= 1;
                    }

                    // Restore state
                    if let Some(cp) = group_char_props.pop() {
                        current_char_props = cp;
                    }
                    if let Some(pp) = group_para_props.pop() {
                        current_para_props = pp;
                    }
                    if let Some(f) = group_in_fonttbl.pop() {
                        in_fonttbl = f;
                    }
                    if let Some(c) = group_in_colortbl.pop() {
                        in_colortbl = c;
                    }
                }
                RtfToken::ControlSymbol('*') => {
                    ignore_next = true;
                }
                RtfToken::ControlWord(word, param) => {
                    if skip_depth.is_some() {
                        // Track depth changes even when skipping
                        continue;
                    }

                    match word.as_str() {
                        // Destinations
                        "fonttbl" => {
                            in_fonttbl = true;
                            fonttbl_depth = depth;
                            *group_in_fonttbl.last_mut().unwrap_or(&mut false) = true;
                        }
                        "colortbl" => {
                            in_colortbl = true;
                            colortbl_depth = depth;
                            *group_in_colortbl.last_mut().unwrap_or(&mut false) = true;
                        }
                        "info" | "stylesheet" | "listtable" | "listoverridetable" => {
                            skip_depth = Some(depth);
                        }

                        // Font table
                        "f" if in_fonttbl => {
                            current_font_idx = param.unwrap_or(0) as usize;
                        }
                        "f" if !in_fonttbl => {
                            let idx = param.unwrap_or(0) as usize;
                            if let Some(font_name) = self.font_table.get(idx) {
                                if !font_name.is_empty() {
                                    current_char_props.font_family = Some(font_name.clone());
                                }
                            }
                        }

                        // Color table
                        "red" if in_colortbl => {
                            color_r = Some(param.unwrap_or(0) as u8);
                        }
                        "green" if in_colortbl => {
                            color_g = Some(param.unwrap_or(0) as u8);
                        }
                        "blue" if in_colortbl => {
                            color_b = Some(param.unwrap_or(0) as u8);
                        }

                        // Apply text color
                        "cf" => {
                            let idx = param.unwrap_or(0) as usize;
                            if let Some(color) = self.color_table.get(idx) {
                                current_char_props.color = Some(*color);
                            }
                        }

                        // Character formatting
                        "b" => {
                            current_char_props.bold = Some(param.unwrap_or(1) != 0);
                        }
                        "i" => {
                            current_char_props.italic = Some(param.unwrap_or(1) != 0);
                        }
                        "ul" => {
                            current_char_props.underline = Some(UnderlineStyle::Single);
                        }
                        "uld" => {
                            current_char_props.underline = Some(UnderlineStyle::Dotted);
                        }
                        "uldb" => {
                            current_char_props.underline = Some(UnderlineStyle::Double);
                        }
                        "ulwave" => {
                            current_char_props.underline = Some(UnderlineStyle::Wave);
                        }
                        "ulnone" => {
                            current_char_props.underline = None;
                        }
                        "strike" => {
                            current_char_props.strikethrough = Some(rw_document::properties::StrikethroughStyle::Single);
                        }
                        "striked" => {
                            current_char_props.strikethrough = Some(rw_document::properties::StrikethroughStyle::Double);
                        }
                        "fs" => {
                            // Font size in half-points
                            if let Some(size) = param {
                                current_char_props.font_size = Some(*size as u32);
                            }
                        }
                        "super" => {
                            current_char_props.superscript = Some(true);
                        }
                        "sub" => {
                            current_char_props.subscript = Some(true);
                        }
                        "nosupersub" => {
                            current_char_props.superscript = None;
                            current_char_props.subscript = None;
                        }
                        "plain" => {
                            // Reset character formatting
                            current_char_props = CharacterProperties::default();
                        }

                        // Paragraph formatting
                        "par" | "pard" if word == "pard" => {
                            // Flush text to current para if any
                            if !current_text.is_empty() {
                                let mut run = TextRun::new(current_text.clone());
                                run.properties = current_char_props.clone();
                                current_para.content.push(Inline::Text(run));
                                current_text.clear();
                            }
                            // Reset paragraph properties
                            current_para_props = ParagraphProperties::default();
                        }
                        "par" => {
                            // Flush current text
                            if !current_text.is_empty() {
                                let mut run = TextRun::new(current_text.clone());
                                run.properties = current_char_props.clone();
                                current_para.content.push(Inline::Text(run));
                                current_text.clear();
                            }
                            // End paragraph
                            current_para.properties = current_para_props.clone();
                            self.blocks.push(Block::Paragraph(current_para));
                            current_para = Paragraph::new();
                        }
                        "ql" => {
                            current_para_props.alignment = Some(Alignment::Left);
                        }
                        "qc" => {
                            current_para_props.alignment = Some(Alignment::Center);
                        }
                        "qr" => {
                            current_para_props.alignment = Some(Alignment::Right);
                        }
                        "qj" => {
                            current_para_props.alignment = Some(Alignment::Justify);
                        }

                        // Line break
                        "line" => {
                            if !current_text.is_empty() {
                                let mut run = TextRun::new(current_text.clone());
                                run.properties = current_char_props.clone();
                                current_para.content.push(Inline::Text(run));
                                current_text.clear();
                            }
                            current_para.content.push(Inline::Break(rw_document::inline::BreakType::Line));
                        }

                        // Tab
                        "tab" => {
                            if !current_text.is_empty() {
                                let mut run = TextRun::new(current_text.clone());
                                run.properties = current_char_props.clone();
                                current_para.content.push(Inline::Text(run));
                                current_text.clear();
                            }
                            current_para.content.push(Inline::Tab);
                        }

                        // Unicode character
                        "u" => {
                            if let Some(code) = param {
                                let code = *code;
                                let code = if code < 0 { (code + 65536) as u32 } else { code as u32 };
                                if let Some(c) = char::from_u32(code) {
                                    current_text.push(c);
                                }
                            }
                        }

                        _ => {}
                    }
                }
                RtfToken::ControlSymbol(c) => {
                    if skip_depth.is_some() {
                        continue;
                    }
                    match c {
                        '-' => current_text.push('\u{00AD}'), // Soft hyphen
                        '~' => current_text.push('\u{00A0}'), // Non-breaking space
                        '_' => current_text.push('\u{2011}'), // Non-breaking hyphen
                        '{' => current_text.push('{'),
                        '}' => current_text.push('}'),
                        '\\' => current_text.push('\\'),
                        _ => {}
                    }
                }
                RtfToken::Text(text) => {
                    if skip_depth.is_some() {
                        continue;
                    }

                    if in_fonttbl {
                        // Accumulate font name
                        current_font_name.push_str(text);
                    } else if in_colortbl {
                        // Color entries separated by ';' in text
                        for c in text.chars() {
                            if c == ';' {
                                // Commit current color
                                let r = color_r.unwrap_or(0);
                                let g = color_g.unwrap_or(0);
                                let b = color_b.unwrap_or(0);
                                self.color_table.push(Color::rgb(r, g, b));
                                color_r = None;
                                color_g = None;
                                color_b = None;
                            }
                        }
                    } else {
                        current_text.push_str(text);
                    }
                }
            }
        }

        // Flush any remaining content
        if !current_text.is_empty() {
            let mut run = TextRun::new(current_text);
            run.properties = current_char_props;
            current_para.content.push(Inline::Text(run));
        }
        if !current_para.content.is_empty() {
            current_para.properties = current_para_props;
            self.blocks.push(Block::Paragraph(current_para));
        }

        Ok(())
    }
}
