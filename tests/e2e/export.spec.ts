import { test, expect } from '@playwright/test';

test.describe('Export', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForSelector('.app-container', { timeout: 30000 });
  });

  test('Export DOCX via File menu', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.fill('# Export Test\n\nContent.');

    const downloadPromise = page.waitForEvent('download', { timeout: 15000 });
    await page.locator('.menu-item:text-is("File")').click();
    await page.locator('#export-docx-btn').click();

    const download = await downloadPromise;
    expect(download.suggestedFilename()).toContain('.docx');
    await expect(page.locator('.status-msg')).toContainText('Exported');
  });

  test('Export ODF via File menu', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.fill('# ODF Export\n\nContent.');

    const downloadPromise = page.waitForEvent('download', { timeout: 15000 });
    await page.locator('.menu-item:text-is("File")').click();
    await page.locator('#export-odf-btn').click();

    const download = await downloadPromise;
    expect(download.suggestedFilename()).toContain('.odt');
    await expect(page.locator('.status-msg')).toContainText('Exported');
  });

  test('DOCX export produces valid ZIP file', async ({ page }) => {
    const result = await page.evaluate(async () => {
      const resp = await fetch('/api/convert', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ content: 'Test', from: 'markdown', to: 'docx' }),
      });
      const data = await resp.json();
      const binary = atob(data.content.substring(0, 4));
      return { first: binary.charCodeAt(0), second: binary.charCodeAt(1), format: data.format };
    });
    expect(result.first).toBe(0x50);
    expect(result.second).toBe(0x4b);
    expect(result.format).toBe('docx');
  });
});
