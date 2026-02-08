use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct SkillLock {
    pub version: String,
    #[serde(default)]
    pub skills: HashMap<String, LockEntry>,
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
}
