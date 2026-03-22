use crate::providers::SkillProvider;
use crate::types::Skill;
use anyhow::{bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Result of skill installation
#[derive(Debug, Clone, PartialEq)]
pub struct InstallResult {
    pub path: PathBuf,
    pub symlink_failed: bool,
}

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
pub fn install_skill(skill: &Skill, config: &InstallConfig) -> Result<InstallResult> {
    install_skill_with_provider(skill, config, None)
}

/// Install a skill with an optional provider (for external sources)
pub fn install_skill_with_provider(
    skill: &Skill,
    config: &InstallConfig,
    provider: Option<&dyn SkillProvider>,
) -> Result<InstallResult> {
    // Create canonical path
    let canonical_path = config.canonical_dir.join(&skill.name);

    // Ensure parent directory exists
    if let Some(parent) = canonical_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create canonical directory: {:?}", parent))?;
    }

    // Fetch content to canonical location
    if let Some(provider) = provider {
        // Use provider to fetch skill content
        provider
            .fetch_skill(skill, &canonical_path)
            .with_context(|| format!("Failed to fetch skill via provider: {:?}", skill.name))?;
    } else {
        // Write skill content directly (for embedded/local skills)
        fs::create_dir_all(&canonical_path)
            .with_context(|| format!("Failed to create skill directory: {:?}", canonical_path))?;

        let skill_file_path = canonical_path.join("SKILL.md");
        fs::write(&skill_file_path, &skill.raw_content)
            .with_context(|| format!("Failed to write skill file: {:?}", skill_file_path))?;

        // Write auxiliary files alongside SKILL.md
        for (rel_path, content) in &skill.auxiliary_files {
            // Security: reject absolute paths and path traversal outside skill root
            let rel = Path::new(rel_path);
            if rel.is_absolute() {
                bail!("Auxiliary file path must be relative, got: {:?}", rel_path);
            }
            // Normalize and ensure path stays within canonical_path
            let file_path = canonical_path.join(rel_path);
            let canonical_file = file_path
                .canonicalize()
                .unwrap_or_else(|_| file_path.clone());
            // Before the file exists we can't canonicalize it; check components instead
            for component in rel.components() {
                use std::path::Component;
                match component {
                    Component::ParentDir => {
                        bail!(
                            "Auxiliary file path must not traverse outside skill directory: {:?}",
                            rel_path
                        );
                    }
                    Component::RootDir => {
                        bail!(
                            "Auxiliary file path must not be rooted (contains root separator): {:?}",
                            rel_path
                        );
                    }
                    Component::Prefix(_) => {
                        bail!(
                            "Auxiliary file path must not contain a path prefix (e.g. drive letter): {:?}",
                            rel_path
                        );
                    }
                    _ => {}
                }
            }
            let _ = canonical_file; // used for future post-write checks
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent).with_context(|| {
                    format!(
                        "Failed to create directory for auxiliary file: {:?}",
                        parent
                    )
                })?;
            }
            fs::write(&file_path, content)
                .with_context(|| format!("Failed to write auxiliary file: {:?}", file_path))?;
        }
    }

    // Track if any symlink failed
    let mut any_symlink_failed = false;

    // Link or copy to target directories
    for target_dir in &config.target_dirs {
        let symlink_failed = link_or_copy_skill(&canonical_path, target_dir, &skill.name, config)?;
        any_symlink_failed = any_symlink_failed || symlink_failed;
    }

    Ok(InstallResult {
        path: canonical_path,
        symlink_failed: any_symlink_failed,
    })
}

/// Link or copy skill from canonical location to target directory
/// Returns true if symlink failed and fallback to copy was used
fn link_or_copy_skill(
    canonical_path: &Path,
    target_dir: &Path,
    skill_name: &str,
    config: &InstallConfig,
) -> Result<bool> {
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
                Ok(_) => Ok(false),
                Err(e) if config.fallback_to_copy => {
                    // Fallback to copy
                    copy_skill(canonical_path, &target_path)?;
                    Ok(true)
                }
                Err(e) => Err(e).context("Failed to create symlink"),
            }
        }
        InstallMode::Copy => {
            copy_skill(canonical_path, &target_path)?;
            Ok(false)
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
            auxiliary_files: Default::default(),
        }
    }

    #[test]
    fn test_install_skill_to_canonical() {
        let temp_dir = TempDir::new().unwrap();
        let canonical_dir = temp_dir.path().join(".agents/skills");

        let config = InstallConfig::new(canonical_dir.clone());
        let skill = create_test_skill();

        let result = install_skill(&skill, &config).unwrap();

        assert_eq!(result.path, canonical_dir.join("test-skill"));
        assert!(result.path.join("SKILL.md").exists());
        assert!(!result.symlink_failed);

        let content = fs::read_to_string(result.path.join("SKILL.md")).unwrap();
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
        let result = install_skill(&skill, &config).unwrap();

        let target_path = target_dir.join("test-skill");
        assert!(target_path.exists());

        // On systems that support symlinks, verify it's a symlink
        #[cfg(unix)]
        {
            let metadata = fs::symlink_metadata(&target_path).unwrap();
            assert!(metadata.file_type().is_symlink());
            assert!(!result.symlink_failed);
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
        let result = install_skill(&skill, &config).unwrap();

        let target_path = target_dir.join("test-skill");
        assert!(target_path.exists());
        assert!(target_path.join("SKILL.md").exists());
        assert!(!result.symlink_failed);

        let content = fs::read_to_string(target_path.join("SKILL.md")).unwrap();
        assert_eq!(content, skill.raw_content);
    }

    #[test]
    fn test_install_skill_with_auxiliary_files() {
        use std::collections::HashMap;

        let temp_dir = TempDir::new().unwrap();
        let canonical_dir = temp_dir.path().join(".agents/skills");

        let mut auxiliary_files = HashMap::new();
        auxiliary_files.insert(
            "scripts/helper.py".to_string(),
            "print('hello')".to_string(),
        );
        auxiliary_files.insert(
            "references/guide.md".to_string(),
            "# Guide\nContent".to_string(),
        );

        let skill = Skill {
            name: "multi-file-skill".to_string(),
            description: "Skill with auxiliary files".to_string(),
            path: None,
            raw_content: "---\nname: multi-file-skill\ndescription: Skill with auxiliary files\n---\n\n# Test"
                .to_string(),
            metadata: SkillMetadata::default(),
            auxiliary_files,
        };

        let config = InstallConfig::new(canonical_dir.clone());
        let result = install_skill(&skill, &config).unwrap();

        assert!(result.path.join("SKILL.md").exists());
        assert!(result.path.join("scripts/helper.py").exists());
        assert!(result.path.join("references/guide.md").exists());

        let helper_content = fs::read_to_string(result.path.join("scripts/helper.py")).unwrap();
        assert_eq!(helper_content, "print('hello')");

        let guide_content = fs::read_to_string(result.path.join("references/guide.md")).unwrap();
        assert_eq!(guide_content, "# Guide\nContent");
    }

    #[test]
    fn test_install_skill_rejects_absolute_auxiliary_path() {
        use std::collections::HashMap;

        let temp_dir = TempDir::new().unwrap();
        let canonical_dir = temp_dir.path().join(".agents/skills");

        let mut auxiliary_files = HashMap::new();
        auxiliary_files.insert("/etc/passwd".to_string(), "malicious".to_string());

        let skill = Skill {
            name: "bad-skill".to_string(),
            description: "Bad skill".to_string(),
            path: None,
            raw_content: "---\nname: bad-skill\ndescription: Bad skill\n---\n".to_string(),
            metadata: SkillMetadata::default(),
            auxiliary_files,
        };

        let config = InstallConfig::new(canonical_dir.clone());
        let result = install_skill(&skill, &config);
        assert!(
            result.is_err(),
            "Expected error for absolute auxiliary path"
        );
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("must be relative"),
            "Expected 'must be relative' in error, got: {err}"
        );
        // Ensure no file was written outside the skill directory
        assert!(!std::path::Path::new("/etc/passwd.malicious_test").exists());
    }

    #[test]
    fn test_install_skill_rejects_path_traversal_auxiliary() {
        use std::collections::HashMap;

        let temp_dir = TempDir::new().unwrap();
        let canonical_dir = temp_dir.path().join(".agents/skills");

        let mut auxiliary_files = HashMap::new();
        auxiliary_files.insert(
            "../../../outside/secret.txt".to_string(),
            "stolen".to_string(),
        );

        let skill = Skill {
            name: "traversal-skill".to_string(),
            description: "Traversal skill".to_string(),
            path: None,
            raw_content: "---\nname: traversal-skill\ndescription: Traversal skill\n---\n"
                .to_string(),
            metadata: SkillMetadata::default(),
            auxiliary_files,
        };

        let config = InstallConfig::new(canonical_dir.clone());
        let result = install_skill(&skill, &config);
        assert!(
            result.is_err(),
            "Expected error for path traversal auxiliary path"
        );
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("must not traverse outside"),
            "Expected traversal error, got: {err}"
        );
        // Confirm no files were written outside temp_dir
        let outside = temp_dir.path().parent().unwrap().join("outside/secret.txt");
        assert!(
            !outside.exists(),
            "File must not be written outside skill dir"
        );
    }

    #[test]
    fn test_install_skill_rejects_rooted_auxiliary_path() {
        // Component::RootDir: on Windows `\foo` is rooted but not absolute (no prefix).
        // We construct a path whose first component is RootDir by using std::path::PathBuf
        // from components so the test is meaningful on all platforms.
        use std::collections::HashMap;
        use std::path::{Component, PathBuf};

        let temp_dir = TempDir::new().unwrap();
        let canonical_dir = temp_dir.path().join(".agents/skills");

        // Build a path that begins with Component::RootDir
        let rooted: PathBuf = [Component::RootDir, Component::Normal("evil".as_ref())]
            .iter()
            .collect();
        let rooted_str = rooted.to_string_lossy().to_string();

        let mut auxiliary_files = HashMap::new();
        auxiliary_files.insert(rooted_str, "evil".to_string());

        let skill = Skill {
            name: "rooted-skill".to_string(),
            description: "Rooted path skill".to_string(),
            path: None,
            raw_content: "---\nname: rooted-skill\ndescription: Rooted path skill\n---\n"
                .to_string(),
            metadata: SkillMetadata::default(),
            auxiliary_files,
        };

        let config = InstallConfig::new(canonical_dir.clone());
        let result = install_skill(&skill, &config);
        // On Unix the rooted path is also absolute and caught by is_absolute(); on Windows it
        // is caught by the Component::RootDir check.  Either way installation must fail.
        assert!(
            result.is_err(),
            "Expected error for rooted auxiliary path (absolute or root component)"
        );
    }

    #[cfg(windows)]
    #[test]
    fn test_install_skill_rejects_prefixed_auxiliary_path() {
        // On Windows, `C:relative` has Component::Prefix but is NOT absolute (no root `/`).
        // is_absolute() returns false, so only the Component::Prefix check catches it.
        use std::collections::HashMap;

        let temp_dir = TempDir::new().unwrap();
        let canonical_dir = temp_dir.path().join(".agents/skills");

        let mut auxiliary_files = HashMap::new();
        // "C:relative" — prefix only, no root separator, not caught by is_absolute()
        auxiliary_files.insert("C:relative\\path".to_string(), "evil".to_string());

        let skill = Skill {
            name: "prefixed-skill".to_string(),
            description: "Prefixed path skill".to_string(),
            path: None,
            raw_content: "---\nname: prefixed-skill\ndescription: Prefixed path skill\n---\n"
                .to_string(),
            metadata: SkillMetadata::default(),
            auxiliary_files,
        };

        let config = InstallConfig::new(canonical_dir.clone());
        let result = install_skill(&skill, &config);
        assert!(
            result.is_err(),
            "Expected error for prefixed auxiliary path (Windows drive-relative)"
        );
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("path prefix"),
            "Expected 'path prefix' in error, got: {err}"
        );
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

        assert!(result.path.exists());
        assert!(result.path.join("SKILL.md").exists());
        assert!(!result.symlink_failed);
    }
}
