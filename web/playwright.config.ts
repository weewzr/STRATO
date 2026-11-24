import { defineConfig, devices } from '@playwright/test';

// Runs against the production build served by `vite preview` under the GitHub Pages base path.
// Locally, set PW_CHANNEL=msedge to use the installed Microsoft Edge instead of a downloaded Chromium.
const port = 4173;
const channel = process.env.PW_CHANNEL;

export default defineConfig({
  testDir: 'e2e',
  outputDir: 'test-results',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: 0,
  workers: process.env.CI ? 2 : 4,
  reporter: process.env.CI ? [['list'], ['html', { open: 'never' }]] : [['list']],
  use: {
    baseURL: `http://localhost:${port}/STRATO/`,
    trace: 'off',
    ...(channel ? { channel } : {}),
  },
  projects: [{ name: 'chromium', use: { ...devices['Desktop Chrome'], ...(channel ? { channel } : {}) } }],
  webServer: {
    command: 'npm run preview',
    url: `http://localhost:${port}/STRATO/`,
    reuseExistingServer: !process.env.CI,
    timeout: 60_000,
  },
});
