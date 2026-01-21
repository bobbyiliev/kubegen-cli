//! Implementation of the `kubegen new` command
//!
//! Creates a new Kubernetes operator project with the basic structure.

use std::path::Path;

use tracing::{debug, info};

use crate::cli::NewArgs;
use crate::error::Result;
use crate::fs::{
    check_conflicts, create_dir_all, format_conflicts, write_file_protected, DryRunContext,
    WriteOptions,
};
use crate::templates::{
    get_template, ProjectContext, SimpleRenderer, StringTemplate, TemplateContext, TemplateRenderer,
};
use crate::validation;

/// Execute the `kubegen new` command
pub fn execute_new(args: &NewArgs) -> Result<()> {
    // Validate project name
    validation::validate_project_name(&args.name)?;

    info!("Creating new operator project: {}", args.name);

    // Build project context
    let project_ctx = ProjectContext::builder()
        .name(&args.name)
        .group(&args.domain)
        .version("v1alpha1")
        .kind("Sample") // Default kind, user can add CRDs later
        .domain(&args.domain)
        .build()
        .ok_or_else(|| {
            crate::error::KubegenError::ValidationError(
                "Failed to build project context".to_string(),
            )
        })?;

    let template_ctx = project_ctx.to_template_context();
    let project_dir = Path::new(&args.name);

    if args.dry_run {
        return execute_dry_run(project_dir, &template_ctx);
    }

    let write_opts = WriteOptions::with_force(args.force);

    // Check for conflicts before proceeding
    let paths_to_create = get_paths_to_create(project_dir);
    let conflicts = check_conflicts(&paths_to_create, &write_opts);
    if !conflicts.is_empty() {
        return Err(crate::error::KubegenError::ValidationError(
            format_conflicts(&conflicts),
        ));
    }

    // Create project structure
    create_project_structure(project_dir, &template_ctx, &write_opts)?;

    info!("Project '{}' created successfully!", args.name);
    info!("Next steps:");
    info!("  cd {}", args.name);
    info!("  cargo build");
    info!("  kubegen add crd MyResource --group {}", args.domain);

    Ok(())
}

/// Get list of paths that will be created
fn get_paths_to_create(project_dir: &Path) -> Vec<std::path::PathBuf> {
    vec![
        project_dir.to_path_buf(),
        project_dir.join("src"),
        project_dir.join("Cargo.toml"),
        project_dir.join("README.md"),
        project_dir.join("Makefile"),
        project_dir.join(".gitignore"),
        project_dir.join("src/main.rs"),
        project_dir.join("src/lib.rs"),
        project_dir.join("src/error.rs"),
    ]
}

/// Execute in dry-run mode
fn execute_dry_run(project_dir: &Path, ctx: &TemplateContext) -> Result<()> {
    let mut dry_run = DryRunContext::new();

    dry_run.plan_dir(project_dir);
    dry_run.plan_dir(project_dir.join("src"));

    // Render templates and plan file creation
    let renderer = SimpleRenderer::new();

    let cargo_content = render_template(&renderer, "project/Cargo.toml.tmpl", ctx)?;
    dry_run.plan_file(project_dir.join("Cargo.toml"), &cargo_content);

    let readme_content = render_template(&renderer, "project/README.md.tmpl", ctx)?;
    dry_run.plan_file(project_dir.join("README.md"), &readme_content);

    let makefile_content = render_template(&renderer, "project/Makefile.tmpl", ctx)?;
    dry_run.plan_file(project_dir.join("Makefile"), &makefile_content);

    let gitignore_content = render_template(&renderer, "project/gitignore.tmpl", ctx)?;
    dry_run.plan_file(project_dir.join(".gitignore"), &gitignore_content);

    let main_content = render_template(&renderer, "project/main.rs.tmpl", ctx)?;
    dry_run.plan_file(project_dir.join("src/main.rs"), &main_content);

    let lib_content = render_template(&renderer, "project/lib.rs.tmpl", ctx)?;
    dry_run.plan_file(project_dir.join("src/lib.rs"), &lib_content);

    let error_content = render_template(&renderer, "project/error.rs.tmpl", ctx)?;
    dry_run.plan_file(project_dir.join("src/error.rs"), &error_content);

    println!("{}", dry_run.format_preview());
    Ok(())
}

/// Create the actual project structure
fn create_project_structure(
    project_dir: &Path,
    ctx: &TemplateContext,
    opts: &WriteOptions,
) -> Result<()> {
    let renderer = SimpleRenderer::new();

    // Create directories
    debug!("Creating project directory: {}", project_dir.display());
    create_dir_all(project_dir)?;
    create_dir_all(project_dir.join("src"))?;

    // Render and write Cargo.toml
    let cargo_content = render_template(&renderer, "project/Cargo.toml.tmpl", ctx)?;
    debug!("Writing Cargo.toml");
    write_file_protected(project_dir.join("Cargo.toml"), &cargo_content, opts)?;

    // Render and write README.md
    let readme_content = render_template(&renderer, "project/README.md.tmpl", ctx)?;
    debug!("Writing README.md");
    write_file_protected(project_dir.join("README.md"), &readme_content, opts)?;

    // Render and write Makefile
    let makefile_content = render_template(&renderer, "project/Makefile.tmpl", ctx)?;
    debug!("Writing Makefile");
    write_file_protected(project_dir.join("Makefile"), &makefile_content, opts)?;

    // Render and write .gitignore
    let gitignore_content = render_template(&renderer, "project/gitignore.tmpl", ctx)?;
    debug!("Writing .gitignore");
    write_file_protected(project_dir.join(".gitignore"), &gitignore_content, opts)?;

    // Render and write main.rs
    let main_content = render_template(&renderer, "project/main.rs.tmpl", ctx)?;
    debug!("Writing src/main.rs");
    write_file_protected(project_dir.join("src/main.rs"), &main_content, opts)?;

    // Render and write lib.rs
    let lib_content = render_template(&renderer, "project/lib.rs.tmpl", ctx)?;
    debug!("Writing src/lib.rs");
    write_file_protected(project_dir.join("src/lib.rs"), &lib_content, opts)?;

    // Render and write error.rs
    let error_content = render_template(&renderer, "project/error.rs.tmpl", ctx)?;
    debug!("Writing src/error.rs");
    write_file_protected(project_dir.join("src/error.rs"), &error_content, opts)?;

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

    fn make_args(name: &str) -> NewArgs {
        NewArgs {
            name: name.to_string(),
            domain: "example.com".to_string(),
            non_interactive: true,
            dry_run: false,
            force: false,
        }
    }

    #[test]
    fn test_execute_new_creates_project() {
        let _lock = CWD_LOCK.lock().unwrap();
        let temp = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let result = execute_new(&make_args("test-operator"));

        // Restore directory before assertions
        std::env::set_current_dir(&original_dir).unwrap();

        assert!(result.is_ok());
        assert!(temp.path().join("test-operator").exists());
        assert!(temp.path().join("test-operator/Cargo.toml").exists());
        assert!(temp.path().join("test-operator/README.md").exists());
        assert!(temp.path().join("test-operator/Makefile").exists());
        assert!(temp.path().join("test-operator/.gitignore").exists());
        assert!(temp.path().join("test-operator/src/main.rs").exists());
        assert!(temp.path().join("test-operator/src/lib.rs").exists());
        assert!(temp.path().join("test-operator/src/error.rs").exists());
    }

    #[test]
    fn test_execute_new_cargo_toml_content() {
        let _lock = CWD_LOCK.lock().unwrap();
        let temp = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let result = execute_new(&make_args("my-operator"));
        std::env::set_current_dir(&original_dir).unwrap();

        assert!(result.is_ok());
        let cargo_content =
            std::fs::read_to_string(temp.path().join("my-operator/Cargo.toml")).unwrap();
        assert!(cargo_content.contains("[package]"));
        assert!(cargo_content.contains("kube"));
    }

    #[test]
    fn test_execute_new_dry_run() {
        let _lock = CWD_LOCK.lock().unwrap();
        let temp = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let mut args = make_args("dry-run-test");
        args.dry_run = true;
        let result = execute_new(&args);

        std::env::set_current_dir(&original_dir).unwrap();

        assert!(result.is_ok());
        // Project should NOT be created in dry-run mode
        assert!(!temp.path().join("dry-run-test").exists());
    }

    #[test]
    fn test_execute_new_fails_existing_without_force() {
        let _lock = CWD_LOCK.lock().unwrap();
        let temp = TempDir::new().unwrap();
        let project_dir = temp.path().join("existing-project");

        // Create existing project
        std::fs::create_dir_all(&project_dir).unwrap();
        std::fs::write(project_dir.join("Cargo.toml"), "existing").unwrap();

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let result = execute_new(&make_args("existing-project"));

        std::env::set_current_dir(&original_dir).unwrap();

        assert!(result.is_err());
    }

    #[test]
    fn test_execute_new_with_force_overwrites() {
        let _lock = CWD_LOCK.lock().unwrap();
        let temp = TempDir::new().unwrap();
        let project_dir = temp.path().join("force-test");

        // Create existing project
        std::fs::create_dir_all(project_dir.join("src")).unwrap();
        std::fs::write(project_dir.join("Cargo.toml"), "old content").unwrap();

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let mut args = make_args("force-test");
        args.force = true;
        let result = execute_new(&args);

        std::env::set_current_dir(&original_dir).unwrap();

        assert!(result.is_ok());
        let cargo_content = std::fs::read_to_string(project_dir.join("Cargo.toml")).unwrap();
        assert!(cargo_content.contains("[package]"));
    }

    #[test]
    fn test_execute_new_invalid_name() {
        let args = NewArgs {
            name: "Invalid Name".to_string(), // Contains space
            domain: "example.com".to_string(),
            non_interactive: true,
            dry_run: false,
            force: false,
        };

        let result = execute_new(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_paths_to_create() {
        let project_dir = Path::new("test-project");
        let paths = get_paths_to_create(project_dir);

        assert!(paths.contains(&project_dir.to_path_buf()));
        assert!(paths.contains(&project_dir.join("src")));
        assert!(paths.contains(&project_dir.join("Cargo.toml")));
        assert!(paths.contains(&project_dir.join("README.md")));
        assert!(paths.contains(&project_dir.join("Makefile")));
        assert!(paths.contains(&project_dir.join(".gitignore")));
        assert!(paths.contains(&project_dir.join("src/main.rs")));
    }

    #[test]
    fn test_render_template() {
        let renderer = SimpleRenderer::new();
        let mut ctx = TemplateContext::new();
        ctx.set("project_name", "test-op");

        let result = render_template(&renderer, "project/Cargo.toml.tmpl", &ctx);
        assert!(result.is_ok());
        let content = result.unwrap();
        assert!(content.contains("name = \"test-op\""));
    }
}
