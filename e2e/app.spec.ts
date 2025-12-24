/**
 * E2E Test Suite for Work Info Manage Web Version
 * 
 * このテストスイートは、Web版（WASM）アプリケーションの主要な機能を検証します。
 * 
 * ## テスト対象
 * 
 * ### メニューナビゲーション
 * - メニュー画面の表示
 * - j/kキーによる選択移動
 * - Enterキーによる画面遷移
 * 
 * ### Dashboard操作
 * - タスク一覧の表示
 * - タスク間のナビゲーション
 * - ビュー切り替え（Development/Internal Review/External Review）
 * 
 * ### タスク詳細
 * - 詳細画面への遷移
 * - Escキーによる戻る操作
 * 
 * ### Calendar機能
 * - カレンダー画面の表示
 * - 矢印キーによる日付ナビゲーション
 * 
 * ### フルフロー
 * - 複数画面を跨ぐ一連の操作
 * - キーボードショートカットの一貫性
 * 
 * ## 実行方法
 * 
 * ```bash
 * # 通常実行
 * npm run test:e2e
 * 
 * # UIモード（推奨）
 * npm run test:e2e:ui
 * 
 * # moonrepo経由
 * moon run test-e2e
 * ```
 * 
 * @see {@link https://playwright.dev/docs/intro | Playwright Documentation}
 * @see {@link ../docs/testing/e2e.md | E2E Testing Guide}
 */
import { test, expect } from '@playwright/test';

test.describe('Work Info Manage - Web Version E2E Tests', () => {
    test.beforeEach(async ({ page }) => {
        await page.goto('/');
        // Wait for WASM to load and initialize
        await page.waitForTimeout(2000);
    });

    test('should display menu screen on load', async ({ page }) => {
        // Check for menu title
        await expect(page.locator('text=WorkInfoManage')).toBeVisible({ timeout: 5000 });

        // Check for menu items
        await expect(page.locator('text=Task Manager')).toBeVisible();
        await expect(page.locator('text=Calendar')).toBeVisible();
        await expect(page.locator('text=All Memos')).toBeVisible();
    });

    test('should navigate menu with j/k keys', async ({ page }) => {
        // Initial state - Task Manager should be selected
        await expect(page.locator('text=>> Task Manager')).toBeVisible();

        // Press 'j' to move down
        await page.keyboard.press('j');
        await page.waitForTimeout(200);

        // Calendar should now be selected
        await expect(page.locator('text=>> Calendar')).toBeVisible();

        // Press 'k' to move up
        await page.keyboard.press('k');
        await page.waitForTimeout(200);

        // Back to Task Manager
        await expect(page.locator('text=>> Task Manager')).toBeVisible();
    });

    test('should navigate to Dashboard with Enter key', async ({ page }) => {
        // Press Enter to select Task Manager
        await page.keyboard.press('Enter');
        await page.waitForTimeout(1000);

        // Verify Dashboard is displayed
        await expect(page.locator('text=Development')).toBeVisible();
        await expect(page.locator('text=Tasks')).toBeVisible();
    });

    test('should navigate tasks in Dashboard', async ({ page }) => {
        // Navigate to Dashboard
        await page.keyboard.press('Enter');
        await page.waitForTimeout(1000);

        // Navigate through tasks with j/k
        await page.keyboard.press('j');
        await page.waitForTimeout(200);

        await page.keyboard.press('k');
        await page.waitForTimeout(200);

        // Should still be on Dashboard
        await expect(page.locator('text=Development')).toBeVisible();
    });

    test('should switch views in Dashboard', async ({ page }) => {
        await page.keyboard.press('Enter');
        await page.waitForTimeout(1000);

        // Switch to Internal Review with '2'
        await page.keyboard.press('2');
        await page.waitForTimeout(300);
        await expect(page.locator('text=Internal Review')).toBeVisible();

        // Switch to External Review with '3'
        await page.keyboard.press('3');
        await page.waitForTimeout(300);
        await expect(page.locator('text=External Review')).toBeVisible();

        // Switch back to Development with '1'
        await page.keyboard.press('1');
        await page.waitForTimeout(300);
        await expect(page.locator('text=Development')).toBeVisible();
    });

    test('should view task details', async ({ page }) => {
        await page.keyboard.press('Enter'); // Go to Dashboard
        await page.waitForTimeout(1000);

        await page.keyboard.press('j'); // Select a task
        await page.waitForTimeout(200);

        await page.keyboard.press('Enter'); // View details
        await page.waitForTimeout(500);

        // Verify Detail screen
        await expect(page.locator('text=Task Details')).toBeVisible();
        await expect(page.locator('text=Title:')).toBeVisible();
    });

    test('should navigate back with Esc key', async ({ page }) => {
        await page.keyboard.press('Enter'); // Dashboard
        await page.waitForTimeout(1000);

        await page.keyboard.press('Enter'); // Detail
        await page.waitForTimeout(500);

        await page.keyboard.press('Escape'); // Back to Dashboard
        await page.waitForTimeout(500);

        await expect(page.locator('text=Development')).toBeVisible();

        await page.keyboard.press('Escape'); // Back to Menu
        await page.waitForTimeout(500);

        await expect(page.locator('text=Task Manager')).toBeVisible();
    });

    test('should navigate to Calendar', async ({ page }) => {
        await page.keyboard.press('j'); // Move to Calendar
        await page.waitForTimeout(200);

        await page.keyboard.press('Enter'); // Select Calendar
        await page.waitForTimeout(1000);

        // Verify Calendar is displayed
        await expect(page.locator('text=Calendar')).toBeVisible();
    });

    test('should navigate dates in Calendar with arrow keys', async ({ page }) => {
        await page.keyboard.press('j'); // Move to Calendar
        await page.keyboard.press('Enter');
        await page.waitForTimeout(1000);

        // Navigate dates
        await page.keyboard.press('ArrowRight');
        await page.waitForTimeout(200);

        await page.keyboard.press('ArrowLeft');
        await page.waitForTimeout(200);

        await page.keyboard.press('ArrowDown');
        await page.waitForTimeout(200);

        await page.keyboard.press('ArrowUp');
        await page.waitForTimeout(200);

        // Should still be on Calendar
        await expect(page.locator('text=Calendar')).toBeVisible();
    });

    test('should handle full navigation flow', async ({ page }) => {
        // Menu -> Dashboard -> Detail -> Back -> Menu
        await page.keyboard.press('Enter');
        await page.waitForTimeout(1000);
        await expect(page.locator('text=Development')).toBeVisible();

        await page.keyboard.press('j');
        await page.keyboard.press('Enter');
        await page.waitForTimeout(500);
        await expect(page.locator('text=Task Details')).toBeVisible();

        await page.keyboard.press('Escape');
        await page.waitForTimeout(500);
        await expect(page.locator('text=Development')).toBeVisible();

        await page.keyboard.press('Escape');
        await page.waitForTimeout(500);
        await expect(page.locator('text=Task Manager')).toBeVisible();
    });

    test('should handle keyboard shortcuts consistently', async ({ page }) => {
        // Test that j/k work consistently across screens

        // In Menu
        await page.keyboard.press('j');
        await page.waitForTimeout(200);
        await page.keyboard.press('k');
        await page.waitForTimeout(200);

        // In Dashboard
        await page.keyboard.press('Enter');
        await page.waitForTimeout(1000);
        await page.keyboard.press('j');
        await page.waitForTimeout(200);
        await page.keyboard.press('k');
        await page.waitForTimeout(200);

        // All navigation should work smoothly
        await expect(page.locator('text=Development')).toBeVisible();
    });
});
