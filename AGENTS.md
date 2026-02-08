# AGENTS.md

This file guides agentic coding tools working in this repository.
It focuses on reliable command usage and code conventions inferred from the current Rust codebase.

## Scope

- Project type: Rust library + CLI binary (`skill_installer`, `my-command`)
- Main crate root: `src/lib.rs`
- Binary entrypoint: `src/bin/my_command.rs`
- Core modules: `types`, `discovery`, `installer`, `lock`, `providers`, `embedded`, `cli`

## Environment Assumptions

- Rust edition: `2021`
- Package manager/build tool: `cargo`
- Default shell examples assume Unix-like environments
- Many tests use temporary directories (`tempfile::TempDir`)

## Build, Lint, and Test Commands

### Install dependencies

- `cargo fetch`

### Build

- Debug build: `cargo build`
- Release build: `cargo build --release`
- Build all targets (library, binary, tests, benches/examples if added): `cargo build --all-targets`

### Run CLI

- Run binary via Cargo: `cargo run --bin my-command -- --help`
- Introspection commands:
  - `cargo run --bin my-command -- commands --output json`
  - `cargo run --bin my-command -- schema --command install-skill --output json-schema`

### Format

- Check formatting only: `cargo fmt --all -- --check`
- Apply formatting: `cargo fmt --all`

### Lint

- Clippy (default): `cargo clippy --all-targets --all-features -- -D warnings`
- If `--all-features` fails due to no features, use: `cargo clippy --all-targets -- -D warnings`

### Test

- Run all tests: `cargo test`
- Run all library tests only: `cargo test --lib`
- Run tests with output: `cargo test -- --nocapture`

### Single-test workflows (important)

- By exact test name substring:
  - `cargo test test_end_to_end_embedded_install`
- By fully-qualified path (preferred when names collide):
  - `cargo test discovery::tests::test_parse_frontmatter`
  - `cargo test lock::tests::test_compute_skill_hash`
  - `cargo test integration_tests::test_github_flow_with_mock_provider`
- Single test with captured output disabled:
  - `cargo test test_github_cli_integration -- --nocapture`
- Re-run failed tests quickly:
  - `cargo test -- --ignored` (only if ignored tests are introduced)

### Docs

- Build docs: `cargo doc --no-deps`
- Build docs and open locally: `cargo doc --no-deps --open`

## Recommended Agent Execution Order

Use this order for implementation tasks unless the user asks otherwise:

1. `cargo fmt --all`
2. `cargo clippy --all-targets -- -D warnings`
3. `cargo test`

For quick iteration during coding:

1. Run the most relevant single test
2. Run module-level tests (`cargo test <module>::tests`)
3. Run full `cargo test` before finishing

## Code Style Guidelines

These conventions are based on current repository patterns and should be preserved.

### Imports

- Keep imports explicit and grouped logically.
- Prefer this grouping when editing existing modules:
  1. `crate::...`
  2. external crates (`anyhow`, `serde`, etc.)
  3. `std::...`
- Keep import lists minimal; remove unused imports immediately.
- Let `rustfmt` normalize ordering/formatting.

### Formatting

- Always use `cargo fmt`; do not hand-format against rustfmt output.
- Keep one field/argument per line when structures become long.
- Use trailing commas in multiline literals and call arguments.
- Keep functions focused and reasonably short.

### Types and Data Modeling

- Prefer strong types and enums over stringly-typed logic.
- Derive traits explicitly as needed (`Debug`, `Clone`, `Serialize`, `Deserialize`, `PartialEq`, `Eq`, `Default`).
- Use serde attributes intentionally:
  - `rename` for stable external field names
  - `rename_all` for enum serialization formats
  - `skip_serializing_if` for optional fields
  - `default` for backward-compatible parsing
- Preserve lock file JSON compatibility (camelCase fields in serialized output).

### Naming Conventions

- Types/traits/enums: `PascalCase`
- Functions/modules/variables: `snake_case`
- Constants: `SCREAMING_SNAKE_CASE`
- Use descriptive names over abbreviations.
- Keep compatibility-driven names when needed (e.g., `Self_` enum variant for serde aliasing).

### Error Handling

- Library APIs should return `anyhow::Result<T>` for fallible operations.
- Add context to I/O and parsing failures via `anyhow::Context`.
- Use `anyhow::bail!` for explicit validation failures.
- Do not silently swallow errors in core flows.
- In tests, `unwrap()` is acceptable for setup/assert-focused code.

### I/O and Filesystem Patterns

- Use `std::fs` with contextual errors for reads/writes/dir operations.
- Ensure parent directories exist before writing files.
- Prefer deterministic behavior for lock file updates and hashing.
- Avoid destructive filesystem operations outside intended target paths.

### Testing Conventions

- Follow the repository's mock-first approach:
  - No external network access in tests
  - Use `MockProvider` for provider-driven flows
- Use `tempfile::TempDir` for isolated filesystem tests.
- Test both success and failure paths (missing frontmatter fields, invalid inputs, etc.).
- When adding features, include at least one focused unit test and one integration-style flow test when applicable.

### CLI and UX Behavior

- Keep CLI output predictable and script-friendly where possible.
- Prefer machine-readable JSON for introspection commands.
- Maintain compatibility for `self` and `embedded` source keywords.
- For interactive flows, support non-interactive bypass flags (`--yes`, `--non-interactive`).

### Documentation and Comments

- Repository docs and code comments should be in English.
- Use doc comments (`///`) on public structs/functions.
- Keep comments concise and focused on intent, not obvious mechanics.

## Cursor/Copilot Rules Check

No repository-local Cursor/Copilot rule files were found at the time of writing:

- `.cursor/rules/` (not present)
- `.cursorrules` (not present)
- `.github/copilot-instructions.md` (not present)

If any of these files are added later, update this AGENTS.md and treat them as higher-priority behavioral constraints for coding agents.

## Agent Safety Notes

- Do not introduce network-dependent tests.
- Do not hardcode secrets or user-specific absolute paths.
- Keep behavior backward-compatible for existing lock file schema and CLI introspection outputs.
