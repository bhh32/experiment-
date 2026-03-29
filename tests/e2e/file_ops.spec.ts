import { test, expect, type Page } from '@playwright/test';

// Helper to save a file via API for test setup
async function saveFileViaApi(page: Page, name: string, content: string) {
  await page.evaluate(
    async ({ name, content }) => {
      await fetch(`/api/files/${name}`, { method: 'PUT', body: content });
    },
    { name, content }
  );
}

async function deleteFileViaApi(page: Page, name: string) {
  await page.evaluate(
    async ({ name }) => {
      await fetch(`/api/files/${name}`, { method: 'DELETE' });
    },
    { name }
  );
}

test.describe('File Operations', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForSelector('.app-container', { timeout: 30000 });
  });

  test('Save button saves file to server', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.fill('# Test Save\n\nSaved content.');

    await page.locator('#save-btn').click();

    // Check status bar shows success
    await expect(page.locator('.status-bar')).toContainText('Saved');

    // Verify via API
    const resp = await page.evaluate(async () => {
      const r = await fetch('/api/files/untitled.md');
      return r.text();
    });
    expect(resp).toContain('# Test Save');

    // Cleanup
    await deleteFileViaApi(page, 'untitled.md');
  });

  test('Open button shows file list dialog', async ({ page }) => {
    // Create a test file first
    await saveFileViaApi(page, 'test-open.md', '# Open Me');

    await page.locator('button.file-btn:text-is("Open")').click();

    // Dialog should appear
    await expect(page.locator('.dialog')).toBeVisible();
    await expect(page.locator('.dialog h3')).toContainText('Open File');

    // File should be listed
    await expect(page.locator('.file-list-item:text("test-open.md")')).toBeVisible();

    // Cleanup
    await page.locator('.dialog-close').click();
    await deleteFileViaApi(page, 'test-open.md');
  });

  test('Clicking file in Open dialog loads it', async ({ page }) => {
    await saveFileViaApi(page, 'load-me.md', '# Loaded Content\n\nParagraph here.');

    await page.locator('button.file-btn:text-is("Open")').click();
    await page.locator('.file-list-item:text("load-me.md")').click();

    // Dialog should close
    await expect(page.locator('.dialog')).not.toBeVisible();

    // Content should be loaded
    const textarea = page.locator('.editor-textarea');
    await expect(textarea).toHaveValue(/Loaded Content/);

    // Filename should update
    await expect(page.locator('.file-name')).toContainText('load-me.md');

    // Status should show opened
    await expect(page.locator('.status-bar')).toContainText('Opened');

    // Cleanup
    await deleteFileViaApi(page, 'load-me.md');
  });

  test('Cancel button closes Open dialog', async ({ page }) => {
    await page.locator('button.file-btn:text-is("Open")').click();
    await expect(page.locator('.dialog')).toBeVisible();

    await page.locator('.dialog-close').click();
    await expect(page.locator('.dialog')).not.toBeVisible();
  });

  test('Clicking overlay closes Open dialog', async ({ page }) => {
    await page.locator('button.file-btn:text-is("Open")').click();
    await expect(page.locator('.dialog')).toBeVisible();

    // Click on overlay (outside dialog)
    await page.locator('.dialog-overlay').click({ position: { x: 10, y: 10 } });
    await expect(page.locator('.dialog')).not.toBeVisible();
  });

  test('Full save and reopen roundtrip', async ({ page }) => {
    // Type content
    const textarea = page.locator('.editor-textarea');
    await textarea.fill('# Roundtrip Test\n\n- Item 1\n- Item 2');

    // Save
    await page.locator('#save-btn').click();
    await expect(page.locator('.status-bar')).toContainText('Saved');

    // Clear editor
    await textarea.fill('');

    // Open the file
    await page.locator('button.file-btn:text-is("Open")').click();
    await page.locator('.file-list-item:text("untitled.md")').click();

    // Verify content restored
    await expect(textarea).toHaveValue(/Roundtrip Test/);
    await expect(textarea).toHaveValue(/Item 1/);

    // Cleanup
    await deleteFileViaApi(page, 'untitled.md');
  });

  test('Ctrl+S saves file', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.fill('Ctrl+S test');
    await textarea.focus();
    await page.keyboard.press('Control+s');

    await expect(page.locator('.status-bar')).toContainText('Saved');

    // Cleanup
    await deleteFileViaApi(page, 'untitled.md');
  });

  test('Empty open dialog shows message', async ({ page }) => {
    // Make sure no files exist
    const files = await page.evaluate(async () => {
      const r = await fetch('/api/files');
      return r.json();
    });
    for (const f of files as Array<{ name: string }>) {
      await deleteFileViaApi(page, f.name);
    }

    await page.locator('button.file-btn:text-is("Open")').click();
    await expect(page.locator('.empty-msg')).toContainText('No documents found');
    await page.locator('.dialog-close').click();
  });
});
