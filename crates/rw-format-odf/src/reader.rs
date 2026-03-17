//! ODF document reader.
//!
//! Parses an .odt ZIP archive and constructs a Document model.

/// ODF XML namespace constants.
pub mod ns {
    pub const OFFICE: &str = "urn:oasis:names:tc:opendocument:xmlns:office:1.0";
    pub const TEXT: &str = "urn:oasis:names:tc:opendocument:xmlns:text:1.0";
    pub const STYLE: &str = "urn:oasis:names:tc:opendocument:xmlns:style:1.0";
    pub const TABLE: &str = "urn:oasis:names:tc:opendocument:xmlns:table:1.0";
    pub const DRAW: &str = "urn:oasis:names:tc:opendocument:xmlns:drawing:1.0";
    pub const FO: &str = "urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0";
    pub const META: &str = "urn:oasis:names:tc:opendocument:xmlns:meta:1.0";
    pub const SVG: &str = "urn:oasis:names:tc:opendocument:xmlns:svg-compatible:1.0";
    pub const XLINK: &str = "http://www.w3.org/1999/xlink";
    pub const DC: &str = "http://purl.org/dc/elements/1.1/";
}
