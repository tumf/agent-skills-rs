use crate::types::Skill;
use anyhow::Result;
use std::path::Path;

/// Trait for skill content providers (GitHub, GitLab, etc.)
pub trait SkillProvider: Send + Sync {
    /// Discover skills from the provider
    fn discover_skills(&self, url: &str, subpath: Option<&str>) -> Result<Vec<Skill>>;

    /// Fetch skill content to a local directory
    fn fetch_skill(&self, skill: &Skill, dest: &Path) -> Result<()>;

    /// Get folder hash for a skill (for lock file)
    fn get_folder_hash(&self, skill: &Skill) -> Result<String>;
}

/// Mock provider for testing
pub struct MockProvider {
    skills: Vec<Skill>,
    folder_hash: String,
}

impl MockProvider {
    pub fn new(skills: Vec<Skill>) -> Self {
        Self {
            skills,
            folder_hash: "mock-hash-123".to_string(),
        }
    }

    pub fn with_hash(mut self, hash: String) -> Self {
        self.folder_hash = hash;
        self
    }
}

impl SkillProvider for MockProvider {
    fn discover_skills(&self, _url: &str, _subpath: Option<&str>) -> Result<Vec<Skill>> {
        Ok(self.skills.clone())
    }

    fn fetch_skill(&self, skill: &Skill, dest: &Path) -> Result<()> {
        std::fs::create_dir_all(dest)?;
        std::fs::write(dest.join("SKILL.md"), &skill.raw_content)?;
        Ok(())
    }

    fn get_folder_hash(&self, _skill: &Skill) -> Result<String> {
        Ok(self.folder_hash.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SkillMetadata;
    use tempfile::TempDir;

    #[test]
    fn test_mock_provider_discover() {
        let skill = Skill {
            name: "test-skill".to_string(),
            description: "Test skill".to_string(),
            path: None,
            raw_content: "---\nname: test-skill\ndescription: Test skill\n---\n\n# Test"
                .to_string(),
            metadata: SkillMetadata::default(),
        };

        let provider = MockProvider::new(vec![skill.clone()]);
        let discovered = provider
            .discover_skills("https://example.com", None)
            .unwrap();

        assert_eq!(discovered.len(), 1);
        assert_eq!(discovered[0].name, "test-skill");
    }

    #[test]
    fn test_mock_provider_fetch() {
        let skill = Skill {
            name: "test-skill".to_string(),
            description: "Test skill".to_string(),
            path: None,
            raw_content: "---\nname: test-skill\ndescription: Test skill\n---\n\n# Test"
                .to_string(),
            metadata: SkillMetadata::default(),
        };

        let provider = MockProvider::new(vec![skill.clone()]);
        let temp_dir = TempDir::new().unwrap();

        provider.fetch_skill(&skill, temp_dir.path()).unwrap();

        assert!(temp_dir.path().join("SKILL.md").exists());
        let content = std::fs::read_to_string(temp_dir.path().join("SKILL.md")).unwrap();
        assert_eq!(content, skill.raw_content);
    }

    #[test]
    fn test_mock_provider_hash() {
        let skill = Skill {
            name: "test-skill".to_string(),
            description: "Test skill".to_string(),
            path: None,
            raw_content: "test".to_string(),
            metadata: SkillMetadata::default(),
        };

        let provider =
            MockProvider::new(vec![skill.clone()]).with_hash("custom-hash-456".to_string());

        let hash = provider.get_folder_hash(&skill).unwrap();
        assert_eq!(hash, "custom-hash-456");
    }
}
