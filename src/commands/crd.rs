//! Implementation of the `kubegen add crd` command
//!
//! Adds a new CRD to an existing Kubernetes operator project.

use std::path::Path;

use tracing::{debug, info};

use crate::cli::CrdArgs;
use crate::error::Result;
use crate::fs::{
    check_conflicts, create_dir_all, format_conflicts, write_file_protected, DryRunContext,
    WriteOptions,
};
use crate::templates::{
    get_template_with_override, CrdContext, SimpleRenderer, StringTemplate, TemplateContext,
    TemplateRenderer,
};
use crate::validation;

/// Execute the `kubegen add crd` command
pub fn execute_add_crd(args: &CrdArgs, template_dir: Option<&Path>) -> Result<()> {
    // Validate CRD kind
    validation::validate_crd_kind(&args.kind)?;

    // Validate API version
    validation::validate_crd_version(&args.api_version)?;

    // Determine group - use provided or default to example.com
    let group = args
        .group
        .clone()
        .unwrap_or_else(|| "example.com".to_string());

    // Validate group
    validation::validate_crd_group(&group)?;

    // Validate we're in a kubegen project (Cargo.toml with kube dependency exists)
    validate_project_structure()?;

    // Get project name for template context
    let project_name = get_project_name()?;

    info!("Adding CRD: {}", args.kind);

    // Build CRD context
    let crd_ctx = CrdContext::builder()
        .group(&group)
        .version(&args.api_version)
        .kind(&args.kind)
        .with_controller(true)
        .with_status(true)
        .build()
        .ok_or_else(|| {
            crate::error::KubegenError::ValidationError("Failed to build CRD context".to_string())
        })?;

    let mut template_ctx = crd_ctx.to_template_context();
    template_ctx.set("project_name", &project_name);

    let crd_dir = Path::new("src").join(&crd_ctx.kind_snake);
    let examples_dir = Path::new("examples");
    let manifests_dir = Path::new("manifests");

    if args.dry_run {
        return execute_dry_run(
            &crd_dir,
            examples_dir,
            manifests_dir,
            &crd_ctx.kind_snake,
            &template_ctx,
            template_dir,
        );
    }

    let write_opts = WriteOptions::with_force(args.force);

    // Check for conflicts before proceeding
    let paths_to_create =
        get_paths_to_create(&crd_dir, examples_dir, manifests_dir, &crd_ctx.kind_snake);
    let conflicts = check_conflicts(&paths_to_create, &write_opts);
    if !conflicts.is_empty() {
        return Err(crate::error::KubegenError::ValidationError(
            format_conflicts(&conflicts),
        ));
    }

    // Create CRD module structure
    create_crd_structure(
        &crd_dir,
        examples_dir,
        manifests_dir,
        &crd_ctx.kind_snake,
        &template_ctx,
        &write_opts,
        template_dir,
    )?;

    info!("CRD '{}' added successfully!", args.kind);
    info!("Next steps:");
    info!("  1. Add 'mod {};' to src/lib.rs", crd_ctx.kind_snake);
    info!("  2. Update src/main.rs to start the controller");
    info!(
        "  3. Apply CRD: kubectl apply -f manifests/{}-crd.yaml",
        crd_ctx.kind_snake
    );
    info!(
        "  4. Apply example CR: kubectl apply -f examples/example-{}.yaml",
        crd_ctx.kind_snake
    );
    info!("  5. Run 'cargo build' to verify");

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

/// Get the project name from Cargo.toml
fn get_project_name() -> Result<String> {
    let cargo_toml = Path::new("Cargo.toml");
    let content = std::fs::read_to_string(cargo_toml).map_err(|e| {
        crate::error::KubegenError::ValidationError(format!("Failed to read Cargo.toml: {}", e))
    })?;

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("name") {
            if let Some(value) = line.split('=').nth(1) {
                let name = value.trim().trim_matches('"').trim_matches('\'');
                return Ok(name.to_string());
            }
        }
    }

    Err(crate::error::KubegenError::ValidationError(
        "Could not find package name in Cargo.toml".to_string(),
    ))
}

/// Get list of paths that will be created
fn get_paths_to_create(
    crd_dir: &Path,
    examples_dir: &Path,
    manifests_dir: &Path,
    kind_snake: &str,
) -> Vec<std::path::PathBuf> {
    vec![
        crd_dir.to_path_buf(),
        crd_dir.join("mod.rs"),
        crd_dir.join("types.rs"),
        crd_dir.join("controller.rs"),
        crd_dir.join("finalizer.rs"),
        crd_dir.join("status.rs"),
        examples_dir.to_path_buf(),
        examples_dir.join(format!("example-{}.yaml", kind_snake)),
        manifests_dir.to_path_buf(),
        manifests_dir.join(format!("{}-crd.yaml", kind_snake)),
    ]
}

/// Execute in dry-run mode
fn execute_dry_run(
    crd_dir: &Path,
    examples_dir: &Path,
    manifests_dir: &Path,
    kind_snake: &str,
    ctx: &TemplateContext,
    template_dir: Option<&Path>,
) -> Result<()> {
    let mut dry_run = DryRunContext::new();

    dry_run.plan_dir(crd_dir);
    dry_run.plan_dir(examples_dir);
    dry_run.plan_dir(manifests_dir);

    let renderer = SimpleRenderer::new();

    let mod_content = render_template(&renderer, "crd/mod.rs.tmpl", ctx, template_dir)?;
    dry_run.plan_file(crd_dir.join("mod.rs"), &mod_content);

    let types_content = render_template(&renderer, "crd/types.rs.tmpl", ctx, template_dir)?;
    dry_run.plan_file(crd_dir.join("types.rs"), &types_content);

    let controller_content =
        render_template(&renderer, "crd/controller.rs.tmpl", ctx, template_dir)?;
    dry_run.plan_file(crd_dir.join("controller.rs"), &controller_content);

    let finalizer_content = render_template(&renderer, "crd/finalizer.rs.tmpl", ctx, template_dir)?;
    dry_run.plan_file(crd_dir.join("finalizer.rs"), &finalizer_content);

    let status_content = render_template(&renderer, "crd/status.rs.tmpl", ctx, template_dir)?;
    dry_run.plan_file(crd_dir.join("status.rs"), &status_content);

    let example_content = render_template(&renderer, "crd/example.yaml.tmpl", ctx, template_dir)?;
    dry_run.plan_file(
        examples_dir.join(format!("example-{}.yaml", kind_snake)),
        &example_content,
    );

    let crd_manifest_content = render_template(&renderer, "crd/crd.yaml.tmpl", ctx, template_dir)?;
    dry_run.plan_file(
        manifests_dir.join(format!("{}-crd.yaml", kind_snake)),
        &crd_manifest_content,
    );

    println!("{}", dry_run.format_preview());
    Ok(())
}

/// Create the CRD module structure
fn create_crd_structure(
    crd_dir: &Path,
    examples_dir: &Path,
    manifests_dir: &Path,
    kind_snake: &str,
    ctx: &TemplateContext,
    opts: &WriteOptions,
    template_dir: Option<&Path>,
) -> Result<()> {
    let renderer = SimpleRenderer::new();

    // Create CRD directory
    debug!("Creating CRD directory: {}", crd_dir.display());
    create_dir_all(crd_dir)?;

    // Create examples directory
    debug!("Creating examples directory: {}", examples_dir.display());
    create_dir_all(examples_dir)?;

    // Create manifests directory
    debug!("Creating manifests directory: {}", manifests_dir.display());
    create_dir_all(manifests_dir)?;

    // Render and write mod.rs
    let mod_content = render_template(&renderer, "crd/mod.rs.tmpl", ctx, template_dir)?;
    debug!("Writing mod.rs");
    write_file_protected(crd_dir.join("mod.rs"), &mod_content, opts)?;

    // Render and write types.rs
    let types_content = render_template(&renderer, "crd/types.rs.tmpl", ctx, template_dir)?;
    debug!("Writing types.rs");
    write_file_protected(crd_dir.join("types.rs"), &types_content, opts)?;

    // Render and write controller.rs
    let controller_content =
        render_template(&renderer, "crd/controller.rs.tmpl", ctx, template_dir)?;
    debug!("Writing controller.rs");
    write_file_protected(crd_dir.join("controller.rs"), &controller_content, opts)?;

    // Render and write finalizer.rs
    let finalizer_content = render_template(&renderer, "crd/finalizer.rs.tmpl", ctx, template_dir)?;
    debug!("Writing finalizer.rs");
    write_file_protected(crd_dir.join("finalizer.rs"), &finalizer_content, opts)?;

    // Render and write status.rs
    let status_content = render_template(&renderer, "crd/status.rs.tmpl", ctx, template_dir)?;
    debug!("Writing status.rs");
    write_file_protected(crd_dir.join("status.rs"), &status_content, opts)?;

    // Render and write example CR YAML
    let example_content = render_template(&renderer, "crd/example.yaml.tmpl", ctx, template_dir)?;
    debug!("Writing example-{}.yaml", kind_snake);
    write_file_protected(
        examples_dir.join(format!("example-{}.yaml", kind_snake)),
        &example_content,
        opts,
    )?;

    // Render and write CRD manifest YAML
    let crd_manifest_content = render_template(&renderer, "crd/crd.yaml.tmpl", ctx, template_dir)?;
    debug!("Writing {}-crd.yaml", kind_snake);
    write_file_protected(
        manifests_dir.join(format!("{}-crd.yaml", kind_snake)),
        &crd_manifest_content,
        opts,
    )?;

    Ok(())
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

    fn make_args(kind: &str) -> CrdArgs {
        CrdArgs {
            kind: kind.to_string(),
            group: Some("example.com".to_string()),
            api_version: "v1alpha1".to_string(),
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
    fn test_execute_add_crd_creates_files() {
        let _lock = CWD_LOCK.lock().unwrap();
        let temp = TempDir::new().unwrap();
        setup_project(&temp);

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let result = execute_add_crd(&make_args("MyResource"), None);
        std::env::set_current_dir(&original_dir).unwrap();

        assert!(result.is_ok());
        assert!(temp.path().join("src/my_resource").exists());
        assert!(temp.path().join("src/my_resource/mod.rs").exists());
        assert!(temp.path().join("src/my_resource/types.rs").exists());
        assert!(temp.path().join("src/my_resource/controller.rs").exists());
        assert!(temp.path().join("src/my_resource/finalizer.rs").exists());
        assert!(temp.path().join("src/my_resource/status.rs").exists());
        // Check example CR YAML
        assert!(temp
            .path()
            .join("examples/example-my_resource.yaml")
            .exists());
        // Check CRD manifest YAML
        assert!(temp.path().join("manifests/my_resource-crd.yaml").exists());
    }

    #[test]
    fn test_execute_add_crd_dry_run() {
        let _lock = CWD_LOCK.lock().unwrap();
        let temp = TempDir::new().unwrap();
        setup_project(&temp);

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let mut args = make_args("DryRunResource");
        args.dry_run = true;
        let result = execute_add_crd(&args, None);

        std::env::set_current_dir(&original_dir).unwrap();

        assert!(result.is_ok());
        assert!(!temp.path().join("src/dry_run_resource").exists());
    }

    #[test]
    fn test_execute_add_crd_invalid_kind() {
        let args = CrdArgs {
            kind: "invalid-kind".to_string(), // Contains hyphen
            group: Some("example.com".to_string()),
            api_version: "v1alpha1".to_string(),
            dry_run: false,
            force: false,
        };

        let result = execute_add_crd(&args, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_add_crd_not_in_project() {
        let _lock = CWD_LOCK.lock().unwrap();
        let temp = TempDir::new().unwrap();
        // Don't create project structure

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let result = execute_add_crd(&make_args("MyResource"), None);

        std::env::set_current_dir(&original_dir).unwrap();

        assert!(result.is_err());
    }

    #[test]
    fn test_execute_add_crd_existing_without_force() {
        let _lock = CWD_LOCK.lock().unwrap();
        let temp = TempDir::new().unwrap();
        setup_project(&temp);

        // Create existing CRD directory
        let crd_dir = temp.path().join("src/my_resource");
        std::fs::create_dir_all(&crd_dir).unwrap();
        std::fs::write(crd_dir.join("mod.rs"), "existing").unwrap();

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let result = execute_add_crd(&make_args("MyResource"), None);

        std::env::set_current_dir(&original_dir).unwrap();

        assert!(result.is_err());
    }

    #[test]
    fn test_execute_add_crd_force_overwrites() {
        let _lock = CWD_LOCK.lock().unwrap();
        let temp = TempDir::new().unwrap();
        setup_project(&temp);

        // Create existing CRD directory
        let crd_dir = temp.path().join("src/my_resource");
        std::fs::create_dir_all(&crd_dir).unwrap();
        std::fs::write(crd_dir.join("mod.rs"), "old content").unwrap();

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let mut args = make_args("MyResource");
        args.force = true;
        let result = execute_add_crd(&args, None);

        std::env::set_current_dir(&original_dir).unwrap();

        assert!(result.is_ok());
        let content = std::fs::read_to_string(crd_dir.join("mod.rs")).unwrap();
        assert!(content.contains("MyResource"));
    }

    #[test]
    fn test_get_paths_to_create() {
        let crd_dir = Path::new("src/my_resource");
        let examples_dir = Path::new("examples");
        let manifests_dir = Path::new("manifests");
        let paths = get_paths_to_create(crd_dir, examples_dir, manifests_dir, "my_resource");

        assert!(paths.contains(&crd_dir.to_path_buf()));
        assert!(paths.contains(&crd_dir.join("mod.rs")));
        assert!(paths.contains(&crd_dir.join("types.rs")));
        assert!(paths.contains(&crd_dir.join("controller.rs")));
        assert!(paths.contains(&crd_dir.join("finalizer.rs")));
        assert!(paths.contains(&crd_dir.join("status.rs")));
        assert!(paths.contains(&examples_dir.to_path_buf()));
        assert!(paths.contains(&examples_dir.join("example-my_resource.yaml")));
        assert!(paths.contains(&manifests_dir.to_path_buf()));
        assert!(paths.contains(&manifests_dir.join("my_resource-crd.yaml")));
    }

    #[test]
    fn test_render_template() {
        let renderer = SimpleRenderer::new();
        let mut ctx = TemplateContext::new();
        ctx.set("kind", "TestResource");
        ctx.set("kind_snake", "test_resource");
        ctx.set("group", "example.com");
        ctx.set("version", "v1");

        let result = render_template(&renderer, "crd/mod.rs.tmpl", &ctx, None);
        assert!(result.is_ok());
        let content = result.unwrap();
        assert!(content.contains("TestResource"));
    }
}
