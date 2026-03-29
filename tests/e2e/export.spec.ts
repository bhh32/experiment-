import { test, expect } from '@playwright/test';

test.describe('Export', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForSelector('.app-container', { timeout: 30000 });
  });

  test('Export DOCX button calls convert API', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.fill('# Export Test\n\nContent to export.');

    // Listen for the download
    const downloadPromise = page.waitForEvent('download', { timeout: 15000 });
    await page.locator('#export-docx-btn').click();

    const download = await downloadPromise;
    expect(download.suggestedFilename()).toContain('.docx');

    // Status should show export success
    await expect(page.locator('.status-bar')).toContainText('Exported');
  });

  test('Export ODF button calls convert API', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.fill('# ODF Export\n\nODF content.');

    const downloadPromise = page.waitForEvent('download', { timeout: 15000 });
    await page.locator('#export-odf-btn').click();

    const download = await downloadPromise;
    expect(download.suggestedFilename()).toContain('.odt');

    await expect(page.locator('.status-bar')).toContainText('Exported');
  });

  test('DOCX export produces valid ZIP file', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.fill('Test');

    // Call convert API directly to verify output format
    const result = await page.evaluate(async () => {
      const resp = await fetch('/api/convert', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ content: 'Test', from: 'markdown', to: 'docx' }),
      });
      const data = await resp.json();
      // Decode first 4 base64 chars to check PK header
      const binary = atob(data.content.substring(0, 4));
      return { first: binary.charCodeAt(0), second: binary.charCodeAt(1), format: data.format };
    });

    // PK is 0x50 0x4B (ZIP header)
    expect(result.first).toBe(0x50);
    expect(result.second).toBe(0x4b);
    expect(result.format).toBe('docx');
  });
});
