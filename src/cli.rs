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
            description: "Install a skill from various sources".to_string(),
            subcommands: None,
            arguments: Some(vec![
                Argument {
                    name: "source".to_string(),
                    description:
                        "Source type or identifier (github, gitlab, local, direct, self, embedded)"
                            .to_string(),
                    arg_type: "string".to_string(),
                    required: true,
                    choices: Some(vec![
                        "github".to_string(),
                        "gitlab".to_string(),
                        "local".to_string(),
                        "direct".to_string(),
                        "self".to_string(),
                        "embedded".to_string(),
                    ]),
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
    let output = IntrospectionOutput::new("commands", json!({ "commands": commands }));
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
    fn test_install_skill_command_has_self_and_embedded() {
        let commands = get_commands();
        let install_cmd = commands.iter().find(|c| c.name == "install-skill").unwrap();

        let source_arg = install_cmd
            .arguments
            .as_ref()
            .unwrap()
            .iter()
            .find(|a| a.name == "source")
            .unwrap();

        let choices = source_arg.choices.as_ref().unwrap();
        assert!(choices.contains(&"self".to_string()));
        assert!(choices.contains(&"embedded".to_string()));
    }

    #[test]
    fn test_output_commands_json() {
        let json = output_commands_json().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["schemaVersion"], "1.0");
        assert_eq!(parsed["type"], "commands");
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

        // Verify source enum includes self and embedded
        let source_prop = &parsed["schema"]["properties"]["source"];
        let enum_values = source_prop["enum"].as_array().unwrap();
        assert!(enum_values.iter().any(|v| v == "self"));
        assert!(enum_values.iter().any(|v| v == "embedded"));
    }

    #[test]
    fn test_schema_includes_yes_and_non_interactive() {
        let schema = get_command_schema("install-skill").unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&schema).unwrap();

        assert!(parsed["schema"]["properties"]["yes"].is_object());
        assert!(parsed["schema"]["properties"]["non-interactive"].is_object());
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
