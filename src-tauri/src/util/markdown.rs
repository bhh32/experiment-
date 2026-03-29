// Markdown rendering via pulldown-cmark
// Phase 2: add this dependency and implement full rendering
// For now, the frontend handles markdown display via a WASM markdown crate

pub fn render_to_html(markdown: &str) -> String {
    // Placeholder - returns raw markdown wrapped in a pre tag
    // Will be replaced with pulldown-cmark once added as a dependency
    format!("<pre>{}</pre>", markdown)
}
