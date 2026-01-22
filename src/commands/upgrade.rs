//! Implementation of the `kubegen upgrade` command
//!
//! Upgrades project files to match current kubegen templates.

use std::path::Path;

use tracing::{debug, info, warn};

use crate::cli::UpgradeArgs;
use crate::error::{KubegenError, Result};
use crate::fs::{
    dir_exists, file_exists, read_to_string, write_file_protected, DryRunContext, WriteOptions,
};
use crate::templates::{
    get_template_with_override, SimpleRenderer, StringTemplate, TemplateContext, TemplateRenderer,
};

/// Files that can be upgraded in a kubegen project
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpgradeableComponent {
    /// Core project files (Makefile, .gitignore, error.rs)
    Project,
}

/// Execute the `kubegen upgrade` command
pub fn execute_upgrade(args: &UpgradeArgs, template_dir: Option<&Path>) -> Result<()> {
    // Validate we're in a kubegen project
    validate_project_structure()?;

    info!("Checking for upgradeable files...");

    // Detect project context from Cargo.toml
    let project_ctx = detect_project_context()?;

    // Determine which components to upgrade
    let components = if let Some(ref only) = args.only {
        parse_component_filter(only)?
    } else {
        vec![UpgradeableComponent::Project]
    };

    if args.dry_run {
        return execute_dry_run(&project_ctx, &components, template_dir);
    }

    let write_opts = WriteOptions::with_force(args.force);

    // Perform the upgrade
    let upgraded = perform_upgrade(&project_ctx, &components, &write_opts, template_dir)?;

    if upgraded == 0 {
        info!("No files needed upgrading. Project is up to date.");
    } else {
        info!("Upgraded {} file(s) successfully!", upgraded);
        info!("Review the changes and run 'cargo build' to verify.");
    }

    Ok(())
}

/// Validate that we're in a kubegen project directory
fn validate_project_structure() -> Result<()> {
    if !file_exists("Cargo.toml") {
        return Err(KubegenError::ValidationError(
            "Not in a Rust project directory (no Cargo.toml found)".to_string(),
        ));
    }

    if !dir_exists("src") {
        return Err(KubegenError::ValidationError(
            "Not in a valid project directory (no src/ directory found)".to_string(),
        ));
    }

    Ok(())
}

/// Detect project context from existing Cargo.toml
fn detect_project_context() -> Result<TemplateContext> {
    let cargo_content = read_to_string("Cargo.toml")?;

    // Extract project name from Cargo.toml
    let project_name = extract_cargo_field(&cargo_content, "name").ok_or_else(|| {
        KubegenError::ValidationError("Could not extract project name from Cargo.toml".to_string())
    })?;

    debug!("Detected project name: {}", project_name);

    let mut ctx = TemplateContext::new();
    ctx.set("project_name", &project_name);

    // Try to extract domain from existing code or use default
    let domain = detect_domain().unwrap_or_else(|| "example.com".to_string());
    ctx.set("domain", &domain);
    ctx.set("group", &domain);

    Ok(ctx)
}

/// Extract a field value from Cargo.toml content
fn extract_cargo_field(content: &str, field: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(&format!("{} =", field)) || trimmed.starts_with(&format!("{}=", field))
        {
            // Extract the value between quotes
            if let Some(start) = trimmed.find('"') {
                if let Some(end) = trimmed[start + 1..].find('"') {
                    return Some(trimmed[start + 1..start + 1 + end].to_string());
                }
            }
        }
    }
    None
}

/// Try to detect the domain from existing project files
fn detect_domain() -> Option<String> {
    // Try to find domain in existing CRD files or lib.rs
    if let Ok(content) = read_to_string("src/lib.rs") {
        // Look for domain patterns like "example.com" in group definitions
        for line in content.lines() {
            if line.contains("group =") || line.contains("group=") {
                if let Some(start) = line.find('"') {
                    if let Some(end) = line[start + 1..].find('"') {
                        let value = &line[start + 1..start + 1 + end];
                        // Check if it looks like a domain
                        if value.contains('.') && !value.contains('/') {
                            return Some(value.to_string());
                        }
                    }
                }
            }
        }
    }
    None
}

/// Parse the --only filter into components
fn parse_component_filter(filter: &str) -> Result<Vec<UpgradeableComponent>> {
    match filter.to_lowercase().as_str() {
        "project" => Ok(vec![UpgradeableComponent::Project]),
        _ => Err(KubegenError::ValidationError(format!(
            "Unknown component '{}'. Valid components: project",
            filter
        ))),
    }
}

/// Execute in dry-run mode
fn execute_dry_run(
    ctx: &TemplateContext,
    components: &[UpgradeableComponent],
    template_dir: Option<&Path>,
) -> Result<()> {
    let mut dry_run = DryRunContext::new();
    let renderer = SimpleRenderer::new();

    for component in components {
        match component {
            UpgradeableComponent::Project => {
                plan_project_upgrades(&mut dry_run, &renderer, ctx, template_dir)?;
            }
        }
    }

    let preview = dry_run.format_preview();
    if preview == "No changes planned." {
        info!("No upgrades available. Project files match current templates.");
    } else {
        println!("{}", preview);
    }

    Ok(())
}

/// Plan project file upgrades for dry-run
fn plan_project_upgrades(
    dry_run: &mut DryRunContext,
    renderer: &SimpleRenderer,
    ctx: &TemplateContext,
    template_dir: Option<&Path>,
) -> Result<()> {
    // Check each upgradeable project file
    let upgradeable_files = [
        ("Makefile", "project/Makefile.tmpl"),
        (".gitignore", "project/gitignore.tmpl"),
        ("src/error.rs", "project/error.rs.tmpl"),
    ];

    for (file_path, template_path) in &upgradeable_files {
        if file_exists(file_path) {
            let current_content = read_to_string(file_path)?;
            let new_content = render_template(renderer, template_path, ctx, template_dir)?;

            if current_content != new_content {
                dry_run.plan_file(file_path, &new_content);
            }
        }
    }

    Ok(())
}

/// Perform the actual upgrade
fn perform_upgrade(
    ctx: &TemplateContext,
    components: &[UpgradeableComponent],
    opts: &WriteOptions,
    template_dir: Option<&Path>,
) -> Result<usize> {
    let renderer = SimpleRenderer::new();
    let mut upgraded_count = 0;

    for component in components {
        match component {
            UpgradeableComponent::Project => {
                upgraded_count += upgrade_project_files(&renderer, ctx, opts, template_dir)?;
            }
        }
    }

    Ok(upgraded_count)
}

/// Upgrade project files
fn upgrade_project_files(
    renderer: &SimpleRenderer,
    ctx: &TemplateContext,
    _opts: &WriteOptions,
    template_dir: Option<&Path>,
) -> Result<usize> {
    let mut upgraded = 0;

    let upgradeable_files = [
        ("Makefile", "project/Makefile.tmpl"),
        (".gitignore", "project/gitignore.tmpl"),
        ("src/error.rs", "project/error.rs.tmpl"),
    ];

    for (file_path, template_path) in &upgradeable_files {
        if file_exists(file_path) {
            let current_content = read_to_string(file_path)?;
            let new_content = render_template(renderer, template_path, ctx, template_dir)?;

            if current_content != new_content {
                debug!("Upgrading {}", file_path);

                // Always use force for upgrades since we're updating existing files
                let upgrade_opts = WriteOptions::with_force(true);
                write_file_protected(file_path, &new_content, &upgrade_opts)?;

                info!("Upgraded: {}", file_path);
                upgraded += 1;
            } else {
                debug!("{} is already up to date", file_path);
            }
        } else {
            warn!("{} not found, skipping", file_path);
        }
    }

    Ok(upgraded)
}

/// Helper to render a template by path
fn render_template(
    renderer: &SimpleRenderer,
    template_path: &str,
    ctx: &TemplateContext,
    template_dir: Option<&Path>,
) -> Result<String> {
    let template_content = get_template_with_override(template_path, template_dir)?;
    let template = StringTemplate::new(template_path, &template_content);
    renderer.render(&template, ctx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::test_utils::CWD_LOCK;
    use tempfile::TempDir;

    fn setup_project(temp: &TempDir) {
        let src_dir = temp.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(
            temp.path().join("Cargo.toml"),
            r#"[package]
name = "test-operator"
version = "0.1.0"
"#,
        )
        .unwrap();
        std::fs::write(src_dir.join("main.rs"), "fn main() {}").unwrap();
        std::fs::write(src_dir.join("lib.rs"), "").unwrap();
    }

    fn make_args() -> UpgradeArgs {
        UpgradeArgs {
            dry_run: false,
            force: false,
            only: None,
        }
    }

    #[test]
    fn test_validate_project_structure_valid() {
        let _lock = CWD_LOCK.lock().unwrap();
        let temp = TempDir::new().unwrap();
        setup_project(&temp);

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let result = validate_project_structure();
        std::env::set_current_dir(&original_dir).unwrap();

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_project_structure_no_cargo() {
        let _lock = CWD_LOCK.lock().unwrap();
        let temp = TempDir::new().unwrap();
        std::fs::create_dir_all(temp.path().join("src")).unwrap();

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let result = validate_project_structure();
        std::env::set_current_dir(&original_dir).unwrap();

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Cargo.toml"));
    }

    #[test]
    fn test_validate_project_structure_no_src() {
        let _lock = CWD_LOCK.lock().unwrap();
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("Cargo.toml"), "[package]").unwrap();

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let result = validate_project_structure();
        std::env::set_current_dir(&original_dir).unwrap();

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("src/"));
    }

    #[test]
    fn test_extract_cargo_field_name() {
        let content = r#"[package]
name = "my-operator"
version = "0.1.0"
"#;
        let name = extract_cargo_field(content, "name");
        assert_eq!(name, Some("my-operator".to_string()));
    }

    #[test]
    fn test_extract_cargo_field_version() {
        let content = r#"[package]
name = "my-operator"
version = "0.1.0"
"#;
        let version = extract_cargo_field(content, "version");
        assert_eq!(version, Some("0.1.0".to_string()));
    }

    #[test]
    fn test_extract_cargo_field_not_found() {
        let content = r#"[package]
name = "my-operator"
"#;
        let edition = extract_cargo_field(content, "edition");
        assert!(edition.is_none());
    }

    #[test]
    fn test_parse_component_filter_project() {
        let result = parse_component_filter("project");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![UpgradeableComponent::Project]);
    }

    #[test]
    fn test_parse_component_filter_case_insensitive() {
        let result = parse_component_filter("PROJECT");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![UpgradeableComponent::Project]);
    }

    #[test]
    fn test_parse_component_filter_invalid() {
        let result = parse_component_filter("invalid");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown component"));
    }

    #[test]
    fn test_detect_project_context() {
        let _lock = CWD_LOCK.lock().unwrap();
        let temp = TempDir::new().unwrap();
        setup_project(&temp);

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let result = detect_project_context();
        std::env::set_current_dir(&original_dir).unwrap();

        assert!(result.is_ok());
        let ctx = result.unwrap();
        assert_eq!(ctx.get("project_name"), Some(&"test-operator".to_string()));
    }

    #[test]
    fn test_execute_upgrade_dry_run() {
        let _lock = CWD_LOCK.lock().unwrap();
        let temp = TempDir::new().unwrap();
        setup_project(&temp);

        // Create a Makefile that differs from template
        std::fs::write(temp.path().join("Makefile"), "# Old Makefile").unwrap();

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let mut args = make_args();
        args.dry_run = true;
        let result = execute_upgrade(&args, None);

        std::env::set_current_dir(&original_dir).unwrap();

        assert!(result.is_ok());
        // File should not be changed in dry-run mode
        let content = std::fs::read_to_string(temp.path().join("Makefile")).unwrap();
        assert_eq!(content, "# Old Makefile");
    }

    #[test]
    fn test_execute_upgrade_updates_makefile() {
        let _lock = CWD_LOCK.lock().unwrap();
        let temp = TempDir::new().unwrap();
        setup_project(&temp);

        // Create a Makefile that differs from template
        std::fs::write(temp.path().join("Makefile"), "# Old Makefile").unwrap();

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let result = execute_upgrade(&make_args(), None);

        std::env::set_current_dir(&original_dir).unwrap();

        assert!(result.is_ok());
        let content = std::fs::read_to_string(temp.path().join("Makefile")).unwrap();
        // Should have been upgraded to new template
        assert!(content.contains("test-operator"));
    }

    #[test]
    fn test_execute_upgrade_not_in_project() {
        let _lock = CWD_LOCK.lock().unwrap();
        let temp = TempDir::new().unwrap();

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let result = execute_upgrade(&make_args(), None);

        std::env::set_current_dir(&original_dir).unwrap();

        assert!(result.is_err());
    }

    #[test]
    fn test_execute_upgrade_with_only_filter() {
        let _lock = CWD_LOCK.lock().unwrap();
        let temp = TempDir::new().unwrap();
        setup_project(&temp);

        std::fs::write(temp.path().join("Makefile"), "# Old").unwrap();

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let mut args = make_args();
        args.only = Some("project".to_string());
        let result = execute_upgrade(&args, None);

        std::env::set_current_dir(&original_dir).unwrap();

        assert!(result.is_ok());
    }

    #[test]
    fn test_upgradeable_component_equality() {
        assert_eq!(UpgradeableComponent::Project, UpgradeableComponent::Project);
    }

    #[test]
    fn test_upgradeable_component_clone() {
        let comp = UpgradeableComponent::Project;
        let cloned = comp;
        assert_eq!(comp, cloned);
    }

    #[test]
    fn test_upgradeable_component_debug() {
        let comp = UpgradeableComponent::Project;
        let debug_str = format!("{:?}", comp);
        assert!(debug_str.contains("Project"));
    }
}
