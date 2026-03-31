import { test, expect } from '@playwright/test';

test.describe('Toolbar Formatting', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForSelector('.app-container', { timeout: 30000 });
  });

  test('Bold button inserts ** markers', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.focus();
    await page.locator('[title="Bold (Ctrl+B)"]').click();
    const value = await textarea.inputValue();
    expect(value).toContain('****');
  });

  test('Italic button inserts * markers', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.focus();
    await page.locator('[title="Italic (Ctrl+I)"]').click();
    const value = await textarea.inputValue();
    expect(value).toContain('**');
  });

  test('Underline button inserts __ markers', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.focus();
    await page.locator('[title="Underline (Ctrl+U)"]').click();
    const value = await textarea.inputValue();
    expect(value).toContain('____');
  });

  test('Strikethrough button inserts ~~ markers', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.focus();
    await page.locator('[title="Strikethrough"]').click();
    const value = await textarea.inputValue();
    expect(value).toContain('~~~~');
  });

  test('H1 button inserts heading marker', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.focus();
    // Use paragraph style dropdown
    await page.locator('.style-select').selectOption('h1');
    const value = await textarea.inputValue();
    expect(value).toContain('# ');
  });

  test('List button inserts bullet marker', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.focus();
    await page.locator('[title="Bullet List"]').click();
    const value = await textarea.inputValue();
    expect(value).toContain('- ');
  });

  test('Numbered list button inserts number marker', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.focus();
    await page.locator('[title="Numbered List"]').click();
    const value = await textarea.inputValue();
    expect(value).toContain('1. ');
  });

  test('Link button inserts link template', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.focus();
    await page.locator('[title="Link (Ctrl+K)"]').click();
    const value = await textarea.inputValue();
    expect(value).toContain('[link text](url)');
  });

  test('Code button inserts code block', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.focus();
    await page.locator('[title="Code Block"]').click();
    const value = await textarea.inputValue();
    expect(value).toContain('```');
  });

  test('HR button inserts horizontal rule', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.focus();
    await page.locator('[title="Horizontal Rule"]').click();
    const value = await textarea.inputValue();
    expect(value).toContain('---');
  });

  test('Bold button updates preview', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.fill('hello');
    await textarea.focus();
    await page.locator('[title="Bold (Ctrl+B)"]').click();
    const value = await textarea.inputValue();
    expect(value).toContain('****');
  });

  test('H1 updates preview with heading', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.fill('My Title');
    await textarea.focus();
    await page.locator('.style-select').selectOption('h1');
    await expect(page.locator('.preview-content h1')).toBeVisible();
  });

  test('Bold wraps selected text', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.fill('select this text');
    await textarea.focus();
    // Select "this"
    await textarea.evaluate((el: HTMLTextAreaElement) => {
      el.setSelectionRange(7, 11);
    });
    await page.locator('[title="Bold (Ctrl+B)"]').click();
    const value = await textarea.inputValue();
    expect(value).toContain('**this**');
  });

  test('Italic wraps selected text', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.fill('select this text');
    await textarea.focus();
    await textarea.evaluate((el: HTMLTextAreaElement) => {
      el.setSelectionRange(7, 11);
    });
    await page.locator('[title="Italic (Ctrl+I)"]').click();
    const value = await textarea.inputValue();
    expect(value).toContain('*this*');
  });

  test('Underline wraps selected text', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.fill('select this text');
    await textarea.focus();
    await textarea.evaluate((el: HTMLTextAreaElement) => {
      el.setSelectionRange(7, 11);
    });
    await page.locator('[title="Underline (Ctrl+U)"]').click();
    const value = await textarea.inputValue();
    expect(value).toContain('__this__');
  });
});
