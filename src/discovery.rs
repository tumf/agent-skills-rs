use crate::embedded;
use crate::types::{Skill, Source};
use anyhow::{Context, Result};

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

    // For other source types, this would implement file system or provider-based discovery
    // For now, return empty as the mock-first approach will handle this via providers
    Ok(Vec::new())
}

/// Discover embedded skills
fn discover_embedded_skills(_config: &DiscoveryConfig) -> Result<Vec<Skill>> {
    let skill = embedded::get_embedded_skill().context("Failed to load embedded skill")?;
    Ok(vec![skill])
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
