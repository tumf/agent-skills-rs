use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;

/// CLI command definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subcommands: Option<Vec<Command>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<Argument>>,
}

/// CLI argument definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Argument {
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub arg_type: String,
    #[serde(default)]
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub choices: Option<Vec<String>>,
}

/// Introspection output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntrospectionOutput {
    #[serde(rename = "schemaVersion")]
    pub schema_version: String,
    #[serde(rename = "type")]
    pub output_type: String,
    pub ok: bool,
    #[serde(flatten)]
    pub data: serde_json::Value,
}

impl IntrospectionOutput {
    pub fn new(output_type: &str, data: serde_json::Value) -> Self {
        Self {
            schema_version: "1.0".to_string(),
            output_type: output_type.to_string(),
            ok: true,
            data,
        }
    }
}

/// Get all commands for introspection
pub fn get_commands() -> Vec<Command> {
    vec![
        Command {
            name: "install-skill".to_string(),
            description: "Install embedded skill(s) bundled in the binary".to_string(),
            subcommands: None,
            arguments: Some(vec![
                Argument {
                    name: "agent".to_string(),
                    description: "Target agent name for agent-specific installation".to_string(),
                    arg_type: "string".to_string(),
                    required: false,
                    choices: None,
                },
                Argument {
                    name: "skill".to_string(),
                    description: "Specific skill name to install (if source contains multiple)"
                        .to_string(),
                    arg_type: "string".to_string(),
                    required: false,
                    choices: None,
                },
                Argument {
                    name: "global".to_string(),
                    description: "Install globally (default: project-local)".to_string(),
                    arg_type: "boolean".to_string(),
                    required: false,
                    choices: None,
                },
                Argument {
                    name: "yes".to_string(),
                    description: "Skip confirmation prompts".to_string(),
                    arg_type: "boolean".to_string(),
                    required: false,
                    choices: None,
                },
                Argument {
                    name: "non-interactive".to_string(),
                    description: "Run in non-interactive mode".to_string(),
                    arg_type: "boolean".to_string(),
                    required: false,
                    choices: None,
                },
            ]),
        },
        Command {
            name: "commands".to_string(),
            description: "List all available commands".to_string(),
            subcommands: None,
            arguments: Some(vec![Argument {
                name: "output".to_string(),
                description: "Output format (json)".to_string(),
                arg_type: "string".to_string(),
                required: false,
                choices: Some(vec!["json".to_string()]),
            }]),
        },
        Command {
            name: "schema".to_string(),
            description: "Get JSON schema for a command".to_string(),
            subcommands: None,
            arguments: Some(vec![
                Argument {
                    name: "command".to_string(),
                    description: "Command name to get schema for".to_string(),
                    arg_type: "string".to_string(),
                    required: true,
                    choices: None,
                },
                Argument {
                    name: "output".to_string(),
                    description: "Output format (json-schema)".to_string(),
                    arg_type: "string".to_string(),
                    required: false,
                    choices: Some(vec!["json-schema".to_string()]),
                },
            ]),
        },
    ]
}

/// Output commands as JSON
pub fn output_commands_json() -> Result<String> {
    let commands = get_commands();
    let output = IntrospectionOutput::new("commands.list", json!({ "commands": commands }));
    serde_json::to_string_pretty(&output).context("Failed to serialize commands")
}

/// Get JSON schema for a specific command
pub fn get_command_schema(command_name: &str) -> Result<String> {
    let commands = get_commands();
    let command = commands
        .iter()
        .find(|c| c.name == command_name)
        .with_context(|| format!("Command not found: {}", command_name))?;

    let mut properties = serde_json::Map::new();
    let mut required = Vec::new();

    if let Some(args) = &command.arguments {
        for arg in args {
            let mut prop = serde_json::Map::new();
            prop.insert("type".to_string(), json!(arg.arg_type));
            prop.insert("description".to_string(), json!(arg.description));

            if let Some(choices) = &arg.choices {
                prop.insert("enum".to_string(), json!(choices));
            }

            properties.insert(arg.name.clone(), json!(prop));

            if arg.required {
                required.push(arg.name.clone());
            }
        }
    }

    let schema = json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "object",
        "properties": properties,
        "required": required,
        "additionalProperties": false
    });

    let output = IntrospectionOutput::new("schema", json!({ "schema": schema }));
    serde_json::to_string_pretty(&output).context("Failed to serialize schema")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_commands() {
        let commands = get_commands();
        assert!(!commands.is_empty());

        let install_cmd = commands.iter().find(|c| c.name == "install-skill");
        assert!(install_cmd.is_some());
    }

    #[test]
    fn test_install_skill_command_has_no_source_argument() {
        let commands = get_commands();
        let install_cmd = commands.iter().find(|c| c.name == "install-skill").unwrap();

        let source_arg = install_cmd
            .arguments
            .as_ref()
            .unwrap()
            .iter()
            .find(|a| a.name == "source");

        assert!(source_arg.is_none());
    }

    #[test]
    fn test_output_commands_json() {
        let json = output_commands_json().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["schemaVersion"], "1.0");
        assert_eq!(parsed["type"], "commands.list");
        assert_eq!(parsed["ok"], true);
        assert!(parsed["commands"].is_array());
    }

    #[test]
    fn test_get_command_schema() {
        let schema = get_command_schema("install-skill").unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&schema).unwrap();

        assert_eq!(parsed["schemaVersion"], "1.0");
        assert_eq!(parsed["type"], "schema");
        assert_eq!(parsed["ok"], true);
        assert!(parsed["schema"]["properties"].is_object());

        // Source is fixed to embedded and should not be user-specified
        assert!(parsed["schema"]["properties"]["source"].is_null());
    }

    #[test]
    fn test_schema_includes_yes_and_non_interactive() {
        let schema = get_command_schema("install-skill").unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&schema).unwrap();

        assert!(parsed["schema"]["properties"]["yes"].is_object());
        assert!(parsed["schema"]["properties"]["non-interactive"].is_object());
    }

    #[test]
    fn test_schema_includes_agent_skill_global() {
        let schema = get_command_schema("install-skill").unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&schema).unwrap();

        assert!(parsed["schema"]["properties"]["agent"].is_object());
        assert!(parsed["schema"]["properties"]["skill"].is_object());
        assert!(parsed["schema"]["properties"]["global"].is_object());

        // Verify descriptions
        assert_eq!(
            parsed["schema"]["properties"]["agent"]["description"],
            "Target agent name for agent-specific installation"
        );
        assert_eq!(
            parsed["schema"]["properties"]["skill"]["description"],
            "Specific skill name to install (if source contains multiple)"
        );
        assert_eq!(
            parsed["schema"]["properties"]["global"]["description"],
            "Install globally (default: project-local)"
        );
    }

    #[test]
    fn test_get_command_schema_not_found() {
        let result = get_command_schema("nonexistent");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Command not found"));
    }
}
