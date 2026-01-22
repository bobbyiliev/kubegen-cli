//! Template context types for kubegen scaffolding
//!
//! Provides strongly-typed context builders for generating operator projects.

use serde::{Deserialize, Serialize};

use super::TemplateContext;

/// Project metadata for scaffolding a new operator
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectContext {
    /// Project name (e.g., "my-operator")
    pub name: String,
    /// Project name in snake_case for Rust identifiers
    pub name_snake: String,
    /// API group (e.g., "example.com")
    pub group: String,
    /// API version (e.g., "v1", "v1alpha1")
    pub version: String,
    /// CRD kind (e.g., "MyResource")
    pub kind: String,
    /// CRD kind in snake_case
    pub kind_snake: String,
    /// Optional domain for the API group
    pub domain: Option<String>,
}

impl ProjectContext {
    /// Create a new ProjectContextBuilder
    pub fn builder() -> ProjectContextBuilder {
        ProjectContextBuilder::default()
    }

    /// Convert to a TemplateContext for rendering
    pub fn to_template_context(&self) -> TemplateContext {
        let mut ctx = TemplateContext::new();
        ctx.set("project_name", &self.name);
        ctx.set("project_name_snake", &self.name_snake);
        ctx.set("group", &self.group);
        ctx.set("version", &self.version);
        ctx.set("kind", &self.kind);
        ctx.set("kind_snake", &self.kind_snake);
        if let Some(ref domain) = self.domain {
            ctx.set("domain", domain);
        }
        ctx
    }
}

/// Builder for ProjectContext
#[derive(Debug, Clone, Default)]
pub struct ProjectContextBuilder {
    name: Option<String>,
    group: Option<String>,
    version: Option<String>,
    kind: Option<String>,
    domain: Option<String>,
}

impl ProjectContextBuilder {
    /// Set the project name
    pub fn name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the API group
    pub fn group<S: Into<String>>(mut self, group: S) -> Self {
        self.group = Some(group.into());
        self
    }

    /// Set the API version
    pub fn version<S: Into<String>>(mut self, version: S) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Set the CRD kind
    pub fn kind<S: Into<String>>(mut self, kind: S) -> Self {
        self.kind = Some(kind.into());
        self
    }

    /// Set the optional domain
    pub fn domain<S: Into<String>>(mut self, domain: S) -> Self {
        self.domain = Some(domain.into());
        self
    }

    /// Build the ProjectContext
    ///
    /// Returns None if required fields are missing
    pub fn build(self) -> Option<ProjectContext> {
        let name = self.name?;
        let kind = self.kind?;

        Some(ProjectContext {
            name_snake: to_snake_case(&name),
            name,
            group: self.group?,
            version: self.version?,
            kind_snake: to_snake_case(&kind),
            kind,
            domain: self.domain,
        })
    }
}

/// CRD-specific context for adding a new CRD to an existing project
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CrdContext {
    /// API group (e.g., "example.com")
    pub group: String,
    /// API version (e.g., "v1", "v1alpha1")
    pub version: String,
    /// CRD kind (e.g., "MyResource")
    pub kind: String,
    /// CRD kind in snake_case
    pub kind_snake: String,
    /// Plural form of the kind (e.g., "myresources")
    pub plural: String,
    /// Short name for the CRD (e.g., "mr")
    pub short_name: String,
    /// Whether to generate a controller
    pub with_controller: bool,
    /// Whether to generate status subresource
    pub with_status: bool,
}

impl CrdContext {
    /// Create a new CrdContextBuilder
    pub fn builder() -> CrdContextBuilder {
        CrdContextBuilder::default()
    }

    /// Convert to a TemplateContext for rendering
    pub fn to_template_context(&self) -> TemplateContext {
        let mut ctx = TemplateContext::new();
        ctx.set("group", &self.group);
        ctx.set("version", &self.version);
        ctx.set("kind", &self.kind);
        ctx.set("kind_snake", &self.kind_snake);
        ctx.set("plural", &self.plural);
        ctx.set("short_name", &self.short_name);
        ctx.set(
            "with_controller",
            if self.with_controller {
                "true"
            } else {
                "false"
            },
        );
        ctx.set(
            "with_status",
            if self.with_status { "true" } else { "false" },
        );
        ctx
    }
}

/// Builder for CrdContext
#[derive(Debug, Clone, Default)]
pub struct CrdContextBuilder {
    group: Option<String>,
    version: Option<String>,
    kind: Option<String>,
    with_controller: bool,
    with_status: bool,
}

impl CrdContextBuilder {
    /// Set the API group
    pub fn group<S: Into<String>>(mut self, group: S) -> Self {
        self.group = Some(group.into());
        self
    }

    /// Set the API version
    pub fn version<S: Into<String>>(mut self, version: S) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Set the CRD kind
    pub fn kind<S: Into<String>>(mut self, kind: S) -> Self {
        self.kind = Some(kind.into());
        self
    }

    /// Enable controller generation
    pub fn with_controller(mut self, enabled: bool) -> Self {
        self.with_controller = enabled;
        self
    }

    /// Enable status subresource
    pub fn with_status(mut self, enabled: bool) -> Self {
        self.with_status = enabled;
        self
    }

    /// Build the CrdContext
    ///
    /// Returns None if required fields are missing
    pub fn build(self) -> Option<CrdContext> {
        let kind = self.kind?;
        let kind_snake = to_snake_case(&kind);
        let plural = pluralize(&kind);
        let short_name = generate_short_name(&kind);

        Some(CrdContext {
            group: self.group?,
            version: self.version?,
            kind_snake,
            kind,
            plural,
            short_name,
            with_controller: self.with_controller,
            with_status: self.with_status,
        })
    }
}

/// Generate plural form of a kind name (simple English rules)
fn pluralize(s: &str) -> String {
    let lower = s.to_lowercase();
    if lower.ends_with('s')
        || lower.ends_with('x')
        || lower.ends_with("ch")
        || lower.ends_with("sh")
    {
        format!("{}es", lower)
    } else if lower.ends_with('y')
        && !lower.ends_with("ay")
        && !lower.ends_with("ey")
        && !lower.ends_with("oy")
        && !lower.ends_with("uy")
    {
        format!("{}ies", &lower[..lower.len() - 1])
    } else {
        format!("{}s", lower)
    }
}

/// Generate a short name from a kind (e.g., "MyResource" -> "mr")
fn generate_short_name(kind: &str) -> String {
    kind.chars()
        .filter(|c| c.is_uppercase())
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

/// Convert a string to snake_case
fn to_snake_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 4);
    let mut prev_lower = false;

    for c in s.chars() {
        if c.is_uppercase() {
            if prev_lower {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
            prev_lower = false;
        } else if c == '-' {
            result.push('_');
            prev_lower = false;
        } else {
            result.push(c);
            prev_lower = c.is_lowercase();
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // to_snake_case tests
    #[test]
    fn test_snake_case_simple() {
        assert_eq!(to_snake_case("MyResource"), "my_resource");
    }

    #[test]
    fn test_snake_case_already_snake() {
        assert_eq!(to_snake_case("my_resource"), "my_resource");
    }

    #[test]
    fn test_snake_case_kebab() {
        assert_eq!(to_snake_case("my-resource"), "my_resource");
    }

    #[test]
    fn test_snake_case_lowercase() {
        assert_eq!(to_snake_case("resource"), "resource");
    }

    #[test]
    fn test_snake_case_uppercase() {
        assert_eq!(to_snake_case("RESOURCE"), "resource");
    }

    #[test]
    fn test_snake_case_mixed() {
        assert_eq!(to_snake_case("MyHTTPServer"), "my_httpserver");
    }

    // ProjectContext tests
    #[test]
    fn test_project_context_builder() {
        let ctx = ProjectContext::builder()
            .name("my-operator")
            .group("example.com")
            .version("v1")
            .kind("MyResource")
            .build();

        assert!(ctx.is_some());
        let ctx = ctx.unwrap();
        assert_eq!(ctx.name, "my-operator");
        assert_eq!(ctx.name_snake, "my_operator");
        assert_eq!(ctx.group, "example.com");
        assert_eq!(ctx.version, "v1");
        assert_eq!(ctx.kind, "MyResource");
        assert_eq!(ctx.kind_snake, "my_resource");
        assert!(ctx.domain.is_none());
    }

    #[test]
    fn test_project_context_builder_with_domain() {
        let ctx = ProjectContext::builder()
            .name("my-operator")
            .group("apps")
            .version("v1alpha1")
            .kind("Database")
            .domain("example.com")
            .build()
            .unwrap();

        assert_eq!(ctx.domain, Some("example.com".to_string()));
    }

    #[test]
    fn test_project_context_builder_missing_name() {
        let ctx = ProjectContext::builder()
            .group("example.com")
            .version("v1")
            .kind("MyResource")
            .build();

        assert!(ctx.is_none());
    }

    #[test]
    fn test_project_context_builder_missing_group() {
        let ctx = ProjectContext::builder()
            .name("my-operator")
            .version("v1")
            .kind("MyResource")
            .build();

        assert!(ctx.is_none());
    }

    #[test]
    fn test_project_context_builder_missing_version() {
        let ctx = ProjectContext::builder()
            .name("my-operator")
            .group("example.com")
            .kind("MyResource")
            .build();

        assert!(ctx.is_none());
    }

    #[test]
    fn test_project_context_builder_missing_kind() {
        let ctx = ProjectContext::builder()
            .name("my-operator")
            .group("example.com")
            .version("v1")
            .build();

        assert!(ctx.is_none());
    }

    #[test]
    fn test_project_context_to_template_context() {
        let project = ProjectContext::builder()
            .name("my-operator")
            .group("example.com")
            .version("v1")
            .kind("MyResource")
            .domain("example.io")
            .build()
            .unwrap();

        let ctx = project.to_template_context();

        assert_eq!(ctx.get("project_name"), Some(&"my-operator".to_string()));
        assert_eq!(
            ctx.get("project_name_snake"),
            Some(&"my_operator".to_string())
        );
        assert_eq!(ctx.get("group"), Some(&"example.com".to_string()));
        assert_eq!(ctx.get("version"), Some(&"v1".to_string()));
        assert_eq!(ctx.get("kind"), Some(&"MyResource".to_string()));
        assert_eq!(ctx.get("kind_snake"), Some(&"my_resource".to_string()));
        assert_eq!(ctx.get("domain"), Some(&"example.io".to_string()));
    }

    #[test]
    fn test_project_context_to_template_context_no_domain() {
        let project = ProjectContext::builder()
            .name("my-operator")
            .group("example.com")
            .version("v1")
            .kind("MyResource")
            .build()
            .unwrap();

        let ctx = project.to_template_context();
        assert!(ctx.get("domain").is_none());
    }

    // CrdContext tests
    #[test]
    fn test_crd_context_builder() {
        let ctx = CrdContext::builder()
            .group("example.com")
            .version("v1")
            .kind("MyResource")
            .build();

        assert!(ctx.is_some());
        let ctx = ctx.unwrap();
        assert_eq!(ctx.group, "example.com");
        assert_eq!(ctx.version, "v1");
        assert_eq!(ctx.kind, "MyResource");
        assert_eq!(ctx.kind_snake, "my_resource");
        assert_eq!(ctx.plural, "myresources");
        assert_eq!(ctx.short_name, "mr");
        assert!(!ctx.with_controller);
        assert!(!ctx.with_status);
    }

    #[test]
    fn test_crd_context_builder_with_options() {
        let ctx = CrdContext::builder()
            .group("example.com")
            .version("v1")
            .kind("MyResource")
            .with_controller(true)
            .with_status(true)
            .build()
            .unwrap();

        assert!(ctx.with_controller);
        assert!(ctx.with_status);
    }

    #[test]
    fn test_crd_context_builder_missing_group() {
        let ctx = CrdContext::builder()
            .version("v1")
            .kind("MyResource")
            .build();

        assert!(ctx.is_none());
    }

    #[test]
    fn test_crd_context_builder_missing_version() {
        let ctx = CrdContext::builder()
            .group("example.com")
            .kind("MyResource")
            .build();

        assert!(ctx.is_none());
    }

    #[test]
    fn test_crd_context_builder_missing_kind() {
        let ctx = CrdContext::builder()
            .group("example.com")
            .version("v1")
            .build();

        assert!(ctx.is_none());
    }

    #[test]
    fn test_crd_context_to_template_context() {
        let crd = CrdContext::builder()
            .group("example.com")
            .version("v1")
            .kind("MyResource")
            .with_controller(true)
            .with_status(false)
            .build()
            .unwrap();

        let ctx = crd.to_template_context();

        assert_eq!(ctx.get("group"), Some(&"example.com".to_string()));
        assert_eq!(ctx.get("version"), Some(&"v1".to_string()));
        assert_eq!(ctx.get("kind"), Some(&"MyResource".to_string()));
        assert_eq!(ctx.get("kind_snake"), Some(&"my_resource".to_string()));
        assert_eq!(ctx.get("plural"), Some(&"myresources".to_string()));
        assert_eq!(ctx.get("short_name"), Some(&"mr".to_string()));
        assert_eq!(ctx.get("with_controller"), Some(&"true".to_string()));
        assert_eq!(ctx.get("with_status"), Some(&"false".to_string()));
    }

    // Serialization tests
    #[test]
    fn test_project_context_serialize() {
        let ctx = ProjectContext::builder()
            .name("my-operator")
            .group("example.com")
            .version("v1")
            .kind("MyResource")
            .build()
            .unwrap();

        let json = serde_json::to_string(&ctx).unwrap();
        assert!(json.contains("\"name\":\"my-operator\""));
        assert!(json.contains("\"group\":\"example.com\""));
    }

    #[test]
    fn test_project_context_deserialize() {
        let json = r#"{
            "name": "my-operator",
            "name_snake": "my_operator",
            "group": "example.com",
            "version": "v1",
            "kind": "MyResource",
            "kind_snake": "my_resource",
            "domain": null
        }"#;

        let ctx: ProjectContext = serde_json::from_str(json).unwrap();
        assert_eq!(ctx.name, "my-operator");
        assert_eq!(ctx.group, "example.com");
    }

    #[test]
    fn test_crd_context_serialize() {
        let ctx = CrdContext::builder()
            .group("example.com")
            .version("v1")
            .kind("MyResource")
            .with_controller(true)
            .build()
            .unwrap();

        let json = serde_json::to_string(&ctx).unwrap();
        assert!(json.contains("\"with_controller\":true"));
    }

    #[test]
    fn test_crd_context_deserialize() {
        let json = r#"{
            "group": "example.com",
            "version": "v1",
            "kind": "MyResource",
            "kind_snake": "my_resource",
            "plural": "myresources",
            "short_name": "mr",
            "with_controller": true,
            "with_status": false
        }"#;

        let ctx: CrdContext = serde_json::from_str(json).unwrap();
        assert!(ctx.with_controller);
        assert!(!ctx.with_status);
        assert_eq!(ctx.plural, "myresources");
        assert_eq!(ctx.short_name, "mr");
    }

    // Clone and equality tests
    #[test]
    fn test_project_context_clone() {
        let ctx = ProjectContext::builder()
            .name("my-operator")
            .group("example.com")
            .version("v1")
            .kind("MyResource")
            .build()
            .unwrap();

        let cloned = ctx.clone();
        assert_eq!(ctx, cloned);
    }

    #[test]
    fn test_crd_context_clone() {
        let ctx = CrdContext::builder()
            .group("example.com")
            .version("v1")
            .kind("MyResource")
            .build()
            .unwrap();

        let cloned = ctx.clone();
        assert_eq!(ctx, cloned);
    }

    #[test]
    fn test_project_context_debug() {
        let ctx = ProjectContext::builder()
            .name("my-operator")
            .group("example.com")
            .version("v1")
            .kind("MyResource")
            .build()
            .unwrap();

        let debug = format!("{:?}", ctx);
        assert!(debug.contains("ProjectContext"));
        assert!(debug.contains("my-operator"));
    }

    #[test]
    fn test_crd_context_debug() {
        let ctx = CrdContext::builder()
            .group("example.com")
            .version("v1")
            .kind("MyResource")
            .build()
            .unwrap();

        let debug = format!("{:?}", ctx);
        assert!(debug.contains("CrdContext"));
        assert!(debug.contains("MyResource"));
    }

    // Pluralize tests
    #[test]
    fn test_pluralize_simple() {
        assert_eq!(pluralize("Pod"), "pods");
        assert_eq!(pluralize("Resource"), "resources");
    }

    #[test]
    fn test_pluralize_ends_with_s() {
        assert_eq!(pluralize("Ingress"), "ingresses");
        assert_eq!(pluralize("Address"), "addresses");
    }

    #[test]
    fn test_pluralize_ends_with_x() {
        assert_eq!(pluralize("Index"), "indexes");
        assert_eq!(pluralize("Box"), "boxes");
    }

    #[test]
    fn test_pluralize_ends_with_ch() {
        assert_eq!(pluralize("Watch"), "watches");
        assert_eq!(pluralize("Batch"), "batches");
    }

    #[test]
    fn test_pluralize_ends_with_sh() {
        assert_eq!(pluralize("Mesh"), "meshes");
        assert_eq!(pluralize("Dash"), "dashes");
    }

    #[test]
    fn test_pluralize_ends_with_y_consonant() {
        assert_eq!(pluralize("Policy"), "policies");
        assert_eq!(pluralize("Entity"), "entities");
    }

    #[test]
    fn test_pluralize_ends_with_y_vowel() {
        assert_eq!(pluralize("Key"), "keys");
        assert_eq!(pluralize("Gateway"), "gateways");
        assert_eq!(pluralize("Boy"), "boys");
    }

    // Short name tests
    #[test]
    fn test_generate_short_name() {
        assert_eq!(generate_short_name("MyResource"), "mr");
        assert_eq!(generate_short_name("Pod"), "p");
        assert_eq!(generate_short_name("HTTPServer"), "https");
        assert_eq!(generate_short_name("CustomResourceDefinition"), "crd");
    }

    #[test]
    fn test_generate_short_name_single_word() {
        assert_eq!(generate_short_name("Database"), "d");
    }

    #[test]
    fn test_generate_short_name_all_uppercase() {
        assert_eq!(generate_short_name("ABC"), "abc");
    }
}
