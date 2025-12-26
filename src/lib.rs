//! # Work Info Manage
//!
//! タスク管理、カレンダー、デイリーレポート、メモ機能を統合したTUIアプリケーション。
//!
//! ## 主要機能
//!
//! - **タスク管理**: Asana/GitHub連携によるタスクの同期と管理
//! - **カレンダー**: 日付ベースのナビゲーションとデイリーレポート作成
//! - **メモ機能**: Markdownベースのメモ作成と管理
//! - **マルチプラットフォーム**: ターミナル版とWeb版（WASM）の両方をサポート
//!
//! ## アーキテクチャ
//!
//! ### ストレージ層 (`storage`)
//! - JSONファイルストレージとPostgreSQLデータベースの両方をサポート
//! - 設定ファイル（`config.toml`）でストレージタイプを選択可能
//! - ストレージタイプ変更時の自動マイグレーション機能
//!
//! ### UI層 (`ui`, `app`)
//! - Ratatui（ターミナル）とYew（Web）による統一されたUI
//! - 画面遷移とキーボードナビゲーションの一貫性
//!
//! ### 入力処理 (`input`)
//! - プラットフォーム非依存の入力ハンドラ
//! - ターミナル版とWeb版で共通のキーイベント処理
//!
//! ### ビジネスロジック (`logic`)
//! - タスク同期、物理演算（デモ用）、レビュアー管理
//!
//! ## 使用方法
//!
//! ### ターミナル版
//! ```bash
//! cargo run
//! ```
//!
//! ### Web版
//! ```bash
//! trunk serve
//! ```
//!
//! ## 設定
//!
//! `config.toml`でストレージバックエンドを設定：
//!
//! ```toml
//! [storage]
//! type = "json"  # または "database"
//! path = "~/task-manage/data.json"
//! ```
//!
//! 詳細は`docs/guides/migration.md`を参照してください。

#![allow(non_snake_case)]

pub mod app;
pub mod ui;
pub mod db;
pub mod logic;
#[cfg(not(target_arch = "wasm32"))]
pub mod api;
#[cfg(not(target_arch = "wasm32"))]
pub mod config;
pub mod storage;
pub mod report;
pub mod memo;
pub mod input;
pub mod testing;
pub mod animation;
