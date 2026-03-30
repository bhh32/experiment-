import { test, expect, type Page } from '@playwright/test';

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

  test('File menu opens dropdown with Save, Open, Export', async ({ page }) => {
    await page.locator('.menu-item:text-is("File")').click();
    await expect(page.locator('.dropdown-menu')).toBeVisible();
    await expect(page.locator('.dropdown-item:has-text("Open")')).toBeVisible();
    await expect(page.locator('#save-btn')).toBeVisible();
    await expect(page.locator('#save-as-btn')).toBeVisible();
    await expect(page.locator('.dropdown-item:has-text("Export as DOCX")')).toBeVisible();
    await expect(page.locator('.dropdown-item:has-text("Export as ODF")')).toBeVisible();
  });

  test('Save via File menu saves file', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.fill('# Test Save');

    await page.locator('.menu-item:text-is("File")').click();
    await page.locator('#save-btn').click();
    await expect(page.locator('.status-msg')).toContainText('Saved');

    // Verify via API
    const resp = await page.evaluate(async () => {
      const r = await fetch('/api/files/untitled.md');
      return r.text();
    });
    expect(resp).toContain('# Test Save');
    await deleteFileViaApi(page, 'untitled.md');
  });

  test('Open shows file list dialog', async ({ page }) => {
    await saveFileViaApi(page, 'test-open.md', '# Open Me');

    await page.locator('.menu-item:text-is("File")').click();
    await page.locator('.dropdown-item:has-text("Open")').click();

    await expect(page.locator('.dialog')).toBeVisible();
    await expect(page.locator('.dialog h3')).toContainText('Open File');
    await expect(page.locator('.file-list-item:text("test-open.md")')).toBeVisible();

    await page.locator('.dialog-close').click();
    await deleteFileViaApi(page, 'test-open.md');
  });

  test('Clicking file in dialog loads it', async ({ page }) => {
    await saveFileViaApi(page, 'load-me.md', '# Loaded Content');

    await page.locator('.menu-item:text-is("File")').click();
    await page.locator('.dropdown-item:has-text("Open")').click();
    await page.locator('.file-list-item:text("load-me.md")').click();

    await expect(page.locator('.dialog')).not.toBeVisible();
    await expect(page.locator('.editor-textarea')).toHaveValue(/Loaded Content/);
    await expect(page.locator('.file-name-display')).toContainText('load-me.md');

    await deleteFileViaApi(page, 'load-me.md');
  });

  test('Cancel closes Open dialog', async ({ page }) => {
    await page.locator('.menu-item:text-is("File")').click();
    await page.locator('.dropdown-item:has-text("Open")').click();
    await expect(page.locator('.dialog')).toBeVisible();
    await page.locator('.dialog-close').click();
    await expect(page.locator('.dialog')).not.toBeVisible();
  });

  test('Save and reopen roundtrip', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.fill('# Roundtrip\n\n- Item 1\n- Item 2');

    await page.locator('.menu-item:text-is("File")').click();
    await page.locator('#save-btn').click();
    await expect(page.locator('.status-msg')).toContainText('Saved');

    await textarea.fill('');

    await page.locator('.menu-item:text-is("File")').click();
    await page.locator('.dropdown-item:has-text("Open")').click();
    await page.locator('.file-list-item:text("untitled.md")').click();

    await expect(textarea).toHaveValue(/Roundtrip/);
    await expect(textarea).toHaveValue(/Item 1/);
    await deleteFileViaApi(page, 'untitled.md');
  });

  test('Ctrl+S saves file', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.fill('Ctrl+S test');
    await textarea.focus();
    await page.keyboard.press('Control+s');
    await expect(page.locator('.status-msg')).toContainText('Saved');
    await deleteFileViaApi(page, 'untitled.md');
  });

  test('Empty open dialog shows message', async ({ page }) => {
    const files = await page.evaluate(async () => {
      const r = await fetch('/api/files');
      return r.json();
    });
    for (const f of files as Array<{ name: string }>) {
      await deleteFileViaApi(page, f.name);
    }

    await page.locator('.menu-item:text-is("File")').click();
    await page.locator('.dropdown-item:has-text("Open")').click();
    await expect(page.locator('.empty-msg')).toContainText('No documents found');
    await page.locator('.dialog-close').click();
  });
});
