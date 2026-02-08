# 設計概要

## 全体方針
Vercel Labs `skills` のインストール挙動をRustライブラリとして再現し、`my-command install-skill` から利用できるようにする。実装は「モックファースト」で外部依存を抽象化し、検証はローカルで完結させる。

## モジュール構成（案）
- `types`: Skill / Agent / Source などの型定義
- `agents`: エージェント設定（skillsDir / globalSkillsDir / detectInstalled）
- `discovery`: SKILL.md 検索とパース
- `installer`: symlink/copy の実体、canonical path 管理
- `lock`: `.skill-lock.json` 読み書き、フォルダハッシュ管理
- `providers`: GitHub/GitLab/Local/Direct/Well-known の取得抽象
- `cli`: `install-skill` と introspection コマンド

## データモデル
- Skill: `name`, `description`, `path`, `raw_content`, `metadata`
- Source: `type`, `url`, `subpath`, `skill_filter`, `ref`
- LockEntry: `source`, `source_type`, `source_url`, `skill_path`, `skill_folder_hash`, `installed_at`, `updated_at`

## スキル発見
- 優先探索ディレクトリ（`skills/`, `.agents/skills/`, `.claude/skills/` 等）を順に探索
- `SKILL.md` を frontmatter 解析し、`name`/`description` を必須とする
- `metadata.internal: true` は `INSTALL_INTERNAL_SKILLS=1` または明示的指定時のみ含める
- 何も見つからない場合は再帰探索（最大深度制限）

## インストール方式
- Canonical path: `.agents/skills/<skill-name>` を単一の真実の源とする
- `symlink` モード: canonical にコピー後、各エージェントの skillsDir に相対シンボリックリンク
- `copy` モード: 各エージェントの skillsDir に直接コピー
- symlink 失敗時は copy にフォールバック
- universal agents（`.agents/skills` を使うエージェント）には重複リンクを作らない

## ロックファイル管理
- `~/.agents/.skill-lock.json` に global インストールのみ記録
- GitHub Tree API 相当のフォルダハッシュを記録し更新検知に利用
- 古いバージョンは自動的に初期化（後方互換を持たない）

## イントロスペクション（原則7）
- `my-command commands --output json`
- `my-command schema --command install-skill --output json-schema`
- JSONのトップレベルは `schemaVersion`, `type`, `ok` を固定

## 外部依存の扱い
- Git clone / HTTP fetch / Tree hash 取得はトレイト化
- テストはモック実装のみで完結
- 実API呼び出しは将来作業（Future work）

## トレードオフ
- 既存CLI（Vercel Labs）と同等のUXを優先し、Rust側の最適化は最小化
- 実通信の検証は後回し（モックで品質担保）
