---
change_id: support-comma-separated-agent
title: Enable comma-separated multiple agent specification
status: draft
created_at: 2026-02-08
---

# Change Proposal

## Why
The `--agent` argument is defined in the CLI but currently assumes a single value and is not connected to actual processing logic. Users expect to be able to distribute to multiple platforms using comma-separated values like `--agent claude,opencode`.

## What Changes
- Allow comma-separated values for `--agent` to enable simultaneous distribution to multiple platforms
- Also permit multiple occurrences of `--agent` (e.g., `--agent claude --agent opencode`)
- Preserve order of parsed agents while removing duplicates
- Reject unknown agent names with an error before starting installation
- Update introspection schema description to reflect the new comma-separated and repeatable behavior
- Maintain existing single-agent behavior (canonical placement, symlink with copy fallback)

## Non-Goals
- Adding new platforms or extending discovery logic
- Adding external providers or network-based validation

## Risks and Mitigations
- Existing CLI schema descriptions may diverge from behavior: update introspection descriptions accordingly
- Behavior when invalid agent names are mixed in: explicitly return an error for unknown agents

## Scope of Impact
- CLI argument parsing
- Installation target resolution
- Introspection description text
