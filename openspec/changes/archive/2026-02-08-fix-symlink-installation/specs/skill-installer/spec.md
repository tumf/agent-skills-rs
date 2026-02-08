# skill-installer Change Specification

## MODIFIED Requirements

### Requirement: Multiple Agent Specification
The `--agent` option for `install-skill` MUST accept multiple platforms via comma-separated values or multiple occurrences, resolve installation targets while preserving order and removing duplicates.

For OpenCode in project scope, no additional targets are created as `.agents/skills` is universal; in global scope, `~/.config/opencode/skills` MUST be resolved as the target.

`install-skill` MUST perform installation to canonical only once and create links (or copy on failure) by passing resolved targets to `InstallConfig.target_dirs`. Independent installations for each target MUST NOT be performed.

#### Scenario: Comma-separated multiple platforms
When running `my-command install-skill --agent claude,opencode --yes`, the skill is distributed to both `.claude/skills` and `.config/opencode/skills` via symlink (falling back to copy on failure).

#### Scenario: Multiple `--agent` occurrences
When running `my-command install-skill --agent claude --agent opencode --yes`, the skill is distributed to both `.claude/skills` and `.config/opencode/skills`.

#### Scenario: Unknown agent name provided
When running `my-command install-skill --agent unknown --yes`, the command exits with an error for the unknown agent and does not start installation.

#### Scenario: Scope-based resolution for OpenCode
When executing `my-command install-skill --agent opencode --yes` in project scope, only the canonical installation to `.agents/skills/<skill-name>` is performed. When executing `my-command install-skill --agent opencode --global --yes`, the canonical is created at `~/.agents/skills/<skill-name>` and `~/.config/opencode/skills/<skill-name>` is resolved as the target.

#### Scenario: Distribution via symbolic links
When executing `my-command install-skill --agent claude --yes`, the actual content is created at `.agents/skills/<skill-name>` and a symbolic link is created at `.claude/skills/<skill-name>` (falling back to copy on failure).
