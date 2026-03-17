//! HTML parser — manual state-machine parser for common HTML elements.
//!
//! Rather than using the full html5ever API, we use a simple tokenizer
//! approach that handles the most common HTML constructs needed for
//! import of word processor documents.

use crate::HtmlError;
use rw_document::{
    Block, Document, Inline, Paragraph, Section, TextRun,
    block::{TableBlock, TableCell, TableCellProperties, TableProperties},
    inline::BreakType,
    properties::{Alignment, CharacterProperties, ParagraphProperties, UnderlineStyle},
    ElementId,
};
use std::path::Path;

/// Read an HTML file and convert it to a Document.
pub fn read_html(path: &Path) -> Result<Document, HtmlError> {
    let content = std::fs::read_to_string(path)?;
    parse_html(&content)
}

/// Parse an HTML string into a Document.
pub fn parse_html(html: &str) -> Result<Document, HtmlError> {
    let mut doc = Document::new();
    doc.sections.clear();

    let mut section = Section::new();
    let tokens = tokenize(html);
    let mut parser = HtmlParser::new(tokens);
    parser.parse(&mut section.content);

    // Extract metadata from <title> if found
    if let Some(ref title) = parser.title {
        doc.metadata.title = Some(title.clone());
    }

    if section.content.is_empty() {
        section.content.push(Block::Paragraph(Paragraph::new()));
    }
    doc.sections.push(section);
    Ok(doc)
}

#[derive(Debug, Clone)]
enum Token {
    OpenTag { name: String, attrs: Vec<(String, String)> },
    CloseTag { name: String },
    SelfClose { name: String, attrs: Vec<(String, String)> },
    Text(String),
}

fn tokenize(html: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = html.char_indices().peekable();

    while let Some((i, c)) = chars.next() {
        if c == '<' {
            // Check for comment
            if html[i..].starts_with("<!--") {
                // Skip until -->
                let mut j = i + 4;
                while j + 2 < html.len() {
                    if &html[j..j + 3] == "-->" {
                        // Advance chars past the comment
                        while let Some(&(pos, _)) = chars.peek() {
                            if pos >= j + 3 {
                                break;
                            }
                            chars.next();
                        }
                        break;
                    }
                    j += 1;
                }
                continue;
            }

            // Collect tag content
            let mut tag_content = String::new();
            for (_, tc) in chars.by_ref() {
                if tc == '>' {
                    break;
                }
                tag_content.push(tc);
            }

            // Parse tag
            let tag_content = tag_content.trim().to_string();
            if tag_content.starts_with('/') {
                // Close tag
                let name = tag_content[1..].trim().to_lowercase();
                tokens.push(Token::CloseTag { name });
            } else {
                let is_self_close = tag_content.ends_with('/');
                let tag_content = if is_self_close {
                    tag_content[..tag_content.len() - 1].trim().to_string()
                } else {
                    tag_content
                };

                // Parse tag name and attributes
                let (name, attrs) = parse_tag(&tag_content);
                let name = name.to_lowercase();

                // Some tags are always self-closing
                let self_close_tags = ["br", "hr", "img", "input", "meta", "link"];
                if is_self_close || self_close_tags.contains(&name.as_str()) {
                    tokens.push(Token::SelfClose { name, attrs });
                } else {
                    tokens.push(Token::OpenTag { name, attrs });
                }
            }
        } else {
            // Text node - collect until next '<'
            let mut text = String::new();
            text.push(c);
            while let Some(&(_, nc)) = chars.peek() {
                if nc == '<' {
                    break;
                }
                text.push(nc);
                chars.next();
            }
            // Decode HTML entities
            let text = decode_entities(&text);
            if !text.is_empty() {
                tokens.push(Token::Text(text));
            }
        }
    }

    tokens
}

fn parse_tag(s: &str) -> (String, Vec<(String, String)>) {
    let mut parts = s.splitn(2, |c: char| c.is_whitespace());
    let name = parts.next().unwrap_or("").to_string();
    let rest = parts.next().unwrap_or("");

    let mut attrs = Vec::new();
    let mut remaining = rest.trim();

    while !remaining.is_empty() {
        // Skip whitespace
        remaining = remaining.trim_start();
        if remaining.is_empty() {
            break;
        }

        // Find attribute name
        let eq_pos = remaining.find('=');
        let space_pos = remaining.find(|c: char| c.is_whitespace());

        match (eq_pos, space_pos) {
            (Some(eq), _) if space_pos.map_or(true, |sp| eq < sp) => {
                let attr_name = remaining[..eq].trim().to_lowercase();
                remaining = &remaining[eq + 1..].trim_start();

                // Get value
                let (value, rest) = if remaining.starts_with('"') {
                    let end = remaining[1..].find('"').map(|p| p + 1).unwrap_or(remaining.len() - 1);
                    (remaining[1..end].to_string(), &remaining[end + 1..])
                } else if remaining.starts_with('\'') {
                    let end = remaining[1..].find('\'').map(|p| p + 1).unwrap_or(remaining.len() - 1);
                    (remaining[1..end].to_string(), &remaining[end + 1..])
                } else {
                    let end = remaining.find(|c: char| c.is_whitespace()).unwrap_or(remaining.len());
                    (remaining[..end].to_string(), &remaining[end..])
                };

                attrs.push((attr_name, value));
                remaining = rest;
            }
            _ => {
                // Boolean attribute
                let end = remaining.find(|c: char| c.is_whitespace()).unwrap_or(remaining.len());
                let attr_name = remaining[..end].to_lowercase();
                if !attr_name.is_empty() {
                    attrs.push((attr_name.clone(), attr_name));
                }
                remaining = &remaining[end..];
            }
        }
    }

    (name, attrs)
}

fn decode_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&nbsp;", "\u{00A0}")
        .replace("&#160;", "\u{00A0}")
}

struct HtmlParser {
    tokens: Vec<Token>,
    pos: usize,
    pub title: Option<String>,
}

impl HtmlParser {
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: 0,
            title: None,
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<&Token> {
        let tok = self.tokens.get(self.pos);
        self.pos += 1;
        tok
    }

    fn parse(&mut self, content: &mut Vec<Block>) {
        // Skip to body content
        while let Some(tok) = self.peek() {
            match tok {
                Token::OpenTag { name, .. } if name == "body" => {
                    self.next();
                    self.parse_body(content);
                    return;
                }
                Token::OpenTag { name, .. } if name == "title" => {
                    self.next();
                    self.title = Some(self.collect_text_until("title"));
                }
                Token::CloseTag { name } if name == "html" => {
                    self.next();
                    return;
                }
                _ => {
                    self.next();
                }
            }
        }
        // If no body found, parse everything as body
        self.pos = 0;
        self.parse_body(content);
    }

    fn collect_text_until(&mut self, end_tag: &str) -> String {
        let mut text = String::new();
        while let Some(tok) = self.peek() {
            match tok {
                Token::CloseTag { name } if name == end_tag => {
                    self.next();
                    break;
                }
                Token::Text(t) => {
                    text.push_str(t);
                    self.next();
                }
                _ => {
                    self.next();
                }
            }
        }
        text
    }

    fn parse_body(&mut self, content: &mut Vec<Block>) {
        while let Some(tok) = self.peek() {
            match tok {
                Token::CloseTag { name } if name == "body" || name == "html" => {
                    self.next();
                    return;
                }
                Token::OpenTag { name, attrs } => {
                    let name = name.clone();
                    let attrs = attrs.clone();
                    self.next();
                    match name.as_str() {
                        "p" => {
                            let para = self.parse_paragraph(&attrs);
                            content.push(Block::Paragraph(para));
                        }
                        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                            let level: u8 = name[1..].parse().unwrap_or(1);
                            let para = self.parse_heading(&name, level);
                            content.push(Block::Paragraph(para));
                        }
                        "ul" | "ol" => {
                            self.parse_list(content, &name);
                        }
                        "table" => {
                            let tbl = self.parse_table();
                            content.push(Block::Table(tbl));
                        }
                        "div" | "article" | "section" | "main" | "header" | "footer" | "nav" | "aside" => {
                            // Treat as transparent container
                            self.parse_body(content);
                        }
                        "br" => {
                            let mut para = Paragraph::new();
                            para.content.push(Inline::Break(BreakType::Line));
                            content.push(Block::Paragraph(para));
                        }
                        "hr" => {
                            // Empty paragraph as separator
                            content.push(Block::Paragraph(Paragraph::new()));
                        }
                        "script" | "style" | "head" => {
                            // Skip until close
                            self.skip_until(&name);
                        }
                        _ => {
                            // Unknown block element, try to parse inline text
                            let text = self.collect_inline_text(&name);
                            if !text.trim().is_empty() {
                                content.push(Block::Paragraph(Paragraph::with_text(text)));
                            }
                        }
                    }
                }
                Token::Text(t) => {
                    let t = t.clone();
                    self.next();
                    let trimmed = t.trim();
                    if !trimmed.is_empty() {
                        content.push(Block::Paragraph(Paragraph::with_text(trimmed)));
                    }
                }
                _ => {
                    self.next();
                }
            }
        }
    }

    fn parse_paragraph(&mut self, attrs: &[(String, String)]) -> Paragraph {
        let mut para = Paragraph::new();

        // Check for style alignment
        for (k, v) in attrs {
            if k == "style" {
                if v.contains("text-align: center") || v.contains("text-align:center") {
                    para.properties.alignment = Some(Alignment::Center);
                } else if v.contains("text-align: right") || v.contains("text-align:right") {
                    para.properties.alignment = Some(Alignment::Right);
                } else if v.contains("text-align: justify") || v.contains("text-align:justify") {
                    para.properties.alignment = Some(Alignment::Justify);
                }
            }
        }

        self.collect_inline_runs(&mut para.content, "p");
        para
    }

    fn parse_heading(&mut self, tag: &str, level: u8) -> Paragraph {
        let mut para = Paragraph::new();
        para.properties.outline_level = Some(level);
        para.properties.paragraph_style = Some(format!("Heading{}", level));

        self.collect_inline_runs(&mut para.content, tag);
        para
    }

    fn collect_inline_runs(&mut self, inlines: &mut Vec<Inline>, end_tag: &str) {
        let mut char_props = CharacterProperties::default();

        loop {
            match self.peek() {
                None => break,
                Some(Token::CloseTag { name }) if name == end_tag => {
                    self.next();
                    break;
                }
                Some(Token::Text(_)) => {
                    if let Some(Token::Text(t)) = self.next().cloned().as_ref() {
                        if !t.is_empty() {
                            let mut run = TextRun::new(t.as_str());
                            run.properties = char_props.clone();
                            inlines.push(Inline::Text(run));
                        }
                    }
                }
                Some(Token::OpenTag { name, .. }) => {
                    let name = name.clone();
                    self.next();
                    match name.as_str() {
                        "b" | "strong" => {
                            let mut props = char_props.clone();
                            props.bold = Some(true);
                            self.collect_inline_runs_with_props(inlines, &name, props);
                        }
                        "i" | "em" => {
                            let mut props = char_props.clone();
                            props.italic = Some(true);
                            self.collect_inline_runs_with_props(inlines, &name, props);
                        }
                        "u" => {
                            let mut props = char_props.clone();
                            props.underline = Some(UnderlineStyle::Single);
                            self.collect_inline_runs_with_props(inlines, &name, props);
                        }
                        "s" | "del" | "strike" => {
                            let mut props = char_props.clone();
                            props.strikethrough = Some(rw_document::properties::StrikethroughStyle::Single);
                            self.collect_inline_runs_with_props(inlines, &name, props);
                        }
                        "span" | "a" | "abbr" | "cite" | "code" | "kbd" | "samp" | "var" | "small" | "big" | "sup" | "sub" => {
                            // Transparent inline wrapper
                            self.collect_inline_runs_with_props(inlines, &name, char_props.clone());
                        }
                        "br" => {
                            inlines.push(Inline::Break(BreakType::Line));
                        }
                        _ => {
                            // Skip unknown
                            self.skip_until(&name);
                        }
                    }
                }
                Some(Token::SelfClose { name, .. }) => {
                    let name = name.clone();
                    self.next();
                    if name == "br" {
                        inlines.push(Inline::Break(BreakType::Line));
                    }
                }
                _ => {
                    self.next();
                }
            }
        }
    }

    fn collect_inline_runs_with_props(
        &mut self,
        inlines: &mut Vec<Inline>,
        end_tag: &str,
        char_props: CharacterProperties,
    ) {
        loop {
            match self.peek() {
                None => break,
                Some(Token::CloseTag { name }) if name == end_tag => {
                    self.next();
                    break;
                }
                Some(Token::Text(_)) => {
                    if let Some(Token::Text(t)) = self.next().cloned().as_ref() {
                        if !t.is_empty() {
                            let mut run = TextRun::new(t.as_str());
                            run.properties = char_props.clone();
                            inlines.push(Inline::Text(run));
                        }
                    }
                }
                Some(Token::OpenTag { name, .. }) => {
                    let name = name.clone();
                    self.next();
                    match name.as_str() {
                        "b" | "strong" => {
                            let mut props = char_props.clone();
                            props.bold = Some(true);
                            self.collect_inline_runs_with_props(inlines, &name, props);
                        }
                        "i" | "em" => {
                            let mut props = char_props.clone();
                            props.italic = Some(true);
                            self.collect_inline_runs_with_props(inlines, &name, props);
                        }
                        "u" => {
                            let mut props = char_props.clone();
                            props.underline = Some(UnderlineStyle::Single);
                            self.collect_inline_runs_with_props(inlines, &name, props);
                        }
                        "br" => {
                            inlines.push(Inline::Break(BreakType::Line));
                        }
                        "span" | "a" | "abbr" | "code" => {
                            self.collect_inline_runs_with_props(inlines, &name, char_props.clone());
                        }
                        _ => {
                            self.skip_until(&name);
                        }
                    }
                }
                Some(Token::SelfClose { name, .. }) => {
                    let name = name.clone();
                    self.next();
                    if name == "br" {
                        inlines.push(Inline::Break(BreakType::Line));
                    }
                }
                _ => {
                    self.next();
                }
            }
        }
    }

    fn collect_inline_text(&mut self, end_tag: &str) -> String {
        let mut text = String::new();
        loop {
            match self.peek() {
                None => break,
                Some(Token::CloseTag { name }) if name == end_tag => {
                    self.next();
                    break;
                }
                Some(Token::Text(_)) => {
                    if let Some(Token::Text(t)) = self.next().cloned().as_ref() {
                        text.push_str(t);
                    }
                }
                _ => {
                    self.next();
                }
            }
        }
        text
    }

    fn parse_list(&mut self, content: &mut Vec<Block>, list_tag: &str) {
        loop {
            match self.peek() {
                None => break,
                Some(Token::CloseTag { name }) if name == list_tag => {
                    self.next();
                    break;
                }
                Some(Token::OpenTag { name, .. }) if name == "li" => {
                    self.next();
                    let mut para = Paragraph::new();
                    para.properties.paragraph_style = Some(match list_tag {
                        "ol" => "ListNumber".to_string(),
                        _ => "ListBullet".to_string(),
                    });
                    self.collect_inline_runs(&mut para.content, "li");
                    content.push(Block::Paragraph(para));
                }
                _ => {
                    self.next();
                }
            }
        }
    }

    fn parse_table(&mut self) -> TableBlock {
        let mut tbl = TableBlock {
            id: ElementId::new(),
            rows: 0,
            cols: 0,
            cells: Vec::new(),
            properties: TableProperties::default(),
        };

        loop {
            match self.peek() {
                None => break,
                Some(Token::CloseTag { name }) if name == "table" => {
                    self.next();
                    break;
                }
                Some(Token::OpenTag { name, .. }) if name == "tr" => {
                    self.next();
                    let cells = self.parse_table_row();
                    let cols = cells.len() as u32;
                    if cols > tbl.cols {
                        tbl.cols = cols;
                    }
                    tbl.rows += 1;
                    tbl.cells.extend(cells);
                }
                _ => {
                    self.next();
                }
            }
        }

        tbl
    }

    fn parse_table_row(&mut self) -> Vec<TableCell> {
        let mut cells = Vec::new();

        loop {
            match self.peek() {
                None => break,
                Some(Token::CloseTag { name }) if name == "tr" => {
                    self.next();
                    break;
                }
                Some(Token::OpenTag { name, .. }) if name == "td" || name == "th" => {
                    let tag = name.clone();
                    self.next();
                    let cell = self.parse_table_cell(&tag);
                    cells.push(cell);
                }
                _ => {
                    self.next();
                }
            }
        }

        cells
    }

    fn parse_table_cell(&mut self, end_tag: &str) -> TableCell {
        let mut cell_content: Vec<Block> = Vec::new();

        loop {
            match self.peek() {
                None => break,
                Some(Token::CloseTag { name }) if name == end_tag => {
                    self.next();
                    break;
                }
                Some(Token::OpenTag { name, attrs }) => {
                    let name = name.clone();
                    let attrs = attrs.clone();
                    self.next();
                    match name.as_str() {
                        "p" => {
                            let para = self.parse_paragraph(&attrs);
                            cell_content.push(Block::Paragraph(para));
                        }
                        _ => {
                            let text = self.collect_inline_text(&name);
                            if !text.trim().is_empty() {
                                cell_content.push(Block::Paragraph(Paragraph::with_text(text.trim())));
                            }
                        }
                    }
                }
                Some(Token::Text(_)) => {
                    if let Some(Token::Text(t)) = self.next().cloned().as_ref() {
                        let trimmed = t.trim();
                        if !trimmed.is_empty() {
                            cell_content.push(Block::Paragraph(Paragraph::with_text(trimmed)));
                        }
                    }
                }
                _ => {
                    self.next();
                }
            }
        }

        if cell_content.is_empty() {
            cell_content.push(Block::Paragraph(Paragraph::new()));
        }

        TableCell {
            id: ElementId::new(),
            content: cell_content,
            properties: TableCellProperties::default(),
            col_span: 1,
            row_span: 1,
        }
    }

    fn skip_until(&mut self, end_tag: &str) {
        let mut depth = 1;
        loop {
            match self.peek() {
                None => break,
                Some(Token::OpenTag { name, .. }) if name == end_tag => {
                    depth += 1;
                    self.next();
                }
                Some(Token::CloseTag { name }) if name == end_tag => {
                    depth -= 1;
                    self.next();
                    if depth == 0 {
                        break;
                    }
                }
                _ => {
                    self.next();
                }
            }
        }
    }
}
