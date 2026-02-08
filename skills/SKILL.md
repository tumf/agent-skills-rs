---
name: skill-installer
description: Rust library for agent skill installation with embedded skill support
---

# Skill Installer

This is an embedded skill that provides Rust library functionality for installing and managing agent skills.

## Features

- Skill discovery and installation
- Support for embedded skills (compile-time bundled)
- Lock file management
- Symlink and copy modes
- CLI introspection support

## Usage

Install this skill using:

```bash
my-command install-skill --yes
```

## Implementation

This skill is bundled into the binary at compile time using `include_str!` and can be installed without external network access.
