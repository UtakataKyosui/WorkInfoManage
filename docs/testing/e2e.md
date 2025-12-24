# E2Eテストセットアップ完了ガイド

## ✅ セットアップ済み

以下のセットアップが完了しました:

### 1. Playwright インストール
- ✅ `@playwright/test` インストール済み
- ✅ Chromiumブラウザ インストール済み
- ✅ TypeScript サポート

### 2. 設定ファイル
- ✅ `playwright.config.ts` - Playwright設定
- ✅ `e2e/app.spec.ts` - E2Eテストスイート
- ✅ `package.json` - npmスクリプト設定

### 3. moonrepo統合
- ✅ `test` - Rustユニットテスト
- ✅ `test-integration` - Rust統合テスト
- ✅ `test-e2e` - PlaywrightE2Eテスト
- ✅ `test-e2e-ui` - Playwright UIモード
- ✅ `test-all` - すべてのテスト実行

## 🚀 テスト実行方法

### E2Eテスト (Playwright)

#### 基本実行
```bash
# moonrepo経由
moon run test-e2e

# または直接npm経由
npm run test:e2e
```

#### UIモード (推奨 - デバッグに便利)
```bash
moon run test-e2e-ui
# または
npm run test:e2e:ui
```

#### ヘッドモード (ブラウザを表示)
```bash
npm run test:e2e:headed
```

#### デバッグモード
```bash
npm run test:e2e:debug
```

### 統合テスト (Rust)
```bash
moon run test-integration
# または
cargo test --test input_handling_test
```

### すべてのテスト
```bash
moon run test-all
```

## 📊 テストカバレッジ

### E2Eテスト内容 (`e2e/app.spec.ts`)

1. **メニュー表示** - 起動時のメニュー画面確認
2. **メニューナビゲーション** - j/kキーでの移動
3. **Dashboard遷移** - Enterキーでの画面遷移
4. **タスクナビゲーション** - Dashboard内でのタスク選択
5. **ビュー切り替え** - 1/2/3キーでのビュー変更
6. **タスク詳細表示** - Enterキーで詳細画面へ
7. **戻るナビゲーション** - Escキーでの画面遷移
8. **Calendar表示** - Calendar画面への遷移
9. **日付ナビゲーション** - 矢印キーでの日付移動
10. **フルフロー** - 複数画面を跨ぐ操作
11. **キーボードショートカット一貫性** - 全画面での動作確認

## 🔧 トラブルシューティング

### ポート8080が使用中の場合
```bash
# 既存のプロセスを終了
lsof -ti:8080 | xargs kill -9

# または moonrepo経由で起動 (自動でkill)
moon run dev-web
```

### テストが失敗する場合
1. **WASMビルドを確認**
   ```bash
   moon run build-web
   ```

2. **開発サーバーが起動しているか確認**
   ```bash
   curl http://127.0.0.1:8080
   ```

3. **ブラウザキャッシュをクリア**
   - Playwrightは自動的にクリーンな状態で実行

### デバッグ方法
```bash
# UIモードで実行 (最も推奨)
npm run test:e2e:ui

# または特定のテストのみ実行
npx playwright test --grep "should display menu"

# ヘッドモードで実行
npm run test:e2e:headed
```

## 📝 テスト追加方法

`e2e/app.spec.ts`に新しいテストを追加:

```typescript
test('新しいテストケース', async ({ page }) => {
  // テストコード
  await page.keyboard.press('j');
  await expect(page.locator('text=期待する文字列')).toBeVisible();
});
```

## 🎯 CI/CD統合

GitHub Actionsで自動実行する場合:

```yaml
# .github/workflows/test.yml
name: Tests
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - uses: actions/setup-node@v3
      
      - name: Install dependencies
        run: |
          cargo install trunk
          npm install
          npx playwright install --with-deps chromium
      
      - name: Run all tests
        run: moon run test-all
```

## 📈 レポート

テスト実行後、HTMLレポートが生成されます:

```bash
# レポートを開く
npx playwright show-report
```

## 🎉 次のステップ

1. **テストを実行してみる**
   ```bash
   moon run test-e2e-ui
   ```

2. **新しいテストケースを追加**
   - `e2e/app.spec.ts`を編集

3. **CI/CDに統合**
   - GitHub Actionsワークフローを追加

4. **カバレッジを拡大**
   - エッジケースのテスト
   - エラーハンドリングのテスト
