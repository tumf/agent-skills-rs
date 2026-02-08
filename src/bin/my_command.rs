use anyhow::Result;
use clap::{Parser, Subcommand};
use skill_installer::*;
use std::path::PathBuf;

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
        /// Target agent name for agent-specific installation
        #[arg(long)]
        agent: Option<String>,
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
            install_skill_command(
                agent.as_deref(),
                skill.as_deref(),
                global,
                yes || non_interactive,
            )?;
        }
    }

    Ok(())
}

fn install_skill_command(
    _agent: Option<&str>,
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

        let install_config = InstallConfig::new(canonical_dir.clone());
        let result = install_skill(skill, &install_config)?;

        println!("  Installed to: {}", result.path.display());
        if result.symlink_failed {
            println!("  Note: Symlink failed, used copy instead.");
        }

        let lock_manager = LockManager::new(lock_path.clone());
        lock_manager.update_entry(&skill.name, &source, &result.path)?;

        println!("  Lock file updated: {}", lock_path.display());
    }

    println!("\nInstallation complete!");
    Ok(())
}
