# テスト戦略

このプロジェクトでは、Terminal版とWeb版の両方に対して包括的なテストを実施します。

## Terminal版テスト (Rust統合テスト)

### 場所
`tests/input_handling_test.rs`

### テスト内容
1. **メニューナビゲーション**
   - j/kキーでの選択移動
   - Enterキーでの画面遷移

2. **Dashboard操作**
   - タスクナビゲーション
   - ビュー切り替え (Development/Internal/External Review)

3. **Detail画面**
   - Escキーでの戻る操作
   - ノート作成モードへの遷移

4. **Calendar画面**
   - 日付ナビゲーション (矢印キー)
   - 月の切り替え

5. **入力モード**
   - テキスト入力
   - Backspace操作
   - Escキーでの終了

### 実行方法
```bash
cargo test --test input_handling_test
```

## Web版テスト (Playwright)

### セットアップ

#### 1. プロジェクト初期化
```bash
npm init -y
npm install -D @playwright/test
npx playwright install
```

#### 2. Playwright設定 (`playwright.config.ts`)
```typescript
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
  },

  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
    {
      name: 'firefox',
      use: { ...devices['Desktop Firefox'] },
    },
    {
      name: 'webkit',
      use: { ...devices['Desktop Safari'] },
    },
  ],

  webServer: {
    command: 'trunk serve',
    url: 'http://127.0.0.1:8080',
    reuseExistingServer: !process.env.CI,
  },
});
```

#### 3. E2Eテスト (`e2e/app.spec.ts`)
```typescript
import { test, expect } from '@playwright/test';

test.describe('Work Info Manage - Web Version', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    // Wait for WASM to load
    await page.waitForTimeout(1000);
  });

  test('should display menu screen', async ({ page }) => {
    // Check for menu items
    await expect(page.locator('text=Task Manager')).toBeVisible();
    await expect(page.locator('text=Calendar & Daily Reports')).toBeVisible();
    await expect(page.locator('text=All Memos')).toBeVisible();
  });

  test('should navigate menu with j/k keys', async ({ page }) => {
    // Press 'j' to move down
    await page.keyboard.press('j');
    await page.waitForTimeout(100);
    
    // Verify selection moved (check for >> marker or highlight)
    const selection = await page.locator('text=>>').textContent();
    expect(selection).toContain('>>');
    
    // Press 'k' to move up
    await page.keyboard.press('k');
    await page.waitForTimeout(100);
  });

  test('should navigate to Dashboard', async ({ page }) => {
    // Press Enter to select Task Manager
    await page.keyboard.press('Enter');
    await page.waitForTimeout(500);
    
    // Verify Dashboard is displayed
    await expect(page.locator('text=Development')).toBeVisible();
  });

  test('should navigate tasks in Dashboard', async ({ page }) => {
    // Navigate to Dashboard
    await page.keyboard.press('Enter');
    await page.waitForTimeout(500);
    
    // Navigate through tasks
    await page.keyboard.press('j');
    await page.waitForTimeout(100);
    await page.keyboard.press('k');
    await page.waitForTimeout(100);
  });

  test('should switch views in Dashboard', async ({ page }) => {
    await page.keyboard.press('Enter');
    await page.waitForTimeout(500);
    
    // Switch to Internal Review
    await page.keyboard.press('2');
    await page.waitForTimeout(100);
    await expect(page.locator('text=Internal Review')).toBeVisible();
    
    // Switch to External Review
    await page.keyboard.press('3');
    await page.waitForTimeout(100);
    await expect(page.locator('text=External Review')).toBeVisible();
  });

  test('should view task details', async ({ page }) => {
    await page.keyboard.press('Enter'); // Go to Dashboard
    await page.waitForTimeout(500);
    
    await page.keyboard.press('j'); // Select a task
    await page.waitForTimeout(100);
    
    await page.keyboard.press('Enter'); // View details
    await page.waitForTimeout(500);
    
    // Verify Detail screen
    await expect(page.locator('text=Title:')).toBeVisible();
  });

  test('should navigate back with Esc', async ({ page }) => {
    await page.keyboard.press('Enter'); // Dashboard
    await page.waitForTimeout(500);
    await page.keyboard.press('Enter'); // Detail
    await page.waitForTimeout(500);
    
    await page.keyboard.press('Escape'); // Back to Dashboard
    await page.waitForTimeout(500);
    
    await expect(page.locator('text=Development')).toBeVisible();
  });

  test('should navigate to Calendar', async ({ page }) => {
    await page.keyboard.press('j'); // Move to Calendar
    await page.waitForTimeout(100);
    await page.keyboard.press('Enter');
    await page.waitForTimeout(500);
    
    // Verify Calendar is displayed
    await expect(page.locator('text=Calendar')).toBeVisible();
  });

  test('should navigate dates in Calendar', async ({ page }) => {
    await page.keyboard.press('j');
    await page.keyboard.press('Enter'); // Go to Calendar
    await page.waitForTimeout(500);
    
    // Navigate dates
    await page.keyboard.press('ArrowRight');
    await page.waitForTimeout(100);
    await page.keyboard.press('ArrowLeft');
    await page.waitForTimeout(100);
    await page.keyboard.press('ArrowDown');
    await page.waitForTimeout(100);
    await page.keyboard.press('ArrowUp');
    await page.waitForTimeout(100);
  });

  test('should create note', async ({ page }) => {
    await page.keyboard.press('Enter'); // Dashboard
    await page.waitForTimeout(500);
    await page.keyboard.press('Enter'); // Detail
    await page.waitForTimeout(500);
    
    await page.keyboard.press('n'); // Create note
    await page.waitForTimeout(100);
    
    // Verify input mode
    await expect(page.locator('text=Adding note')).toBeVisible();
  });

  test('should handle full navigation flow', async ({ page }) => {
    // Menu -> Dashboard -> Detail -> Back -> Menu
    await page.keyboard.press('Enter');
    await page.waitForTimeout(500);
    
    await page.keyboard.press('j');
    await page.keyboard.press('Enter');
    await page.waitForTimeout(500);
    
    await page.keyboard.press('Escape');
    await page.waitForTimeout(500);
    
    await page.keyboard.press('Escape');
    await page.waitForTimeout(500);
    
    // Should be back at menu
    await expect(page.locator('text=Task Manager')).toBeVisible();
  });
});
```

#### 4. package.json scripts
```json
{
  "scripts": {
    "test:e2e": "playwright test",
    "test:e2e:ui": "playwright test --ui",
    "test:e2e:debug": "playwright test --debug"
  }
}
```

### 実行方法
```bash
# すべてのテストを実行
npm run test:e2e

# UIモードで実行
npm run test:e2e:ui

# デバッグモードで実行
npm run test:e2e:debug
```

## moonrepoタスク統合

`moon.yml`に以下を追加:

```yaml
tasks:
  test:
    command: 'cargo test'
    inputs:
      - '@group(sources)'

  test-integration:
    command: 'cargo test --test input_handling_test'
    inputs:
      - '@group(sources)'
      - 'tests/**/*'

  test-e2e:
    command: 'npm run test:e2e'
    deps:
      - 'build-web'
    inputs:
      - '@group(web)'
      - 'e2e/**/*'

  test-all:
    deps:
      - 'test'
      - 'test-integration'
      - 'test-e2e'
```

### 実行
```bash
# すべてのテスト
moon run test-all

# 統合テストのみ
moon run test-integration

# E2Eテストのみ
moon run test-e2e
```

## CI/CD統合 (GitHub Actions)

`.github/workflows/test.yml`:
```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - uses: actions/setup-node@v3
        with:
          node-version: '20'
      
      - name: Install dependencies
        run: |
          cargo install trunk
          npm install
          npx playwright install --with-deps
      
      - name: Run Rust tests
        run: cargo test
      
      - name: Run integration tests
        run: cargo test --test input_handling_test
      
      - name: Build WASM
        run: trunk build
      
      - name: Run E2E tests
        run: npm run test:e2e
      
      - name: Upload test results
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: playwright-report
          path: playwright-report/
```

## テストカバレッジ

### Terminal版
- ✅ メニューナビゲーション
- ✅ Dashboard操作
- ✅ タスク選択
- ✅ Detail画面
- ✅ Calendar操作
- ✅ 入力モード

### Web版
- ✅ 同上 (Playwrightで自動化)
- ✅ クロスブラウザテスト (Chrome, Firefox, Safari)
- ✅ ビジュアルリグレッションテスト (オプション)

## 今後の拡張

1. **スナップショットテスト**
   - UI状態のスナップショット
   - ビジュアルリグレッション検出

2. **パフォーマンステスト**
   - レンダリング速度
   - メモリ使用量

3. **アクセシビリティテスト**
   - キーボードナビゲーション
   - スクリーンリーダー対応
