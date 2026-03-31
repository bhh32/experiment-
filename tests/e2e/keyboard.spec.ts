import { test, expect } from '@playwright/test';

test.describe('Keyboard Shortcuts', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForSelector('.app-container', { timeout: 30000 });
  });

  test('Ctrl+B inserts bold markers', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.focus();
    await page.keyboard.press('Control+b');
    const value = await textarea.inputValue();
    expect(value).toContain('****');
  });

  test('Ctrl+I inserts italic markers', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.focus();
    await page.keyboard.press('Control+i');
    const value = await textarea.inputValue();
    expect(value).toContain('**');
  });

  test('Ctrl+K inserts link template', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.focus();
    await page.keyboard.press('Control+k');
    const value = await textarea.inputValue();
    expect(value).toContain('[link text](url)');
  });

  test('Ctrl+U inserts underline markers', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.focus();
    await page.keyboard.press('Control+u');
    const value = await textarea.inputValue();
    expect(value).toContain('____');
  });

  test('Tab inserts spaces', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.focus();
    await page.keyboard.press('Tab');
    const value = await textarea.inputValue();
    expect(value).toContain('    ');
  });
});
