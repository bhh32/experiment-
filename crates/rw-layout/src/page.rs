use crate::line::LayoutLine;
use crate::LayoutRect;
use rw_document::properties::PageProperties;

/// A laid-out page, containing positioned content.
#[derive(Debug, Clone)]
pub struct LayoutPage {
    /// Page index (0-based)
    pub page_number: usize,
    /// Page dimensions in twips
    pub bounds: LayoutRect,
    /// The content area (inside margins)
    pub content_area: LayoutRect,
    /// Laid-out lines on this page
    pub lines: Vec<LayoutLine>,
    /// Header content (if any)
    pub header: Option<Vec<LayoutLine>>,
    /// Footer content (if any)
    pub footer: Option<Vec<LayoutLine>>,
    /// Footnotes on this page
    pub footnotes: Vec<LayoutLine>,
    /// Page properties used for this page
    pub properties: PageProperties,
}

impl LayoutPage {
    pub fn new(page_number: usize, properties: PageProperties) -> Self {
        let width = properties.width.to_points();
        let height = properties.height.to_points();
        let margins = &properties.margins;

        let content_x = margins.left.to_points();
        let content_y = margins.top.to_points();
        let content_w = width - margins.left.to_points() - margins.right.to_points();
        let content_h = height - margins.top.to_points() - margins.bottom.to_points();

        Self {
            page_number,
            bounds: LayoutRect::new(0.0, 0.0, width, height),
            content_area: LayoutRect::new(content_x, content_y, content_w, content_h),
            lines: Vec::new(),
            header: None,
            footer: None,
            footnotes: Vec::new(),
            properties,
        }
    }
}
