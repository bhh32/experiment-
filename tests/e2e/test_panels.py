"""Tests for collapsible preview and sidebar panels."""
import pytest
from playwright.sync_api import Page, expect

BASE_URL = "http://localhost:8080"
SCREENSHOT_DIR = "test-results"


@pytest.fixture(autouse=True)
def navigate(page: Page):
    page.goto(BASE_URL)
    page.wait_for_selector(".app-container", timeout=30000)


# --- Preview Panel ---

def test_preview_visible_by_default(page: Page):
    expect(page.locator(".preview-pane")).to_be_visible()
    expect(page.locator(".preview-scroll-area")).to_be_visible()
    page.screenshot(path=f"{SCREENSHOT_DIR}/panels-default-layout.png")


def test_preview_border_toggle_visible(page: Page):
    expect(page.locator(".preview-border-toggle")).to_be_visible()


def test_clicking_preview_toggle_hides_preview(page: Page):
    page.locator(".preview-border-toggle").click()
    expect(page.locator(".preview-pane")).not_to_be_visible()
    page.screenshot(path=f"{SCREENSHOT_DIR}/panels-preview-collapsed.png")


def test_editor_expands_when_preview_hidden(page: Page):
    page.locator(".preview-border-toggle").click()
    expect(page.locator(".editor-pane")).to_have_class("editor-pane editor-pane-full")
    page.screenshot(path=f"{SCREENSHOT_DIR}/panels-editor-full-width.png")


def test_editor_centers_when_preview_collapsed(page: Page):
    page.locator(".editor-textarea").fill("# Centered Editor\n\nThis should be centered.")
    page.locator(".preview-border-toggle").click()
    expect(page.locator(".editor-pane")).to_have_class("editor-pane editor-pane-full")
    page.screenshot(path=f"{SCREENSHOT_DIR}/panels-editor-centered.png")


def test_clicking_preview_toggle_again_restores(page: Page):
    page.locator(".preview-border-toggle").click()
    expect(page.locator(".preview-pane")).not_to_be_visible()
    page.locator(".preview-border-toggle").click()
    expect(page.locator(".preview-pane")).to_be_visible()
    page.screenshot(path=f"{SCREENSHOT_DIR}/panels-preview-restored.png")


def test_editor_returns_to_half_when_preview_restored(page: Page):
    page.locator(".preview-border-toggle").click()
    page.locator(".preview-border-toggle").click()
    expect(page.locator(".editor-pane")).not_to_have_class("editor-pane-full")


# --- Sidebar Panel ---

def test_sidebar_visible_by_default(page: Page):
    expect(page.locator(".styles-sidebar")).to_be_visible()


def test_sidebar_border_toggle_visible(page: Page):
    expect(page.locator(".sidebar-border-toggle")).to_be_visible()


def test_clicking_sidebar_toggle_hides_sidebar(page: Page):
    page.locator(".sidebar-border-toggle").click()
    expect(page.locator(".styles-sidebar")).not_to_be_visible()
    page.screenshot(path=f"{SCREENSHOT_DIR}/panels-sidebar-collapsed.png")


def test_clicking_sidebar_toggle_again_restores(page: Page):
    page.locator(".sidebar-border-toggle").click()
    page.locator(".sidebar-border-toggle").click()
    expect(page.locator(".styles-sidebar")).to_be_visible()
    page.screenshot(path=f"{SCREENSHOT_DIR}/panels-sidebar-restored.png")


# --- Sidebar Tabs ---

def test_styles_tab_active_by_default(page: Page):
    expect(page.locator(".sidebar-tab.active")).to_contain_text("Styles")
    expect(page.locator(".styles-list")).to_be_visible()


def test_properties_tab_shows_document_info(page: Page):
    page.locator('.sidebar-tab:text-is("Properties")').click()
    expect(page.locator('.sidebar-tab:text-is("Properties")')).to_have_class("sidebar-tab active")
    expect(page.locator(".properties-list").first).to_be_visible()
    page.screenshot(path=f"{SCREENSHOT_DIR}/panels-properties-tab.png")


def test_properties_shows_font_info(page: Page):
    page.locator('.sidebar-tab:text-is("Properties")').click()
    text = page.locator(".properties-list").first.text_content()
    assert "pt" in text


def test_properties_shows_word_count(page: Page):
    page.locator(".editor-textarea").fill("Hello world this is a test.")
    page.locator('.sidebar-tab:text-is("Properties")').click()
    stats = page.locator(".properties-list").nth(1)
    text = stats.text_content()
    assert "Words" in text
    page.screenshot(path=f"{SCREENSHOT_DIR}/panels-properties-stats.png")


def test_switching_back_to_styles(page: Page):
    page.locator('.sidebar-tab:text-is("Properties")').click()
    expect(page.locator(".properties-list").first).to_be_visible()
    page.locator('.sidebar-tab:text-is("Styles")').click()
    expect(page.locator(".styles-list")).to_be_visible()


# --- Both panels collapsed ---

def test_both_panels_collapsed(page: Page):
    page.locator(".editor-textarea").fill("# Focus Mode\n\nNo distractions.")
    page.locator(".preview-border-toggle").click()
    page.locator(".sidebar-border-toggle").click()
    expect(page.locator(".preview-pane")).not_to_be_visible()
    expect(page.locator(".styles-sidebar")).not_to_be_visible()
    expect(page.locator(".editor-pane")).to_have_class("editor-pane editor-pane-full")
    page.screenshot(path=f"{SCREENSHOT_DIR}/panels-both-collapsed.png")
