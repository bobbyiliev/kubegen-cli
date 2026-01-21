//! Implementation of the `kubegen add metrics` command
//!
//! Adds Prometheus metrics support to an existing Kubernetes operator project.

use std::path::Path;

use tracing::{debug, info};

use crate::cli::MetricsArgs;
use crate::error::Result;
use crate::fs::{
    check_conflicts, create_dir_all, format_conflicts, write_file_protected, DryRunContext,
    WriteOptions,
};
use crate::templates::{
    get_template, SimpleRenderer, StringTemplate, TemplateContext, TemplateRenderer,
};

/// Execute the `kubegen add metrics` command
pub fn execute_add_metrics(args: &MetricsArgs) -> Result<()> {
    // Validate we're in a kubegen project
    validate_project_structure()?;

    info!("Adding metrics support (port: {})", args.port);

    // Build template context
    let mut ctx = TemplateContext::new();
    ctx.set("metrics_port", args.port.to_string());

    let metrics_dir = Path::new("src").join("metrics");

    if args.dry_run {
        return execute_dry_run(&metrics_dir, &ctx);
    }

    let write_opts = WriteOptions::with_force(args.force);

    // Check for conflicts before proceeding
    let paths_to_create = get_paths_to_create(&metrics_dir);
    let conflicts = check_conflicts(&paths_to_create, &write_opts);
    if !conflicts.is_empty() {
        return Err(crate::error::KubegenError::ValidationError(
            format_conflicts(&conflicts),
        ));
    }

    // Create metrics module structure
    create_metrics_structure(&metrics_dir, &ctx, &write_opts)?;

    info!("Metrics support added successfully!");
    info!("Next steps:");
    info!("  1. Add 'pub mod metrics;' to src/lib.rs");
    info!("  2. Call metrics::init() in main.rs before starting controllers");
    info!("  3. Run 'cargo build' to verify");

    Ok(())
}

/// Validate that we're in a kubegen project directory
fn validate_project_structure() -> Result<()> {
    let cargo_toml = Path::new("Cargo.toml");
    if !cargo_toml.exists() {
        return Err(crate::error::KubegenError::ValidationError(
            "Not in a Rust project directory (Cargo.toml not found)".to_string(),
        ));
    }

    let src_dir = Path::new("src");
    if !src_dir.exists() {
        return Err(crate::error::KubegenError::ValidationError(
            "Not in a valid project directory (src/ not found)".to_string(),
        ));
    }

    Ok(())
}

/// Get list of paths that will be created
fn get_paths_to_create(metrics_dir: &Path) -> Vec<std::path::PathBuf> {
    vec![metrics_dir.to_path_buf(), metrics_dir.join("mod.rs")]
}

/// Execute in dry-run mode
fn execute_dry_run(metrics_dir: &Path, ctx: &TemplateContext) -> Result<()> {
    let mut dry_run = DryRunContext::new();

    dry_run.plan_dir(metrics_dir);

    let renderer = SimpleRenderer::new();

    let mod_content = render_template(&renderer, "metrics/mod.rs.tmpl", ctx)?;
    dry_run.plan_file(metrics_dir.join("mod.rs"), &mod_content);

    println!("{}", dry_run.format_preview());
    Ok(())
}

/// Create the metrics module structure
fn create_metrics_structure(
    metrics_dir: &Path,
    ctx: &TemplateContext,
    opts: &WriteOptions,
) -> Result<()> {
    let renderer = SimpleRenderer::new();

    // Create metrics directory
    debug!("Creating metrics directory: {}", metrics_dir.display());
    create_dir_all(metrics_dir)?;

    // Render and write mod.rs
    let mod_content = render_template(&renderer, "metrics/mod.rs.tmpl", ctx)?;
    debug!("Writing mod.rs");
    write_file_protected(metrics_dir.join("mod.rs"), &mod_content, opts)?;

    Ok(())
}

/// Helper to render a template by path
fn render_template(
    renderer: &SimpleRenderer,
    template_path: &str,
    ctx: &TemplateContext,
) -> Result<String> {
    let template_content = get_template(template_path)?;
    let template = StringTemplate::new(template_path, &template_content);
    renderer.render(&template, ctx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::test_utils::CWD_LOCK;
    use tempfile::TempDir;

    fn make_args() -> MetricsArgs {
        MetricsArgs {
            port: 8080,
            dry_run: false,
            force: false,
        }
    }

    fn setup_project(temp: &TempDir) {
        // Create minimal project structure
        std::fs::create_dir_all(temp.path().join("src")).unwrap();
        std::fs::write(
            temp.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"",
        )
        .unwrap();
    }

    #[test]
    fn test_execute_add_metrics_creates_files() {
        let _lock = CWD_LOCK.lock().unwrap();
        let temp = TempDir::new().unwrap();
        setup_project(&temp);

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let result = execute_add_metrics(&make_args());
        std::env::set_current_dir(&original_dir).unwrap();

        assert!(result.is_ok());
        assert!(temp.path().join("src/metrics").exists());
        assert!(temp.path().join("src/metrics/mod.rs").exists());
    }

    #[test]
    fn test_execute_add_metrics_dry_run() {
        let _lock = CWD_LOCK.lock().unwrap();
        let temp = TempDir::new().unwrap();
        setup_project(&temp);

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let mut args = make_args();
        args.dry_run = true;
        let result = execute_add_metrics(&args);

        std::env::set_current_dir(&original_dir).unwrap();

        assert!(result.is_ok());
        assert!(!temp.path().join("src/metrics").exists());
    }

    #[test]
    fn test_execute_add_metrics_not_in_project() {
        let _lock = CWD_LOCK.lock().unwrap();
        let temp = TempDir::new().unwrap();
        // Don't create project structure

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let result = execute_add_metrics(&make_args());

        std::env::set_current_dir(&original_dir).unwrap();

        assert!(result.is_err());
    }

    #[test]
    fn test_execute_add_metrics_existing_without_force() {
        let _lock = CWD_LOCK.lock().unwrap();
        let temp = TempDir::new().unwrap();
        setup_project(&temp);

        // Create existing metrics directory
        let metrics_dir = temp.path().join("src/metrics");
        std::fs::create_dir_all(&metrics_dir).unwrap();
        std::fs::write(metrics_dir.join("mod.rs"), "existing").unwrap();

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let result = execute_add_metrics(&make_args());

        std::env::set_current_dir(&original_dir).unwrap();

        assert!(result.is_err());
    }

    #[test]
    fn test_execute_add_metrics_force_overwrites() {
        let _lock = CWD_LOCK.lock().unwrap();
        let temp = TempDir::new().unwrap();
        setup_project(&temp);

        // Create existing metrics directory
        let metrics_dir = temp.path().join("src/metrics");
        std::fs::create_dir_all(&metrics_dir).unwrap();
        std::fs::write(metrics_dir.join("mod.rs"), "old content").unwrap();

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let mut args = make_args();
        args.force = true;
        let result = execute_add_metrics(&args);

        std::env::set_current_dir(&original_dir).unwrap();

        assert!(result.is_ok());
        let content = std::fs::read_to_string(metrics_dir.join("mod.rs")).unwrap();
        assert!(content.contains("prometheus") || content.contains("metrics"));
    }

    #[test]
    fn test_execute_add_metrics_custom_port() {
        let _lock = CWD_LOCK.lock().unwrap();
        let temp = TempDir::new().unwrap();
        setup_project(&temp);

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let mut args = make_args();
        args.port = 9090;
        let result = execute_add_metrics(&args);

        std::env::set_current_dir(&original_dir).unwrap();

        assert!(result.is_ok());
        let content = std::fs::read_to_string(temp.path().join("src/metrics/mod.rs")).unwrap();
        assert!(content.contains("9090"));
    }

    #[test]
    fn test_get_paths_to_create() {
        let metrics_dir = Path::new("src/metrics");
        let paths = get_paths_to_create(metrics_dir);

        assert!(paths.contains(&metrics_dir.to_path_buf()));
        assert!(paths.contains(&metrics_dir.join("mod.rs")));
    }
}
