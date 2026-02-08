use crate::types::Skill;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Installation mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallMode {
    Symlink,
    Copy,
}

/// Installation configuration
#[derive(Debug, Clone)]
pub struct InstallConfig {
    pub mode: InstallMode,
    pub canonical_dir: PathBuf,
    pub target_dirs: Vec<PathBuf>,
    pub fallback_to_copy: bool,
}

impl InstallConfig {
    pub fn new(canonical_dir: PathBuf) -> Self {
        Self {
            mode: InstallMode::Symlink,
            canonical_dir,
            target_dirs: Vec::new(),
            fallback_to_copy: true,
        }
    }
}

/// Install a skill to the canonical location and link/copy to target directories
pub fn install_skill(skill: &Skill, config: &InstallConfig) -> Result<PathBuf> {
    // Create canonical path
    let canonical_path = config.canonical_dir.join(&skill.name);

    // Ensure parent directory exists
    if let Some(parent) = canonical_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create canonical directory: {:?}", parent))?;
    }

    // Write skill content to canonical location
    fs::create_dir_all(&canonical_path)
        .with_context(|| format!("Failed to create skill directory: {:?}", canonical_path))?;

    let skill_file_path = canonical_path.join("SKILL.md");
    fs::write(&skill_file_path, &skill.raw_content)
        .with_context(|| format!("Failed to write skill file: {:?}", skill_file_path))?;

    // Link or copy to target directories
    for target_dir in &config.target_dirs {
        link_or_copy_skill(&canonical_path, target_dir, &skill.name, config)?;
    }

    Ok(canonical_path)
}

/// Link or copy skill from canonical location to target directory
fn link_or_copy_skill(
    canonical_path: &Path,
    target_dir: &Path,
    skill_name: &str,
    config: &InstallConfig,
) -> Result<()> {
    fs::create_dir_all(target_dir)
        .with_context(|| format!("Failed to create target directory: {:?}", target_dir))?;

    let target_path = target_dir.join(skill_name);

    // Remove existing if present
    if target_path.exists() {
        if target_path.is_dir() {
            fs::remove_dir_all(&target_path)?;
        } else {
            fs::remove_file(&target_path)?;
        }
    }

    match config.mode {
        InstallMode::Symlink => {
            // Try to create symlink
            #[cfg(unix)]
            let result = std::os::unix::fs::symlink(canonical_path, &target_path);
            #[cfg(windows)]
            let result = std::os::windows::fs::symlink_dir(canonical_path, &target_path);

            match result {
                Ok(_) => Ok(()),
                Err(e) if config.fallback_to_copy => {
                    // Fallback to copy
                    copy_skill(canonical_path, &target_path)?;
                    Ok(())
                }
                Err(e) => Err(e).context("Failed to create symlink"),
            }
        }
        InstallMode::Copy => {
            copy_skill(canonical_path, &target_path)?;
            Ok(())
        }
    }
}

/// Copy skill directory
fn copy_skill(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_skill(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SkillMetadata;
    use tempfile::TempDir;

    fn create_test_skill() -> Skill {
        Skill {
            name: "test-skill".to_string(),
            description: "Test skill".to_string(),
            path: None,
            raw_content: "---\nname: test-skill\ndescription: Test skill\n---\n\n# Test"
                .to_string(),
            metadata: SkillMetadata::default(),
        }
    }

    #[test]
    fn test_install_skill_to_canonical() {
        let temp_dir = TempDir::new().unwrap();
        let canonical_dir = temp_dir.path().join(".agents/skills");

        let config = InstallConfig::new(canonical_dir.clone());
        let skill = create_test_skill();

        let result = install_skill(&skill, &config).unwrap();

        assert_eq!(result, canonical_dir.join("test-skill"));
        assert!(result.join("SKILL.md").exists());

        let content = fs::read_to_string(result.join("SKILL.md")).unwrap();
        assert_eq!(content, skill.raw_content);
    }

    #[test]
    fn test_install_skill_with_symlink() {
        let temp_dir = TempDir::new().unwrap();
        let canonical_dir = temp_dir.path().join(".agents/skills");
        let target_dir = temp_dir.path().join("target/skills");

        let mut config = InstallConfig::new(canonical_dir.clone());
        config.target_dirs.push(target_dir.clone());
        config.mode = InstallMode::Symlink;

        let skill = create_test_skill();
        install_skill(&skill, &config).unwrap();

        let target_path = target_dir.join("test-skill");
        assert!(target_path.exists());

        // On systems that support symlinks, verify it's a symlink
        #[cfg(unix)]
        {
            let metadata = fs::symlink_metadata(&target_path).unwrap();
            assert!(metadata.file_type().is_symlink());
        }
    }

    #[test]
    fn test_install_skill_with_copy() {
        let temp_dir = TempDir::new().unwrap();
        let canonical_dir = temp_dir.path().join(".agents/skills");
        let target_dir = temp_dir.path().join("target/skills");

        let mut config = InstallConfig::new(canonical_dir.clone());
        config.target_dirs.push(target_dir.clone());
        config.mode = InstallMode::Copy;

        let skill = create_test_skill();
        install_skill(&skill, &config).unwrap();

        let target_path = target_dir.join("test-skill");
        assert!(target_path.exists());
        assert!(target_path.join("SKILL.md").exists());

        let content = fs::read_to_string(target_path.join("SKILL.md")).unwrap();
        assert_eq!(content, skill.raw_content);
    }

    #[test]
    fn test_embedded_skill_installation() {
        let temp_dir = TempDir::new().unwrap();
        let canonical_dir = temp_dir.path().join(".agents/skills");

        let config = InstallConfig::new(canonical_dir.clone());

        // Get embedded skill
        let skill = crate::embedded::get_embedded_skill().unwrap();

        // Install it
        let result = install_skill(&skill, &config).unwrap();

        assert!(result.exists());
        assert!(result.join("SKILL.md").exists());
    }
}
