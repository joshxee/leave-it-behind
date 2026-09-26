import { defineConfig, devices } from '@playwright/test';

// The web build must exist before running: `E2E=1 scripts/build-web.sh`.
// Viewport matches the fixed e2e canvas size (WINDOW_SIZE in src/lib.rs).
const PORT = Number(process.env.PORT ?? 4173);

export default defineConfig({
  testDir: './specs',
  snapshotPathTemplate: '{testDir}/__screenshots__/{testFilePath}/{arg}{ext}',
  // Wasm startup under SwiftShader is slow.
  timeout: 120_000,
  expect: {
    timeout: 60_000,
    toHaveScreenshot: { maxDiffPixelRatio: 0.02, animations: 'disabled' },
  },
  fullyParallel: false,
  workers: 1,
  forbidOnly: !!process.env.CI,
  // A test that passes only on retry is reported as flaky by build-report.mjs.
  retries: process.env.CI ? 1 : 0,
  reporter: [
    ['list'],
    ['html', { outputFolder: 'playwright-report', open: 'never' }],
    ['json', { outputFile: 'results.json' }],
  ],
  use: {
    baseURL: `http://127.0.0.1:${PORT}`,
    viewport: { width: 1280, height: 720 },
    screenshot: 'on',
    trace: 'retain-on-failure',
    video: 'retain-on-failure',
  },
  projects: [
    {
      name: 'chromium',
      use: {
        ...devices['Desktop Chrome'],
        viewport: { width: 1280, height: 720 },
        deviceScaleFactor: 1,
        launchOptions: {
          args: ['--use-angle=swiftshader', '--enable-unsafe-swiftshader', '--ignore-gpu-blocklist'],
        },
      },
    },
  ],
  webServer: {
    command: `bash ../scripts/serve-web.sh`,
    url: `http://127.0.0.1:${PORT}/index.html`,
    env: { PORT: String(PORT) },
    reuseExistingServer: !process.env.CI,
    timeout: 60_000,
  },
});
