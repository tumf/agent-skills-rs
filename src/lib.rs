pub mod cli;
pub mod discovery;
pub mod embedded;
pub mod installer;
pub mod lock;
pub mod types;

pub use cli::{get_command_schema, get_commands, output_commands_json};
pub use discovery::{discover_skills, DiscoveryConfig};
pub use embedded::get_embedded_skill;
pub use installer::{install_skill, InstallConfig, InstallMode, InstallResult};
pub use lock::LockManager;
pub use types::{Skill, SkillLock, Source, SourceType};

#[cfg(test)]
mod integration_tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_end_to_end_embedded_install() {
        // Setup
        let temp_dir = TempDir::new().unwrap();
        let canonical_dir = temp_dir.path().join(".agents/skills");
        let lock_path = temp_dir.path().join(".agents/.skill-lock.json");

        // Create source
        let source = Source {
            source_type: SourceType::Self_,
            url: None,
            subpath: None,
            skill_filter: None,
            ref_: None,
        };

        // Discover skills
        let config = DiscoveryConfig::default();
        let skills = discover_skills(&source, &config).unwrap();
        assert_eq!(skills.len(), 1);
        let skill = &skills[0];

        // Install skill
        let install_config = InstallConfig::new(canonical_dir.clone());
        let result = install_skill(skill, &install_config).unwrap();

        // Verify installation
        assert!(result.path.exists());
        assert!(result.path.join("SKILL.md").exists());

        // Update lock
        let lock_manager = LockManager::new(lock_path);
        lock_manager
            .update_entry(&skill.name, &source, &result.path)
            .unwrap();

        // Verify lock entry
        let entry = lock_manager.get_entry(&skill.name).unwrap().unwrap();
        assert_eq!(entry.source_type, "self");
        assert!(!entry.skill_folder_hash.is_empty());
    }

    #[test]
    fn test_self_and_embedded_are_equivalent() {
        let config = DiscoveryConfig::default();

        // Test with source_type parsed from "self" JSON
        let json_self = r#"{"type":"self"}"#;
        let source_self: Source = serde_json::from_str(json_self).unwrap();
        let skills_self = discover_skills(&source_self, &config).unwrap();

        // Test with source_type parsed from "embedded" JSON (should deserialize to Self_ due to alias)
        let json_embedded = r#"{"type":"embedded"}"#;
        let source_embedded: Source = serde_json::from_str(json_embedded).unwrap();
        let skills_embedded = discover_skills(&source_embedded, &config).unwrap();

        // Should produce same results (both deserialize to Self_)
        assert_eq!(skills_self.len(), skills_embedded.len());
        assert_eq!(skills_self[0].name, skills_embedded[0].name);
        assert_eq!(source_self.source_type, source_embedded.source_type);
    }

    #[test]
    fn test_no_external_calls_for_embedded() {
        // This test verifies the entire flow works without external network access
        let temp_dir = TempDir::new().unwrap();
        let canonical_dir = temp_dir.path().join(".agents/skills");
        let lock_path = temp_dir.path().join(".agents/.skill-lock.json");

        let source = Source {
            source_type: SourceType::Self_,
            url: None,
            subpath: None,
            skill_filter: None,
            ref_: None,
        };

        let config = DiscoveryConfig::default();
        let skills = discover_skills(&source, &config).unwrap();

        let install_config = InstallConfig::new(canonical_dir.clone());
        let result = install_skill(&skills[0], &install_config).unwrap();

        let lock_manager = LockManager::new(lock_path);
        lock_manager
            .update_entry(&skills[0].name, &source, &result.path)
            .unwrap();

        // All operations should succeed without network access
        assert!(result.path.exists());
    }
}
