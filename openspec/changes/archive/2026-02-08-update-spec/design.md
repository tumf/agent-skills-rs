# 設計: update-spec

## 変更の概要

既存の `openspec/specs/` 配下の仕様書を、最近実装された以下の変更に合わせて更新する：

1. `install-skill` コマンドから `source` 引数を削除（常に埋め込みスキルをインストール）
2. `--global` のデフォルト値を `false` に変更（project-local がデフォルト）
3. ロックファイル更新を project/global 両方で行うように変更

## 変更内容の詳細

### skill-installer/spec.md の更新

#### 削除した記述
- `self` / `embedded` 引数の選択に関する記述
- "global のみでロックファイルを更新" という制限

#### 追加/更新した記述
- **Requirement: 埋め込みスキルのインストール**
  - `install-skill` コマンドは引数なしで自動的に埋め込みスキルをソースとして解決
  - `self` / `embedded` の選択肢は不要

- **Requirement: インストールスコープのデフォルト**（新規追加）
  - デフォルトは project-local スコープ（`./.agents/...`）
  - `--global` フラグ指定時のみ global スコープ（`~/.agents/...`）
  - 各スコープでの動作を Scenario で明確化

- **Requirement: ロックファイル更新**
  - project/global 両方でロックファイルを更新
  - スコープに応じてパスが異なる
    - project-local: `./.agents/.skill-lock.json`
    - global: `~/.agents/.skill-lock.json`

### cli-introspection/spec.md の更新

#### 削除した記述
- `self` / `embedded` が有効値として含まれるという Scenario

#### 更新した記述
- **Scenario: install-skill のスキーマ取得**
  - `--global` のデフォルト値が `false`（project-local）であることを明記

## テストによる検証

すべての要件は既存のテストスイートで検証可能：

- **埋め込みスキルの自動解決**: `test_discover_embedded_skills`, `test_embedded_skill_installation`, `test_end_to_end_embedded_install`
- **CLI introspection**: `test_get_commands`, `test_get_command_schema`, `test_schema_includes_agent_skill_global`
- **ロックファイル更新**: `test_lock_manager_update_entry`, `test_embedded_source_lock_entry`
- **後方互換性**: `test_skill_lock_deserializes_integer_version`, `test_load_legacy_lock_with_integer_version`

## 影響範囲

- **コードへの影響**: なし（仕様書のみの更新）
- **互換性**: 既存の実装を正確に反映する更新のため、互換性への影響なし
- **ドキュメント**: `openspec/specs/` 配下の仕様書が最新の実装と一致

## 実装方針

1. 仕様書から古い記述を削除
2. 新しい要件と Scenario を追加
3. すべての Scenario が現在の実装と一致することを確認
4. `cargo test` で要件が検証可能であることを確認
