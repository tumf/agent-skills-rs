use crate::types::{LockEntry, SkillLock, Source};
use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

/// Lock file manager
pub struct LockManager {
    lock_path: PathBuf,
}

impl LockManager {
    pub fn new(lock_path: PathBuf) -> Self {
        Self { lock_path }
    }

    /// Load lock file, creating new if it doesn't exist
    pub fn load(&self) -> Result<SkillLock> {
        if !self.lock_path.exists() {
            return Ok(SkillLock::new());
        }

        let content = fs::read_to_string(&self.lock_path)
            .with_context(|| format!("Failed to read lock file: {:?}", self.lock_path))?;

        let lock: SkillLock = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse lock file: {:?}", self.lock_path))?;

        Ok(lock)
    }

    /// Save lock file
    pub fn save(&self, lock: &SkillLock) -> Result<()> {
        if let Some(parent) = self.lock_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create lock directory: {:?}", parent))?;
        }

        let content =
            serde_json::to_string_pretty(lock).context("Failed to serialize lock file")?;

        fs::write(&self.lock_path, content)
            .with_context(|| format!("Failed to write lock file: {:?}", self.lock_path))?;

        Ok(())
    }

    /// Update or add a skill entry in the lock file
    pub fn update_entry(&self, skill_name: &str, source: &Source, skill_path: &Path) -> Result<()> {
        let folder_hash = compute_skill_hash(skill_path)?;
        self.update_entry_with_hash(skill_name, source, skill_path, folder_hash)
    }

    /// Update or add a skill entry with a custom hash (for provider-based installs)
    pub fn update_entry_with_hash(
        &self,
        skill_name: &str,
        source: &Source,
        skill_path: &Path,
        folder_hash: String,
    ) -> Result<()> {
        let mut lock = self.load()?;
        let now = chrono::Utc::now();

        // Normalize source type for embedded/self
        let normalized_source_type = if source.source_type.is_embedded() {
            "self".to_string()
        } else {
            format!("{:?}", source.source_type).to_lowercase()
        };

        let entry = lock
            .skills
            .entry(skill_name.to_string())
            .or_insert_with(|| LockEntry {
                source: if source.source_type.is_embedded() {
                    "Self".to_string()
                } else {
                    format!("{:?}", source.source_type)
                },
                source_type: normalized_source_type.clone(),
                source_url: source.url.clone(),
                skill_path: skill_path.to_string_lossy().to_string(),
                skill_folder_hash: folder_hash.clone(),
                installed_at: now,
                updated_at: now,
            });

        // Update entry
        entry.source_type = normalized_source_type;
        entry.skill_folder_hash = folder_hash;
        entry.updated_at = now;
        entry.skill_path = skill_path.to_string_lossy().to_string();

        self.save(&lock)?;
        Ok(())
    }

    /// Get entry for a skill
    pub fn get_entry(&self, skill_name: &str) -> Result<Option<LockEntry>> {
        let lock = self.load()?;
        Ok(lock.skills.get(skill_name).cloned())
    }

    /// Remove entry for a skill
    pub fn remove_entry(&self, skill_name: &str) -> Result<()> {
        let mut lock = self.load()?;
        lock.skills.remove(skill_name);
        self.save(&lock)?;
        Ok(())
    }
}

/// Compute hash for a skill directory
pub fn compute_skill_hash(skill_path: &Path) -> Result<String> {
    let mut hasher = Sha256::new();

    // For embedded skills, hash the SKILL.md content
    let skill_file = skill_path.join("SKILL.md");
    if skill_file.exists() {
        let content = fs::read(&skill_file)
            .with_context(|| format!("Failed to read skill file: {:?}", skill_file))?;
        hasher.update(&content);
    }

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SourceType;
    use tempfile::TempDir;

    #[test]
    fn test_lock_manager_new_file() {
        let temp_dir = TempDir::new().unwrap();
        let lock_path = temp_dir.path().join(".skill-lock.json");

        let manager = LockManager::new(lock_path.clone());
        let lock = manager.load().unwrap();

        assert_eq!(lock.version, "1.0");
        assert!(lock.skills.is_empty());
    }

    #[test]
    fn test_lock_manager_save_load() {
        let temp_dir = TempDir::new().unwrap();
        let lock_path = temp_dir.path().join(".skill-lock.json");

        let manager = LockManager::new(lock_path.clone());
        let mut lock = SkillLock::new();

        let now = chrono::Utc::now();
        lock.skills.insert(
            "test-skill".to_string(),
            LockEntry {
                source: "Embedded".to_string(),
                source_type: "embedded".to_string(),
                source_url: None,
                skill_path: "/path/to/skill".to_string(),
                skill_folder_hash: "abc123".to_string(),
                installed_at: now,
                updated_at: now,
            },
        );

        manager.save(&lock).unwrap();

        let loaded = manager.load().unwrap();
        assert_eq!(loaded.skills.len(), 1);
        assert!(loaded.skills.contains_key("test-skill"));
    }

    #[test]
    fn test_lock_manager_update_entry() {
        let temp_dir = TempDir::new().unwrap();
        let lock_path = temp_dir.path().join(".skill-lock.json");
        let skill_dir = temp_dir.path().join("skill");

        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), "test content").unwrap();

        let manager = LockManager::new(lock_path.clone());
        let source = Source {
            source_type: SourceType::Self_,
            url: None,
            subpath: None,
            skill_filter: None,
            ref_: None,
        };

        manager
            .update_entry("test-skill", &source, &skill_dir)
            .unwrap();

        let entry = manager.get_entry("test-skill").unwrap().unwrap();
        assert_eq!(entry.source_type, "self");
        assert!(!entry.skill_folder_hash.is_empty());
    }

    #[test]
    fn test_compute_skill_hash() {
        let temp_dir = TempDir::new().unwrap();
        let skill_dir = temp_dir.path().join("skill");

        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), "test content").unwrap();

        let hash1 = compute_skill_hash(&skill_dir).unwrap();
        assert!(!hash1.is_empty());

        // Same content should produce same hash
        let hash2 = compute_skill_hash(&skill_dir).unwrap();
        assert_eq!(hash1, hash2);

        // Different content should produce different hash
        fs::write(skill_dir.join("SKILL.md"), "different content").unwrap();
        let hash3 = compute_skill_hash(&skill_dir).unwrap();
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_embedded_source_lock_entry() {
        let temp_dir = TempDir::new().unwrap();
        let lock_path = temp_dir.path().join(".skill-lock.json");
        let skill_dir = temp_dir.path().join("skill");

        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), "embedded skill").unwrap();

        let manager = LockManager::new(lock_path);
        let source = Source {
            source_type: SourceType::Self_,
            url: None,
            subpath: None,
            skill_filter: None,
            ref_: None,
        };

        manager
            .update_entry("embedded-skill", &source, &skill_dir)
            .unwrap();

        let entry = manager.get_entry("embedded-skill").unwrap().unwrap();
        assert_eq!(entry.source_type, "self");
        assert!(entry.source_url.is_none());
        assert!(!entry.skill_folder_hash.is_empty());
    }

    #[test]
    fn test_load_legacy_lock_with_integer_version() {
        let temp_dir = TempDir::new().unwrap();
        let lock_path = temp_dir.path().join(".skill-lock.json");

        fs::write(
            &lock_path,
            r#"{
  "version": 3,
  "skills": {}
}"#,
        )
        .unwrap();

        let manager = LockManager::new(lock_path);
        let lock = manager.load().unwrap();

        assert_eq!(lock.version, "3");
        assert!(lock.skills.is_empty());
    }

    #[test]
    fn test_load_legacy_array_format() {
        // Test loading vercel-lab/AgentSkills format
        let temp_dir = TempDir::new().unwrap();
        let lock_path = temp_dir.path().join(".skill-lock.json");

        fs::write(
            &lock_path,
            r#"{
  "skills": [
    {
      "name": "skill-1",
      "path": "/path/to/skill1",
      "source_type": "github"
    },
    {
      "name": "skill-2",
      "path": "",
      "source_type": "github"
    },
    {
      "name": "skill-3",
      "path": "/path/to/skill3",
      "source_type": "self"
    }
  ]
}"#,
        )
        .unwrap();

        let manager = LockManager::new(lock_path.clone());
        let lock = manager.load().unwrap();

        // Should auto-migrate to new format
        assert_eq!(lock.version, "1.0");
        assert_eq!(lock.skills.len(), 2); // Only skills with non-empty paths

        assert!(lock.skills.contains_key("skill-1"));
        assert!(lock.skills.contains_key("skill-3"));
        assert!(!lock.skills.contains_key("skill-2")); // Empty path

        let entry = lock.skills.get("skill-1").unwrap();
        assert_eq!(entry.source_type, "github");
        assert_eq!(entry.skill_path, "/path/to/skill1");

        // Save and verify new format is persisted
        manager.save(&lock).unwrap();
        let reloaded = manager.load().unwrap();
        assert_eq!(reloaded.version, "1.0");
        assert_eq!(reloaded.skills.len(), 2);
    }
}
