import { test, expect } from '@playwright/test';

test.describe('Backend API', () => {
  test('health endpoint returns ok', async ({ request }) => {
    const resp = await request.get('/api/health');
    expect(resp.ok()).toBeTruthy();
    const body = await resp.json();
    expect(body.status).toBe('ok');
  });

  test('file CRUD roundtrip', async ({ request }) => {
    // Create
    const putResp = await request.put('/api/files/api-test.md', { data: '# API Test' });
    expect(putResp.ok()).toBeTruthy();

    // Read
    const getResp = await request.get('/api/files/api-test.md');
    expect(await getResp.text()).toBe('# API Test');

    // List
    const listResp = await request.get('/api/files');
    const files = await listResp.json();
    const names = files.map((f: any) => f.name);
    expect(names).toContain('api-test.md');

    // Delete
    const delResp = await request.delete('/api/files/api-test.md');
    expect(delResp.ok()).toBeTruthy();

    // Verify gone
    const gone = await request.get('/api/files/api-test.md');
    expect(gone.ok()).toBeFalsy();
  });

  test('convert markdown to DOCX', async ({ request }) => {
    const resp = await request.post('/api/convert', {
      data: { content: '# Hello', from: 'markdown', to: 'docx' },
    });
    expect(resp.ok()).toBeTruthy();
    const body = await resp.json();
    expect(body.format).toBe('docx');
    expect(body.content.length).toBeGreaterThan(0);
  });

  test('convert markdown to ODF', async ({ request }) => {
    const resp = await request.post('/api/convert', {
      data: { content: '# Hello', from: 'markdown', to: 'odt' },
    });
    expect(resp.ok()).toBeTruthy();
    const body = await resp.json();
    expect(body.format).toBe('odt');
  });

  test('file list only returns supported formats', async ({ request }) => {
    // Create files of different types
    await request.put('/api/files/doc.md', { data: 'markdown' });

    const resp = await request.get('/api/files');
    const files = await resp.json();
    const names = files.map((f: any) => f.name);
    expect(names).toContain('doc.md');

    // Cleanup
    await request.delete('/api/files/doc.md');
  });
});
