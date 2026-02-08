# skill-installer Change Specification

## ADDED Requirements

### Requirement: Multiple Agent Specification
The `--agent` option for `install-skill` MUST accept multiple platforms via comma-separated values or multiple occurrences, resolve installation targets while preserving order and removing duplicates.

#### Scenario: Comma-separated multiple platforms
When running `my-command install-skill --agent claude,opencode --yes`, the skill is distributed to both `.claude/skills` and `.config/opencode/skills` via symlink (falling back to copy on failure).

#### Scenario: Multiple `--agent` occurrences
When running `my-command install-skill --agent claude --agent opencode --yes`, the skill is distributed to both `.claude/skills` and `.config/opencode/skills`.

#### Scenario: Unknown agent name provided
When running `my-command install-skill --agent unknown --yes`, the command exits with an error for the unknown agent and does not start installation.
