import { test, expect } from '@playwright/test';

test.describe('Editor View', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForSelector('.app-container', { timeout: 30000 });
  });

  test('opens directly to editor (no home page)', async ({ page }) => {
    await expect(page.locator('.app-container')).toBeVisible();
    await expect(page.locator('.editor-textarea')).toBeVisible();
  });

  test('shows menu bar with File, Edit, View, etc.', async ({ page }) => {
    await expect(page.locator('.menu-item:text-is("File")')).toBeVisible();
    await expect(page.locator('.menu-item:text-is("Edit")')).toBeVisible();
    await expect(page.locator('.menu-item:text-is("View")')).toBeVisible();
    await expect(page.locator('.menu-item:text-is("Insert")')).toBeVisible();
    await expect(page.locator('.menu-item:text-is("Format")')).toBeVisible();
    await expect(page.locator('.menu-item:text-is("Styles")')).toBeVisible();
    await expect(page.locator('.menu-item:text-is("Table")')).toBeVisible();
    await expect(page.locator('.menu-item:text-is("Tools")')).toBeVisible();
    await expect(page.locator('.menu-item:text-is("Help")')).toBeVisible();
  });

  test('shows toolbar with formatting buttons', async ({ page }) => {
    const buttons = ['B', 'I', 'H1', 'H2', 'H3', 'List', '1.', 'Link', 'Code', 'HR'];
    for (const label of buttons) {
      await expect(page.locator(`.tool-btn:text-is("${label}")`)).toBeVisible();
    }
  });

  test('shows font, size, and line height selectors', async ({ page }) => {
    await expect(page.locator('.font-select')).toBeVisible();
    await expect(page.locator('.font-size-select')).toBeVisible();
    await expect(page.locator('.line-height-select')).toBeVisible();
  });

  test('shows preview mode toggle', async ({ page }) => {
    await expect(page.locator('.mode-btn:text-is("Print")')).toBeVisible();
    await expect(page.locator('.mode-btn:text-is("DOCX")')).toBeVisible();
    await expect(page.locator('.mode-btn:text-is("ODF")')).toBeVisible();
  });

  test('typing updates live preview', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.fill('# Hello World\n\nThis is **bold** text.');
    await expect(page.locator('.preview-content h1')).toContainText('Hello World');
    await expect(page.locator('.preview-content strong')).toContainText('bold');
  });

  test('textarea has spellcheck enabled', async ({ page }) => {
    const spellcheck = await page.locator('.editor-textarea').getAttribute('spellcheck');
    expect(spellcheck).toBe('true');
  });

  test('shows untitled.md as default filename', async ({ page }) => {
    await expect(page.locator('.file-name-display')).toContainText('untitled.md');
  });

  test('DOCX preview mode activates', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.fill('Test content');
    await page.locator('.mode-btn:text-is("DOCX")').click();
    await expect(page.locator('.mode-btn:text-is("DOCX")')).toHaveClass(/active/);
  });

  test('ODF preview mode activates', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.fill('Test content');
    await page.locator('.mode-btn:text-is("ODF")').click();
    await expect(page.locator('.mode-btn:text-is("ODF")')).toHaveClass(/active/);
  });

  test('Print preview mode is default', async ({ page }) => {
    await expect(page.locator('.mode-btn:text-is("Print")')).toHaveClass(/active/);
  });

  test('shows styles sidebar', async ({ page }) => {
    await expect(page.locator('.styles-sidebar')).toBeVisible();
    await expect(page.locator('.style-item:has-text("Heading 1")')).toBeVisible();
    await expect(page.locator('.style-item:has-text("Body Text")')).toBeVisible();
    await expect(page.locator('.style-item:has-text("Code")')).toBeVisible();
    await expect(page.locator('.style-item:has-text("Quote")')).toBeVisible();
  });

  test('shows status bar with word and character counts', async ({ page }) => {
    await expect(page.locator('.status-bar')).toBeVisible();
    await expect(page.locator('.status-bar')).toContainText('Words:');
    await expect(page.locator('.status-bar')).toContainText('Characters:');

    const textarea = page.locator('.editor-textarea');
    await textarea.fill('Hello world test');
    await expect(page.locator('.status-bar')).toContainText('Words: 3');
  });

  test('preview shows document page on gray background', async ({ page }) => {
    const scrollArea = page.locator('.preview-scroll-area');
    const bg = await scrollArea.evaluate(el => getComputedStyle(el).backgroundColor);
    // Gray background (rgb(128, 128, 128) = #808080)
    expect(bg).toContain('128');
  });

  test('alignment buttons are visible (L, C, R, J)', async ({ page }) => {
    await expect(page.locator('.tool-btn:text-is("L")')).toBeVisible();
    await expect(page.locator('.tool-btn:text-is("C")')).toBeVisible();
    await expect(page.locator('.tool-btn:text-is("R")')).toBeVisible();
    await expect(page.locator('.tool-btn:text-is("J")')).toBeVisible();
  });
});
