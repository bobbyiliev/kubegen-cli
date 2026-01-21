//! Embedded templates compiled into the binary
//!
//! Uses rust-embed to compile all templates from the templates/ directory
//! into the binary for easy distribution.

use rust_embed::Embed;

use crate::error::{KubegenError, Result};

/// Embedded template assets
#[derive(Embed)]
#[folder = "templates/"]
#[prefix = ""]
pub struct TemplateAssets;

/// Template categories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateCategory {
    /// Project scaffolding templates
    Project,
    /// CRD-related templates
    Crd,
    /// Webhook templates
    Webhook,
    /// Metrics templates
    Metrics,
}

impl TemplateCategory {
    /// Get the directory prefix for this category
    pub fn prefix(&self) -> &'static str {
        match self {
            TemplateCategory::Project => "project/",
            TemplateCategory::Crd => "crd/",
            TemplateCategory::Webhook => "webhook/",
            TemplateCategory::Metrics => "metrics/",
        }
    }
}

/// Get an embedded template by path
///
/// # Arguments
/// * `path` - The template path relative to the templates/ directory
///
/// # Returns
/// The template content as a string, or an error if not found
pub fn get_template(path: &str) -> Result<String> {
    TemplateAssets::get(path)
        .map(|file| String::from_utf8_lossy(file.data.as_ref()).to_string())
        .ok_or_else(|| KubegenError::TemplateNotFound {
            template_name: path.to_string(),
        })
}

/// Get an embedded template by category and name
///
/// # Arguments
/// * `category` - The template category
/// * `name` - The template filename (without the category prefix)
///
/// # Returns
/// The template content as a string, or an error if not found
pub fn get_template_by_category(category: TemplateCategory, name: &str) -> Result<String> {
    let path = format!("{}{}", category.prefix(), name);
    get_template(&path)
}

/// List all templates in a category
///
/// # Arguments
/// * `category` - The template category to list
///
/// # Returns
/// A vector of template names (without the category prefix)
pub fn list_templates(category: TemplateCategory) -> Vec<String> {
    let prefix = category.prefix();
    TemplateAssets::iter()
        .filter(|path| path.starts_with(prefix))
        .map(|path| path.strip_prefix(prefix).unwrap_or(&path).to_string())
        .collect()
}

/// List all available templates
///
/// # Returns
/// A vector of all template paths
pub fn list_all_templates() -> Vec<String> {
    TemplateAssets::iter().map(|s| s.to_string()).collect()
}

/// Check if a template exists
///
/// # Arguments
/// * `path` - The template path to check
///
/// # Returns
/// true if the template exists, false otherwise
pub fn template_exists(path: &str) -> bool {
    TemplateAssets::get(path).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_category_prefix() {
        assert_eq!(TemplateCategory::Project.prefix(), "project/");
        assert_eq!(TemplateCategory::Crd.prefix(), "crd/");
        assert_eq!(TemplateCategory::Webhook.prefix(), "webhook/");
        assert_eq!(TemplateCategory::Metrics.prefix(), "metrics/");
    }

    #[test]
    fn test_get_template_project_cargo() {
        let content = get_template("project/Cargo.toml.tmpl");
        assert!(content.is_ok());
        let content = content.unwrap();
        assert!(content.contains("{{project_name}}"));
        assert!(content.contains("[package]"));
    }

    #[test]
    fn test_get_template_project_main() {
        let content = get_template("project/main.rs.tmpl");
        assert!(content.is_ok());
        let content = content.unwrap();
        assert!(content.contains("{{project_name}}"));
        assert!(content.contains("tokio::main"));
    }

    #[test]
    fn test_get_template_project_error() {
        let content = get_template("project/error.rs.tmpl");
        assert!(content.is_ok());
        let content = content.unwrap();
        assert!(content.contains("{{project_name}}"));
        assert!(content.contains("KubeError"));
        assert!(content.contains("FinalizerError"));
        assert!(content.contains("thiserror"));
    }

    #[test]
    fn test_get_template_crd_types() {
        let content = get_template("crd/types.rs.tmpl");
        assert!(content.is_ok());
        let content = content.unwrap();
        assert!(content.contains("{{kind}}"));
        assert!(content.contains("{{group}}"));
        assert!(content.contains("{{version}}"));
    }

    #[test]
    fn test_get_template_crd_controller() {
        let content = get_template("crd/controller.rs.tmpl");
        assert!(content.is_ok());
        let content = content.unwrap();
        assert!(content.contains("{{kind}}"));
        assert!(content.contains("reconcile"));
        // Check for enhanced reconciler features
        assert!(content.contains("FINALIZER"));
        assert!(content.contains("finalizer"));
        assert!(content.contains("cleanup_resource"));
        assert!(content.contains("update_status"));
        assert!(content.contains("error_policy"));
        assert!(content.contains("DEFAULT_REQUEUE_INTERVAL"));
        assert!(content.contains("ERROR_REQUEUE_INTERVAL"));
    }

    #[test]
    fn test_get_template_webhook_mod() {
        let content = get_template("webhook/mod.rs.tmpl");
        assert!(content.is_ok());
        let content = content.unwrap();
        assert!(content.contains("{{kind_snake}}"));
        assert!(content.contains("TlsConfig"));
        assert!(content.contains("run_server"));
        assert!(content.contains("tls()"));
        assert!(content.contains("cert_path"));
        assert!(content.contains("key_path"));
    }

    #[test]
    fn test_get_template_webhook_validating() {
        let content = get_template("webhook/validating.rs.tmpl");
        assert!(content.is_ok());
        let content = content.unwrap();
        assert!(content.contains("{{kind}}"));
        assert!(content.contains("validate"));
    }

    #[test]
    fn test_get_template_webhook_mutating() {
        let content = get_template("webhook/mutating.rs.tmpl");
        assert!(content.is_ok());
        let content = content.unwrap();
        assert!(content.contains("{{kind}}"));
        assert!(content.contains("mutate"));
    }

    #[test]
    fn test_get_template_webhook_validating_config() {
        let content = get_template("webhook/validating-webhook-config.yaml.tmpl");
        assert!(content.is_ok());
        let content = content.unwrap();
        assert!(content.contains("ValidatingWebhookConfiguration"));
        assert!(content.contains("{{kind_snake}}"));
        assert!(content.contains("{{group}}"));
    }

    #[test]
    fn test_get_template_webhook_mutating_config() {
        let content = get_template("webhook/mutating-webhook-config.yaml.tmpl");
        assert!(content.is_ok());
        let content = content.unwrap();
        assert!(content.contains("MutatingWebhookConfiguration"));
        assert!(content.contains("{{kind_snake}}"));
        assert!(content.contains("{{group}}"));
    }

    #[test]
    fn test_get_template_webhook_certificate() {
        let content = get_template("webhook/certificate.yaml.tmpl");
        assert!(content.is_ok());
        let content = content.unwrap();
        assert!(content.contains("cert-manager.io/v1"));
        assert!(content.contains("Certificate"));
        assert!(content.contains("{{service_name}}"));
        assert!(content.contains("{{namespace}}"));
    }

    #[test]
    fn test_get_template_webhook_issuer() {
        let content = get_template("webhook/issuer.yaml.tmpl");
        assert!(content.is_ok());
        let content = content.unwrap();
        assert!(content.contains("cert-manager.io/v1"));
        assert!(content.contains("Issuer"));
        assert!(content.contains("selfSigned"));
    }

    #[test]
    fn test_get_template_metrics() {
        let content = get_template("metrics/prometheus.rs.tmpl");
        assert!(content.is_ok());
        let content = content.unwrap();
        assert!(content.contains("{{project_name}}"));
        assert!(content.contains("prometheus"));
    }

    #[test]
    fn test_get_template_not_found() {
        let result = get_template("nonexistent/template.tmpl");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Template not found"));
    }

    #[test]
    fn test_get_template_by_category() {
        let content = get_template_by_category(TemplateCategory::Project, "Cargo.toml.tmpl");
        assert!(content.is_ok());
        assert!(content.unwrap().contains("[package]"));
    }

    #[test]
    fn test_list_templates_project() {
        let templates = list_templates(TemplateCategory::Project);
        assert!(!templates.is_empty());
        assert!(templates.iter().any(|t| t.contains("Cargo.toml")));
        assert!(templates.iter().any(|t| t.contains("main.rs")));
    }

    #[test]
    fn test_list_templates_crd() {
        let templates = list_templates(TemplateCategory::Crd);
        assert!(!templates.is_empty());
        assert!(templates.iter().any(|t| t.contains("types.rs")));
        assert!(templates.iter().any(|t| t.contains("controller.rs")));
    }

    #[test]
    fn test_list_all_templates() {
        let templates = list_all_templates();
        assert!(!templates.is_empty());
        // Should have templates from all categories
        assert!(templates.iter().any(|t| t.starts_with("project/")));
        assert!(templates.iter().any(|t| t.starts_with("crd/")));
        assert!(templates.iter().any(|t| t.starts_with("webhook/")));
        assert!(templates.iter().any(|t| t.starts_with("metrics/")));
    }

    #[test]
    fn test_template_exists() {
        assert!(template_exists("project/Cargo.toml.tmpl"));
        assert!(template_exists("crd/types.rs.tmpl"));
        assert!(!template_exists("nonexistent.tmpl"));
    }

    #[test]
    fn test_template_category_equality() {
        assert_eq!(TemplateCategory::Project, TemplateCategory::Project);
        assert_ne!(TemplateCategory::Project, TemplateCategory::Crd);
    }

    #[test]
    fn test_template_category_clone() {
        let cat = TemplateCategory::Webhook;
        let cloned = cat;
        assert_eq!(cat, cloned);
    }

    #[test]
    fn test_template_category_debug() {
        let cat = TemplateCategory::Metrics;
        let debug_str = format!("{:?}", cat);
        assert!(debug_str.contains("Metrics"));
    }
}
