use crate::types::{Skill, SkillMetadata};
use anyhow::{Context, Result};
use std::collections::HashMap;

const AGENT_SKILLS_RUST_CONTENT: &str = include_str!("../skills/agent-skills-rs/SKILL.md");

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

/// Register an embedded skill from SKILL.md content and optional auxiliary files.
///
/// `skill_md` must contain valid YAML frontmatter with `name` and `description` fields.
/// `auxiliary` is a slice of `(relative_path, content)` pairs for extra files bundled
/// with the skill (e.g. `("scripts/helper.py", include_str!("scripts/helper.py"))`).
pub fn register_embedded_skill(skill_md: &str, auxiliary: &[(&str, &str)]) -> Result<Skill> {
    let (metadata, name, description) = parse_frontmatter(skill_md)?;

    let mut auxiliary_files = HashMap::new();
    for (rel_path, content) in auxiliary {
        auxiliary_files.insert(rel_path.to_string(), content.to_string());
    }

    Ok(Skill {
        name,
        description,
        path: None,
        raw_content: skill_md.to_string(),
        metadata,
        auxiliary_files,
    })
}

/// Get the embedded skill definition
pub fn get_embedded_skill() -> Result<Skill> {
    register_embedded_skill(AGENT_SKILLS_RUST_CONTENT, &[])
}

/// Get all embedded skill definitions
pub fn get_embedded_skills() -> Result<Vec<Skill>> {
    Ok(vec![register_embedded_skill(AGENT_SKILLS_RUST_CONTENT, &[])?])
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
    fn test_register_embedded_skill_with_aux_files() {
        let skill_md = r#"---
name: my-skill
description: A skill with aux files
---

# My Skill
"#;
        let skill = register_embedded_skill(
            skill_md,
            &[
                ("scripts/helper.py", "print('hello')"),
                ("references/guide.md", "# Guide"),
            ],
        )
        .unwrap();

        assert_eq!(skill.name, "my-skill");
        assert_eq!(skill.description, "A skill with aux files");
        assert_eq!(skill.raw_content, skill_md);
        assert_eq!(skill.auxiliary_files.len(), 2);
        assert_eq!(
            skill.auxiliary_files.get("scripts/helper.py").unwrap(),
            "print('hello')"
        );
        assert_eq!(
            skill.auxiliary_files.get("references/guide.md").unwrap(),
            "# Guide"
        );
    }

    #[test]
    fn test_register_embedded_skill_no_aux_files() {
        let skill_md = r#"---
name: simple-skill
description: A simple skill
---

# Simple Skill
"#;
        let skill = register_embedded_skill(skill_md, &[]).unwrap();
        assert_eq!(skill.name, "simple-skill");
        assert!(skill.auxiliary_files.is_empty());
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
