# cli-introspection Specification

## Purpose
Provides machine-readable introspection capabilities for CLI commands, enabling agents and automation tools to discover available commands and their schemas programmatically.

## Requirements
### Requirement: JSON Output for Command List
CLI MUST provide `commands --output json` and return JSON with `schemaVersion`, `type`, and `ok` at the top level.

#### Scenario: Retrieving command list
When running `my-command commands --output json`, JSON containing `ok: true` and `type: commands.list` is output to stdout.

### Requirement: JSON Schema Output for Command Schema
CLI MUST provide `schema --command install-skill --output json-schema` and return the JSON Schema for arguments and options.

#### Scenario: Retrieving install-skill schema
When running `my-command schema --command install-skill --output json-schema`, a JSON Schema defining the arguments for `install-skill` (`--agent`, `--skill`, `--yes`, `--global`, etc.) is output to stdout. The default value for `--global` is `false` (project-local).
