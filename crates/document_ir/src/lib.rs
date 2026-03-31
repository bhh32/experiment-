mod ir;
mod parse;
mod render_html;
mod render_docx;
mod read_docx;

pub use ir::*;
pub use parse::parse_markdown;
pub use render_html::{render_to_html, render_footnotes};
pub use render_docx::render_to_docx;
pub use read_docx::read_docx;
