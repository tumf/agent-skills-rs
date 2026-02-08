# 変更提案: Rustスキルインストーラライブラリの追加

## 背景
Vercel Labs の `skills` CLI が提供している「スキル発見 → インストール → ロック管理」の挙動を、Rust製CLIの `my-command install-skill` から再現したい。加えて Agentic CLI Design 原則7（Introspectable）を満たす自己記述的なCLI設計を組み込みたい。

## 目的
- `my-command install-skill` でユーザー環境にスキルを正確にインストールできる Rust ライブラリを提供する
- Vercel Labs `skills` のインストールプロセス（canonical path、symlink/copy、ロックファイル、スキル発見）を忠実に再現する
- CLIが自己記述可能であること（`commands --output json` / `schema --command ...`）を提供する

## 範囲
- RustライブラリのコアAPI（スキル発見、インストール、ロックファイル管理）
- `my-command install-skill` への組み込みに必要なCLI仕様
- イントロスペクション用CLI出力仕様（JSON）

## 非範囲
- 実際の外部API（GitHub/GitLab）への認証・実通信検証
- UIやGUI統合
- 既存のスキルエコシステム仕様の拡張（Agent Skills Spec自体の変更）

## 主要成果物
- Rustライブラリ仕様（API設計・データモデル）
- CLIの自己記述出力仕様
- スキル発見/インストール/ロックの要件定義

## 依存関係
- ファイルシステム（symlink/copy）
- Git clone 相当の取得手段（本実装は抽象化し、テストはモックで担保）
- GitHub Tree API 相当のフォルダハッシュ取得（抽象化し、テストはモックで担保）

## 受け入れ基準
- Vercel Labs `skills` と同等のスキル発見/インストール挙動を再現できる
- `my-command commands --output json` および `my-command schema --command install-skill --output json-schema` が利用できる
- 非対話モードで完走可能（`--yes` / `--non-interactive`）
