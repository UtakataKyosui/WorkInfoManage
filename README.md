# TaskManagerWithLLM

AsanaとGitHubを統合したタスク管理TUIアプリケーション。タスクの進捗状況をリアルタイムで可視化し、PRレビューの状態を自動的に反映します。

## 主な機能

- ✅ **Asana連携**: カスタムフィールド（ステータス、タスク種別）の自動取得
- ✅ **GitHub PR連携**: タスクコメントからPRリンクを自動抽出
- ✅ **自動ステータス推論**: PRレビュー状況に基づいた詳細ステータス
- ✅ **TUIインターフェース**: 3カラムレイアウトでタスクを視覚化
- ✅ **期限管理**: 期限超過タスクを赤文字で警告
- ✅ **キーボードナビゲーション**: 快適な操作性
- ✅ **起動時自動同期**: アプリ起動時に最新データを取得
- ✅ **日報作成機能**: Markdownで日報を作成・編集・プレビュー (NEW!)
- ✅ **設定可能なストレージ**: JSON/Database切り替え可能

## 必要要件

- Rust 1.70.0以上
- PostgreSQL 14以上
- Asana Personal Access Token
- GitHub Personal Access Token

## セットアップ

### 1. リポジトリをクローン

```bash
git clone <repository-url>
cd TaskManagerWithLLM
```

### 2. ストレージの設定

TaskManagerは2種類のストレージバックエンドをサポートしています：

#### オプション1: JSONファイルストレージ（推奨・簡単）

`config.toml`を作成：

```toml
[storage]
type = "json"
path = "~/task-manage/data.json"
```

このオプションでは、データベースのセットアップは不要です。タスクデータは`~/task-manage/data.json`に保存されます。

#### オプション2: PostgreSQLデータベース

`config.toml`を作成：

```toml
[storage]
type = "database"
url = "${DATABASE_URL}"
```

データベースを起動：

```bash
# スクリプトを使用してデータベースを起動（推奨）
./scripts/start-db.sh

# または、手動でDocker Composeを使用
docker-compose up -d db

# または、ローカルのPostgreSQLを使用
createdb taskmanager
```

**Note:** `./scripts/start-db.sh` を使用すると、Docker Compose v1/v2の両方に対応し、自動的にデータベースを起動します。

### 3. 環境変数の設定

`.env.example`を参考に`.env`ファイルを作成：

```bash
cp .env.example .env
```

`.env`ファイルを編集：

```bash
# Database
DATABASE_URL=postgresql://user:password@localhost:5432/taskmanager

# Asana API
ASANA_ACCESS_TOKEN=your_asana_personal_access_token
ASANA_WORKSPACE_GID=your_workspace_gid
ASANA_USER_GID=your_user_gid

# GitHub API
GITHUB_PERSONAL_ACCESS_TOKEN=your_github_personal_access_token
```

#### 環境変数の取得方法

**Asana Access Token:**
1. https://app.asana.com/0/my-apps にアクセス
2. "Personal access tokens" セクション
3. "+ Create new token" をクリック
4. トークンをコピーして `ASANA_ACCESS_TOKEN` に設定

**Asana Workspace GID:**
```bash
curl -H "Authorization: Bearer YOUR_TOKEN" \
  https://app.asana.com/api/1.0/workspaces
```
レスポンスから `gid` を取得

**Asana User GID:**
```bash
curl -H "Authorization: Bearer YOUR_TOKEN" \
  https://app.asana.com/api/1.0/users/me
```
レスポンスから `gid` を取得

**GitHub Personal Access Token:**
1. GitHub Settings → Developer settings → Personal access tokens → Tokens (classic)
2. "Generate new token (classic)" をクリック
3. スコープ: `repo`, `read:user`, `read:org` を選択
4. トークンをコピーして `GITHUB_PERSONAL_ACCESS_TOKEN` に設定

### 4. レビュアー設定

`reviewers.json`を編集してレビュアーのGitHub usernameを設定：

```json
{
    "internal_reviewers": [
        "your-github-username",
        "teammate1",
        "teammate2"
    ],
    "external_reviewers": [
        "external-auditor",
        "client-reviewer"
    ]
}
```

- **internal_reviewers**: 社内レビュアーのGitHub username
- **external_reviewers**: 外部レビュアーのGitHub username

### 5. ビルドと実行

```bash
# ビルド
cargo build --release

# 実行
./target/release/TaskManagerWithLLM
```

または開発モード：

```bash
cargo run
```

## 使い方

### TUI操作

### タスク管理

- `↑`/`↓`: タスク選択
- `Enter`: タスク詳細表示
- `Esc`: 前の画面に戻る
- `Tab`: ビュー切り替え（Development/Internal Review/External Review）
- `t`: タイマー開始/停止
- `n`: ノート追加
- `s`: Asana/GitHubと同期

### 日報作成

#### カレンダー画面

- `Shift+Tab`: カレンダー画面に切り替え
- `Esc`: タスク画面に戻る
- `Enter`: 日付を選択（日報作成/プレビュー）
- `←`/`→`: 月移動 ✅
- `↑`/`↓`: 週単位で日付移動 ✅

**カレンダー表示機能:**
- 月間カレンダーグリッド表示 ✅
- 選択中の日付をハイライト（青背景） ✅
- 今日の日付を緑色で表示 ✅
- 日報が存在する日付をシアン色で表示 ✅

#### エディタ画面

- `Shift+Enter`: 改行
- `Ctrl+S`: 保存してプレビュー表示（実装予定）
- `Esc`: カレンダーに戻る

#### プレビュー画面

- `e`: 編集モードに移行
- `↑`/`↓`: スクロール（実装予定）
- `Esc`: カレンダーに戻る

**日報の保存場所**: `~/task-manage/daily-report/YYYY-MM-DD.md`

**実装状況:**
- ✅ カレンダーUI（月間表示、日付選択、ナビゲーション）
- ✅ ファイルストレージ（保存・読み込み）
- ⏳ Markdownエディタ（tui-textarea統合）
- ⏳ プレビュー画面（Markdownレンダリング）

### 画面構成

```
┌─ Development ─────┐┌─ Internal Review ──┐┌─ External Review ──┐
│ Not Started       ││ UnChecked          ││ UnChecked          │
│ - Task A          ││ - Task B           ││ - Task E           │
│ In Progress       ││ Checked            ││ - Task D           │
└───────────────────┘└────────────────────┘└────────────────────┘

┌─ Task Details ────────────────────────────────────────────────┐
│ Title: [タスク名]                                              │
│ Status: In Progress                                           │
│ Type: 実装                                                     │
│ Due Date: 2024-12-31 (または ⚠ Due Date (OVERDUE): 赤字表示) │
│ Description: ...                                              │
└───────────────────────────────────────────────────────────────┘

┌─ Info ────────────────────────────────────────────────────────┐
│ Controls: ↑↓: Select | s: Sync | Esc/q: Quit                 │
│ Status: Loaded 8 tasks. Press 's' to refresh.                │
└───────────────────────────────────────────────────────────────┘
```

### ステータスの判定ロジック

#### 1. PRがない場合
Asanaカスタムフィールドの「ステータス」を使用：
- **Ready** → `Not Started`
- **In Progress** → `In Progress`
- **In Review** → `In Review`（PRがないため詳細不明）

#### 2. PRがある場合
PRのレビュー状況を詳細に反映：

**External Review（外部レビュー）:**
- `external_reviewers`リストの誰かがアサインされている
- ✅ **Checked**: 外部レビュアーが承認済み
- ⚠️ **UnChecked**: 外部レビュアー未承認

**Internal Review（内部レビュー）:**
- `internal_reviewers`リストの誰かがアサインされている（または外部レビュアーなし）
- ✅ **Checked**: 内部レビュアーが承認済み
- ⚠️ **UnChecked**: 内部レビュアー未承認

#### PR検出方法

タスクのコメント（stories）から以下の条件でPRを検出：
1. コメント本文に「**PRを作成しました**」が含まれる
2. そのコメント内にGitHub PR URLが含まれる
3. URL形式: `https://github.com/owner/repo/pull/123`

## トラブルシューティング

### タスクが同期されない

1. `.env`ファイルの設定を確認
2. `logs/sync_detail.log`でエラーを確認：
   ```bash
   tail -100 logs/sync_detail.log
   ```

### 期限が表示されない

Asanaタスクに期日（due_on）が設定されているか確認してください。

### PRが検出されない

1. タスクのコメントに「PRを作成しました」というテキストがあるか確認
2. そのコメントにGitHub PR URLが含まれているか確認
3. `logs/sync_detail.log`で「Found PR in comments」が表示されているか確認

### レビューステータスが正しくない

1. `reviewers.json`にレビュアーのGitHub usernameが正しく設定されているか確認
2. PRに実際にレビュアーがアサインされているか確認
3. GitHub APIトークンに`repo`スコープがあるか確認

### ターミナルが壊れる

アプリ終了時に自動的にクリーンアップされますが、問題が発生した場合：
```bash
reset
```

## 開発

### テスト実行

```bash
cargo test
```

### ログ確認

```bash
# 同期ログ
tail -f logs/sync_detail.log

# アプリケーションログ
tail -f logs/sync.log
```

### データベースマイグレーション

```bash
cd migration
cargo run
```

## ライセンス

MIT

## 貢献

Issue、Pull Requestを歓迎します！
