import { test, expect } from '@playwright/test';

test.describe('Preview', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForSelector('.app-container', { timeout: 30000 });
  });

  test('GFM table renders in preview', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.fill('| Col A | Col B |\n|-------|-------|\n| val1  | val2  |');
    await expect(page.locator('.preview-content table')).toBeVisible();
    await expect(page.locator('.preview-content td').first()).toContainText('val1');
  });

  test('Code block renders in preview', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.fill('```rust\nfn main() {}\n```');
    await expect(page.locator('.preview-content code')).toBeVisible();
    await expect(page.locator('.preview-content code')).toContainText('fn main()');
  });

  test('Bold text renders in preview', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.fill('Some **bold words** here');
    await expect(page.locator('.preview-content strong')).toContainText('bold words');
  });

  test('Task list renders in preview', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.fill('- [x] Done\n- [ ] Todo');
    // Comrak renders checkboxes as input elements
    const checked = page.locator('.preview-content input[checked]');
    await expect(checked).toBeVisible();
  });

  test('Preview API returns markdown HTML', async ({ page }) => {
    const result = await page.evaluate(async () => {
      const resp = await fetch('/api/preview', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ content: '**bold**', mode: 'markdown' }),
      });
      return resp.json();
    });
    expect(result.html).toContain('<strong>bold</strong>');
  });

  test('Preview API returns DOCX-styled HTML', async ({ page }) => {
    const result = await page.evaluate(async () => {
      const resp = await fetch('/api/preview', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ content: 'test', mode: 'docx' }),
      });
      return resp.json();
    });
    expect(result.html).toContain('Calibri');
  });

  test('Preview API returns ODF-styled HTML', async ({ page }) => {
    const result = await page.evaluate(async () => {
      const resp = await fetch('/api/preview', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ content: 'test', mode: 'odt' }),
      });
      return resp.json();
    });
    expect(result.html).toContain('Liberation Serif');
  });

  test('XSS is blocked in preview', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.fill('<script>alert("xss")</script>');
    const content = await page.locator('.preview-content').innerHTML();
    expect(content).not.toContain('<script>');
  });
});
