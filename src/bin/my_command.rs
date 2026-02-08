use anyhow::Result;
use clap::{Parser, Subcommand};
use skill_installer::*;
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
fn resolve_target_dirs(agents: &[String], base_dir: &Path) -> Result<Vec<PathBuf>> {
    let known_agents = [
        ("claude", ".claude/skills"),
        ("opencode", ".config/opencode/skills"),
    ];

    let mut target_dirs = Vec::new();

    for agent in agents {
        let found = known_agents.iter().find(|(name, _)| *name == agent);
        if let Some((_, path)) = found {
            target_dirs.push(base_dir.join(path));
        } else {
            anyhow::bail!("Unknown agent: '{}'. Known agents: claude, opencode", agent);
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
        resolve_target_dirs(&normalized_agents, &base_dir)?
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

        // Install to canonical directory
        let install_config = InstallConfig::new(canonical_dir.clone());
        let result = install_skill(skill, &install_config)?;

        println!("  Installed to: {}", result.path.display());
        if result.symlink_failed {
            println!("  Note: Symlink failed, used copy instead.");
        }

        // Install to target agent directories
        for target_dir in &target_dirs {
            let target_config = InstallConfig::new(target_dir.clone());
            let target_result = install_skill(skill, &target_config)?;
            println!("  Installed to: {}", target_result.path.display());
            if target_result.symlink_failed {
                println!("  Note: Symlink failed, used copy instead.");
            }
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
        let result = resolve_target_dirs(&agents, &base_dir).unwrap();
        assert_eq!(result, vec![PathBuf::from("/home/user/.claude/skills")]);
    }

    #[test]
    fn test_resolve_target_dirs_opencode() {
        let agents = vec!["opencode".to_string()];
        let base_dir = PathBuf::from("/home/user");
        let result = resolve_target_dirs(&agents, &base_dir).unwrap();
        assert_eq!(
            result,
            vec![PathBuf::from("/home/user/.config/opencode/skills")]
        );
    }

    #[test]
    fn test_resolve_target_dirs_multiple() {
        let agents = vec!["claude".to_string(), "opencode".to_string()];
        let base_dir = PathBuf::from("/home/user");
        let result = resolve_target_dirs(&agents, &base_dir).unwrap();
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
        let result = resolve_target_dirs(&agents, &base_dir);
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
        let result = resolve_target_dirs(&agents, &base_dir).unwrap();
        assert_eq!(result, Vec::<PathBuf>::new());
    }
}
