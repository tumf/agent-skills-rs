# Skill Installer

A Rust library for installing and managing agent skills with embedded skill support.

## Overview

This library provides a Rust implementation of skill installation functionality similar to Vercel Labs' `skills` CLI. It supports:

- Skill discovery from multiple sources (GitHub, GitLab, local, direct, and embedded)
- Installation with symlink or copy modes
- Canonical path management
- Lock file management for tracking installed skills
- Embedded skills bundled at compile time
- CLI introspection support (Agentic CLI Design Principle 7)

## Features

### Core Functionality

- **Skill Discovery**: Find and parse SKILL.md files from various sources
- **Installation Modes**: 
  - Symlink mode with automatic fallback to copy
  - Copy mode for direct file copying
- **Canonical Path**: Single source of truth at `.agents/skills/<skill-name>`
- **Lock Management**: Track installed skills with deterministic hashing
- **Embedded Skills**: Bundle skills into the binary at compile time using `include_str!`

### Embedded Skills

The CLI installs skill(s) embedded in the binary by default.

## Architecture

### Modules

- `types`: Core data structures (Skill, Source, LockEntry, etc.)
- `embedded`: Compile-time embedded skill definitions
- `discovery`: Skill discovery and parsing logic
- `installer`: Installation with symlink/copy support
- `lock`: Lock file management
- `cli`: CLI command definitions and introspection

## Usage

### Basic Installation Flow

```rust
use skill_installer::*;
use tempfile::TempDir;

// Setup
let temp_dir = TempDir::new().unwrap();
let canonical_dir = temp_dir.path().join(".agents/skills");
let lock_path = temp_dir.path().join(".agents/.skill-lock.json");

// Create source (embedded skill)
let source = Source {
    source_type: SourceType::Self_,
    url: None,
    subpath: None,
    skill_filter: None,
    ref_: None,
};

// Discover skills
let config = DiscoveryConfig::default();
let skills = discover_skills(&source, &config).unwrap();

// Install skill
let install_config = InstallConfig::new(canonical_dir.clone());
let installed_path = install_skill(&skills[0], &install_config).unwrap();

// Update lock
let lock_manager = LockManager::new(lock_path);
lock_manager.update_entry(&skills[0].name, &source, &installed_path).unwrap();
```

### CLI Introspection

```rust
use skill_installer::*;

// Get all commands as JSON
let commands_json = output_commands_json().unwrap();
println!("{}", commands_json);

// Get schema for a specific command
let schema = get_command_schema("install-skill").unwrap();
println!("{}", schema);
```

## Testing

The library follows a **mock-first** approach for testing:

- All tests run without external network access
- External dependencies (Git, HTTP, Tree API) are abstracted via traits
- Integration tests use temporary directories
- Embedded skill tests verify compile-time bundling

Run tests:

```bash
cargo test
```

Run linting:

```bash
cargo clippy
```

Format code:

```bash
cargo fmt
```

## Design Principles

This library implements **Agentic CLI Design Principle 7: Introspectable**:

- Self-describing commands via `commands --output json`
- JSON Schema output via `schema --command <name> --output json-schema`
- Machine-readable output formats
- Embedded skills enable offline installation

## Data Model

### Skill

```rust
pub struct Skill {
    pub name: String,
    pub description: String,
    pub path: Option<String>,
    pub raw_content: String,
    pub metadata: SkillMetadata,
}
```

### Source

```rust
pub struct Source {
    pub source_type: SourceType,
    pub url: Option<String>,
    pub subpath: Option<String>,
    pub skill_filter: Option<String>,
    pub ref_: Option<String>,
}

pub enum SourceType {
    Github,
    Gitlab,
    Local,
    Direct,
    #[serde(rename = "self", alias = "embedded")]
    Self_,
}
```

### LockEntry

```rust
pub struct LockEntry {
    pub source: String,
    pub source_type: String,
    pub source_url: Option<String>,
    pub skill_path: String,
    pub skill_folder_hash: String,
    pub installed_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

## Future Work

- GitHub/GitLab real API integration with authentication
- Additional source types
- UI/TUI integration
- Advanced skill filtering and search

## License

This project is part of the agent skills ecosystem.
