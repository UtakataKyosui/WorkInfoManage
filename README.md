# WorkInfoManage

AsanaとGitHubを統合したタスク管理TUIアプリケーション。タスクの進捗状況をリアルタイムで可視化し、PRレビューの状態を自動的に反映します。

![Tests](https://github.com/UtakataKyosui/WorkInfoManage/workflows/Tests/badge.svg)
![Quick Check](https://github.com/UtakataKyosui/WorkInfoManage/workflows/Quick%20Check/badge.svg)

## ✨ 特徴

- 🎯 **統合タスク管理**: AsanaとGitHubの情報を一元管理
- 🚀 **リアルタイム同期**: タスクとPRの状態を自動更新
- 💻 **クロスプラットフォーム**: Terminal版とWeb版の両方をサポート
- 🎨 **美しいUI**: Ratatuiベースの洗練されたターミナルUI
- 🌐 **WASM対応**: ブラウザで動作するWeb版
- 📊 **日報機能**: Markdownエディタ付きカレンダー
- 🔄 **自動テスト**: Rust統合テスト + PlaywrightE2E

## 🚀 クイックスタート

### Terminal版

```bash
# ビルド & 実行
cargo run

# または moonrepo経由
moon run dev
```

### Web版

```bash
# 開発サーバー起動
trunk serve

# または moonrepo経由
moon run dev-web
```

ブラウザで http://127.0.0.1:8080 を開く

## 🧪 テスト

### すべてのテスト実行
```bash
moon run test-all
```

### 個別実行
```bash
# Rustユニットテスト
moon run test

# Rust統合テスト
moon run test-integration

# PlaywrightE2Eテスト
moon run test-e2e

# E2E UIモード (推奨)
moon run test-e2e-ui
```

詳細は [docs/testing/e2e.md](docs/testing/e2e.md) を参照

## 📁 プロジェクト構成

```
WorkInfoManage/
├── src/
│   ├── app.rs              # アプリケーションロジック
│   ├── input/              # 共有入力処理
│   │   ├── key_event.rs    # プラットフォーム非依存キーイベント
│   │   └── handlers.rs     # 入力ハンドラー
│   ├── testing/            # 共有テストデータ
│   ├── ui/                 # UI コンポーネント
│   ├── storage/            # ストレージ抽象化
│   └── logic/              # ビジネスロジック
├── src/bin/
│   └── web_demo.rs         # Web版エントリポイント
├── tests/
│   └── input_handling_test.rs  # Rust統合テスト
├── e2e/
│   └── app.spec.ts         # PlaywrightE2Eテスト
└── .github/
    └── workflows/          # CI/CDワークフロー
```

## 🛠️ 技術スタック

### Terminal版
- **Rust** - システムプログラミング言語
- **Ratatui** - ターミナルUI フレームワーク
- **Crossterm** - クロスプラットフォームターミナル制御
- **SeaORM** - Rust ORM
- **Tokio** - 非同期ランタイム

### Web版
- **Rust + WASM** - WebAssembly
- **Ratzilla** - WASMターミナルバックエンド
- **Trunk** - WASMビルドツール

### テスト
- **Rust** - 統合テスト
- **Playwright** - E2Eテスト
- **moonrepo** - タスクランナー

## 📊 コード品質

### リファクタリング成果

| 項目 | 削減量 |
|------|--------|
| `main.rs` | 80行 (12%) |
| `web_demo.rs` | 273行 (51%) |
| **合計削減** | **353行 (29%)** |

### 新規共有モジュール

| モジュール | 行数 | 機能 |
|-----------|------|------|
| `input/` | 465行 | プラットフォーム非依存入力処理 |
| `testing/` | 141行 | 共有テストデータ |
| `logic/physics.rs` | 21行 | アニメーション |

## 🔄 CI/CD

GitHub Actionsで自動テスト実行:

- ✅ Rustユニットテスト
- ✅ Rust統合テスト  
- ✅ PlaywrightE2Eテスト
- ✅ Clippy & フォーマットチェック
- ✅ Linux & WASMビルド

詳細は [.github/CICD.md](.github/CICD.md) を参照

## 📝 ドキュメント

### 開発ガイド
- [docs/development/moonrepo.md](docs/development/moonrepo.md) - moonrepo使用方法
- [docs/guides/migration.md](docs/guides/migration.md) - ストレージマイグレーションガイド

### テスト
- [docs/testing/overview.md](docs/testing/overview.md) - テスト戦略全体
- [docs/testing/e2e.md](docs/testing/e2e.md) - E2Eテスト詳細ガイド

### CI/CD
- [.github/CICD.md](.github/CICD.md) - CI/CD設定

### APIドキュメント
Rustdocを生成：
```bash
cargo doc --no-deps --open
```

## 🤝 コントリビューション

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 ライセンス

ISC License

## 👤 作者

UtakataKyosui

## 🙏 謝辞

- [Ratatui](https://github.com/ratatui-org/ratatui) - 素晴らしいTUIフレームワーク
- [Playwright](https://playwright.dev/) - 強力なE2Eテストツール
- [moonrepo](https://moonrepo.dev/) - モダンなタスクランナー
