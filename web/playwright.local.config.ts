import { defineConfig, devices } from '@playwright/test';

// Runs against the production build served by `vite preview` under the GitHub Pages base path.
// Locally, set PW_CHANNEL=msedge to use the installed Microsoft Edge instead of a downloaded Chromium.
const port = 4311;
const channel = process.env.PW_CHANNEL;

export default defineConfig({
  testDir: 'e2e',
  outputDir: 'test-results',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: 0,
  workers: process.env.CI ? 2 : 4,
  reporter: process.env.CI ? [['list'], ['html', { open: 'never', outputFolder: process.env.PW_REPORT_DIR ?? 'playwright-report' }]] : [['list']],
  use: {
    baseURL: `http://localhost:${port}/STRATO/`,
    trace: 'off',
    ...(channel ? { channel } : {}),
  },
  // The frame-rate tests (tagged @perf) form their own project with one worker. Continuous
  // integration runs `--project=chromium` and then `--project=performance` in a separate step, so a
  // frame rate is never measured while another browser shares the machine. Locally, run
  // `npx playwright test --project=performance` on its own for a measurement that means something.
  projects: [
    { name: 'chromium', grepInvert: /@perf/, use: { ...devices['Desktop Chrome'], ...(channel ? { channel } : {}) } },
    { name: 'performance', grep: /@perf/, workers: 1, fullyParallel: false, use: { ...devices['Desktop Chrome'], ...(channel ? { channel } : {}) } },
  ],
  webServer: {
    command: 'npx vite preview --port 4311 --strictPort',
    url: `http://localhost:${port}/STRATO/`,
    reuseExistingServer: !process.env.CI,
    timeout: 60_000,
  },
});
