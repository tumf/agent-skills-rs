use crate::embedded;
use crate::types::{Skill, SkillMetadata, Source};
use anyhow::{Context, Result};
use serde_yaml::Value as YamlValue;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[cfg(test)]
use crate::types::SourceType;

/// Configuration for skill discovery
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    pub allow_internal: bool,
    pub max_depth: usize,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            allow_internal: std::env::var("INSTALL_INTERNAL_SKILLS")
                .map(|v| v == "1")
                .unwrap_or(false),
            max_depth: 3,
        }
    }
}

/// Discover skills based on source specification
pub fn discover_skills(source: &Source, config: &DiscoveryConfig) -> Result<Vec<Skill>> {
    // Handle embedded sources
    if source.source_type.is_embedded() {
        return discover_embedded_skills(config);
    }

    // For local sources, perform file system discovery
    if matches!(source.source_type, crate::types::SourceType::Local) {
        let base_path = source
            .url
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        return discover_local_skills(&base_path, config);
    }

    // For other source types (github, gitlab, direct), this would implement provider-based discovery
    // For now, return empty as the mock-first approach will handle this via providers
    Ok(Vec::new())
}

/// Discover embedded skills
fn discover_embedded_skills(_config: &DiscoveryConfig) -> Result<Vec<Skill>> {
    let skill = embedded::get_embedded_skill().context("Failed to load embedded skill")?;
    Ok(vec![skill])
}

/// Discover skills from local file system
fn discover_local_skills(base_path: &Path, config: &DiscoveryConfig) -> Result<Vec<Skill>> {
    let mut skills = Vec::new();

    // Priority search directories
    let priority_dirs = vec![
        "skills",
        ".agents/skills",
        ".claude/skills",
        ".config/opencode/skills",
    ];

    // First try priority directories
    for dir in &priority_dirs {
        let search_path = base_path.join(dir);
        if search_path.exists() {
            skills.extend(search_directory(&search_path, config, 0)?);
        }
    }

    // If no skills found and max_depth allows, do recursive search
    if skills.is_empty() && config.max_depth > 0 {
        skills.extend(search_directory(base_path, config, 0)?);
    }

    Ok(skills)
}

/// Search a directory for SKILL.md files
fn search_directory(dir: &Path, config: &DiscoveryConfig, depth: usize) -> Result<Vec<Skill>> {
    if depth > config.max_depth {
        return Ok(Vec::new());
    }

    let mut skills = Vec::new();

    for entry in WalkDir::new(dir)
        .max_depth(config.max_depth - depth)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.file_name() == Some(std::ffi::OsStr::new("SKILL.md")) {
            if let Ok(skill) = parse_skill_file(path, config) {
                skills.push(skill);
            }
        }
    }

    Ok(skills)
}

/// Parse a SKILL.md file
fn parse_skill_file(path: &Path, config: &DiscoveryConfig) -> Result<Skill> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read SKILL.md at {:?}", path))?;

    // Parse frontmatter
    let (frontmatter, _body) = parse_frontmatter(&content)?;

    // Extract required fields
    let name = frontmatter
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'name' in frontmatter"))?
        .to_string();

    let description = frontmatter
        .get("description")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'description' in frontmatter"))?
        .to_string();

    // Extract metadata
    let mut metadata = SkillMetadata::default();
    if let Some(internal) = frontmatter.get("internal").and_then(|v| v.as_bool()) {
        metadata.internal = internal;
    }

    // Filter internal skills if not allowed
    if metadata.internal && !config.allow_internal {
        anyhow::bail!("Internal skill not allowed");
    }

    // Store other frontmatter fields in extra
    for (key, value) in frontmatter {
        if key != "name" && key != "description" && key != "internal" {
            if let Ok(json_value) = serde_yaml::from_value::<serde_json::Value>(value) {
                metadata.extra.insert(key, json_value);
            }
        }
    }

    Ok(Skill {
        name,
        description,
        path: Some(path.to_string_lossy().to_string()),
        raw_content: content,
        metadata,
    })
}

/// Parse frontmatter from markdown content
fn parse_frontmatter(content: &str) -> Result<(HashMap<String, YamlValue>, String)> {
    let lines: Vec<&str> = content.lines().collect();

    if lines.is_empty() || lines[0] != "---" {
        anyhow::bail!("Missing frontmatter delimiter");
    }

    let mut frontmatter_end = None;
    for (i, line) in lines.iter().enumerate().skip(1) {
        if *line == "---" {
            frontmatter_end = Some(i);
            break;
        }
    }

    let frontmatter_end = frontmatter_end.ok_or_else(|| anyhow::anyhow!("Unclosed frontmatter"))?;

    let frontmatter_str = lines[1..frontmatter_end].join("\n");
    let frontmatter: HashMap<String, YamlValue> =
        serde_yaml::from_str(&frontmatter_str).context("Failed to parse frontmatter YAML")?;

    let body = lines[(frontmatter_end + 1)..].join("\n");

    Ok((frontmatter, body))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_discover_embedded_skills() {
        let config = DiscoveryConfig::default();
        let source = Source {
            source_type: SourceType::Self_,
            url: None,
            subpath: None,
            skill_filter: None,
            ref_: None,
        };

        let skills = discover_skills(&source, &config).unwrap();
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "skill-installer");
    }

    #[test]
    fn test_discover_self_skills() {
        let config = DiscoveryConfig::default();
        let source = Source {
            source_type: SourceType::Self_,
            url: None,
            subpath: None,
            skill_filter: None,
            ref_: None,
        };

        let skills = discover_skills(&source, &config).unwrap();
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "skill-installer");
    }

    #[test]
    fn test_embedded_discovery_no_external_call() {
        // This test verifies that embedded discovery doesn't make external calls
        let config = DiscoveryConfig::default();
        let source = Source {
            source_type: SourceType::Self_,
            url: None,
            subpath: None,
            skill_filter: None,
            ref_: None,
        };

        // Should succeed without any network access
        let result = discover_skills(&source, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_frontmatter() {
        let content = r#"---
name: test-skill
description: Test skill description
internal: false
---

# Test Skill

This is the skill content.
"#;

        let (frontmatter, body) = parse_frontmatter(content).unwrap();
        assert_eq!(
            frontmatter.get("name").unwrap().as_str().unwrap(),
            "test-skill"
        );
        assert_eq!(
            frontmatter.get("description").unwrap().as_str().unwrap(),
            "Test skill description"
        );
        assert!(body.contains("# Test Skill"));
    }

    #[test]
    fn test_discover_local_skills_from_priority_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let skills_dir = temp_dir.path().join("skills");
        fs::create_dir_all(&skills_dir).unwrap();

        let skill_content = r#"---
name: test-skill
description: A test skill
---

# Test Skill
"#;
        fs::write(skills_dir.join("SKILL.md"), skill_content).unwrap();

        let config = DiscoveryConfig::default();
        let skills = discover_local_skills(temp_dir.path(), &config).unwrap();

        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "test-skill");
        assert_eq!(skills[0].description, "A test skill");
    }

    #[test]
    fn test_discover_local_skills_filters_internal() {
        let temp_dir = TempDir::new().unwrap();
        let skills_dir = temp_dir.path().join("skills");
        fs::create_dir_all(&skills_dir).unwrap();

        let internal_skill = r#"---
name: internal-skill
description: Internal skill
internal: true
---

# Internal Skill
"#;
        fs::write(skills_dir.join("SKILL.md"), internal_skill).unwrap();

        let config = DiscoveryConfig {
            allow_internal: false,
            max_depth: 3,
        };

        let skills = discover_local_skills(temp_dir.path(), &config).unwrap();
        assert_eq!(skills.len(), 0);
    }

    #[test]
    fn test_discover_local_skills_allows_internal_with_flag() {
        let temp_dir = TempDir::new().unwrap();
        let skills_dir = temp_dir.path().join("skills");
        fs::create_dir_all(&skills_dir).unwrap();

        let internal_skill = r#"---
name: internal-skill
description: Internal skill
internal: true
---

# Internal Skill
"#;
        fs::write(skills_dir.join("SKILL.md"), internal_skill).unwrap();

        let config = DiscoveryConfig {
            allow_internal: true,
            max_depth: 3,
        };

        let skills = discover_local_skills(temp_dir.path(), &config).unwrap();
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "internal-skill");
        assert!(skills[0].metadata.internal);
    }

    #[test]
    fn test_parse_skill_file_missing_name() {
        let temp_dir = TempDir::new().unwrap();
        let skill_path = temp_dir.path().join("SKILL.md");

        let content = r#"---
description: Missing name
---

# Skill
"#;
        fs::write(&skill_path, content).unwrap();

        let config = DiscoveryConfig::default();
        let result = parse_skill_file(&skill_path, &config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing 'name'"));
    }

    #[test]
    fn test_parse_skill_file_missing_description() {
        let temp_dir = TempDir::new().unwrap();
        let skill_path = temp_dir.path().join("SKILL.md");

        let content = r#"---
name: test-skill
---

# Skill
"#;
        fs::write(&skill_path, content).unwrap();

        let config = DiscoveryConfig::default();
        let result = parse_skill_file(&skill_path, &config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Missing 'description'"));
    }
}
