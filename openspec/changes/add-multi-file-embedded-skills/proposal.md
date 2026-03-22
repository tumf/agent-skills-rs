# Change: Add multi-file embedded skill support

## Problem/Context

- The current `embedded.rs` module only supports embedding a single `SKILL.md` file per skill via `include_str!`.
- Downstream consumers (e.g. `cflx`) bundle skills that contain multiple files: `SKILL.md`, `scripts/cflx.py`, `references/*.md`, and `.skill` binary manifests.
- When these consumers use `SourceType::Self_` to discover and install embedded skills, only `SKILL.md` content is written to disk — auxiliary files (scripts, references) are lost.
- The `install_skill` function in `installer.rs` writes only `skill.raw_content` as `SKILL.md` for embedded/local skills without a provider, so there is no mechanism to materialize additional files.

## Proposed Solution

### 1. Extend the `Skill` type with auxiliary file storage

Add an `auxiliary_files` field to `Skill` (in `types.rs`):

```rust
pub struct Skill {
    pub name: String,
    pub description: String,
    pub path: Option<String>,
    pub raw_content: String,       // SKILL.md content
    pub metadata: SkillMetadata,
    /// Additional files bundled with the skill, keyed by relative path.
    /// Example: {"scripts/cflx.py": "<file content>", "references/apply.md": "..."}
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub auxiliary_files: HashMap<String, String>,
}
```

### 2. Support `include_str!` registration for multiple files

Instead of hardcoding a single `const`, provide a registration pattern in `embedded.rs`:

```rust
pub fn register_embedded_skill(
    skill_md_content: &'static str,
    auxiliary: &[(&str, &'static str)],  // (relative_path, content)
) -> Result<Skill> { ... }
```

Downstream consumers call this at compile time or in a `get_embedded_skills()` override to register their skills with all files.

Alternatively, provide a macro:

```rust
embedded_skill! {
    skill_md: include_str!("../skills/cflx-workflow/SKILL.md"),
    files: {
        "scripts/cflx.py" => include_str!("../skills/cflx-workflow/scripts/cflx.py"),
        "references/cflx-apply.md" => include_str!("../skills/cflx-workflow/references/cflx-apply.md"),
    }
}
```

### 3. Update installer to write auxiliary files

In `installer.rs`, when installing a skill without a provider (embedded path), write `auxiliary_files` to disk alongside `SKILL.md`:

```rust
// After writing SKILL.md
for (rel_path, content) in &skill.auxiliary_files {
    let file_path = canonical_path.join(rel_path);
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&file_path, content)?;
}
```

### 4. Expose `get_embedded_skills()` as the public API for consumers

Keep `get_embedded_skills()` as the entry point. The default implementation returns the library's own skill. Consumers override or extend via the registration API.

## Acceptance Criteria

- `Skill` struct has an `auxiliary_files: HashMap<String, String>` field.
- `embedded.rs` provides `register_embedded_skill()` (or equivalent macro) that accepts SKILL.md content plus a list of `(relative_path, content)` pairs.
- `install_skill()` writes all auxiliary files to disk under the correct relative paths within the skill directory.
- Existing single-file embedded skill behavior is preserved (backward compatible).
- `discover_skills` with `SourceType::Self_` returns skills with populated `auxiliary_files`.
- All existing tests pass; new tests cover multi-file install and round-trip verification.

## Out of Scope

- Binary file embedding (`include_bytes!`) — text files only for now.
- Build script (`build.rs`) auto-discovery of skill files (consumers manage their own `include_str!` calls).
- Changes to lock file format or provider interface.
