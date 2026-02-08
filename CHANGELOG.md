# Changelog

All notable changes to this project will be documented in this file.

## [0.1.0] - 2026-02-08

### Added

- Core type system for skills, sources, and lock entries
- Embedded skill support with compile-time bundling (`include_str!`)
- `self` and `embedded` source type aliases (both deserialize to the same internal type)
- Skill discovery module with embedded skill support
- Installation module with symlink and copy modes
- Canonical path management at `.agents/skills/<skill-name>`
- Lock file management with SHA-256 hashing
- CLI introspection support (Agentic CLI Design Principle 7)
  - `commands --output json` for listing all commands
  - `schema --command <name> --output json-schema` for command schemas
- `my-command` binary demonstrating library usage
  - `my-command install-skill self --yes` for installing embedded skills
  - `my-command install-skill embedded --yes` (alias for `self`)
- Comprehensive test suite (32 tests, all passing)
  - Unit tests for all modules
  - Integration tests for end-to-end flows
  - Mock-first approach (no external network access required)

### Technical Details

- **Language**: Rust 2021 edition
- **Dependencies**: 
  - serde/serde_json for serialization
  - anyhow/thiserror for error handling
  - sha2 for hashing
  - chrono for timestamps
  - clap for CLI parsing
- **Test Coverage**: All core functionality covered
- **Code Quality**: Passes `cargo clippy` and `cargo fmt`

### Documentation

- Comprehensive README with usage examples
- Inline documentation for all public APIs
- Design documentation in `openspec/changes/add-rust-skill-installer/design.md`
- Task completion tracking in `openspec/changes/add-rust-skill-installer/tasks.md`

### Future Work

Deferred to future releases (see tasks.md):
- GitHub/GitLab real API integration with authentication
- Rate limiting and token management
- CI environment testing
