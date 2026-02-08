# cli-introspection Change Specification

## ADDED Requirements

### Requirement: Agent Description in install-skill Schema
The `agent` argument description in `schema --command install-skill --output json-schema` MUST explicitly indicate that comma-separated values and multiple occurrences are allowed.

#### Scenario: Updated agent description
When running `my-command schema --command install-skill --output json-schema`, the `agent` description includes text indicating "comma-separated" and "repeatable" behavior.
