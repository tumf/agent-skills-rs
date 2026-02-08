# Change Proposal: Symbolic Link Operation and OpenCode Path Fix for install-skill

## Why
The current `install-skill` implementation performs reinstallation to each agent-specific directory after installing to canonical, creating copies instead of symbolic links. Additionally, OpenCode paths are not correctly handled according to scope (project/global).

## What Changes
- Treat canonical (`.agents/skills/<skill>` or `~/.agents/skills/<skill>`) as the single source of truth and create symbolic links to agent-specific directories.
- Correctly resolve OpenCode installation destinations according to scope.

### Scope
- `install-skill` command installation flow
- Agent-specific target directory resolution

### Implementation Approach
- Install to canonical only once and pass agent-specific directories to `InstallConfig.target_dirs` to create links.
- For OpenCode in project scope, `.agents/skills` is universal, so no additional target dir is created.
- For OpenCode in global scope, `~/.config/opencode/skills` is treated as the target dir.

### Impact and Risks
- Directories already installed using the copy method may be overwritten, so deletion/replacement behavior must be clarified in advance.
- When symbolic link creation fails, fallback to copy as per existing specification.

### Non-Goals
- Adding new agents
- Changing existing lock file format
