## Implementation Tasks

- [x] 1.1 Add `auxiliary_files: HashMap<String, String>` field to `Skill` struct in `src/types.rs` with `#[serde(default, skip_serializing_if = "HashMap::is_empty")]` (verification: `cargo test test_skill_serialization` passes; existing deserialization without the field works due to `default`)
- [x] 1.2 Add `register_embedded_skill(skill_md: &str, auxiliary: &[(&str, &str)]) -> Result<Skill>` function in `src/embedded.rs` that parses frontmatter from `skill_md` and populates `auxiliary_files` from the slice (verification: new unit test `test_register_embedded_skill_with_aux_files` in `src/embedded.rs`)
- [x] 1.3 Update `get_embedded_skills()` in `src/embedded.rs` to use `register_embedded_skill` for the existing `agent-skills-rs` skill (verification: `cargo test test_embedded_skill_content` still passes)
- [x] 1.4 Export `register_embedded_skill` from `src/lib.rs` (verification: `cargo build` succeeds; `use agent_skills_rs::register_embedded_skill;` compiles)
- [x] 1.5 Update `install_skill_with_provider` in `src/installer.rs` to write `skill.auxiliary_files` entries to disk alongside SKILL.md when no provider is given (verification: new test `test_install_skill_with_auxiliary_files` creates a skill with aux files and asserts all files exist on disk)
- [x] 1.6 Add integration test in `src/lib.rs` `integration_tests` module: end-to-end embedded install with auxiliary files, verifying all files exist and content matches (verification: `cargo test test_end_to_end_embedded_install_with_aux_files`)
- [x] 1.7 Ensure backward compatibility: existing tests pass without modification (verification: `cargo test` — all existing tests green)
- [x] 1.8 Update README.md usage examples to document multi-file embedded skill registration (verification: README contains `register_embedded_skill` example)

## Future Work

- Provide a declarative `embedded_skill!` macro for ergonomic multi-file registration
- Support `include_bytes!` for binary assets (images, compiled scripts)
- Auto-discover skill files via `build.rs` to reduce manual `include_str!` boilerplate

## Acceptance #1 Failure Follow-up

- [x] Validate `auxiliary_files` paths before writing (reject absolute paths and `..` traversal outside skill root)
- [x] Add installer tests that assert malicious auxiliary paths are rejected and no files are written outside the canonical skill directory

## Acceptance #2 Failure Follow-up

- [x] Harden auxiliary path validation to reject platform-rooted/prefixed paths (e.g., `Component::RootDir` and `Component::Prefix`) in addition to `is_absolute` and `ParentDir`
- [x] Add platform-aware tests for rooted/prefixed auxiliary paths (Windows) to ensure writes cannot escape the canonical skill directory
