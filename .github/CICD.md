# GitHub Actions CI/CD セットアップ

## 🚀 作成したワークフロー

### 1. `test.yml` - メインテストワークフロー

**トリガー**: `main`, `develop`ブランチへのpush/PR

#### ジョブ構成

##### `rust-tests` - Rustテスト
- ユニットテスト (`cargo test --lib`)
- 統合テスト (`cargo test --test input_handling_test`)
- キャッシュ最適化 (cargo registry, index, build)

##### `e2e-tests` - E2Eテスト
- WASMビルド (`trunk build --release`)
- Playwright E2Eテスト実行
- テストレポート・ビデオのアップロード

##### `lint` - コード品質チェック
- フォーマットチェック (`cargo fmt`)
- Clippy (`cargo clippy`)

##### `build` - ビルドチェック
- Linux (`x86_64-unknown-linux-gnu`)
- WASM (`wasm32-unknown-unknown`)

##### `all-tests-passed` - 統合チェック
- すべてのジョブが成功したか確認

### 2. `quick-check.yml` - クイックバリデーション

**トリガー**: `main`以外のブランチへのpush/PR

- フォーマットチェック
- Clippy
- ビルドチェック

高速フィードバック用の軽量ワークフロー

## 📊 ワークフロー実行時間の目安

| ジョブ | 推定時間 |
|--------|----------|
| rust-tests | 3-5分 |
| e2e-tests | 5-8分 |
| lint | 1-2分 |
| build (Linux) | 3-5分 |
| build (WASM) | 4-6分 |
| **合計** | **10-15分** |

## 🔧 最適化ポイント

### キャッシュ戦略
```yaml
- uses: actions/cache@v4
  with:
    path: ~/.cargo/registry
    key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}
```

- Cargo registry
- Cargo index
- ビルド成果物
- npm依存関係

### 並列実行
- 各ジョブは並列実行
- マトリックスビルド (Linux/WASM)

## 📝 テスト結果の確認

### GitHub UI
1. リポジトリの「Actions」タブ
2. 該当のワークフロー実行を選択
3. 各ジョブの詳細を確認

### アーティファクト
- **playwright-report**: HTMLテストレポート (30日保持)
- **test-videos**: テスト実行ビデオ (7日保持)

ダウンロード方法:
1. ワークフロー実行ページ
2. 「Artifacts」セクション
3. ダウンロード

## 🎯 ステータスバッジ

README.mdに追加:

```markdown
![Tests](https://github.com/UtakataKyosui/WorkInfoManage/workflows/Tests/badge.svg)
![Quick Check](https://github.com/UtakataKyosui/WorkInfoManage/workflows/Quick%20Check/badge.svg)
```

## 🔐 シークレット設定

現在は不要ですが、将来的に必要になる可能性:

### Asana API (オプション)
```
Settings > Secrets and variables > Actions > New repository secret
Name: ASANA_ACCESS_TOKEN
Value: <your-token>
```

### GitHub Token
自動的に提供される `GITHUB_TOKEN` を使用

## 🚨 トラブルシューティング

### E2Eテストが失敗する場合

#### 1. ローカルで再現
```bash
moon run test-e2e
```

#### 2. アーティファクトを確認
- playwright-report をダウンロード
- test-videos で失敗箇所を確認

#### 3. デバッグモード
```yaml
- name: Run E2E tests
  run: npm run test:e2e
  env:
    DEBUG: pw:api
```

### ビルドが失敗する場合

#### キャッシュをクリア
1. Actions タブ
2. 「Caches」
3. 該当キャッシュを削除

#### 依存関係の更新
```bash
cargo update
npm update
```

### タイムアウトする場合

```yaml
- name: Run E2E tests
  run: npm run test:e2e
  timeout-minutes: 15  # デフォルトは360分
```

## 📈 CI/CD改善案

### 1. 条件付き実行
```yaml
- name: Run E2E tests
  if: github.event_name == 'pull_request'
  run: npm run test:e2e
```

### 2. マトリックステスト拡張
```yaml
strategy:
  matrix:
    os: [ubuntu-latest, macos-latest, windows-latest]
    rust: [stable, nightly]
```

### 3. デプロイメント追加
```yaml
deploy:
  needs: [rust-tests, e2e-tests]
  if: github.ref == 'refs/heads/main'
  runs-on: ubuntu-latest
  steps:
    - name: Deploy to GitHub Pages
      # ...
```

### 4. 通知設定
```yaml
- name: Notify on failure
  if: failure()
  uses: 8398a7/action-slack@v3
  with:
    status: ${{ job.status }}
```

## 🎓 ベストプラクティス

### 1. 失敗時の継続
```yaml
continue-on-error: true  # 警告のみ
```

### 2. 条件付きステップ
```yaml
if: always()  # 常に実行
if: success() # 成功時のみ
if: failure() # 失敗時のみ
```

### 3. 環境変数
```yaml
env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1
```

### 4. タイムアウト設定
```yaml
timeout-minutes: 30
```

## 📚 参考リンク

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Playwright CI Guide](https://playwright.dev/docs/ci)
- [Rust CI Best Practices](https://doc.rust-lang.org/cargo/guide/continuous-integration.html)

## ✅ チェックリスト

- [x] test.yml ワークフロー作成
- [x] quick-check.yml ワークフロー作成
- [x] キャッシュ設定
- [x] アーティファクトアップロード
- [x] マトリックスビルド
- [ ] ステータスバッジ追加 (README.md)
- [ ] 初回実行確認
- [ ] アーティファクトダウンロード確認

## 🎉 次のステップ

1. **コミット & プッシュ**
   ```bash
   git add .github/workflows/
   git commit -m "Add CI/CD workflows"
   git push
   ```

2. **GitHub Actionsタブで確認**
   - ワークフローの実行状況
   - テスト結果
   - アーティファクト

3. **README.mdにバッジ追加**
   ```markdown
   ![Tests](https://github.com/UtakataKyosui/WorkInfoManage/workflows/Tests/badge.svg)
   ```

4. **継続的改善**
   - テストカバレッジの拡大
   - パフォーマンス最適化
   - デプロイメント自動化
