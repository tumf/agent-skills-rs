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
        /// Source type or identifier (github, gitlab, local, direct, self, embedded)
        source: String,
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
                println!("  install-skill <source> [--yes] [--non-interactive]");
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
            source,
            yes,
            non_interactive,
        } => {
            install_skill_command(&source, yes || non_interactive)?;
        }
    }

    Ok(())
}

fn install_skill_command(source_str: &str, auto_confirm: bool) -> Result<()> {
    // Determine source type
    let source_type = match source_str.to_lowercase().as_str() {
        "self" | "embedded" => SourceType::Self_,
        "github" => SourceType::Github,
        "gitlab" => SourceType::Gitlab,
        "local" => SourceType::Local,
        "direct" => SourceType::Direct,
        _ => {
            anyhow::bail!("Unknown source type: {}", source_str);
        }
    };

    let source = Source {
        source_type,
        url: None,
        subpath: None,
        skill_filter: None,
        ref_: None,
    };

    // Setup paths
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let canonical_dir = PathBuf::from(&home).join(".agents/skills");
    let lock_path = PathBuf::from(&home).join(".agents/.skill-lock.json");

    println!("Discovering skills from source: {}", source_str);

    // Discover skills
    let config = DiscoveryConfig::default();
    let skills = discover_skills(&source, &config)?;

    if skills.is_empty() {
        println!("No skills found.");
        return Ok(());
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
        let installed_path = install_skill(skill, &install_config)?;

        println!("  Installed to: {}", installed_path.display());

        // Update lock file
        let lock_manager = LockManager::new(lock_path.clone());
        lock_manager.update_entry(&skill.name, &source, &installed_path)?;

        println!("  Lock file updated.");
    }

    println!("\nInstallation complete!");
    Ok(())
}
