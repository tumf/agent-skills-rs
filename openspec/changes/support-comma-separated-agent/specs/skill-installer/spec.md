# skill-installer 変更仕様

## MODIFIED Requirements

### Requirement: Agent オプションの複数指定
`install-skill` の `--agent` はカンマ区切りまたは複数回指定で複数 platform を受け付け、指定順を維持した上で重複を除外して配布先を解決しなければならない (MUST)。

#### Scenario: カンマ区切りで複数 platform を指定
`my-command install-skill --agent claude,opencode --yes` を実行すると、`.claude/skills` と `.config/opencode/skills` の両方に symlink（失敗時は copy）で配布される。

#### Scenario: `--agent` の複数回指定
`my-command install-skill --agent claude --agent opencode --yes` を実行すると、`.claude/skills` と `.config/opencode/skills` の両方に配布される。

#### Scenario: 未知の agent 名を含む場合
`my-command install-skill --agent unknown --yes` を実行すると、未知の agent としてエラー終了し、配布を開始しない。
