# cli-introspection 変更仕様

## MODIFIED Requirements

### Requirement: install-skill の agent 説明
`schema --command install-skill --output json-schema` の `agent` 引数説明は、カンマ区切りおよび複数回指定を許可する旨を明示しなければならない (MUST)。

#### Scenario: agent 説明の更新
`my-command schema --command install-skill --output json-schema` を実行すると、`agent` の description に「comma-separated」と「repeatable」に相当する説明が含まれる。
