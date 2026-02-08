# skill-installer Specification

## Purpose
Rust library for installing and managing agent skills with embedded skill support. Provides skill discovery from multiple sources, installation with symlink/copy modes, canonical path management, and lock file tracking.
## Requirements
### Requirement: Priority Directory Discovery
`discover` MUST search priority directories (`skills/`, `skills/.curated/`, `.agents/skills/`, `.claude/skills/`, etc.) in order and return only skills whose `SKILL.md` frontmatter contains both `name` and `description` fields.

#### Scenario: Priority search with internal filter
Given a repository with `skills/alpha/SKILL.md` and `.agents/skills/beta/SKILL.md`, where `beta`'s frontmatter has `metadata.internal: true` set. When `INSTALL_INTERNAL_SKILLS` is unset, `discover` returns only `alpha`.

### Requirement: Embedded Skill Installation
CLI MUST use the `install-skill` command to automatically resolve `SKILL.md` and other content embedded at compile time as the source. No external source specification is required.

#### Scenario: Automatic resolution of embedded skills
When running `my-command install-skill --yes`, the implementation resolves skills from embedded content and proceeds to installation without calling external providers (clone/fetch).

### Requirement: Default Installation Scope
CLI MUST install to project-local scope (`./.agents/...`) when the `--global` flag is not specified. Only when the `--global` flag is specified should it install to global scope (`~/.agents/...`).

#### Scenario: Default is project-local installation
When running `my-command install-skill --yes` (without `--global`), the skill is placed in `./.agents/skills/<skill-name>` and the lock file is recorded in `./.agents/.skill-lock.json`.

#### Scenario: Global installation with --global flag
When running `my-command install-skill --global --yes`, the skill is placed in `~/.agents/skills/<skill-name>` and the lock file is recorded in `~/.agents/.skill-lock.json`.

### Requirement: Canonical Path and Installation Mode
`install` MUST treat the canonical path (`.agents/skills/<skill-name>`) as the single source of truth and support installation in either `symlink` or `copy` mode. When `symlink` fails, it MUST fallback to `copy`.

#### Scenario: Fallback on symlink failure
When `install` is executed in an environment where `symlink` is not permitted, the installation result includes `symlinkFailed = true` and the actual files are placed as with `copy`.

#### Scenario: Common canonical processing for embedded skills
When running `my-command install-skill --yes`, embedded skills are placed in `.agents/skills/<skill-name>` and subsequent agent-specific distribution follows the same symlink/copy rules as regular sources.

### Requirement: Lock File Update
`install` MUST record `source`, `sourceType`, `sourceUrl`, and `skillFolderHash` in the scope-appropriate lock file upon successful installation.
- project-local scope: `./.agents/.skill-lock.json`
- global scope: `~/.agents/.skill-lock.json`

#### Scenario: Lock update in project-local scope
When running `my-command install-skill --yes` (without `--global`), the lock file is recorded in `./.agents/.skill-lock.json`.

#### Scenario: Lock update in global scope
When running `my-command install-skill --global --yes`, the lock file is recorded in `~/.agents/.skill-lock.json`.

#### Scenario: Lock update for embedded skills
When installing embedded skills, the lock file records `sourceType` as `self` and `skillFolderHash` is stored as a deterministic value computed without using external APIs.

### Requirement: Multiple Agent Specification
The `--agent` option for `install-skill` MUST accept multiple platforms via comma-separated values or multiple occurrences, resolve installation targets while preserving order and removing duplicates.

#### Scenario: Comma-separated multiple platforms
When running `my-command install-skill --agent claude,opencode --yes`, the skill is distributed to both `.claude/skills` and `.config/opencode/skills` via symlink (falling back to copy on failure).

#### Scenario: Multiple `--agent` occurrences
When running `my-command install-skill --agent claude --agent opencode --yes`, the skill is distributed to both `.claude/skills` and `.config/opencode/skills`.

#### Scenario: Unknown agent name provided
When running `my-command install-skill --agent unknown --yes`, the command exits with an error for the unknown agent and does not start installation.

