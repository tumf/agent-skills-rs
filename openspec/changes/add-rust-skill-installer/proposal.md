# 変更提案: Rustスキルインストーラライブラリの追加

## 背景
Vercel Labs の `skills` CLI が提供している「スキル発見 → インストール → ロック管理」の挙動を、Rust製CLIの `my-command install-skill` から再現したい。加えて Agentic CLI Design 原則7（Introspectable）を満たす自己記述的なCLI設計を組み込みたい。

追加要件として、このライブラリの主要目的を「CLI/エージェントの自己記述能力向上」と定義する。特に `my-command` 自身が自分向けスキルをインストールできるように、`SKILL.md` 等の実コンテンツをコンパイル時埋め込み（`include_str!` 相当）で扱う経路を正式サポートする。

## 目的
- `my-command install-skill` でユーザー環境にスキルを正確にインストールできる Rust ライブラリを提供する
- Vercel Labs `skills` のインストールプロセス（canonical path、symlink/copy、ロックファイル、スキル発見）を忠実に再現する
- CLIが自己記述可能であること（`commands --output json` / `schema --command ...`）を提供する
- `my-command install-skill self`（または `embedded`）で、コンパイル時埋め込みスキルを外部通信なしでインストール可能にする

## 範囲
- RustライブラリのコアAPI（スキル発見、インストール、ロックファイル管理）
- RustライブラリのコアAPI（スキル発見、インストール、ロックファイル管理、埋め込みスキルソース）
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
- コンパイル時埋め込みスキル（self/embedded source）の要件定義

## 依存関係
- ファイルシステム（symlink/copy）
- Git clone 相当の取得手段（本実装は抽象化し、テストはモックで担保）
- GitHub Tree API 相当のフォルダハッシュ取得（抽象化し、テストはモックで担保）
- コンパイル時埋め込み機構（`include_str!` 相当）

## 受け入れ基準
- Vercel Labs `skills` と同等のスキル発見/インストール挙動を再現できる
- `my-command commands --output json` および `my-command schema --command install-skill --output json-schema` が利用できる
- 非対話モードで完走可能（`--yes` / `--non-interactive`）
- `my-command install-skill self` または `my-command install-skill embedded` により、外部通信なしで埋め込み `SKILL.md` をインストールできる
