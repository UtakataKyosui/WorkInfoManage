# moonrepo Tasks

このプロジェクトはmoonrepoで管理されています。以下のタスクが利用可能です:

## 開発タスク

### ターミナル版の起動
```bash
moon run dev
```

### Web版の起動
```bash
moon run dev-web
```
ポート8080で起動します。http://127.0.0.1:8080/ でアクセス可能。

## ビルドタスク

### ターミナル版のリリースビルド
```bash
moon run build
```

### Web版のリリースビルド
```bash
moon run build-web
```

## チェック・リントタスク

### コードチェック
```bash
moon run check        # ターミナル版
moon run check-web    # Web版
```

### Clippy (リンター)
```bash
moon run clippy
```

### フォーマットチェック
```bash
moon run fmt-check
```

### フォーマット適用
```bash
moon run fmt
```

### すべてのリント実行
```bash
moon run lint
```

## テストタスク

### ユニットテスト実行
```bash
moon run test
```

## CI/CDタスク

### CI用の全チェック実行
```bash
moon run ci
```
以下を実行します:
- フォーマットチェック
- Clippy
- コードチェック (ターミナル & Web)
- テスト
- ビルド

## クリーンアップ

### ビルド成果物の削除
```bash
moon run clean
```

## その他の便利なコマンド

### タスク一覧の表示
```bash
moon query tasks
```

### プロジェクト情報の表示
```bash
moon query projects
```

### タスクの依存関係グラフ表示
```bash
moon query touched-files
```

## moonrepoの利点

1. **タスクキャッシング**: 変更がない場合、タスクをスキップして高速化
2. **並列実行**: 依存関係のないタスクを並列実行
3. **一貫性**: すべての開発者が同じコマンドを使用
4. **CI/CD統合**: GitHub Actionsなどで簡単に使用可能
