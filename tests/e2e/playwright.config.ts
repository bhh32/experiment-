import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: '.',
  testMatch: '*.spec.ts',
  timeout: 60000,
  expect: { timeout: 10000 },
  use: {
    baseURL: 'http://localhost:8080',
    launchOptions: {
      executablePath: '/opt/chrome-linux/chrome',
      args: ['--no-sandbox', '--disable-gpu', '--disable-dev-shm-usage'],
    },
    // Capture screenshot after every test for visual confirmation
    screenshot: 'on',
    video: 'off',
  },
  // Save all test artifacts (screenshots) to test-results/
  outputDir: 'test-results',
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
});
