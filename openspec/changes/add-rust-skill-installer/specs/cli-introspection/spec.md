# Capability: CLI自己記述（Introspectable）

## ADDED Requirements

### Requirement: コマンド一覧のJSON出力
CLI MUST は `commands --output json` を提供し、トップレベルに `schemaVersion`, `type`, `ok` を含むJSONを返すこと。

#### Scenario: コマンド一覧の取得
`my-command commands --output json` を実行すると、`ok: true` と `type: commands.list` を含むJSONがstdoutに出力される。

### Requirement: コマンドスキーマのJSON Schema出力
CLI MUST は `schema --command install-skill --output json-schema` を提供し、引数・オプションのJSON Schemaを返すこと。

#### Scenario: install-skill のスキーマ取得
`my-command schema --command install-skill --output json-schema` を実行すると、`install-skill` の引数（`--agent`, `--skill`, `--yes`, `--global` など）が定義されたJSON Schemaがstdoutに出力される。
