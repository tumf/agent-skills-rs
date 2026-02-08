---
name: agent-skills-rust
description: Guide for integrating the agent-skills-rs Rust library and optional CLI workflow into a project. Use when adding embedded skill discovery/installation, lock-file updates, agent-specific install paths, or command introspection support in Rust codebases.
---

# Integrate agent-skills-rs

Use this skill to implement or explain how to add `agent-skills-rs` into a Rust project.

## Follow this workflow

1. Add dependency and verify compile.
2. Implement library-based installation flow.
3. Optionally expose a CLI command for installation.
4. Verify with formatting, linting, and tests.

## Add dependency

Add to `Cargo.toml`:

```toml
[dependencies]
agent-skills-rs = "0.1"
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
4. Update `.agents/.skill-lock.json` via `LockManager`.

Use this baseline snippet:

```rust
use agent_skills_rs::{
    discover_skills, install_skill, DiscoveryConfig, InstallConfig, LockManager, Source, SourceType,
};
use anyhow::Result;
use std::path::PathBuf;

pub fn install_embedded_skills(base_dir: PathBuf) -> Result<()> {
    let canonical_dir = base_dir.join(".agents/skills");
    let lock_path = base_dir.join(".agents/.skill-lock.json");

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
- `get_command_schema("install-skill")` for `schema --command install-skill --output json-schema`

Keep the install command script-friendly:

- Support `--yes` and `--non-interactive`
- Avoid mandatory interactive prompts
- Print deterministic output paths

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
