//! # rw-widgets
//!
//! Custom UI widgets for Rust Writer, built on libcosmic/iced.
//!
//! ## Widgets
//!
//! - **Ruler**: Horizontal and vertical rulers showing margins, indents, tab stops
//! - **ColorPicker**: Full color picker with palette, custom color, recent colors
//! - **StyleGallery**: Visual gallery of paragraph/character styles with live preview
//! - **StatusBar**: Bottom status bar with page info, word count, zoom slider, view modes
//! - **FontPicker**: Font selection dropdown with preview
//! - **ZoomSlider**: Zoom control with percentage display and preset buttons
//! - **PageNavigator**: Page thumbnail sidebar for document navigation
//! - **SymbolPicker**: Special character insertion dialog
//! - **FormatPainter**: Format painter cursor mode indicator
//! - **DocumentCanvas**: The main document editing/rendering area widget
//! - **PageView**: Single page renderer within the document canvas

pub mod color_picker;
pub mod document_canvas;
pub mod font_picker;
pub mod page_navigator;
pub mod ruler;
pub mod status_bar;
pub mod style_gallery;
pub mod symbol_picker;
pub mod zoom_slider;
