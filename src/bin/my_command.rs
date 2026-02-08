use agent_skills_rs::*;
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "my-command")]
#[command(about = "A CLI tool with skill installation support", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
#[allow(clippy::enum_variant_names)]
enum Commands {
    /// List all available commands with JSON output
    Commands {
        #[arg(long, value_name = "FORMAT")]
        output: Option<String>,
    },
    /// Get JSON schema for a command
    Schema {
        #[arg(long, value_name = "COMMAND")]
        command: String,
        #[arg(long, value_name = "FORMAT")]
        output: Option<String>,
    },
    /// Install a skill
    InstallSkill {
        /// Target agent name(s) for agent-specific installation (can be comma-separated or specified multiple times)
        #[arg(long)]
        agent: Vec<String>,
        /// Specific skill name to install
        #[arg(long)]
        skill: Option<String>,
        /// Install globally (default: project-local)
        #[arg(long)]
        global: bool,
        /// Skip confirmation prompts
        #[arg(long)]
        yes: bool,
        /// Run in non-interactive mode
        #[arg(long)]
        non_interactive: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Commands { output } => {
            if output.as_deref() == Some("json") {
                let json = output_commands_json()?;
                println!("{}", json);
            } else {
                println!("Available commands:");
                println!("  commands --output json");
                println!("  schema --command <name> --output json-schema");
                println!("  install-skill [--global] [--yes] [--non-interactive]");
            }
        }
        Commands::Schema { command, output } => {
            if output.as_deref() == Some("json-schema") {
                let schema = get_command_schema(&command)?;
                println!("{}", schema);
            } else {
                println!("Use --output json-schema to get the schema");
            }
        }
        Commands::InstallSkill {
            agent,
            skill,
            global,
            yes,
            non_interactive,
        } => {
            install_skill_command(&agent, skill.as_deref(), global, yes || non_interactive)?;
        }
    }

    Ok(())
}

/// Parse agent names from CLI input, handling comma-separated values and deduplication
fn parse_agents(agents: &[String]) -> Result<Vec<String>> {
    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::new();

    for agent_str in agents {
        for agent in agent_str.split(',') {
            let trimmed = agent.trim();
            if !trimmed.is_empty() && seen.insert(trimmed.to_string()) {
                result.push(trimmed.to_string());
            }
        }
    }

    Ok(result)
}

/// Resolve agent names to target directories
/// For OpenCode, returns target dir only in global scope (since project scope uses .agents/skills as universal location)
fn resolve_target_dirs(
    agents: &[String],
    base_dir: &Path,
    is_global: bool,
) -> Result<Vec<PathBuf>> {
    let mut target_dirs = Vec::new();

    for agent in agents {
        match agent.as_str() {
            "claude" => {
                // Claude always uses agent-specific directory
                target_dirs.push(base_dir.join(".claude/skills"));
            }
            "opencode" => {
                if is_global {
                    // Global scope: use ~/.config/opencode/skills as target
                    target_dirs.push(base_dir.join(".config/opencode/skills"));
                }
                // Project scope: .agents/skills is universal, no additional target dir needed
            }
            _ => {
                anyhow::bail!("Unknown agent: '{}'. Known agents: claude, opencode", agent);
            }
        }
    }

    Ok(target_dirs)
}

fn install_skill_command(
    agents: &[String],
    skill_filter: Option<&str>,
    is_global: bool,
    auto_confirm: bool,
) -> Result<()> {
    let source = Source {
        source_type: SourceType::Self_,
        url: None,
        subpath: None,
        skill_filter: skill_filter.map(|s| s.to_string()),
        ref_: None,
    };

    // Setup paths
    let base_dir = if is_global {
        PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".to_string()))
    } else {
        std::env::current_dir()?
    };
    let canonical_dir = base_dir.join(".agents/skills");
    let lock_path = base_dir.join(".agents/.skill-lock.json");

    // Parse and normalize agent names
    let normalized_agents = parse_agents(agents)?;

    if is_global {
        println!("Discovering embedded skills (scope: global)");
    } else {
        println!("Discovering embedded skills (scope: project)");
    }

    // Discover skills
    let config = DiscoveryConfig::default();
    let mut skills = discover_skills(&source, &config)?;

    if skills.is_empty() {
        println!("No skills found.");
        return Ok(());
    }

    // Filter by skill name if specified
    if let Some(filter) = skill_filter {
        skills.retain(|s| s.name == filter);
        if skills.is_empty() {
            println!("No skill matching '{}' found.", filter);
            return Ok(());
        }
    }

    println!("Found {} skill(s):", skills.len());
    for skill in &skills {
        println!("  - {} ({})", skill.name, skill.description);
    }

    // Resolve target directories if agents specified
    let target_dirs = if !normalized_agents.is_empty() {
        resolve_target_dirs(&normalized_agents, &base_dir, is_global)?
    } else {
        Vec::new()
    };

    // Install each skill
    for skill in &skills {
        if !auto_confirm {
            println!("\nInstall skill '{}'? (y/n)", skill.name);
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if !input.trim().eq_ignore_ascii_case("y") {
                println!("Skipped.");
                continue;
            }
        }

        println!("Installing skill '{}'...", skill.name);

        // Install to canonical directory and link/copy to target directories
        let mut install_config = InstallConfig::new(canonical_dir.clone());
        install_config.target_dirs = target_dirs.clone();
        let result = install_skill(skill, &install_config)?;

        println!("  Installed to: {}", result.path.display());

        // Report target directories
        for target_dir in &target_dirs {
            let target_path = target_dir.join(&skill.name);
            println!("  Linked to: {}", target_path.display());
        }

        if result.symlink_failed {
            println!("  Note: Some symlinks failed, used copy fallback.");
        }

        let lock_manager = LockManager::new(lock_path.clone());
        lock_manager.update_entry(&skill.name, &source, &result.path)?;

        println!("  Lock file updated: {}", lock_path.display());
    }

    println!("\nInstallation complete!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_skills_rs::types::SkillMetadata;

    #[test]
    fn test_parse_agents_single() {
        let agents = vec!["claude".to_string()];
        let result = parse_agents(&agents).unwrap();
        assert_eq!(result, vec!["claude"]);
    }

    #[test]
    fn test_parse_agents_comma_separated() {
        let agents = vec!["claude,opencode".to_string()];
        let result = parse_agents(&agents).unwrap();
        assert_eq!(result, vec!["claude", "opencode"]);
    }

    #[test]
    fn test_parse_agents_multiple_args() {
        let agents = vec!["claude".to_string(), "opencode".to_string()];
        let result = parse_agents(&agents).unwrap();
        assert_eq!(result, vec!["claude", "opencode"]);
    }

    #[test]
    fn test_parse_agents_mixed() {
        let agents = vec!["claude".to_string(), "opencode,claude".to_string()];
        let result = parse_agents(&agents).unwrap();
        // Should deduplicate, keeping first occurrence
        assert_eq!(result, vec!["claude", "opencode"]);
    }

    #[test]
    fn test_parse_agents_with_whitespace() {
        let agents = vec!["claude , opencode".to_string()];
        let result = parse_agents(&agents).unwrap();
        assert_eq!(result, vec!["claude", "opencode"]);
    }

    #[test]
    fn test_parse_agents_empty() {
        let agents = vec![];
        let result = parse_agents(&agents).unwrap();
        assert_eq!(result, Vec::<String>::new());
    }

    #[test]
    fn test_parse_agents_empty_string() {
        let agents = vec!["".to_string()];
        let result = parse_agents(&agents).unwrap();
        assert_eq!(result, Vec::<String>::new());
    }

    #[test]
    fn test_resolve_target_dirs_claude() {
        let agents = vec!["claude".to_string()];
        let base_dir = PathBuf::from("/home/user");
        let result = resolve_target_dirs(&agents, &base_dir, false).unwrap();
        assert_eq!(result, vec![PathBuf::from("/home/user/.claude/skills")]);
    }

    #[test]
    fn test_resolve_target_dirs_opencode_project_scope() {
        let agents = vec!["opencode".to_string()];
        let base_dir = PathBuf::from("/home/user/project");
        let result = resolve_target_dirs(&agents, &base_dir, false).unwrap();
        // Project scope: no additional target dir (uses canonical .agents/skills)
        assert_eq!(result, Vec::<PathBuf>::new());
    }

    #[test]
    fn test_resolve_target_dirs_opencode_global_scope() {
        let agents = vec!["opencode".to_string()];
        let base_dir = PathBuf::from("/home/user");
        let result = resolve_target_dirs(&agents, &base_dir, true).unwrap();
        // Global scope: adds ~/.config/opencode/skills as target
        assert_eq!(
            result,
            vec![PathBuf::from("/home/user/.config/opencode/skills")]
        );
    }

    #[test]
    fn test_resolve_target_dirs_multiple_project_scope() {
        let agents = vec!["claude".to_string(), "opencode".to_string()];
        let base_dir = PathBuf::from("/home/user/project");
        let result = resolve_target_dirs(&agents, &base_dir, false).unwrap();
        // Project scope: only claude gets target dir
        assert_eq!(
            result,
            vec![PathBuf::from("/home/user/project/.claude/skills")]
        );
    }

    #[test]
    fn test_resolve_target_dirs_multiple_global_scope() {
        let agents = vec!["claude".to_string(), "opencode".to_string()];
        let base_dir = PathBuf::from("/home/user");
        let result = resolve_target_dirs(&agents, &base_dir, true).unwrap();
        // Global scope: both get target dirs
        assert_eq!(
            result,
            vec![
                PathBuf::from("/home/user/.claude/skills"),
                PathBuf::from("/home/user/.config/opencode/skills")
            ]
        );
    }

    #[test]
    fn test_resolve_target_dirs_unknown_agent() {
        let agents = vec!["unknown".to_string()];
        let base_dir = PathBuf::from("/home/user");
        let result = resolve_target_dirs(&agents, &base_dir, false);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unknown agent: 'unknown'"));
    }

    #[test]
    fn test_resolve_target_dirs_empty() {
        let agents = vec![];
        let base_dir = PathBuf::from("/home/user");
        let result = resolve_target_dirs(&agents, &base_dir, false).unwrap();
        assert_eq!(result, Vec::<PathBuf>::new());
    }

    #[test]
    #[cfg(unix)]
    fn test_agent_specific_installation_with_symlinks() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let base_dir = temp_dir.path();

        // Setup paths
        let canonical_dir = base_dir.join(".agents/skills");
        let claude_target = base_dir.join(".claude/skills");

        // Create a test skill
        let skill = Skill {
            name: "test-skill".to_string(),
            description: "Test skill".to_string(),
            path: None,
            raw_content: "---\nname: test-skill\ndescription: Test skill\n---\n\n# Test"
                .to_string(),
            metadata: SkillMetadata::default(),
        };

        // Install with target_dirs
        let mut install_config = InstallConfig::new(canonical_dir.clone());
        install_config.target_dirs = vec![claude_target.clone()];

        let result = install_skill(&skill, &install_config).unwrap();

        // Verify canonical installation
        assert_eq!(result.path, canonical_dir.join("test-skill"));
        assert!(result.path.exists());
        assert!(result.path.join("SKILL.md").exists());

        // Verify target symlink on Unix
        let target_path = claude_target.join("test-skill");
        assert!(target_path.exists());

        let metadata = std::fs::symlink_metadata(&target_path).unwrap();
        assert!(
            metadata.file_type().is_symlink(),
            "Target should be a symlink"
        );
        assert!(!result.symlink_failed, "Symlink should not have failed");

        // Verify symlink points to canonical
        let link_target = std::fs::read_link(&target_path).unwrap();
        assert_eq!(link_target, canonical_dir.join("test-skill"));
    }

    #[test]
    fn test_opencode_project_scope_no_target_dir() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let base_dir = temp_dir.path();

        // Setup paths
        let canonical_dir = base_dir.join(".agents/skills");

        // Resolve target dirs for opencode in project scope
        let agents = vec!["opencode".to_string()];
        let target_dirs = resolve_target_dirs(&agents, base_dir, false).unwrap();

        // Should be empty (no additional target dir needed)
        assert_eq!(target_dirs.len(), 0);

        // Create a test skill
        let skill = Skill {
            name: "test-skill".to_string(),
            description: "Test skill".to_string(),
            path: None,
            raw_content: "---\nname: test-skill\ndescription: Test skill\n---\n\n# Test"
                .to_string(),
            metadata: SkillMetadata::default(),
        };

        // Install with empty target_dirs
        let mut install_config = InstallConfig::new(canonical_dir.clone());
        install_config.target_dirs = target_dirs;

        let result = install_skill(&skill, &install_config).unwrap();

        // Verify only canonical installation exists
        assert_eq!(result.path, canonical_dir.join("test-skill"));
        assert!(result.path.exists());
        assert!(result.path.join("SKILL.md").exists());
        assert!(!result.symlink_failed);
    }

    #[test]
    fn test_opencode_global_scope_has_target_dir() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let base_dir = temp_dir.path();

        // Setup paths
        let canonical_dir = base_dir.join(".agents/skills");

        // Resolve target dirs for opencode in global scope
        let agents = vec!["opencode".to_string()];
        let target_dirs = resolve_target_dirs(&agents, base_dir, true).unwrap();

        // Should have one target dir
        assert_eq!(target_dirs.len(), 1);
        assert_eq!(target_dirs[0], base_dir.join(".config/opencode/skills"));

        // Create a test skill
        let skill = Skill {
            name: "test-skill".to_string(),
            description: "Test skill".to_string(),
            path: None,
            raw_content: "---\nname: test-skill\ndescription: Test skill\n---\n\n# Test"
                .to_string(),
            metadata: SkillMetadata::default(),
        };

        // Install with target_dirs
        let mut install_config = InstallConfig::new(canonical_dir.clone());
        install_config.target_dirs = target_dirs.clone();

        let result = install_skill(&skill, &install_config).unwrap();

        // Verify canonical installation
        assert_eq!(result.path, canonical_dir.join("test-skill"));
        assert!(result.path.exists());

        // Verify target installation
        let target_path = target_dirs[0].join("test-skill");
        assert!(target_path.exists());
        assert!(target_path.join("SKILL.md").exists());
    }
}
