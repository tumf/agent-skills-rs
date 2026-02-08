# Capability: スキルインストール基盤

## ADDED Requirements

### Requirement: スキル発見の優先探索
`discover` MUST は優先ディレクトリ（`skills/`, `skills/.curated/`, `.agents/skills/`, `.claude/skills/` など）を順に探索し、`SKILL.md` の frontmatter に `name` と `description` が存在するもののみをスキルとして返すこと。

#### Scenario: 優先探索と internal フィルタ
あるリポジトリに `skills/alpha/SKILL.md` と `.agents/skills/beta/SKILL.md` が存在し、`beta` の frontmatter に `metadata.internal: true` が設定されている。`INSTALL_INTERNAL_SKILLS` が未設定の場合、`discover` は `alpha` のみを返す。

### Requirement: Canonical path とインストール方式
`install` MUST は canonical path（`.agents/skills/<skill-name>`）を単一の真実の源として扱い、`symlink` または `copy` のモードでインストールできること。`symlink` 失敗時は `copy` にフォールバックすること。

#### Scenario: symlink 失敗時のフォールバック
`symlink` が許可されていない環境で `install` を実行した場合、インストール結果は `symlinkFailed = true` を含み、実ファイルは `copy` と同様に配置される。

### Requirement: ロックファイル更新（globalのみ）
`install` MUST は global スコープで成功した場合、`~/.agents/.skill-lock.json` に `source`, `sourceType`, `sourceUrl`, `skillFolderHash` を記録すること。ローカルスコープでは記録しないこと。

#### Scenario: GitHub ソースのロック更新
GitHub 由来のスキルを global にインストールした場合、ロックファイルに `skillFolderHash` が保存される。local ソースの場合は `skillFolderHash` が空文字でも良い。
