# 設計概要

## 全体方針
Vercel Labs `skills` のインストール挙動をRustライブラリとして再現し、`my-command install-skill` から利用できるようにする。実装は「モックファースト」で外部依存を抽象化し、検証はローカルで完結させる。

本変更では、ライブラリの主目的を「自己記述能力向上」とし、`my-command` 自身向けの埋め込みスキル配布経路（self/embedded）を第一級要件として扱う。

## モジュール構成（案）
- `types`: Skill / Agent / Source などの型定義
- `agents`: エージェント設定（skillsDir / globalSkillsDir / detectInstalled）
- `discovery`: SKILL.md 検索とパース
- `installer`: symlink/copy の実体、canonical path 管理
- `lock`: `.skill-lock.json` 読み書き、フォルダハッシュ管理
- `providers`: GitHub/GitLab/Local/Direct/Well-known の取得抽象
- `embedded`: コンパイル時埋め込みスキル定義（`include_str!` 相当）
- `cli`: `install-skill` と introspection コマンド

## データモデル
- Skill: `name`, `description`, `path`, `raw_content`, `metadata`
- Source: `type`, `url`, `subpath`, `skill_filter`, `ref`
- LockEntry: `source`, `source_type`, `source_url`, `skill_path`, `skill_folder_hash`, `installed_at`, `updated_at`

補足:
- `Source.type` に `self` / `embedded` を追加し、同義エイリアスとして解釈する
- `embedded` ソースは外部URLやcloneを伴わず、バイナリに埋め込まれた `SKILL.md` 群を入力とする

## スキル発見
- 優先探索ディレクトリ（`skills/`, `.agents/skills/`, `.claude/skills/` 等）を順に探索
- `SKILL.md` を frontmatter 解析し、`name`/`description` を必須とする
- `metadata.internal: true` は `INSTALL_INTERNAL_SKILLS=1` または明示的指定時のみ含める
- 何も見つからない場合は再帰探索（最大深度制限）
- `self` / `embedded` 指定時はファイルシステム探索を行わず、埋め込み定義を discovery 結果として返す

## インストール方式
- Canonical path: `.agents/skills/<skill-name>` を単一の真実の源とする
- `symlink` モード: canonical にコピー後、各エージェントの skillsDir に相対シンボリックリンク
- `copy` モード: 各エージェントの skillsDir に直接コピー
- symlink 失敗時は copy にフォールバック
- universal agents（`.agents/skills` を使うエージェント）には重複リンクを作らない
- `install-skill self` / `install-skill embedded` は provider 解決をスキップし、埋め込みコンテンツを canonical path に展開して同一の後段処理（symlink/copy、lock更新判定）へ渡す

## ロックファイル管理
- `~/.agents/.skill-lock.json` に global インストールのみ記録
- GitHub Tree API 相当のフォルダハッシュを記録し更新検知に利用
- 古いバージョンは自動的に初期化（後方互換を持たない）
- `embedded` ソースの `skill_folder_hash` は埋め込みコンテンツ由来の決定的ハッシュ（実装詳細は任意）を許可し、外部API依存を禁止する

## イントロスペクション（原則7）
- `my-command commands --output json`
- `my-command schema --command install-skill --output json-schema`
- JSONのトップレベルは `schemaVersion`, `type`, `ok` を固定
- `schema` 出力には `self` / `embedded` を指定可能な入力定義を含める

## 外部依存の扱い
- Git clone / HTTP fetch / Tree hash 取得はトレイト化
- テストはモック実装のみで完結
- 実API呼び出しは将来作業（Future work）
- `embedded` 経路の検証では外部通信を禁止し、ローカルテストのみで完結させる

## トレードオフ
- 既存CLI（Vercel Labs）と同等のUXを優先し、Rust側の最適化は最小化
- 実通信の検証は後回し（モックで品質担保）
