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

  test('shows file menu with all buttons', async ({ page }) => {
    await expect(page.locator('button.file-btn:text-is("Open")')).toBeVisible();
    await expect(page.locator('#save-btn')).toBeVisible();
    await expect(page.locator('#save-as-btn')).toBeVisible();
    await expect(page.locator('#export-docx-btn')).toBeVisible();
    await expect(page.locator('#export-odf-btn')).toBeVisible();
  });

  test('shows toolbar with formatting buttons', async ({ page }) => {
    const buttons = ['B', 'I', 'H1', 'H2', 'H3', 'List', '1.', 'Link', 'Code', 'HR'];
    for (const label of buttons) {
      await expect(page.locator(`.tool-btn:text-is("${label}")`)).toBeVisible();
    }
  });

  test('shows preview mode toggle', async ({ page }) => {
    await expect(page.locator('.mode-btn:text-is("Markdown")')).toBeVisible();
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
    await expect(page.locator('.file-name')).toContainText('untitled.md');
  });

  test('DOCX preview mode shows Calibri styling', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.fill('Test content');

    await page.locator('.mode-btn:text-is("DOCX")').click();
    await expect(page.locator('.mode-btn:text-is("DOCX")')).toHaveClass(/active/);

    const content = await page.locator('.preview-content').innerHTML();
    expect(content).toContain('Calibri');
  });

  test('ODF preview mode shows Liberation Serif styling', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.fill('Test content');

    await page.locator('.mode-btn:text-is("ODF")').click();
    await expect(page.locator('.mode-btn:text-is("ODF")')).toHaveClass(/active/);

    const content = await page.locator('.preview-content').innerHTML();
    expect(content).toContain('Liberation Serif');
  });

  test('markdown preview mode is default and active', async ({ page }) => {
    await expect(page.locator('.mode-btn:text-is("Markdown")')).toHaveClass(/active/);
  });
});
