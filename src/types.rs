use serde::{Deserialize, Serialize};
use std::collections::HashMap;

fn deserialize_lock_version<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum LockVersion {
        String(String),
        Integer(i64),
    }

    match LockVersion::deserialize(deserializer)? {
        LockVersion::String(version) => Ok(version),
        LockVersion::Integer(version) => Ok(version.to_string()),
    }
}

/// Legacy lock entry format (vercel-lab/AgentSkills)
#[derive(Debug, Clone, Deserialize)]
struct LegacyLockEntry {
    name: String,
    path: String,
    source_type: String,
}

/// Helper enum for deserializing both old and new lock file formats
#[derive(Deserialize)]
#[serde(untagged)]
enum SkillLockFormat {
    New {
        #[serde(deserialize_with = "deserialize_lock_version")]
        version: String,
        #[serde(default)]
        skills: HashMap<String, LockEntry>,
    },
    Legacy {
        skills: Vec<LegacyLockEntry>,
    },
}

impl From<SkillLockFormat> for SkillLock {
    fn from(format: SkillLockFormat) -> Self {
        match format {
            SkillLockFormat::New { version, skills } => SkillLock { version, skills },
            SkillLockFormat::Legacy { skills } => {
                let now = chrono::Utc::now();
                let mut skill_map = HashMap::new();

                for legacy_entry in skills {
                    // Skip entries with empty paths (not actually installed)
                    if legacy_entry.path.is_empty() {
                        continue;
                    }

                    let entry = LockEntry {
                        source: legacy_entry.source_type.clone(),
                        source_type: legacy_entry.source_type,
                        source_url: None,
                        skill_path: legacy_entry.path,
                        skill_folder_hash: String::new(), // Will be computed on next update
                        installed_at: now,
                        updated_at: now,
                    };
                    skill_map.insert(legacy_entry.name, entry);
                }

                SkillLock {
                    version: "1.0".to_string(),
                    skills: skill_map,
                }
            }
        }
    }
}

/// Represents a skill definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Skill {
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    pub raw_content: String,
    #[serde(default)]
    pub metadata: SkillMetadata,
}

/// Metadata for a skill
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct SkillMetadata {
    #[serde(default)]
    pub internal: bool,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Source type for skill installation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SourceType {
    Github,
    Gitlab,
    Local,
    Direct,
    #[serde(rename = "self", alias = "embedded")]
    Self_,
}

impl SourceType {
    /// Check if this is an embedded source type (self or embedded)
    pub fn is_embedded(&self) -> bool {
        matches!(self, SourceType::Self_)
    }
}

/// Source specification for skill installation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Source {
    #[serde(rename = "type")]
    pub source_type: SourceType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subpath: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_filter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ref_: Option<String>,
}

/// Lock entry for installed skills
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LockEntry {
    pub source: String,
    #[serde(rename = "sourceType")]
    pub source_type: String,
    #[serde(rename = "sourceUrl", skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
    #[serde(rename = "skillPath")]
    pub skill_path: String,
    #[serde(rename = "skillFolderHash")]
    pub skill_folder_hash: String,
    #[serde(rename = "installedAt")]
    pub installed_at: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Lock file structure
#[derive(Debug, Clone, Serialize, Default, PartialEq)]
pub struct SkillLock {
    pub version: String,
    pub skills: HashMap<String, LockEntry>,
}

impl<'de> serde::Deserialize<'de> for SkillLock {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let format = SkillLockFormat::deserialize(deserializer)?;
        Ok(format.into())
    }
}

impl SkillLock {
    pub fn new() -> Self {
        Self {
            version: "1.0".to_string(),
            skills: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_type_embedded_alias() {
        // Test that "self" and "embedded" are recognized
        let self_json = r#"{"type":"self"}"#;
        let embedded_json = r#"{"type":"embedded"}"#;

        let self_source: Source = serde_json::from_str(self_json).unwrap();
        let embedded_source: Source = serde_json::from_str(embedded_json).unwrap();

        assert!(self_source.source_type.is_embedded());
        assert!(embedded_source.source_type.is_embedded());
    }

    #[test]
    fn test_source_type_is_embedded() {
        assert!(SourceType::Self_.is_embedded());
        assert!(!SourceType::Github.is_embedded());
        assert!(!SourceType::Local.is_embedded());
    }

    #[test]
    fn test_skill_serialization() {
        let skill = Skill {
            name: "test-skill".to_string(),
            description: "Test skill".to_string(),
            path: Some("/path/to/skill".to_string()),
            raw_content: "# Test\nContent".to_string(),
            metadata: SkillMetadata::default(),
        };

        let json = serde_json::to_string(&skill).unwrap();
        let deserialized: Skill = serde_json::from_str(&json).unwrap();
        assert_eq!(skill, deserialized);
    }

    #[test]
    fn test_lock_entry_serialization() {
        let entry = LockEntry {
            source: "embedded".to_string(),
            source_type: "embedded".to_string(),
            source_url: None,
            skill_path: "/path/to/skill".to_string(),
            skill_folder_hash: "abc123".to_string(),
            installed_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: LockEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(entry, deserialized);
    }

    #[test]
    fn test_skill_lock_deserializes_integer_version() {
        let json = r#"{
            "version": 3,
            "skills": {}
        }"#;

        let lock: SkillLock = serde_json::from_str(json).unwrap();
        assert_eq!(lock.version, "3");
        assert!(lock.skills.is_empty());
    }

    #[test]
    fn test_legacy_lock_format_migration() {
        // Test migrating from vercel-lab/AgentSkills format
        let legacy_json = r#"{
            "skills": [
                {
                    "name": "test-skill-1",
                    "path": "/path/to/skill1",
                    "source_type": "github"
                },
                {
                    "name": "test-skill-2",
                    "path": "",
                    "source_type": "github"
                },
                {
                    "name": "test-skill-3",
                    "path": "/path/to/skill3",
                    "source_type": "self"
                }
            ]
        }"#;

        let lock: SkillLock = serde_json::from_str(legacy_json).unwrap();

        // Should have version 1.0 after migration
        assert_eq!(lock.version, "1.0");

        // Should only include skills with non-empty paths
        assert_eq!(lock.skills.len(), 2);
        assert!(lock.skills.contains_key("test-skill-1"));
        assert!(lock.skills.contains_key("test-skill-3"));
        assert!(!lock.skills.contains_key("test-skill-2")); // Empty path, should be skipped

        // Check migrated entry structure
        let entry1 = lock.skills.get("test-skill-1").unwrap();
        assert_eq!(entry1.source, "github");
        assert_eq!(entry1.source_type, "github");
        assert_eq!(entry1.skill_path, "/path/to/skill1");
        assert_eq!(entry1.skill_folder_hash, ""); // Will be computed on next update

        let entry3 = lock.skills.get("test-skill-3").unwrap();
        assert_eq!(entry3.source, "self");
        assert_eq!(entry3.source_type, "self");
    }

    #[test]
    fn test_new_lock_format_still_works() {
        // Ensure new format continues to work
        let new_json = r#"{
            "version": "1.0",
            "skills": {
                "test-skill": {
                    "source": "github",
                    "sourceType": "github",
                    "skillPath": "/path/to/skill",
                    "skillFolderHash": "abc123",
                    "installedAt": "2024-01-01T00:00:00Z",
                    "updatedAt": "2024-01-01T00:00:00Z"
                }
            }
        }"#;

        let lock: SkillLock = serde_json::from_str(new_json).unwrap();

        assert_eq!(lock.version, "1.0");
        assert_eq!(lock.skills.len(), 1);
        assert!(lock.skills.contains_key("test-skill"));

        let entry = lock.skills.get("test-skill").unwrap();
        assert_eq!(entry.skill_folder_hash, "abc123");
    }
}
