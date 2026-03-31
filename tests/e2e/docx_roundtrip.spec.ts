import { test, expect, type Page } from '@playwright/test';

async function deleteFile(page: Page, name: string) {
  await page.evaluate(async (n) => {
    await fetch(`/api/files/${n}`, { method: 'DELETE' });
  }, name);
}

test.describe('DOCX Import/Export Round-trip', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForSelector('.app-container', { timeout: 30000 });
  });

  test('heading text survives round-trip', async ({ page }) => {
    // Export
    const exported = await page.evaluate(async () => {
      const resp = await fetch('/api/convert', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ content: '# My Research Paper\n\n## Introduction', from: 'markdown', to: 'docx' }),
      });
      return resp.json();
    });

    // Save DOCX
    await page.evaluate(async (b64: string) => {
      const binary = atob(b64);
      const bytes = new Uint8Array(binary.length);
      for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
      await fetch('/api/files/rt-heading.docx', { method: 'PUT', body: new Blob([bytes]) });
    }, exported.content);

    // Import
    const imported = await page.evaluate(async () => {
      const resp = await fetch('/api/files/rt-heading.docx');
      return resp.text();
    });

    expect(imported).toContain('My Research Paper');
    expect(imported).toContain('Introduction');
    expect(imported).toContain('# ');
    expect(imported).toContain('## ');

    await deleteFile(page, 'rt-heading.docx');
  });

  test('bold and italic survive round-trip', async ({ page }) => {
    const exported = await page.evaluate(async () => {
      const resp = await fetch('/api/convert', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ content: 'This has **bold** and *italic* text.', from: 'markdown', to: 'docx' }),
      });
      return resp.json();
    });

    await page.evaluate(async (b64: string) => {
      const binary = atob(b64);
      const bytes = new Uint8Array(binary.length);
      for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
      await fetch('/api/files/rt-formatting.docx', { method: 'PUT', body: new Blob([bytes]) });
    }, exported.content);

    const imported = await page.evaluate(async () => {
      const resp = await fetch('/api/files/rt-formatting.docx');
      return resp.text();
    });

    expect(imported).toContain('**bold**');
    expect(imported).toContain('*italic*');

    await deleteFile(page, 'rt-formatting.docx');
  });

  test('centered heading survives round-trip', async ({ page }) => {
    const exported = await page.evaluate(async () => {
      const resp = await fetch('/api/convert', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ content: '{center}# Centered Title', from: 'markdown', to: 'docx' }),
      });
      return resp.json();
    });

    await page.evaluate(async (b64: string) => {
      const binary = atob(b64);
      const bytes = new Uint8Array(binary.length);
      for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
      await fetch('/api/files/rt-center.docx', { method: 'PUT', body: new Blob([bytes]) });
    }, exported.content);

    const imported = await page.evaluate(async () => {
      const resp = await fetch('/api/files/rt-center.docx');
      return resp.text();
    });

    expect(imported).toContain('{center}');
    expect(imported).toContain('Centered Title');

    await deleteFile(page, 'rt-center.docx');
  });

  test('table survives round-trip', async ({ page }) => {
    const md = '| Name | Score |\n|---|---|\n| Alice | 95 |\n| Bob | 87 |';
    const exported = await page.evaluate(async (content: string) => {
      const resp = await fetch('/api/convert', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ content, from: 'markdown', to: 'docx' }),
      });
      return resp.json();
    }, md);

    await page.evaluate(async (b64: string) => {
      const binary = atob(b64);
      const bytes = new Uint8Array(binary.length);
      for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
      await fetch('/api/files/rt-table.docx', { method: 'PUT', body: new Blob([bytes]) });
    }, exported.content);

    const imported = await page.evaluate(async () => {
      const resp = await fetch('/api/files/rt-table.docx');
      return resp.text();
    });

    expect(imported).toContain('Alice');
    expect(imported).toContain('95');
    expect(imported).toContain('|');

    await deleteFile(page, 'rt-table.docx');
  });

  test('bullet list survives round-trip', async ({ page }) => {
    const md = '- First item\n- Second item\n- Third item';
    const exported = await page.evaluate(async (content: string) => {
      const resp = await fetch('/api/convert', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ content, from: 'markdown', to: 'docx' }),
      });
      return resp.json();
    }, md);

    await page.evaluate(async (b64: string) => {
      const binary = atob(b64);
      const bytes = new Uint8Array(binary.length);
      for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
      await fetch('/api/files/rt-list.docx', { method: 'PUT', body: new Blob([bytes]) });
    }, exported.content);

    const imported = await page.evaluate(async () => {
      const resp = await fetch('/api/files/rt-list.docx');
      return resp.text();
    });

    expect(imported).toContain('First item');
    expect(imported).toContain('Second item');
    expect(imported).toContain('- ');

    await deleteFile(page, 'rt-list.docx');
  });

  test('full document round-trip and screenshot', async ({ page }) => {
    const original = [
      '{center}# Software Engineering Report',
      '{center}## CS 410',
      '{center}Spring 2026',
      '',
      '{pagebreak}',
      '',
      '## Introduction',
      '',
      'This report covers **key findings** from the Q4 analysis. Team velocity increased by *23%* compared to the previous quarter.',
      '',
      '### Metrics',
      '',
      '- Uptime: **99.97%**',
      '- MTTR: *28 minutes*',
      '- Sprint velocity: 52 points',
      '',
      '| Metric | Value |',
      '|---|---|',
      '| Deployments | 4.8/week |',
      '| Code coverage | 89% |',
    ].join('\n');

    // Export to DOCX
    const exported = await page.evaluate(async (content: string) => {
      const resp = await fetch('/api/convert', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ content, from: 'markdown', to: 'docx' }),
      });
      return resp.json();
    }, original);

    // Save DOCX
    await page.evaluate(async (b64: string) => {
      const binary = atob(b64);
      const bytes = new Uint8Array(binary.length);
      for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
      await fetch('/api/files/rt-full.docx', { method: 'PUT', body: new Blob([bytes]) });
    }, exported.content);

    // Import back
    const imported = await page.evaluate(async () => {
      const resp = await fetch('/api/files/rt-full.docx');
      return resp.text();
    });

    // Load into editor and take screenshot
    const textarea = page.locator('.editor-textarea');
    await textarea.fill(imported);
    await page.waitForTimeout(800);
    await page.screenshot({ path: 'test-results/roundtrip-imported.png', type: 'png' });

    // Verify key content survived
    expect(imported).toContain('Software Engineering Report');
    expect(imported).toContain('Introduction');
    expect(imported).toContain('**key findings**');
    expect(imported).toContain('*23%*');
    expect(imported).toContain('99.97%');
    expect(imported).toContain('|');

    await deleteFile(page, 'rt-full.docx');
  });
});
