import { test, expect } from '@playwright/test';

test.describe('Newlines and Page Breaks', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForSelector('.app-container', { timeout: 30000 });
  });

  // ─── Newline Handling ───

  test('single newline creates a line break in Print preview', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.fill('Line one\nLine two');
    await page.waitForTimeout(300);

    const html = await page.locator('.preview-content').innerHTML();
    // With hardbreaks enabled, single newline should produce a <br> tag
    expect(html).toContain('<br');
  });

  test('single newline creates a line break in DOCX preview', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.fill('Line one\nLine two');

    await page.locator('.mode-btn:text-is("DOCX")').click();
    await page.waitForTimeout(300);

    const html = await page.locator('.preview-content').innerHTML();
    expect(html).toContain('<br');
  });

  test('single newline creates a line break in ODF preview', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.fill('Line one\nLine two');

    await page.locator('.mode-btn:text-is("ODF")').click();
    await page.waitForTimeout(300);

    const html = await page.locator('.preview-content').innerHTML();
    expect(html).toContain('<br');
  });

  test('double newline creates a new paragraph', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.fill('Paragraph one.\n\nParagraph two.');
    await page.waitForTimeout(300);

    const paragraphs = await page.locator('.preview-content p').count();
    expect(paragraphs).toBeGreaterThanOrEqual(2);
  });

  test('both lines are visible in preview (not merged)', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.fill('First line\nSecond line');
    await page.waitForTimeout(300);

    const text = await page.locator('.preview-content').textContent();
    expect(text).toContain('First line');
    expect(text).toContain('Second line');
  });

  // ─── Page Break Handling ───

  test('page break creates separate page divs in DOCX preview', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.fill('# Page 1 Content\n\nSome text on page one.\n\n{pagebreak}\n\n# Page 2 Content\n\nSome text on page two.');

    await page.locator('.mode-btn:text-is("DOCX")').click();
    await page.waitForTimeout(300);

    const pages = await page.locator('.docx-page').count();
    expect(pages).toBe(2);
  });

  test('page break creates separate page divs in ODF preview', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.fill('# Page 1\n\nContent.\n\n{pagebreak}\n\n# Page 2\n\nMore content.');

    await page.locator('.mode-btn:text-is("ODF")').click();
    await page.waitForTimeout(300);

    const pages = await page.locator('.odt-page').count();
    expect(pages).toBe(2);
  });

  test('page break creates separate page divs in Print preview', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.fill('Page one text.\n\n{pagebreak}\n\nPage two text.');
    await page.waitForTimeout(300);

    const pages = await page.locator('.md-page').count();
    expect(pages).toBe(2);
  });

  test('multiple page breaks create correct number of pages', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.fill('Page 1\n\n{pagebreak}\n\nPage 2\n\n{pagebreak}\n\nPage 3');

    await page.locator('.mode-btn:text-is("DOCX")').click();
    await page.waitForTimeout(300);

    const pages = await page.locator('.docx-page').count();
    expect(pages).toBe(3);
  });

  test('page divs have proper page dimensions (min-height 11in)', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.fill('Short page.\n\n{pagebreak}\n\nAnother short page.');

    await page.locator('.mode-btn:text-is("DOCX")').click();
    await page.waitForTimeout(300);

    const minHeight = await page.locator('.docx-page').first().evaluate(
      el => getComputedStyle(el).minHeight
    );
    // 11in = 1056px at 96dpi — browser resolves CSS 'in' units to px
    expect(parseFloat(minHeight)).toBeGreaterThanOrEqual(1056);
  });

  test('content before page break appears on first page', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.fill('First page content here.\n\n{pagebreak}\n\nSecond page content here.');

    await page.locator('.mode-btn:text-is("DOCX")').click();
    await page.waitForTimeout(300);

    const firstPage = await page.locator('.docx-page').first().textContent();
    const secondPage = await page.locator('.docx-page').last().textContent();
    expect(firstPage).toContain('First page content here');
    expect(secondPage).toContain('Second page content here');
  });

  test('PgBrk toolbar button inserts {pagebreak} marker', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.fill('Before');
    await textarea.focus();
    // Move cursor to end
    await page.keyboard.press('End');
    await page.locator('.tool-btn:text-is("PgBrk")').click();

    const value = await textarea.inputValue();
    expect(value).toContain('{pagebreak}');
  });

  test('no page break gives single page', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.fill('Just a single page of content.');

    await page.locator('.mode-btn:text-is("DOCX")').click();
    await page.waitForTimeout(300);

    const pages = await page.locator('.docx-page').count();
    expect(pages).toBe(1);
  });

  // ─── Multi-page document looks correct ───

  test('multi-page document has gap between pages', async ({ page }) => {
    const textarea = page.locator('.editor-textarea');
    await textarea.fill('Page 1\n\n{pagebreak}\n\nPage 2');

    await page.locator('.mode-btn:text-is("DOCX")').click();
    await page.waitForTimeout(300);

    // The preview container should use flex with gap for page spacing
    const display = await page.locator('.docx-preview').evaluate(
      el => getComputedStyle(el).display
    );
    expect(display).toBe('flex');

    const gap = await page.locator('.docx-preview').evaluate(
      el => getComputedStyle(el).gap
    );
    expect(gap).not.toBe('');
    expect(gap).not.toBe('normal');
  });
});
