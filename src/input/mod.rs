//! # 入力処理システム
//!
//! このモジュールは、ターミナル版とWeb版で共通の入力処理ロジックを提供します。
//!
//! ## 設計思想
//!
//! プラットフォーム非依存の入力ハンドラを実装することで、以下を実現：
//!
//! - **コードの再利用**: 同じロジックをターミナル版とWeb版で共有
//! - **一貫性**: すべてのプラットフォームで同じキーボード操作
//! - **テスト容易性**: プラットフォーム固有の依存なしでテスト可能
//!
//! ## 主要コンポーネント
//!
//! ### [`KeyEvent`]
//! プラットフォーム非依存のキーイベント表現。
//! - `crossterm::event::KeyEvent`（ターミナル）からの変換
//! - `web_sys::KeyboardEvent`（Web）からの変換
//!
//! ### [`InputHandler`]
//! 各画面の入力処理を実装するトレイト。
//! - `handle_key`: キーイベントを処理し、アプリケーション状態を更新
//! - すべての画面（Menu, Dashboard, Detail, Calendar等）で実装
//!
//! ## 使用例
//!
//! ```rust,no_run
//! use work_info_manage::input::{InputHandler, KeyEvent, KeyCode};
//! use work_info_manage::app::App;
//!
//! fn handle_input(app: &mut App, key: KeyEvent) {
//!     match app.current_screen {
//!         work_info_manage::app::CurrentScreen::Menu => {
//!             // MenuのInputHandlerを使用
//!             InputHandler::handle_menu(app, key);
//!         }
//!         _ => {}
//!     }
//! }
//! ```
//!
//! ## テスト
//!
//! 統合テスト（`tests/input_handling_test.rs`）で、すべての画面の入力処理を検証。
//! 詳細は`docs/testing/overview.md`を参照。

// Shared input handling logic for all screens
// This module contains platform-agnostic input handlers

pub mod handlers;
pub mod key_event;

pub use handlers::InputHandler;
pub use key_event::{KeyCode, KeyEvent, KeyModifiers};
