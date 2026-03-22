## MODIFIED Requirements

### Requirement: Embedded Skill Installation

CLI MUST use the `install-skill` command to automatically resolve `SKILL.md` and other content embedded at compile time as the source. No external source specification is required.

Skills MAY include auxiliary files (scripts, reference documents) alongside `SKILL.md`. When auxiliary files are registered via `register_embedded_skill()`, the installer SHALL write all auxiliary files to disk under the correct relative paths within the skill directory.

#### Scenario: Automatic resolution of embedded skills

When running `my-command install-skill --yes`, the implementation resolves skills from embedded content and proceeds to installation without calling external providers (clone/fetch).

#### Scenario: Multi-file embedded skill installation

- **GIVEN** a skill is registered via `register_embedded_skill()` with `SKILL.md` content and auxiliary files `{"scripts/helper.py": "<content>", "references/guide.md": "<content>"}`
- **WHEN** the skill is installed via `install_skill()`
- **THEN** the canonical directory contains `SKILL.md`, `scripts/helper.py`, and `references/guide.md`
- **AND** each file's content matches the registered content exactly

#### Scenario: Backward-compatible single-file embedded skill

- **GIVEN** a skill is registered with only `SKILL.md` content and no auxiliary files
- **WHEN** the skill is installed via `install_skill()`
- **THEN** only `SKILL.md` is written to the canonical directory
- **AND** existing behavior is preserved identically
