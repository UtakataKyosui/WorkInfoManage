# ドキュメント

このディレクトリには、Work Info Manageプロジェクトの各種ドキュメントが含まれています。

## 📁 ディレクトリ構成

```
docs/
├── README.md                    # このファイル
├── development/                 # 開発関連ドキュメント
│   └── moonrepo.md             # moonrepoタスクランナーの使用方法
├── guides/                      # ガイド・チュートリアル
│   └── migration.md            # ストレージマイグレーションガイド
└── testing/                     # テスト関連ドキュメント
    ├── overview.md             # テスト戦略全体
    └── e2e.md                  # E2Eテスト詳細ガイド
```

## 📚 ドキュメント一覧

### 開発ガイド

#### [moonrepo.md](development/moonrepo.md)
moonrepoタスクランナーの使用方法。以下のタスクについて説明：
- 開発タスク（`dev`, `dev-web`）
- ビルドタスク（`build`, `build-web`）
- テストタスク（`test`, `test-integration`, `test-e2e`）
- リント・フォーマット（`lint`, `fmt`）

### ガイド・チュートリアル

#### [migration.md](guides/migration.md)
ストレージバックエンドの移行ガイド：
- JSONファイルストレージとPostgreSQLデータベースの切り替え
- 自動マイグレーション機能の説明
- トラブルシューティング

### テストドキュメント

#### [overview.md](testing/overview.md)
テスト戦略の全体像：
- Terminal版テスト（Rust統合テスト）
- Web版テスト（Playwright E2E）
- moonrepoタスク統合
- CI/CD統合

#### [e2e.md](testing/e2e.md)
E2Eテストの詳細ガイド：
- Playwrightのセットアップ
- テスト実行方法（通常/UI/デバッグモード）
- テストカバレッジ
- トラブルシューティング

## 🔗 関連ドキュメント

### プロジェクトルート
- [README.md](../README.md) - プロジェクト概要
- [Cargo.toml](../Cargo.toml) - Rust依存関係
- [package.json](../package.json) - Node.js依存関係

### CI/CD
- [.github/CICD.md](../.github/CICD.md) - GitHub Actions設定

### APIドキュメント

Rustdocを生成して、コード内のドキュメントコメントを確認：

```bash
cargo doc --no-deps --open
```

主要モジュールのドキュメント：
- `src/lib.rs` - プロジェクト全体の概要
- `src/storage/mod.rs` - ストレージシステムの詳細
- `src/input/mod.rs` - 入力処理システムの説明

## 📝 ドキュメントの更新

ドキュメントを更新する際は、以下のガイドラインに従ってください：

1. **Markdownフォーマット**: GitHub Flavored Markdown（GFM）を使用
2. **コードブロック**: 言語指定を含める（```rust, ```bash, ```toml等）
3. **リンク**: 相対パスを使用し、ファイル移動時は更新する
4. **日本語**: 主要ドキュメントは日本語で記述
5. **構造**: 見出しレベルを適切に使用（H1は1つのみ）

## 🎯 ドキュメント作成のベストプラクティス

### Rustドキュメントコメント
- モジュールレベル: `//!` を使用
- 関数/構造体レベル: `///` を使用
- 例を含める: `# Examples` セクション
- テストを含める: `# Safety`, `# Panics`, `# Errors` セクション

### TSDocコメント
- `/**` で開始し、`*/` で終了
- `@param`, `@returns`, `@see` タグを活用
- 例を含める: `@example` タグ

### Markdownドキュメント
- 目次を含める（長いドキュメントの場合）
- コード例を豊富に含める
- スクリーンショットや図を活用（必要に応じて）
- 関連ドキュメントへのリンクを含める
