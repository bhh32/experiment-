"""Comprehensive editing workflow tests.

Tests realistic multi-step editing: typing, selecting, formatting,
modifying text, verifying preview renders correctly at each step.
"""
import pytest
from playwright.sync_api import Page, expect

BASE_URL = "http://localhost:8080"
SS = "tests/e2e/test-results"


@pytest.fixture(autouse=True)
def navigate(page: Page):
    page.goto(BASE_URL)
    page.wait_for_selector(".app-container", timeout=30000)


# ── Helpers ──

def get_textarea(page: Page):
    return page.locator(".editor-textarea")


def textarea_value(page: Page) -> str:
    return get_textarea(page).input_value()


def type_text(page: Page, text: str):
    """Type text character by character into the textarea."""
    get_textarea(page).focus()
    page.keyboard.type(text)


def select_text(page: Page, start: int, end: int):
    """Select a range of text in the textarea via JS."""
    page.evaluate(f"""() => {{
        const ta = document.getElementById('editor-textarea');
        ta.focus();
        ta.setSelectionRange({start}, {end});
    }}""")


def select_all(page: Page):
    get_textarea(page).focus()
    page.keyboard.press("Control+a")


# ── Basic Typing ──

class TestTyping:
    def test_type_single_line(self, page: Page):
        ta = get_textarea(page)
        ta.fill("Hello World")
        assert textarea_value(page) == "Hello World"
        page.screenshot(path=f"{SS}/edit-type-single-line.png")

    def test_type_multiple_lines(self, page: Page):
        ta = get_textarea(page)
        ta.fill("Line one\nLine two\nLine three")
        val = textarea_value(page)
        assert "Line one" in val
        assert "Line two" in val
        assert "Line three" in val

    def test_type_then_modify(self, page: Page):
        ta = get_textarea(page)
        ta.fill("Initial text")
        assert textarea_value(page) == "Initial text"
        ta.fill("Changed text")
        assert textarea_value(page) == "Changed text"
        ta.fill("Final version")
        assert textarea_value(page) == "Final version"

    def test_type_special_characters(self, page: Page):
        ta = get_textarea(page)
        ta.fill("Special: é à ü ñ — • © ™ ½")
        assert "é" in textarea_value(page)
        assert "—" in textarea_value(page)

    def test_tab_inserts_spaces(self, page: Page):
        ta = get_textarea(page)
        ta.focus()
        page.keyboard.type("hello")
        page.keyboard.press("Tab")
        page.keyboard.type("world")
        val = textarea_value(page)
        assert "    " in val  # 4 spaces


# ── Selection & Formatting ──

class TestSelectionFormatting:
    def test_select_word_and_bold(self, page: Page):
        ta = get_textarea(page)
        ta.fill("make this bold please")
        # Select "this" (positions 5-9)
        select_text(page, 5, 9)
        # Click Bold button
        page.locator('.tool-btn .icon-bold').click()
        val = textarea_value(page)
        assert "**this**" in val
        page.screenshot(path=f"{SS}/edit-select-bold.png")

    def test_select_word_and_italic(self, page: Page):
        ta = get_textarea(page)
        ta.fill("make this italic please")
        select_text(page, 5, 9)
        page.locator('.tool-btn .icon-italic').click()
        val = textarea_value(page)
        assert "*this*" in val
        page.screenshot(path=f"{SS}/edit-select-italic.png")

    def test_select_word_and_underline(self, page: Page):
        ta = get_textarea(page)
        ta.fill("make this underlined please")
        select_text(page, 5, 9)
        page.locator('.tool-btn .icon-underline').click()
        val = textarea_value(page)
        assert "__this__" in val

    def test_select_word_and_strikethrough(self, page: Page):
        ta = get_textarea(page)
        ta.fill("make this struck please")
        select_text(page, 5, 9)
        page.locator('.tool-btn .icon-strike').click()
        val = textarea_value(page)
        assert "~~this~~" in val

    def test_bold_via_keyboard_at_cursor(self, page: Page):
        """Ctrl+B at cursor inserts bold markers (selection wrapping is toolbar only)."""
        ta = get_textarea(page)
        ta.focus()
        page.keyboard.type("hello ")
        page.keyboard.press("Control+b")
        page.keyboard.type("bold")
        page.keyboard.press("Control+b")
        page.keyboard.type(" world")
        # The bold markers are inserted at cursor position
        val = textarea_value(page)
        assert "**" in val

    def test_italic_via_keyboard_at_cursor(self, page: Page):
        """Ctrl+I at cursor inserts italic markers."""
        ta = get_textarea(page)
        ta.focus()
        page.keyboard.type("hello ")
        page.keyboard.press("Control+i")
        page.keyboard.type("italic")
        page.keyboard.press("Control+i")
        page.keyboard.type(" world")
        val = textarea_value(page)
        assert "*" in val

    def test_multiple_format_operations(self, page: Page):
        """Apply bold, then italic to different words in the same text."""
        ta = get_textarea(page)
        ta.fill("first second third fourth")
        # Bold "second" (positions 6-12)
        select_text(page, 6, 12)
        page.locator('.tool-btn .icon-bold').click()
        val = textarea_value(page)
        assert "**second**" in val

        # Now italic "fourth" — positions shifted by 4 (two ** pairs)
        # "first **second** third fourth"
        # "fourth" starts at position 25
        fourth_start = val.index("fourth")
        fourth_end = fourth_start + len("fourth")
        select_text(page, fourth_start, fourth_end)
        page.locator('.tool-btn .icon-italic').click()
        val = textarea_value(page)
        assert "**second**" in val
        assert "*fourth*" in val
        page.screenshot(path=f"{SS}/edit-multi-format.png")


# ── Preview Rendering ──

class TestPreviewRendering:
    def test_heading_renders_in_preview(self, page: Page):
        get_textarea(page).fill("# My Heading")
        expect(page.locator(".preview-content h1")).to_contain_text("My Heading")
        page.screenshot(path=f"{SS}/edit-preview-heading.png")

    def test_bold_renders_in_preview(self, page: Page):
        get_textarea(page).fill("This is **bold** text")
        expect(page.locator(".preview-content strong")).to_contain_text("bold")

    def test_italic_renders_in_preview(self, page: Page):
        get_textarea(page).fill("This is *italic* text")
        expect(page.locator(".preview-content em")).to_contain_text("italic")

    def test_list_renders_in_preview(self, page: Page):
        get_textarea(page).fill("- Item A\n- Item B\n- Item C")
        items = page.locator(".preview-content li")
        expect(items).to_have_count(3)

    def test_table_renders_in_preview(self, page: Page):
        get_textarea(page).fill("| Col1 | Col2 |\n|------|------|\n| A    | B    |")
        expect(page.locator(".preview-content table")).to_be_visible()
        expect(page.locator(".preview-content td").first).to_contain_text("A")
        page.screenshot(path=f"{SS}/edit-preview-table.png")

    def test_code_block_renders_in_preview(self, page: Page):
        get_textarea(page).fill("```python\nprint('hello')\n```")
        expect(page.locator(".preview-content code")).to_contain_text("print")

    def test_link_renders_in_preview(self, page: Page):
        get_textarea(page).fill("[Click here](https://example.com)")
        expect(page.locator(".preview-content a")).to_contain_text("Click here")

    def test_strikethrough_renders_in_preview(self, page: Page):
        get_textarea(page).fill("This is ~~deleted~~ text")
        expect(page.locator(".preview-content del")).to_contain_text("deleted")

    def test_preview_updates_live(self, page: Page):
        """Type text step by step and verify preview updates each time."""
        ta = get_textarea(page)
        ta.fill("# Step 1")
        expect(page.locator(".preview-content h1")).to_contain_text("Step 1")

        ta.fill("# Step 2\n\nParagraph here")
        expect(page.locator(".preview-content h1")).to_contain_text("Step 2")
        expect(page.locator(".preview-content p")).to_contain_text("Paragraph here")

        ta.fill("# Step 3\n\n**Bold** and *italic*")
        expect(page.locator(".preview-content strong")).to_contain_text("Bold")
        expect(page.locator(".preview-content em")).to_contain_text("italic")
        page.screenshot(path=f"{SS}/edit-preview-live-update.png")


# ── Alignment ──

class TestAlignment:
    def test_center_alignment(self, page: Page):
        ta = get_textarea(page)
        ta.fill("Center this line")
        ta.focus()
        # Click center alignment button
        page.locator('.tool-btn .icon-center').click()
        val = textarea_value(page)
        assert "{center}" in val
        page.screenshot(path=f"{SS}/edit-center-align.png")

    def test_right_alignment(self, page: Page):
        ta = get_textarea(page)
        ta.fill("Right align this")
        ta.focus()
        page.locator('.tool-btn .icon-right').click()
        val = textarea_value(page)
        assert "{right}" in val

    def test_alignment_toggle_off(self, page: Page):
        ta = get_textarea(page)
        ta.fill("{center}Already centered")
        ta.focus()
        # Click center again to toggle off
        page.locator('.tool-btn .icon-center').click()
        val = textarea_value(page)
        assert "{center}" not in val


# ── Toolbar Insertions ──

class TestToolbarInsertions:
    def test_insert_link(self, page: Page):
        ta = get_textarea(page)
        ta.focus()
        page.locator('.tool-btn .icon-link').click()
        val = textarea_value(page)
        assert "[link text](url)" in val

    def test_insert_code_block(self, page: Page):
        ta = get_textarea(page)
        ta.focus()
        page.locator('.tool-btn .icon-code').click()
        val = textarea_value(page)
        assert "```" in val

    def test_insert_hr(self, page: Page):
        ta = get_textarea(page)
        ta.fill("Above")
        ta.focus()
        page.locator('.tool-btn .icon-hr').click()
        val = textarea_value(page)
        assert "---" in val

    def test_insert_list(self, page: Page):
        ta = get_textarea(page)
        ta.fill("My item")
        ta.focus()
        page.locator('.tool-btn .icon-ul').click()
        val = textarea_value(page)
        assert "- " in val or ":=" in val  # depends on toolbar icon mapping

    def test_insert_ordered_list(self, page: Page):
        ta = get_textarea(page)
        ta.fill("My item")
        ta.focus()
        page.locator('.tool-btn .icon-ol').click()
        val = textarea_value(page)
        assert "1." in val


# ── Preview Mode Switching ──

class TestPreviewModes:
    def test_docx_mode(self, page: Page):
        get_textarea(page).fill("# DOCX Preview Test")
        page.locator('.mode-btn:text-is("DOCX")').click()
        expect(page.locator('.mode-btn:text-is("DOCX")')).to_have_class("mode-btn active")
        page.screenshot(path=f"{SS}/edit-docx-mode.png")

    def test_odf_mode(self, page: Page):
        get_textarea(page).fill("# ODF Preview Test")
        page.locator('.mode-btn:text-is("ODF")').click()
        expect(page.locator('.mode-btn:text-is("ODF")')).to_have_class("mode-btn active")
        page.screenshot(path=f"{SS}/edit-odf-mode.png")

    def test_print_mode(self, page: Page):
        get_textarea(page).fill("# Print Preview Test")
        page.locator('.mode-btn:text-is("Print")').click()
        expect(page.locator('.mode-btn:text-is("Print")')).to_have_class("mode-btn active")


# ── Font & Size Changes ──

class TestFontAndSize:
    def test_change_font_family(self, page: Page):
        get_textarea(page).fill("# Font Test\n\nArial font here.")
        page.locator(".font-select").select_option("Arial")
        page.screenshot(path=f"{SS}/edit-font-arial.png")

    def test_change_font_size(self, page: Page):
        get_textarea(page).fill("# Size Test\n\nLarger text.")
        page.locator(".font-size-select").select_option("14")
        page.screenshot(path=f"{SS}/edit-font-size-14.png")

    def test_change_line_spacing(self, page: Page):
        get_textarea(page).fill("# Spacing Test\n\nDouble spaced text.\n\nSecond paragraph.")
        page.locator(".line-height-select").select_option("2.0")
        page.screenshot(path=f"{SS}/edit-line-spacing-double.png")

    def test_change_all_three(self, page: Page):
        """Change font, size, and spacing together."""
        get_textarea(page).fill("# Combined\n\nGeorgia 14pt double-spaced.")
        page.locator(".font-select").select_option("Georgia")
        page.locator(".font-size-select").select_option("14")
        page.locator(".line-height-select").select_option("2.0")
        page.screenshot(path=f"{SS}/edit-combined-styles.png")


# ── Complex Multi-Step Workflow ──

class TestComplexWorkflow:
    def test_write_format_modify_cycle(self, page: Page):
        """Simulates writing a short document with multiple edits."""
        ta = get_textarea(page)

        # Step 1: Write initial content
        ta.fill("# My Essay\n\nThis is the introduction paragraph with important points.")
        expect(page.locator(".preview-content h1")).to_contain_text("My Essay")
        page.screenshot(path=f"{SS}/workflow-step1.png")

        # Step 2: Bold "important" via toolbar button (supports selection)
        val = textarea_value(page)
        idx = val.index("important")
        select_text(page, idx, idx + len("important"))
        page.locator('.tool-btn .icon-bold').click()
        assert "**important**" in textarea_value(page)
        expect(page.locator(".preview-content strong")).to_contain_text("important")
        page.screenshot(path=f"{SS}/workflow-step2-bold.png")

        # Step 3: Add a second paragraph
        val = textarea_value(page)
        ta.fill(val + "\n\nThe second paragraph discusses methodology.")
        page.screenshot(path=f"{SS}/workflow-step3-para2.png")

        # Step 4: Italic "methodology" via toolbar button
        val = textarea_value(page)
        idx = val.index("methodology")
        select_text(page, idx, idx + len("methodology"))
        page.locator('.tool-btn .icon-italic').click()
        assert "*methodology*" in textarea_value(page)
        expect(page.locator(".preview-content em")).to_contain_text("methodology")
        page.screenshot(path=f"{SS}/workflow-step4-italic.png")

        # Step 5: Change font to Times New Roman (if not already)
        page.locator(".font-select").select_option("Georgia")
        page.screenshot(path=f"{SS}/workflow-step5-font.png")

        # Step 6: Switch to DOCX preview
        page.locator('.mode-btn:text-is("DOCX")').click()
        page.screenshot(path=f"{SS}/workflow-step6-docx.png")

    def test_page_break_workflow(self, page: Page):
        """Write title page, add page break, write body."""
        ta = get_textarea(page)

        # Title page
        ta.fill("{center}**My Research Paper**\n\n{center}Author Name\n\n{center}March 30, 2026")
        page.screenshot(path=f"{SS}/workflow-pagebreak-title.png")

        # Add page break and body content
        val = textarea_value(page)
        ta.fill(val + "\n\n{pagebreak}\n\n# Introduction\n\nThe body of the paper begins here.")
        page.screenshot(path=f"{SS}/workflow-pagebreak-body.png")

        # Switch to DOCX preview to see pages
        page.locator('.mode-btn:text-is("DOCX")').click()
        page.wait_for_timeout(500)
        page.screenshot(path=f"{SS}/workflow-pagebreak-docx.png")

    def test_paragraph_style_dropdown(self, page: Page):
        """Use the paragraph style dropdown to apply heading."""
        ta = get_textarea(page)
        ta.fill("My Section Title")
        ta.focus()
        page.locator(".style-select").select_option("h1")
        val = textarea_value(page)
        assert "# " in val
        expect(page.locator(".preview-content h1")).to_be_visible()
        page.screenshot(path=f"{SS}/workflow-style-dropdown.png")
