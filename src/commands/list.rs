//! Implementation of the `kubegen list` command
//!
//! Lists CRDs, webhooks, and other components in the current project.

use std::fs;
use std::path::Path;

use crate::error::{KubegenError, Result};

/// Information about a detected CRD
#[derive(Debug)]
pub struct CrdInfo {
    pub kind: String,
    pub group: String,
    pub version: String,
}

/// Information about a detected webhook
#[derive(Debug)]
pub struct WebhookInfo {
    pub kind: String,
    pub validating: bool,
    pub mutating: bool,
}

/// Information about metrics configuration
#[derive(Debug)]
pub struct MetricsInfo {
    pub enabled: bool,
    pub port: Option<u16>,
}

/// All detected project components
#[derive(Debug)]
pub struct ProjectComponents {
    pub crds: Vec<CrdInfo>,
    pub webhooks: Vec<WebhookInfo>,
    pub metrics: MetricsInfo,
}

/// Execute the `kubegen list` command
pub fn execute_list() -> Result<()> {
    // Check if we're in a kubegen project
    if !Path::new("Cargo.toml").exists() {
        return Err(KubegenError::ProjectNotFound);
    }

    let components = detect_components()?;

    print_components(&components);

    Ok(())
}

/// Detect all components in the current project
fn detect_components() -> Result<ProjectComponents> {
    let crds = detect_crds()?;
    let webhooks = detect_webhooks(&crds)?;
    let metrics = detect_metrics()?;

    Ok(ProjectComponents {
        crds,
        webhooks,
        metrics,
    })
}

/// Detect CRDs from manifests directory
fn detect_crds() -> Result<Vec<CrdInfo>> {
    let mut crds = Vec::new();
    let manifests_dir = Path::new("manifests");

    if !manifests_dir.exists() {
        return Ok(crds);
    }

    let entries = fs::read_dir(manifests_dir)?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path
            .extension()
            .is_some_and(|ext| ext == "yaml" || ext == "yml")
        {
            if let Some(crd) = parse_crd_manifest(&path) {
                crds.push(crd);
            }
        }
    }

    Ok(crds)
}

/// Parse a CRD manifest file to extract info
fn parse_crd_manifest(path: &Path) -> Option<CrdInfo> {
    let content = fs::read_to_string(path).ok()?;

    // Check if this is a CRD
    if !content.contains("kind: CustomResourceDefinition") {
        return None;
    }

    // Extract group from spec.group
    let group = extract_crd_group(&content)?;

    // Extract kind from spec.names.kind
    let kind = extract_crd_kind(&content)?;

    // Extract version from spec.versions[0].name
    let version = extract_version_from_crd(&content).unwrap_or_else(|| "v1alpha1".to_string());

    Some(CrdInfo {
        kind,
        group,
        version,
    })
}

/// Extract group from CRD spec.group
fn extract_crd_group(content: &str) -> Option<String> {
    // Look for "group:" that's under "spec:" section
    let mut in_spec = false;
    for line in content.lines() {
        let trimmed = line.trim();

        // Track when we enter spec section
        if trimmed == "spec:" {
            in_spec = true;
            continue;
        }

        // If in spec and line starts with "group:", extract it
        if in_spec && trimmed.starts_with("group:") {
            let value = trimmed.strip_prefix("group:")?.trim();
            let value = value.trim_matches('"').trim_matches('\'');
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }

        // Exit spec if we hit another top-level section
        if in_spec
            && !line.starts_with(' ')
            && !line.starts_with('\t')
            && !trimmed.is_empty()
            && trimmed.ends_with(':')
        {
            break;
        }
    }
    None
}

/// Extract kind from CRD spec.names.kind
fn extract_crd_kind(content: &str) -> Option<String> {
    // Look for "kind:" that's under "spec.names" section
    let mut in_names = false;
    for line in content.lines() {
        let trimmed = line.trim();

        // Track when we enter names section
        if trimmed == "names:" {
            in_names = true;
            continue;
        }

        // If in names and line starts with "kind:", extract it
        if in_names && trimmed.starts_with("kind:") {
            let value = trimmed.strip_prefix("kind:")?.trim();
            let value = value.trim_matches('"').trim_matches('\'');
            if !value.is_empty() && value != "CustomResourceDefinition" {
                return Some(value.to_string());
            }
        }

        // Exit names if we hit another section at same or higher level
        if in_names
            && !trimmed.is_empty()
            && !trimmed.starts_with("kind:")
            && !trimmed.starts_with("listKind:")
            && !trimmed.starts_with("plural:")
            && !trimmed.starts_with("singular:")
            && !trimmed.starts_with("shortNames:")
            && !trimmed.starts_with("-")
        {
            // Check if this is a sibling key (same indentation as names)
            if !line.starts_with("      ") && !line.starts_with("\t\t\t") {
                break;
            }
        }
    }
    None
}

/// Extract version from CRD spec.versions array
fn extract_version_from_crd(content: &str) -> Option<String> {
    // Look for "- name: v1alpha1" pattern after "versions:" section
    let mut in_versions = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "versions:" {
            in_versions = true;
            continue;
        }
        if in_versions && trimmed.starts_with("- name:") {
            let version = trimmed.strip_prefix("- name:")?.trim();
            return Some(version.to_string());
        }
        // Exit versions section if we hit another top-level key
        if in_versions && !line.starts_with(' ') && !line.starts_with('\t') && !trimmed.is_empty() {
            break;
        }
    }
    None
}

/// Detect webhooks from manifests directory
fn detect_webhooks(crds: &[CrdInfo]) -> Result<Vec<WebhookInfo>> {
    let mut webhook_map: std::collections::HashMap<String, WebhookInfo> =
        std::collections::HashMap::new();
    let manifests_dir = Path::new("manifests");

    if !manifests_dir.exists() {
        return Ok(Vec::new());
    }

    // Scan manifests directory and subdirectories for webhook configs
    scan_for_webhooks(manifests_dir, &mut webhook_map)?;

    // Also check src directory for webhook modules
    let src_dir = Path::new("src");
    if src_dir.exists() {
        if let Ok(entries) = fs::read_dir(src_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    if name.ends_with("_webhook") || name == "webhook" {
                        // Try to match with a CRD
                        let kind_prefix = name.strip_suffix("_webhook").unwrap_or("");
                        for crd in crds {
                            let crd_snake = to_snake_case(&crd.kind);
                            if crd_snake == kind_prefix || kind_prefix.is_empty() {
                                let entry =
                                    webhook_map.entry(crd.kind.clone()).or_insert(WebhookInfo {
                                        kind: crd.kind.clone(),
                                        validating: false,
                                        mutating: false,
                                    });
                                // If we found a webhook module but no manifest info,
                                // assume both types might be present
                                if !entry.validating && !entry.mutating {
                                    // Check the file content for clues
                                    if let Ok(content) = fs::read_to_string(&path) {
                                        if content.contains("validat") {
                                            entry.validating = true;
                                        }
                                        if content.contains("mutat") {
                                            entry.mutating = true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(webhook_map.into_values().collect())
}

/// Recursively scan directory for webhook manifests
fn scan_for_webhooks(
    dir: &Path,
    webhook_map: &mut std::collections::HashMap<String, WebhookInfo>,
) -> Result<()> {
    let entries = fs::read_dir(dir)?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            // Recursively scan subdirectories
            scan_for_webhooks(&path, webhook_map)?;
        } else if path
            .extension()
            .is_some_and(|ext| ext == "yaml" || ext == "yml")
        {
            if let Some((kind, is_validating)) = parse_webhook_manifest(&path) {
                let entry = webhook_map.entry(kind.clone()).or_insert(WebhookInfo {
                    kind,
                    validating: false,
                    mutating: false,
                });
                if is_validating {
                    entry.validating = true;
                } else {
                    entry.mutating = true;
                }
            }
        }
    }

    Ok(())
}

/// Parse a webhook manifest file
fn parse_webhook_manifest(path: &Path) -> Option<(String, bool)> {
    let content = fs::read_to_string(path).ok()?;

    let is_validating = content.contains("kind: ValidatingWebhookConfiguration");
    let is_mutating = content.contains("kind: MutatingWebhookConfiguration");

    if !is_validating && !is_mutating {
        return None;
    }

    // Try to extract the kind from metadata.name or filename
    let kind = extract_webhook_kind(&content, path);

    Some((kind, is_validating))
}

/// Extract the resource kind from webhook configuration
fn extract_webhook_kind(content: &str, path: &Path) -> String {
    // Try to extract from metadata.name like "widget-validating-webhook"
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("name:") && !trimmed.contains("service") {
            if let Some(name) = trimmed.strip_prefix("name:").map(|s| s.trim()) {
                let name = name.trim_matches('"').trim_matches('\'');
                // Extract kind from patterns like "widget-validating-webhook"
                if let Some(kind) = name
                    .strip_suffix("-validating-webhook")
                    .or_else(|| name.strip_suffix("-mutating-webhook"))
                    .or_else(|| name.strip_suffix("_validating_webhook"))
                    .or_else(|| name.strip_suffix("_mutating_webhook"))
                {
                    return to_pascal_case(kind);
                }
            }
        }
    }

    // Fallback to filename
    if let Some(filename) = path.file_stem().and_then(|s| s.to_str()) {
        if let Some(kind) = filename
            .strip_suffix("-validating-webhook-config")
            .or_else(|| filename.strip_suffix("-mutating-webhook-config"))
            .or_else(|| filename.strip_suffix("-validating-webhook"))
            .or_else(|| filename.strip_suffix("-mutating-webhook"))
            .or_else(|| filename.strip_suffix("_validating_webhook"))
            .or_else(|| filename.strip_suffix("_mutating_webhook"))
        {
            return to_pascal_case(kind);
        }
    }

    "Unknown".to_string()
}

/// Detect metrics configuration
fn detect_metrics() -> Result<MetricsInfo> {
    // Check for metrics as a single file or a directory
    let metrics_file = Path::new("src/metrics.rs");
    let metrics_dir = Path::new("src/metrics");
    let metrics_mod_file = Path::new("src/metrics/mod.rs");

    let metrics_content = if metrics_file.exists() {
        Some(fs::read_to_string(metrics_file)?)
    } else if metrics_mod_file.exists() {
        Some(fs::read_to_string(metrics_mod_file)?)
    } else if metrics_dir.exists() {
        // Metrics directory exists, try to read any .rs file to find port
        if let Ok(entries) = fs::read_dir(metrics_dir) {
            entries
                .flatten()
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
                .find_map(|e| fs::read_to_string(e.path()).ok())
        } else {
            None
        }
    } else {
        None
    };

    match metrics_content {
        Some(content) => {
            let port = extract_metrics_port(&content);
            Ok(MetricsInfo {
                enabled: true,
                port,
            })
        }
        None => Ok(MetricsInfo {
            enabled: false,
            port: None,
        }),
    }
}

/// Extract metrics port from metrics.rs content
fn extract_metrics_port(content: &str) -> Option<u16> {
    // Look for patterns like "port: 8080" or "8080" in serve_metrics calls
    for line in content.lines() {
        // Look for port number in common patterns
        if line.contains("8080") {
            return Some(8080);
        }
        if line.contains("9090") {
            return Some(9090);
        }
        // Try to find port = <number> pattern
        if let Some(pos) = line.find("port") {
            let rest = &line[pos..];
            for word in rest.split_whitespace() {
                if let Ok(port) = word
                    .trim_matches(|c: char| !c.is_ascii_digit())
                    .parse::<u16>()
                {
                    if port > 1000 {
                        return Some(port);
                    }
                }
            }
        }
    }
    None
}

/// Print detected components
fn print_components(components: &ProjectComponents) {
    // Print CRDs
    if components.crds.is_empty() {
        println!("CRDs: none");
    } else {
        println!("CRDs:");
        for crd in &components.crds {
            println!("  - {} ({}/{})", crd.kind, crd.group, crd.version);
        }
    }

    println!();

    // Print Webhooks
    if components.webhooks.is_empty() {
        println!("Webhooks: none");
    } else {
        println!("Webhooks:");
        for webhook in &components.webhooks {
            let types: Vec<&str> = [
                if webhook.validating {
                    Some("validating")
                } else {
                    None
                },
                if webhook.mutating {
                    Some("mutating")
                } else {
                    None
                },
            ]
            .into_iter()
            .flatten()
            .collect();

            if types.is_empty() {
                println!("  - {}", webhook.kind);
            } else {
                println!("  - {} ({})", webhook.kind, types.join(", "));
            }
        }
    }

    println!();

    // Print Metrics
    if components.metrics.enabled {
        if let Some(port) = components.metrics.port {
            println!("Metrics: enabled (port {})", port);
        } else {
            println!("Metrics: enabled");
        }
    } else {
        println!("Metrics: disabled");
    }
}

/// Convert snake_case to PascalCase
fn to_pascal_case(s: &str) -> String {
    s.split(['_', '-'])
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}

/// Convert PascalCase to snake_case
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
        } else {
            result.push(c);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("widget"), "Widget");
        assert_eq!(to_pascal_case("my_resource"), "MyResource");
        assert_eq!(to_pascal_case("my-resource"), "MyResource");
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("Widget"), "widget");
        assert_eq!(to_snake_case("MyResource"), "my_resource");
    }

    #[test]
    fn test_extract_crd_group() {
        let yaml = r#"
apiVersion: v1
kind: CustomResourceDefinition
metadata:
  name: widgets.example.com
spec:
  group: example.com
  names:
    kind: Widget
"#;
        assert_eq!(extract_crd_group(yaml), Some("example.com".to_string()));
    }

    #[test]
    fn test_extract_crd_kind() {
        let yaml = r#"
apiVersion: v1
kind: CustomResourceDefinition
metadata:
  name: widgets.example.com
spec:
  group: example.com
  names:
    kind: Widget
    plural: widgets
"#;
        assert_eq!(extract_crd_kind(yaml), Some("Widget".to_string()));
    }

    #[test]
    fn test_extract_version_from_crd() {
        let yaml = r#"
spec:
  versions:
    - name: v1alpha1
      served: true
    - name: v1
      served: true
"#;
        assert_eq!(extract_version_from_crd(yaml), Some("v1alpha1".to_string()));
    }

    #[test]
    fn test_extract_metrics_port() {
        let content = r#"
pub async fn serve_metrics(port: u16) {
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
}
"#;
        assert_eq!(extract_metrics_port(content), Some(8080));
    }
}
