# Usage Guide

## Quick Start

### Building the Project

```bash
cargo build --release
```

### Running the CLI

The `my-command` binary demonstrates the library functionality:

#### Install Embedded Skill

Install the skill bundled in the binary (no network access required):

```bash
my-command install-skill self --yes
```

Or use the `embedded` alias:

```bash
my-command install-skill embedded --yes
```

Both commands produce identical results.

#### List Available Commands

```bash
my-command commands --output json
```

Output:
```json
{
  "schemaVersion": "1.0",
  "type": "commands",
  "ok": true,
  "commands": [...]
}
```

#### Get Command Schema

```bash
my-command schema --command install-skill --output json-schema
```

Output includes JSON Schema with `self` and `embedded` as valid enum values for the `source` parameter.

## Library Usage

### Basic Installation Flow

```rust
use skill_installer::*;
use std::path::PathBuf;

// Setup paths
let canonical_dir = PathBuf::from(".agents/skills");
let lock_path = PathBuf::from(".agents/.skill-lock.json");

// Create embedded source
let source = Source {
    source_type: SourceType::Self_,
    url: None,
    subpath: None,
    skill_filter: None,
    ref_: None,
};

// Discover skills
let config = DiscoveryConfig::default();
let skills = discover_skills(&source, &config)?;

// Install each skill
for skill in &skills {
    let install_config = InstallConfig::new(canonical_dir.clone());
    let installed_path = install_skill(skill, &install_config)?;
    
    // Update lock file
    let lock_manager = LockManager::new(lock_path.clone());
    lock_manager.update_entry(&skill.name, &source, &installed_path)?;
}
```

### Introspection

```rust
use skill_installer::*;

// Get all commands
let commands = get_commands();

// Output as JSON
let json = output_commands_json()?;
println!("{}", json);

// Get schema for specific command
let schema = get_command_schema("install-skill")?;
println!("{}", schema);
```

### Custom Embedded Skills

To add your own embedded skills:

1. Create a SKILL.md file in the `skills/` directory
2. Add frontmatter with `name` and `description`
3. The file will be bundled at compile time via `include_str!`

Example `skills/SKILL.md`:

```markdown
---
name: my-skill
description: My custom skill
---

# My Custom Skill

Skill content here...
```

## Testing

Run all tests:

```bash
cargo test
```

Run specific test:

```bash
cargo test test_end_to_end_embedded_install
```

Run with verbose output:

```bash
cargo test -- --nocapture
```

## Verification

After installing a skill with `my-command install-skill self --yes`:

1. Check installed skill:
   ```bash
   ls -la ~/.agents/skills/skill-installer/
   cat ~/.agents/skills/skill-installer/SKILL.md
   ```

2. Check lock file:
   ```bash
   cat ~/.agents/.skill-lock.json
   ```

The lock file should contain:
- `version`: "1.0"
- `source_type`: "self_"
- `skill_folder_hash`: SHA-256 hash of the skill content
- `installed_at` and `updated_at` timestamps

## Integration

To integrate this library into your own CLI:

1. Add dependency to `Cargo.toml`:
   ```toml
   [dependencies]
   skill_installer = { path = "path/to/skill_installer" }
   ```

2. Import and use:
   ```rust
   use skill_installer::*;
   ```

3. Follow the basic installation flow shown above

## Troubleshooting

### Lock File Version Mismatch

If you encounter a lock file version error, the existing lock file uses a different format. Options:

1. Use a fresh directory for testing
2. Remove the existing lock file (it will be recreated)
3. Update the library to support backward compatibility (future work)

### Symlink Failures

The library automatically falls back to copy mode if symlink creation fails. This is expected on systems with restricted symlink permissions.
