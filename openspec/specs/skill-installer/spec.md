# skill-installer Specification

## Purpose
TBD - created by archiving change add-rust-skill-installer. Update Purpose after archive.
## Requirements
### Requirement: スキル発見の優先探索
`discover` MUST は優先ディレクトリ（`skills/`, `skills/.curated/`, `.agents/skills/`, `.claude/skills/` など）を順に探索し、`SKILL.md` の frontmatter に `name` と `description` が存在するもののみをスキルとして返すこと。

#### Scenario: 優先探索と internal フィルタ
あるリポジトリに `skills/alpha/SKILL.md` と `.agents/skills/beta/SKILL.md` が存在し、`beta` の frontmatter に `metadata.internal: true` が設定されている。`INSTALL_INTERNAL_SKILLS` が未設定の場合、`discover` は `alpha` のみを返す。

### Requirement: 埋め込みスキルソース（self/embedded）の提供
CLI MUST は `install-skill self` または `install-skill embedded` を受け付け、コンパイル時に埋め込まれた `SKILL.md` 等の実コンテンツをソースとして解決できること。`self` と `embedded` は同義として扱うこと。

#### Scenario: self 指定で埋め込みスキルを解決
`my-command install-skill self --yes` を実行した場合、実装は外部provider（clone/fetch）を呼び出さず、埋め込みコンテンツからスキルを解決してインストール処理へ進む。

### Requirement: Canonical path とインストール方式
`install` MUST は canonical path（`.agents/skills/<skill-name>`）を単一の真実の源として扱い、`symlink` または `copy` のモードでインストールできること。`symlink` 失敗時は `copy` にフォールバックすること。

#### Scenario: symlink 失敗時のフォールバック
`symlink` が許可されていない環境で `install` を実行した場合、インストール結果は `symlinkFailed = true` を含み、実ファイルは `copy` と同様に配置される。

#### Scenario: embedded ソースでも canonical 処理を共通適用
`my-command install-skill embedded --yes` を実行した場合、埋め込みスキルは `.agents/skills/<skill-name>` に配置され、その後の agent 別配布は通常ソースと同じ symlink/copy 規則で実行される。

### Requirement: ロックファイル更新（globalのみ）
`install` MUST は global スコープで成功した場合、`~/.agents/.skill-lock.json` に `source`, `sourceType`, `sourceUrl`, `skillFolderHash` を記録すること。ローカルスコープでは記録しないこと。

#### Scenario: GitHub ソースのロック更新
GitHub 由来のスキルを global にインストールした場合、ロックファイルに `skillFolderHash` が保存される。local ソースの場合は `skillFolderHash` が空文字でも良い。

#### Scenario: embedded ソースのロック更新
埋め込みスキルを global にインストールした場合、ロックファイルには `sourceType` として `embedded`（または同義の `self`）が記録され、`skillFolderHash` は外部APIを使わず算出された決定的な値で保存される。

