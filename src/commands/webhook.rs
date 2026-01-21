//! Implementation of the `kubegen add webhook` command
//!
//! Adds admission webhook support to an existing Kubernetes operator project.

use std::path::Path;

use tracing::{debug, info};

use crate::cli::WebhookArgs;
use crate::error::Result;
use crate::fs::{
    check_conflicts, create_dir_all, format_conflicts, write_file_protected, DryRunContext,
    WriteOptions,
};
use crate::templates::{
    get_template, SimpleRenderer, StringTemplate, TemplateContext, TemplateRenderer,
};
use crate::validation::{self, pluralize, to_snake_case};

/// Execute the `kubegen add webhook` command
pub fn execute_add_webhook(args: &WebhookArgs) -> Result<()> {
    // Validate CRD kind
    validation::validate_crd_kind(&args.kind)?;

    // Validate we're in a kubegen project
    validate_project_structure()?;

    // Require at least one webhook type
    if !args.validating && !args.mutating {
        return Err(crate::error::KubegenError::ValidationError(
            "At least one of --validating or --mutating must be specified".to_string(),
        ));
    }

    info!("Adding webhook for: {}", args.kind);

    // Get project name from Cargo.toml for default service name
    let project_name = get_project_name()?;

    // Build template context
    let kind_snake = to_snake_case(&args.kind);
    let plural = pluralize(&kind_snake);
    let group = args
        .group
        .clone()
        .unwrap_or_else(|| format!("{}.example.com", kind_snake));
    let service_name = args
        .service_name
        .clone()
        .unwrap_or_else(|| format!("{}-webhook", project_name));

    let mut ctx = TemplateContext::new();
    ctx.set("kind", &args.kind);
    ctx.set("kind_snake", &kind_snake);
    ctx.set("plural", &plural);
    ctx.set("group", &group);
    ctx.set("service_name", &service_name);
    ctx.set("namespace", &args.namespace);
    ctx.set("with_validating", args.validating.to_string());
    ctx.set("with_mutating", args.mutating.to_string());

    let webhook_dir = Path::new("src").join("webhook");
    let manifests_dir = Path::new("manifests").join("webhook");

    if args.dry_run {
        return execute_dry_run(&webhook_dir, &manifests_dir, &ctx, args);
    }

    let write_opts = WriteOptions::with_force(args.force);

    // Check for conflicts before proceeding
    let paths_to_create = get_paths_to_create(&webhook_dir, &manifests_dir, args);
    let conflicts = check_conflicts(&paths_to_create, &write_opts);
    if !conflicts.is_empty() {
        return Err(crate::error::KubegenError::ValidationError(
            format_conflicts(&conflicts),
        ));
    }

    // Create webhook module structure
    create_webhook_structure(&webhook_dir, &manifests_dir, &ctx, &write_opts, args)?;

    info!("Webhook support added successfully!");
    info!("Next steps:");
    info!("  1. Add 'pub mod webhook;' to src/lib.rs");
    info!("  2. Configure webhook server in main.rs");
    info!("  3. Apply manifests with 'kubectl apply -f manifests/webhook/'");
    info!("  4. Run 'cargo build' to verify");

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

    // Simple parsing to extract package name
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
    webhook_dir: &Path,
    manifests_dir: &Path,
    args: &WebhookArgs,
) -> Vec<std::path::PathBuf> {
    let mut paths = vec![webhook_dir.to_path_buf(), webhook_dir.join("mod.rs")];

    if args.validating {
        paths.push(webhook_dir.join("validating.rs"));
        paths.push(manifests_dir.join("validating-webhook-config.yaml"));
    }
    if args.mutating {
        paths.push(webhook_dir.join("mutating.rs"));
        paths.push(manifests_dir.join("mutating-webhook-config.yaml"));
    }

    // Add cert-manager resources (always generated for webhooks)
    paths.push(manifests_dir.join("certificate.yaml"));
    paths.push(manifests_dir.join("issuer.yaml"));

    // Add manifests directory
    paths.push(manifests_dir.to_path_buf());

    paths
}

/// Execute in dry-run mode
fn execute_dry_run(
    webhook_dir: &Path,
    manifests_dir: &Path,
    ctx: &TemplateContext,
    args: &WebhookArgs,
) -> Result<()> {
    let mut dry_run = DryRunContext::new();

    dry_run.plan_dir(webhook_dir);
    dry_run.plan_dir(manifests_dir);

    let renderer = SimpleRenderer::new();

    let mod_content = render_template(&renderer, "webhook/mod.rs.tmpl", ctx)?;
    dry_run.plan_file(webhook_dir.join("mod.rs"), &mod_content);

    if args.validating {
        let validating_content = render_template(&renderer, "webhook/validating.rs.tmpl", ctx)?;
        dry_run.plan_file(webhook_dir.join("validating.rs"), &validating_content);

        let validating_config = render_template(
            &renderer,
            "webhook/validating-webhook-config.yaml.tmpl",
            ctx,
        )?;
        dry_run.plan_file(
            manifests_dir.join("validating-webhook-config.yaml"),
            &validating_config,
        );
    }

    if args.mutating {
        let mutating_content = render_template(&renderer, "webhook/mutating.rs.tmpl", ctx)?;
        dry_run.plan_file(webhook_dir.join("mutating.rs"), &mutating_content);

        let mutating_config =
            render_template(&renderer, "webhook/mutating-webhook-config.yaml.tmpl", ctx)?;
        dry_run.plan_file(
            manifests_dir.join("mutating-webhook-config.yaml"),
            &mutating_config,
        );
    }

    // Always generate cert-manager resources for webhooks
    let certificate_content = render_template(&renderer, "webhook/certificate.yaml.tmpl", ctx)?;
    dry_run.plan_file(manifests_dir.join("certificate.yaml"), &certificate_content);

    let issuer_content = render_template(&renderer, "webhook/issuer.yaml.tmpl", ctx)?;
    dry_run.plan_file(manifests_dir.join("issuer.yaml"), &issuer_content);

    println!("{}", dry_run.format_preview());
    Ok(())
}

/// Create the webhook module structure
fn create_webhook_structure(
    webhook_dir: &Path,
    manifests_dir: &Path,
    ctx: &TemplateContext,
    opts: &WriteOptions,
    args: &WebhookArgs,
) -> Result<()> {
    let renderer = SimpleRenderer::new();

    // Create webhook directory
    debug!("Creating webhook directory: {}", webhook_dir.display());
    create_dir_all(webhook_dir)?;

    // Create manifests directory
    debug!("Creating manifests directory: {}", manifests_dir.display());
    create_dir_all(manifests_dir)?;

    // Render and write mod.rs
    let mod_content = render_template(&renderer, "webhook/mod.rs.tmpl", ctx)?;
    debug!("Writing mod.rs");
    write_file_protected(webhook_dir.join("mod.rs"), &mod_content, opts)?;

    // Render and write validating.rs if requested
    if args.validating {
        let validating_content = render_template(&renderer, "webhook/validating.rs.tmpl", ctx)?;
        debug!("Writing validating.rs");
        write_file_protected(webhook_dir.join("validating.rs"), &validating_content, opts)?;

        // Write validating webhook configuration manifest
        let validating_config = render_template(
            &renderer,
            "webhook/validating-webhook-config.yaml.tmpl",
            ctx,
        )?;
        debug!("Writing validating-webhook-config.yaml");
        write_file_protected(
            manifests_dir.join("validating-webhook-config.yaml"),
            &validating_config,
            opts,
        )?;
    }

    // Render and write mutating.rs if requested
    if args.mutating {
        let mutating_content = render_template(&renderer, "webhook/mutating.rs.tmpl", ctx)?;
        debug!("Writing mutating.rs");
        write_file_protected(webhook_dir.join("mutating.rs"), &mutating_content, opts)?;

        // Write mutating webhook configuration manifest
        let mutating_config =
            render_template(&renderer, "webhook/mutating-webhook-config.yaml.tmpl", ctx)?;
        debug!("Writing mutating-webhook-config.yaml");
        write_file_protected(
            manifests_dir.join("mutating-webhook-config.yaml"),
            &mutating_config,
            opts,
        )?;
    }

    // Always generate cert-manager resources for webhooks
    let certificate_content = render_template(&renderer, "webhook/certificate.yaml.tmpl", ctx)?;
    debug!("Writing certificate.yaml");
    write_file_protected(
        manifests_dir.join("certificate.yaml"),
        &certificate_content,
        opts,
    )?;

    let issuer_content = render_template(&renderer, "webhook/issuer.yaml.tmpl", ctx)?;
    debug!("Writing issuer.yaml");
    write_file_protected(manifests_dir.join("issuer.yaml"), &issuer_content, opts)?;

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

    fn make_args(kind: &str, validating: bool, mutating: bool) -> WebhookArgs {
        WebhookArgs {
            kind: kind.to_string(),
            validating,
            mutating,
            group: None,
            service_name: None,
            namespace: "default".to_string(),
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
    fn test_execute_add_webhook_validating() {
        let _lock = CWD_LOCK.lock().unwrap();
        let temp = TempDir::new().unwrap();
        setup_project(&temp);

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let result = execute_add_webhook(&make_args("MyResource", true, false));
        std::env::set_current_dir(&original_dir).unwrap();

        assert!(result.is_ok());
        assert!(temp.path().join("src/webhook").exists());
        assert!(temp.path().join("src/webhook/mod.rs").exists());
        assert!(temp.path().join("src/webhook/validating.rs").exists());
        assert!(!temp.path().join("src/webhook/mutating.rs").exists());
        // Check manifests
        assert!(temp
            .path()
            .join("manifests/webhook/validating-webhook-config.yaml")
            .exists());
        assert!(!temp
            .path()
            .join("manifests/webhook/mutating-webhook-config.yaml")
            .exists());
        // Check cert-manager resources
        assert!(temp
            .path()
            .join("manifests/webhook/certificate.yaml")
            .exists());
        assert!(temp.path().join("manifests/webhook/issuer.yaml").exists());
    }

    #[test]
    fn test_execute_add_webhook_mutating() {
        let _lock = CWD_LOCK.lock().unwrap();
        let temp = TempDir::new().unwrap();
        setup_project(&temp);

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let result = execute_add_webhook(&make_args("MyResource", false, true));
        std::env::set_current_dir(&original_dir).unwrap();

        assert!(result.is_ok());
        assert!(temp.path().join("src/webhook").exists());
        assert!(temp.path().join("src/webhook/mod.rs").exists());
        assert!(!temp.path().join("src/webhook/validating.rs").exists());
        assert!(temp.path().join("src/webhook/mutating.rs").exists());
        // Check manifests
        assert!(!temp
            .path()
            .join("manifests/webhook/validating-webhook-config.yaml")
            .exists());
        assert!(temp
            .path()
            .join("manifests/webhook/mutating-webhook-config.yaml")
            .exists());
        // Check cert-manager resources
        assert!(temp
            .path()
            .join("manifests/webhook/certificate.yaml")
            .exists());
        assert!(temp.path().join("manifests/webhook/issuer.yaml").exists());
    }

    #[test]
    fn test_execute_add_webhook_both() {
        let _lock = CWD_LOCK.lock().unwrap();
        let temp = TempDir::new().unwrap();
        setup_project(&temp);

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let result = execute_add_webhook(&make_args("MyResource", true, true));
        std::env::set_current_dir(&original_dir).unwrap();

        assert!(result.is_ok());
        assert!(temp.path().join("src/webhook/mod.rs").exists());
        assert!(temp.path().join("src/webhook/validating.rs").exists());
        assert!(temp.path().join("src/webhook/mutating.rs").exists());
        // Check manifests
        assert!(temp
            .path()
            .join("manifests/webhook/validating-webhook-config.yaml")
            .exists());
        assert!(temp
            .path()
            .join("manifests/webhook/mutating-webhook-config.yaml")
            .exists());
        // Check cert-manager resources
        assert!(temp
            .path()
            .join("manifests/webhook/certificate.yaml")
            .exists());
        assert!(temp.path().join("manifests/webhook/issuer.yaml").exists());
    }

    #[test]
    fn test_execute_add_webhook_neither_fails() {
        let _lock = CWD_LOCK.lock().unwrap();
        let temp = TempDir::new().unwrap();
        setup_project(&temp);

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let result = execute_add_webhook(&make_args("MyResource", false, false));
        std::env::set_current_dir(&original_dir).unwrap();

        assert!(result.is_err());
    }

    #[test]
    fn test_execute_add_webhook_dry_run() {
        let _lock = CWD_LOCK.lock().unwrap();
        let temp = TempDir::new().unwrap();
        setup_project(&temp);

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let mut args = make_args("MyResource", true, false);
        args.dry_run = true;
        let result = execute_add_webhook(&args);

        std::env::set_current_dir(&original_dir).unwrap();

        assert!(result.is_ok());
        assert!(!temp.path().join("src/webhook").exists());
    }

    #[test]
    fn test_execute_add_webhook_invalid_kind() {
        let args = WebhookArgs {
            kind: "invalid-kind".to_string(),
            validating: true,
            mutating: false,
            group: None,
            service_name: None,
            namespace: "default".to_string(),
            dry_run: false,
            force: false,
        };

        let result = execute_add_webhook(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_add_webhook_not_in_project() {
        let _lock = CWD_LOCK.lock().unwrap();
        let temp = TempDir::new().unwrap();
        // Don't create project structure

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let result = execute_add_webhook(&make_args("MyResource", true, false));

        std::env::set_current_dir(&original_dir).unwrap();

        assert!(result.is_err());
    }

    #[test]
    fn test_execute_add_webhook_existing_without_force() {
        let _lock = CWD_LOCK.lock().unwrap();
        let temp = TempDir::new().unwrap();
        setup_project(&temp);

        // Create existing webhook directory
        let webhook_dir = temp.path().join("src/webhook");
        std::fs::create_dir_all(&webhook_dir).unwrap();
        std::fs::write(webhook_dir.join("mod.rs"), "existing").unwrap();

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let result = execute_add_webhook(&make_args("MyResource", true, false));

        std::env::set_current_dir(&original_dir).unwrap();

        assert!(result.is_err());
    }

    #[test]
    fn test_execute_add_webhook_force_overwrites() {
        let _lock = CWD_LOCK.lock().unwrap();
        let temp = TempDir::new().unwrap();
        setup_project(&temp);

        // Create existing webhook directory
        let webhook_dir = temp.path().join("src/webhook");
        std::fs::create_dir_all(&webhook_dir).unwrap();
        std::fs::write(webhook_dir.join("mod.rs"), "old content").unwrap();

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let mut args = make_args("MyResource", true, false);
        args.force = true;
        let result = execute_add_webhook(&args);

        std::env::set_current_dir(&original_dir).unwrap();

        assert!(result.is_ok());
        let content = std::fs::read_to_string(webhook_dir.join("mod.rs")).unwrap();
        assert!(content.contains("webhook") || content.contains("Webhook"));
    }

    #[test]
    fn test_get_paths_to_create_validating_only() {
        let webhook_dir = Path::new("src/webhook");
        let manifests_dir = Path::new("manifests/webhook");
        let args = make_args("Test", true, false);
        let paths = get_paths_to_create(webhook_dir, manifests_dir, &args);

        assert!(paths.contains(&webhook_dir.to_path_buf()));
        assert!(paths.contains(&webhook_dir.join("mod.rs")));
        assert!(paths.contains(&webhook_dir.join("validating.rs")));
        assert!(!paths.contains(&webhook_dir.join("mutating.rs")));
        assert!(paths.contains(&manifests_dir.join("validating-webhook-config.yaml")));
        assert!(!paths.contains(&manifests_dir.join("mutating-webhook-config.yaml")));
        // cert-manager resources are always included
        assert!(paths.contains(&manifests_dir.join("certificate.yaml")));
        assert!(paths.contains(&manifests_dir.join("issuer.yaml")));
    }

    #[test]
    fn test_get_paths_to_create_both() {
        let webhook_dir = Path::new("src/webhook");
        let manifests_dir = Path::new("manifests/webhook");
        let args = make_args("Test", true, true);
        let paths = get_paths_to_create(webhook_dir, manifests_dir, &args);

        assert!(paths.contains(&webhook_dir.join("validating.rs")));
        assert!(paths.contains(&webhook_dir.join("mutating.rs")));
        assert!(paths.contains(&manifests_dir.join("validating-webhook-config.yaml")));
        assert!(paths.contains(&manifests_dir.join("mutating-webhook-config.yaml")));
        // cert-manager resources are always included
        assert!(paths.contains(&manifests_dir.join("certificate.yaml")));
        assert!(paths.contains(&manifests_dir.join("issuer.yaml")));
    }

    #[test]
    fn test_manifest_content() {
        let _lock = CWD_LOCK.lock().unwrap();
        let temp = TempDir::new().unwrap();
        setup_project(&temp);

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let mut args = make_args("MyResource", true, true);
        args.group = Some("mygroup.example.com".to_string());
        args.service_name = Some("my-webhook".to_string());
        args.namespace = "my-namespace".to_string();
        let result = execute_add_webhook(&args);
        std::env::set_current_dir(&original_dir).unwrap();

        assert!(result.is_ok());

        // Check validating config content
        let validating_config = std::fs::read_to_string(
            temp.path()
                .join("manifests/webhook/validating-webhook-config.yaml"),
        )
        .unwrap();
        assert!(validating_config.contains("ValidatingWebhookConfiguration"));
        assert!(validating_config.contains("mygroup.example.com"));
        assert!(validating_config.contains("my-webhook"));
        assert!(validating_config.contains("my-namespace"));
        assert!(validating_config.contains("my_resources")); // plural

        // Check mutating config content
        let mutating_config = std::fs::read_to_string(
            temp.path()
                .join("manifests/webhook/mutating-webhook-config.yaml"),
        )
        .unwrap();
        assert!(mutating_config.contains("MutatingWebhookConfiguration"));
        assert!(mutating_config.contains("mygroup.example.com"));
        assert!(mutating_config.contains("my-webhook"));
        assert!(mutating_config.contains("my-namespace"));

        // Check certificate content
        let certificate =
            std::fs::read_to_string(temp.path().join("manifests/webhook/certificate.yaml"))
                .unwrap();
        assert!(certificate.contains("cert-manager.io/v1"));
        assert!(certificate.contains("Certificate"));
        assert!(certificate.contains("my-webhook"));
        assert!(certificate.contains("my-namespace"));
        assert!(certificate.contains("my-webhook-tls")); // secretName
        assert!(certificate.contains("my-webhook-issuer")); // issuerRef

        // Check issuer content
        let issuer =
            std::fs::read_to_string(temp.path().join("manifests/webhook/issuer.yaml")).unwrap();
        assert!(issuer.contains("cert-manager.io/v1"));
        assert!(issuer.contains("Issuer"));
        assert!(issuer.contains("my-webhook-issuer"));
        assert!(issuer.contains("my-namespace"));
        assert!(issuer.contains("selfSigned"));
    }
}
