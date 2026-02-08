---
name: agent-skills-rust
description: Guide for integrating the agent-skills-rs Rust library and optional CLI workflow into a project. Use when adding embedded skill discovery/installation, lock-file updates, agent-specific install paths, or command introspection support in Rust codebases.
---

# Integrate agent-skills-rs

Use this skill to implement or explain how to add `agent-skills-rs` into a Rust project.

## Recommended implementation order

1. Rename command surface to `install-skills` everywhere (routing, help, usage, introspection, schema, tests).
2. Add `agent-skills-rs = "<current release>"` and verify compile.
3. Implement argument parsing for `install-skills <source> [--global]`.
4. Implement install destination and lock-path resolution for project and global mode.
5. Restrict source schemes to `self` and `local:<path>`, fail fast on unknown schemes.
6. Stabilize JSON output shape and wire integration tests through full CLI paths.
7. Run format, lint, and tests.

## Add dependency

Add to `Cargo.toml`:

```toml
[dependencies]
agent-skills-rs = "<current release>"
```

If local development is required, use a path dependency instead:

```toml
[dependencies]
agent-skills-rs = { path = "../agent-skills-rust" }
```

## Implement the core install flow

Use the library API in this order:

1. Build a `Source` (embedded source uses `SourceType::Self_`).
2. Run discovery with `discover_skills`.
3. Install each skill with `install_skill` and `InstallConfig`.
4. Update lock file via `LockManager` using the same scope base as install destination.

Use this baseline snippet:

```rust
use agent_skills_rs::{
    discover_skills, install_skill, DiscoveryConfig, InstallConfig, LockManager, Source, SourceType,
};
use anyhow::Result;
use std::path::PathBuf;

pub fn install_skills(base_dir: PathBuf, global: bool) -> Result<()> {
    let home_dir = std::env::var("HOME").map(PathBuf::from)?;
    let (canonical_dir, lock_path) = if global {
        (
            home_dir.join(".agents/skills"),
            home_dir.join(".agents/.skill-lock.json"),
        )
    } else {
        (
            base_dir.join(".agents/skills"),
            base_dir.join(".agents/.skill-lock.json"),
        )
    };

    let source = Source {
        source_type: SourceType::Self_,
        url: None,
        subpath: None,
        skill_filter: None,
        ref_: None,
    };

    let config = DiscoveryConfig::default();
    let skills = discover_skills(&source, &config)?;

    let install_config = InstallConfig::new(canonical_dir);
    let lock_manager = LockManager::new(lock_path);

    for skill in &skills {
        let result = install_skill(skill, &install_config)?;
        lock_manager.update_entry(&skill.name, &source, &result.path)?;
    }

    Ok(())
}
```

## Source parsing and validation

Support only these source forms:

- `self` (embedded skills)
- `local:<path>`

Reject any unknown scheme immediately with an explicit message listing allowed schemes.

## Install and lock path policy

Define this early and keep it consistent across implementation and tests.

- Default install dir: `./.agents/skills`
- Global install dir (`--global`): `~/.agents/skills`
- Default lock path: `./.agents/.skill-lock.json`
- Global lock path (`--global`): `~/.agents/.skill-lock.json`

Use the same scope choice for both install destination and lock file. Do not mix project install with global lock (or vice versa).

## Add optional agent-specific install targets

Use canonical install path as the source of truth:

- Project scope: `<project>/.agents/skills/<skill-name>`
- Global scope: `~/.agents/skills/<skill-name>`

If the target agent needs mirrored locations, set `InstallConfig::target_dirs`.

Common target directories:

- Claude: `.claude/skills`
- OpenCode (global scope): `.config/opencode/skills`

Prefer symlink behavior with copy fallback (default behavior of installer).

## Add optional CLI introspection support

Expose these APIs when building a CLI:

- `output_commands_json()` for `commands --output json`
- `get_command_schema("install-skills")` for `schema --command install-skills --output json-schema`

Keep the install command script-friendly:

- Support `--yes` and `--non-interactive`
- Avoid mandatory interactive prompts
- Print deterministic output paths
- Keep JSON output stable, for example:
  - `schemaVersion`
  - `type`
  - `ok`
  - `skills[]`

When renaming commands, update all of these in one pass to avoid drift:

- `src/main.rs`
- `src/cli/introspection.rs`
- integration tests that check `commands` and `schema` flows

## Integration test checklist

Use integration tests to validate real CLI behavior end-to-end, not only library helpers.

- Routing works for `install-skills`
- Help/usage text uses `install-skills`
- `commands --output json` includes `install-skills`
- `schema --command install-skills --output json-schema` works
- Project mode writes under current project path (set `current_dir` explicitly in tests)
- Global mode writes under home path
- JSON output fields match schema exactly

## Validate implementation

Run these checks in order:

```bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test
```

## Troubleshooting

- If no skills are discovered, confirm embedded skill data exists in the build.
- If links fail on the host OS, rely on copy fallback and still update lock entries.
- If command schema is missing fields, verify CLI argument definitions and introspection wiring.
- If existing lock files fail to parse, support compatibility reads for both legacy map-style and newer array-style lock formats.
- If replacing an existing symlink target fails with `remove_dir_all`, retry with `remove_file` fallback.
- If project/global mode behaves unexpectedly in tests, ensure `current_dir` is explicitly set for project-scoped assertions.
