import { test, expect } from '@playwright/test';

test.describe('DOCX Export Formatting', () => {

  // Helper: convert markdown and return the base64 DOCX content
  async function convertToDocx(request: any, content: string, opts?: { font?: string; font_size?: number; line_height?: number }) {
    const resp = await request.post('/api/convert', {
      data: {
        content,
        from: 'markdown',
        to: 'docx',
        font: opts?.font,
        font_size: opts?.font_size,
        line_height: opts?.line_height,
      },
    });
    expect(resp.ok()).toBeTruthy();
    return resp.json();
  }

  // Helper: decode base64 DOCX, extract document.xml text
  async function getDocxXml(page: any, base64Content: string): Promise<string> {
    return page.evaluate(async (b64: string) => {
      // Decode base64 to array buffer
      const binary = atob(b64);
      const bytes = new Uint8Array(binary.length);
      for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);

      // Use JSZip or manual ZIP parsing — simpler: just search for document.xml in the bytes
      // ZIP files have local file headers with filenames
      const text = new TextDecoder('utf-8', { fatal: false }).decode(bytes);
      // Find the XML between <?xml and </w:document>
      const start = text.indexOf('<?xml');
      const end = text.indexOf('</w:document>');
      if (start >= 0 && end >= 0) {
        return text.substring(start, end + '</w:document>'.length);
      }
      return text;
    }, base64Content);
  }

  test('default font is Times New Roman', async ({ request, page }) => {
    const data = await convertToDocx(request, '# Test\n\nBody text.');
    await page.goto('/');
    const xml = await getDocxXml(page, data.content);
    expect(xml).toContain('w:ascii="Times New Roman"');
  });

  test('default font size is 12pt (24 half-points)', async ({ request, page }) => {
    const data = await convertToDocx(request, 'Body text.');
    await page.goto('/');
    const xml = await getDocxXml(page, data.content);
    expect(xml).toContain('w:val="24"');
  });

  test('page size is US Letter (8.5 x 11 inches)', async ({ request, page }) => {
    const data = await convertToDocx(request, 'Test');
    await page.goto('/');
    const xml = await getDocxXml(page, data.content);
    // 8.5" = 12240 twips, 11" = 15840 twips
    expect(xml).toContain('w:w="12240"');
    expect(xml).toContain('w:h="15840"');
  });

  test('margins are 1 inch all around', async ({ request, page }) => {
    const data = await convertToDocx(request, 'Test');
    await page.goto('/');
    const xml = await getDocxXml(page, data.content);
    // 1" = 1440 twips
    expect(xml).toContain('w:top="1440"');
    expect(xml).toContain('w:right="1440"');
    expect(xml).toContain('w:bottom="1440"');
    expect(xml).toContain('w:left="1440"');
  });

  test('line spacing is double (480 twips)', async ({ request, page }) => {
    const data = await convertToDocx(request, 'Body text.');
    await page.goto('/');
    const xml = await getDocxXml(page, data.content);
    expect(xml).toContain('w:line="480"');
  });

  test('headings use named styles (Heading1, Heading2)', async ({ request, page }) => {
    const data = await convertToDocx(request, '# H1\n\n## H2\n\n### H3');
    await page.goto('/');
    const xml = await getDocxXml(page, data.content);
    expect(xml).toContain('w:val="Heading1"');
    expect(xml).toContain('w:val="Heading2"');
    expect(xml).toContain('w:val="Heading3"');
  });

  test('page break uses w:pageBreakBefore', async ({ request, page }) => {
    const data = await convertToDocx(request, 'Page 1\n\n{pagebreak}\n\nPage 2');
    await page.goto('/');
    const xml = await getDocxXml(page, data.content);
    expect(xml).toContain('pageBreakBefore');
  });

  test('bold text produces w:b element', async ({ request, page }) => {
    const data = await convertToDocx(request, '**bold words**');
    await page.goto('/');
    const xml = await getDocxXml(page, data.content);
    expect(xml).toContain('<w:b />');
  });

  test('italic text produces w:i element', async ({ request, page }) => {
    const data = await convertToDocx(request, '*italic words*');
    await page.goto('/');
    const xml = await getDocxXml(page, data.content);
    expect(xml).toContain('<w:i />');
  });

  test('centered heading has w:jc center', async ({ request, page }) => {
    const data = await convertToDocx(request, '{center}# Centered Title');
    await page.goto('/');
    const xml = await getDocxXml(page, data.content);
    expect(xml).toContain('w:val="center"');
  });

  test('custom font override works', async ({ request, page }) => {
    const data = await convertToDocx(request, 'Test', { font: 'Arial' });
    await page.goto('/');
    const xml = await getDocxXml(page, data.content);
    expect(xml).toContain('w:ascii="Arial"');
  });

  test('custom font size override works', async ({ request, page }) => {
    const data = await convertToDocx(request, 'Test', { font_size: 14 });
    await page.goto('/');
    const xml = await getDocxXml(page, data.content);
    // 14pt = 28 half-points
    expect(xml).toContain('w:val="28"');
  });

  test('DOCX is valid ZIP file with PK header', async ({ request }) => {
    const data = await convertToDocx(request, '# Test');
    const binary = Buffer.from(data.content, 'base64');
    expect(binary[0]).toBe(0x50); // P
    expect(binary[1]).toBe(0x4B); // K
  });
});
