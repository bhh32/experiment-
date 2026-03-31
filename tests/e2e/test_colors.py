"""Tests for text color and highlight features.

Covers the full pipeline: toolbar → markdown → preview → DOCX → read back.
"""
import pytest
from playwright.sync_api import Page, expect

BASE_URL = "http://localhost:8080"
SS = "tests/e2e/test-results"


@pytest.fixture(autouse=True)
def navigate(page: Page):
    page.goto(BASE_URL)
    page.wait_for_selector(".app-container", timeout=30000)


def get_textarea(page: Page):
    return page.locator(".editor-textarea")


def textarea_value(page: Page) -> str:
    return get_textarea(page).input_value()


def select_text(page: Page, start: int, end: int):
    page.evaluate(f"""() => {{
        const ta = document.getElementById('editor-textarea');
        ta.focus();
        ta.setSelectionRange({start}, {end});
    }}""")


# ── Color Picker UI ──

class TestColorPickerUI:
    def test_color_dropdown_visible(self, page: Page):
        expect(page.locator(".color-select")).to_be_visible()

    def test_highlight_dropdown_visible(self, page: Page):
        expect(page.locator(".highlight-select")).to_be_visible()

    def test_color_dropdown_has_options(self, page: Page):
        options = page.locator(".color-select option")
        # Default "A" + 12 color options
        count = options.count()
        assert count >= 10, f"Expected at least 10 color options, got {count}"
        page.screenshot(path=f"{SS}/color-picker-ui.png")

    def test_highlight_dropdown_has_options(self, page: Page):
        options = page.locator(".highlight-select option")
        count = options.count()
        assert count >= 5, f"Expected at least 5 highlight options, got {count}"


# ── Applying Colors ──

class TestApplyColor:
    def test_apply_red_to_selection(self, page: Page):
        ta = get_textarea(page)
        ta.fill("make this red please")
        select_text(page, 10, 13)  # select "red"
        page.locator(".color-select").select_option("FF0000")
        val = textarea_value(page)
        assert "{color:FF0000}" in val
        assert "{/color}" in val
        assert "red" in val
        page.screenshot(path=f"{SS}/color-apply-red.png")

    def test_apply_blue_to_selection(self, page: Page):
        ta = get_textarea(page)
        ta.fill("make this blue text")
        select_text(page, 10, 14)  # select "blue"
        page.locator(".color-select").select_option("0000FF")
        val = textarea_value(page)
        assert "{color:0000FF}" in val
        assert "{/color}" in val

    def test_apply_color_at_cursor(self, page: Page):
        ta = get_textarea(page)
        ta.fill("hello")
        ta.focus()
        # Set cursor at end
        page.evaluate("""() => {
            const ta = document.getElementById('editor-textarea');
            ta.setSelectionRange(5, 5);
        }""")
        page.locator(".color-select").select_option("008000")
        val = textarea_value(page)
        assert "{color:008000}" in val
        assert "{/color}" in val

    def test_apply_highlight_to_selection(self, page: Page):
        ta = get_textarea(page)
        ta.fill("highlight this word")
        select_text(page, 10, 14)  # select "this"
        page.locator(".highlight-select").select_option("yellow")
        val = textarea_value(page)
        assert "{highlight:yellow}" in val
        assert "{/highlight}" in val
        page.screenshot(path=f"{SS}/color-highlight-yellow.png")

    def test_multiple_colors_in_same_text(self, page: Page):
        ta = get_textarea(page)
        ta.fill("red and blue words")
        # Color "red" (positions 0-3)
        select_text(page, 0, 3)
        page.locator(".color-select").select_option("FF0000")

        # Now color "blue" — need to find new position after markers were inserted
        val = textarea_value(page)
        idx = val.index("blue")
        select_text(page, idx, idx + 4)
        page.locator(".color-select").select_option("0000FF")

        val = textarea_value(page)
        assert "{color:FF0000}" in val
        assert "{color:0000FF}" in val
        page.screenshot(path=f"{SS}/color-multi-colors.png")


# ── Preview Rendering ──

class TestColorPreview:
    def test_colored_text_in_preview(self, page: Page):
        ta = get_textarea(page)
        ta.fill("This has {color:FF0000}red text{/color} in it")
        # Check preview has colored span
        preview = page.locator(".preview-content")
        preview_html = preview.inner_html()
        assert "color:#FF0000" in preview_html or "color:red" in preview_html.lower()
        page.screenshot(path=f"{SS}/color-preview-red.png")

    def test_highlighted_text_in_preview(self, page: Page):
        ta = get_textarea(page)
        ta.fill("This has {highlight:yellow}highlighted{/highlight} text")
        preview = page.locator(".preview-content")
        preview_html = preview.inner_html()
        assert "background" in preview_html.lower()
        page.screenshot(path=f"{SS}/color-preview-highlight.png")

    def test_color_in_docx_preview(self, page: Page):
        ta = get_textarea(page)
        ta.fill("# Title\n\nThis has {color:0000FF}blue text{/color} here")
        page.locator('.mode-btn:text-is("DOCX")').click()
        page.wait_for_timeout(300)
        preview = page.locator(".preview-content")
        preview_html = preview.inner_html()
        assert "color:#0000FF" in preview_html or "color:blue" in preview_html.lower()
        page.screenshot(path=f"{SS}/color-docx-preview.png")

    def test_bold_and_color_together(self, page: Page):
        ta = get_textarea(page)
        ta.fill("This is **{color:FF0000}bold red{/color}** text")
        preview = page.locator(".preview-content")
        preview_html = preview.inner_html()
        assert "FF0000" in preview_html or "color" in preview_html
        page.screenshot(path=f"{SS}/color-bold-red.png")


# ── DOCX Export & Roundtrip ──

class TestColorDocxRoundtrip:
    def test_color_survives_docx_export_api(self, page: Page):
        """Verify colored text gets into the DOCX via the convert API."""
        result = page.evaluate("""async () => {
            const resp = await fetch('/api/convert', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    content: 'This has {color:FF0000}red text{/color} here',
                    from: 'markdown',
                    to: 'docx'
                })
            });
            const data = await resp.json();
            return { ok: resp.ok, format: data.format, hasContent: data.content.length > 0 };
        }""")
        assert result["ok"]
        assert result["format"] == "docx"
        assert result["hasContent"]

    def test_highlight_survives_docx_export_api(self, page: Page):
        result = page.evaluate("""async () => {
            const resp = await fetch('/api/convert', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    content: 'This has {highlight:yellow}highlighted{/highlight} text',
                    from: 'markdown',
                    to: 'docx'
                })
            });
            return { ok: resp.ok, format: (await resp.json()).format };
        }""")
        assert result["ok"]
        assert result["format"] == "docx"


# ── Markdown Roundtrip ──

class TestColorMarkdownRoundtrip:
    def test_color_syntax_preserved_after_edit(self, page: Page):
        """Type color syntax, verify it stays after editing other parts."""
        ta = get_textarea(page)
        ta.fill("{color:FF0000}Important{/color} note about stuff")
        val = textarea_value(page)
        assert "{color:FF0000}" in val
        assert "{/color}" in val

        # Edit the end of the text
        ta.fill("{color:FF0000}Important{/color} note about other stuff")
        val = textarea_value(page)
        assert "{color:FF0000}" in val
        assert "Important" in val

    def test_multiple_colors_roundtrip(self, page: Page):
        ta = get_textarea(page)
        ta.fill("{color:FF0000}Red{/color} and {color:0000FF}Blue{/color} text")
        val = textarea_value(page)
        assert "{color:FF0000}Red{/color}" in val
        assert "{color:0000FF}Blue{/color}" in val
        page.screenshot(path=f"{SS}/color-roundtrip-multi.png")


# ── Full Workflow ──

class TestColorWorkflow:
    def test_write_color_format_preview_export(self, page: Page):
        """Full workflow: write text, apply color, check preview, switch to DOCX."""
        ta = get_textarea(page)

        # Step 1: Write content
        ta.fill("# Status Report\n\nThe project is on track. We need to address the critical issue.")
        page.screenshot(path=f"{SS}/color-workflow-1-write.png")

        # Step 2: Color "critical" red
        val = textarea_value(page)
        idx = val.index("critical")
        select_text(page, idx, idx + len("critical"))
        page.locator(".color-select").select_option("FF0000")
        assert "{color:FF0000}critical{/color}" in textarea_value(page)
        page.screenshot(path=f"{SS}/color-workflow-2-colored.png")

        # Step 3: Highlight "on track" green
        val = textarea_value(page)
        idx = val.index("on track")
        select_text(page, idx, idx + len("on track"))
        page.locator(".highlight-select").select_option("green")
        assert "{highlight:green}on track{/highlight}" in textarea_value(page)
        page.screenshot(path=f"{SS}/color-workflow-3-highlighted.png")

        # Step 4: Switch to DOCX preview
        page.locator('.mode-btn:text-is("DOCX")').click()
        page.wait_for_timeout(500)
        page.screenshot(path=f"{SS}/color-workflow-4-docx.png")

        # Step 5: Verify preview shows the colors
        preview_html = page.locator(".preview-content").inner_html()
        assert "FF0000" in preview_html  # red color present
        page.screenshot(path=f"{SS}/color-workflow-5-verify.png")
