/**
 * Playwright E2E Test Configuration
 * 
 * Web版（WASM）アプリケーションのE2Eテスト設定。
 * 
 * ## 主要設定
 * 
 * - **testDir**: テストファイルの場所（`./e2e`）
 * - **fullyParallel**: テストの並列実行を有効化
 * - **retries**: CI環境では2回リトライ
 * - **reporter**: HTMLレポート生成
 * 
 * ## ブラウザ設定
 * 
 * - Chromium（Desktop Chrome）でテスト実行
 * - 必要に応じて他のブラウザ（Firefox, WebKit）を追加可能
 * 
 * ## 開発サーバー
 * 
 * - テスト実行前に自動的に`trunk serve`を起動
 * - ポート8080でWASMアプリケーションを提供
 * - ローカル開発時は既存のサーバーを再利用
 * 
 * @see {@link https://playwright.dev/docs/test-configuration | Playwright Configuration}
 */
import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
    testDir: './e2e',
    fullyParallel: true,
    forbidOnly: !!process.env.CI,
    retries: process.env.CI ? 2 : 0,
    workers: process.env.CI ? 1 : undefined,
    reporter: 'html',

    use: {
        baseURL: 'http://127.0.0.1:8080',
        trace: 'on-first-retry',
        screenshot: 'only-on-failure',
    },

    projects: [
        {
            name: 'chromium',
            use: { ...devices['Desktop Chrome'] },
        },
    ],

    // Run local dev server before starting tests
    webServer: {
        command: 'trunk serve',
        url: 'http://127.0.0.1:8080',
        reuseExistingServer: !process.env.CI,
        timeout: 120 * 1000,
    },
});
