//! `plix mod new` command - create a new mod project from template

use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;
use tracing::info;

/// Available template names
const TEMPLATES: &[&str] = &["chat-filter", "world-query", "timers-net"];

/// Run the `new` command
pub fn run(name: &str, template: &str, output: Option<&Path>) -> Result<()> {
    // Validate mod ID
    validate_mod_id(name)?;

    // Check template exists
    if !TEMPLATES.contains(&template) {
        bail!(
            "Unknown template '{}'. Available templates: {}",
            template,
            TEMPLATES.join(", ")
        );
    }

    // Determine output directory
    let output_dir = output.map(|p| p.to_path_buf()).unwrap_or_else(|| {
        std::env::current_dir()
            .unwrap_or_default()
            .join(name)
    });

    if output_dir.exists() {
        bail!("Output directory already exists: {}", output_dir.display());
    }

    info!("Creating mod '{}' from template '{}'", name, template);

    // Find templates directory (relative to executable or in repo)
    let templates_dir = find_templates_dir()?;
    let template_path = templates_dir.join(template);

    if !template_path.exists() {
        bail!(
            "Template directory not found: {}",
            template_path.display()
        );
    }

    // Copy template files
    copy_template(&template_path, &output_dir, name)?;

    info!("Created mod project at {}", output_dir.display());
    println!("\nNext steps:");
    println!("  cd {}", name);
    println!("  plix-mod build");
    println!("  plix-mod pack");

    Ok(())
}

/// Validate mod ID format
fn validate_mod_id(id: &str) -> Result<()> {
    if id.len() < 3 {
        bail!("Mod ID must be at least 3 characters");
    }
    if id.len() > 64 {
        bail!("Mod ID must be at most 64 characters");
    }

    // Must be lowercase alphanumeric + hyphens
    for (i, c) in id.chars().enumerate() {
        if !c.is_ascii_lowercase() && !c.is_ascii_digit() && c != '-' {
            bail!(
                "Mod ID must contain only lowercase letters, digits, and hyphens (invalid char '{}' at position {})",
                c,
                i
            );
        }
    }

    // Cannot start or end with hyphen
    if id.starts_with('-') || id.ends_with('-') {
        bail!("Mod ID cannot start or end with a hyphen");
    }

    // Cannot have consecutive hyphens
    if id.contains("--") {
        bail!("Mod ID cannot contain consecutive hyphens");
    }

    Ok(())
}

/// Find the templates directory
fn find_templates_dir() -> Result<std::path::PathBuf> {
    // Try relative to current directory (for development)
    let dev_path = std::env::current_dir()?.join("templates/mods");
    if dev_path.exists() {
        return Ok(dev_path);
    }

    // Try relative to executable
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            // Check ../templates/mods (installed layout)
            let installed_path = exe_dir.join("../templates/mods");
            if installed_path.exists() {
                return Ok(installed_path);
            }

            // Check share directory
            let share_path = exe_dir.join("../share/plix/templates/mods");
            if share_path.exists() {
                return Ok(share_path);
            }
        }
    }

    // Check environment variable
    if let Ok(path) = std::env::var("PLIX_MOD_TEMPLATES") {
        let env_path = std::path::PathBuf::from(path);
        if env_path.exists() {
            return Ok(env_path);
        }
    }

    bail!(
        "Could not find templates directory. Set PLIX_MOD_TEMPLATES environment variable or run from repository root."
    )
}

/// Copy template files, replacing placeholders
fn copy_template(src: &Path, dst: &Path, mod_name: &str) -> Result<()> {
    fs::create_dir_all(dst)?;

    for entry in walkdir::WalkDir::new(src) {
        let entry = entry?;
        let src_path = entry.path();
        let relative = src_path.strip_prefix(src)?;
        let dst_path = dst.join(relative);

        if entry.file_type().is_dir() {
            fs::create_dir_all(&dst_path)?;
        } else if entry.file_type().is_file() {
            // Read content and replace placeholders
            let content = fs::read_to_string(src_path)
                .with_context(|| format!("Failed to read {}", src_path.display()))?;

            let replaced = replace_placeholders(&content, mod_name);

            // Create parent directories if needed
            if let Some(parent) = dst_path.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::write(&dst_path, replaced)
                .with_context(|| format!("Failed to write {}", dst_path.display()))?;
        }
    }

    Ok(())
}

/// Replace template placeholders with actual values
fn replace_placeholders(content: &str, mod_name: &str) -> String {
    // Convert mod-name to mod_name for Rust identifiers
    let rust_name = mod_name.replace('-', "_");

    content
        .replace("{{mod_id}}", mod_name)
        .replace("{{mod_name}}", mod_name)
        .replace("{{rust_name}}", &rust_name)
        .replace("{{MOD_ID}}", mod_name)
        .replace("{{MOD_NAME}}", mod_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_mod_id_valid() {
        assert!(validate_mod_id("my-mod").is_ok());
        assert!(validate_mod_id("mod123").is_ok());
        assert!(validate_mod_id("my-cool-mod").is_ok());
    }

    #[test]
    fn test_validate_mod_id_too_short() {
        assert!(validate_mod_id("ab").is_err());
    }

    #[test]
    fn test_validate_mod_id_too_long() {
        let long_id = "a".repeat(65);
        assert!(validate_mod_id(&long_id).is_err());
    }

    #[test]
    fn test_validate_mod_id_uppercase() {
        assert!(validate_mod_id("MyMod").is_err());
    }

    #[test]
    fn test_validate_mod_id_hyphen_edges() {
        assert!(validate_mod_id("-mod").is_err());
        assert!(validate_mod_id("mod-").is_err());
        assert!(validate_mod_id("mod--name").is_err());
    }

    #[test]
    fn test_replace_placeholders() {
        let content = "name = \"{{mod_id}}\"\nmod {{rust_name}}";
        let result = replace_placeholders(content, "my-mod");
        assert_eq!(result, "name = \"my-mod\"\nmod my_mod");
    }
}
