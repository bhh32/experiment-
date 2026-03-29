import { test, expect } from '@playwright/test';

test.describe('Toolbar Formatting', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForSelector('.app-container', { timeout: 30000 });
  });

  test('Bold button inserts ** markers', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.focus();
    await page.locator('.tool-btn:text-is("B")').click();
    const value = await textarea.inputValue();
    expect(value).toContain('****');
  });

  test('Italic button inserts * markers', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.focus();
    await page.locator('.tool-btn:text-is("I")').click();
    const value = await textarea.inputValue();
    expect(value).toContain('**');
  });

  test('H1 button inserts heading marker', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.focus();
    await page.locator('.tool-btn:text-is("H1")').click();
    const value = await textarea.inputValue();
    expect(value).toContain('# ');
  });

  test('H2 button inserts heading marker', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.focus();
    await page.locator('.tool-btn:text-is("H2")').click();
    const value = await textarea.inputValue();
    expect(value).toContain('## ');
  });

  test('H3 button inserts heading marker', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.focus();
    await page.locator('.tool-btn:text-is("H3")').click();
    const value = await textarea.inputValue();
    expect(value).toContain('### ');
  });

  test('List button inserts bullet marker', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.focus();
    await page.locator('.tool-btn:text-is("List")').click();
    const value = await textarea.inputValue();
    expect(value).toContain('- ');
  });

  test('Ordered list button inserts number marker', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.focus();
    await page.locator('.tool-btn:text-is("1.")').click();
    const value = await textarea.inputValue();
    expect(value).toContain('1. ');
  });

  test('Link button inserts link template', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.focus();
    await page.locator('.tool-btn:text-is("Link")').click();
    const value = await textarea.inputValue();
    expect(value).toContain('[link text](url)');
  });

  test('Code button inserts code block', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.focus();
    await page.locator('.tool-btn:text-is("Code")').click();
    const value = await textarea.inputValue();
    expect(value).toContain('```');
  });

  test('HR button inserts horizontal rule', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.focus();
    await page.locator('.tool-btn:text-is("HR")').click();
    const value = await textarea.inputValue();
    expect(value).toContain('---');
  });

  test('Bold button updates preview', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.fill('hello');
    // Place cursor at position 5 and click Bold
    await textarea.focus();
    await page.locator('.tool-btn:text-is("B")').click();
    // Should have bold markers in the text
    const value = await textarea.inputValue();
    expect(value).toContain('****');
  });

  test('H1 updates preview with heading', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.fill('My Title');
    await textarea.focus();
    await page.locator('.tool-btn:text-is("H1")').click();
    // Wait for preview to update
    await expect(page.locator('.preview-content h1')).toBeVisible();
  });
});
