# skill-installer 変更仕様

## MODIFIED Requirements

### Requirement: 複数エージェント指定時のターゲット解決
`install-skill` は `--agent` で指定されたエージェントに応じてターゲットディレクトリを解決し、スコープに応じた正しいパスを使用しなければならない。OpenCode は project スコープでは universal であるため追加のターゲットを作成せず、global スコープでは `~/.config/opencode/skills` をターゲットとして解決することを MUST とする。

#### Scenario: OpenCode のスコープ別解決
`my-command install-skill --agent opencode --yes` を project スコープで実行した場合、`.agents/skills/<skill-name>` への canonical インストールのみが行われる。`my-command install-skill --agent opencode --global --yes` を実行した場合、canonical は `~/.agents/skills/<skill-name>` に作成され、ターゲットとして `~/.config/opencode/skills/<skill-name>` が解決される。

### Requirement: canonical とターゲットの一回インストール
`install-skill` は canonical へのインストールを 1 回だけ行い、`InstallConfig.target_dirs` に解決済みターゲットを渡してリンク（失敗時は copy）を作成しなければならない。ターゲットごとに独立したインストールを行わないことを MUST とする。

#### Scenario: シンボリックリンクによる配布
`my-command install-skill --agent claude --yes` を実行した場合、`.agents/skills/<skill-name>` に実体を作成し、`.claude/skills/<skill-name>` にはシンボリックリンクを作成する（失敗時は copy へフォールバック）。
