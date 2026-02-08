use crate::types::{Skill, SkillMetadata};
use anyhow::{Context, Result};

const AGENT_SKILLS_RUST_CONTENT: &str = include_str!("../.agents/skills/agent-skills-rs/SKILL.md");

/// Parse frontmatter from markdown content
fn parse_frontmatter(content: &str) -> Result<(SkillMetadata, String, String)> {
    let lines: Vec<&str> = content.lines().collect();

    // Check for frontmatter
    if lines.is_empty() || !lines[0].trim().starts_with("---") {
        anyhow::bail!("Missing frontmatter in skill definition");
    }

    // Find the closing ---
    let end_idx = lines
        .iter()
        .skip(1)
        .position(|line| line.trim() == "---")
        .context("Frontmatter not properly closed")?;

    let frontmatter_lines = &lines[1..end_idx + 1];
    let mut name = None;
    let mut description = None;

    for line in frontmatter_lines {
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            let value = value.trim();
            match key {
                "name" => name = Some(value.to_string()),
                "description" => description = Some(value.to_string()),
                _ => {}
            }
        }
    }

    let name = name.context("Missing 'name' in frontmatter")?;
    let description = description.context("Missing 'description' in frontmatter")?;

    Ok((SkillMetadata::default(), name, description))
}

/// Get the embedded skill definition
pub fn get_embedded_skill() -> Result<Skill> {
    let (metadata, name, description) = parse_frontmatter(AGENT_SKILLS_RUST_CONTENT)?;

    Ok(Skill {
        name,
        description,
        path: None,
        raw_content: AGENT_SKILLS_RUST_CONTENT.to_string(),
        metadata,
    })
}

/// Get all embedded skill definitions
pub fn get_embedded_skills() -> Result<Vec<Skill>> {
    let mut skills = Vec::new();

    // Parse agent-skills-rs
    let (metadata, name, description) = parse_frontmatter(AGENT_SKILLS_RUST_CONTENT)?;
    skills.push(Skill {
        name,
        description,
        path: None,
        raw_content: AGENT_SKILLS_RUST_CONTENT.to_string(),
        metadata,
    });

    Ok(skills)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_skill_not_empty() {
        assert!(!AGENT_SKILLS_RUST_CONTENT.is_empty());
        assert!(AGENT_SKILLS_RUST_CONTENT.contains("---"));
    }

    #[test]
    fn test_embedded_skill_has_required_frontmatter() {
        let skill = get_embedded_skill().expect("Failed to parse embedded skill");
        assert!(!skill.name.is_empty(), "Skill name must not be empty");
        assert!(
            !skill.description.is_empty(),
            "Skill description must not be empty"
        );
    }

    #[test]
    fn test_embedded_skill_content() {
        let skill = get_embedded_skill().unwrap();
        assert_eq!(skill.name, "agent-skills-rs");
        assert!(skill.description.contains("Rust library"));
        assert_eq!(skill.raw_content, AGENT_SKILLS_RUST_CONTENT);
    }

    #[test]
    fn test_parse_frontmatter_valid() {
        let content = r#"---
name: test-skill
description: A test skill
---

# Test Skill

Content here.
"#;
        let (metadata, name, description) = parse_frontmatter(content).unwrap();
        assert_eq!(name, "test-skill");
        assert_eq!(description, "A test skill");
        assert!(!metadata.internal);
    }

    #[test]
    fn test_parse_frontmatter_missing_name() {
        let content = r#"---
description: A test skill
---

Content
"#;
        let result = parse_frontmatter(content);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("name"));
    }

    #[test]
    fn test_parse_frontmatter_missing_description() {
        let content = r#"---
name: test-skill
---

Content
"#;
        let result = parse_frontmatter(content);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("description"));
    }

    #[test]
    fn test_parse_frontmatter_not_closed() {
        let content = r#"---
name: test-skill
description: A test skill

# No closing frontmatter
"#;
        let result = parse_frontmatter(content);
        assert!(result.is_err());
    }
}
