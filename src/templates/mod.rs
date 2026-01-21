//! Template rendering abstraction for kubegen
//!
//! Provides traits and implementations for template rendering with variable substitution.
//! This module defines a flexible template system that can be used to generate
//! Kubernetes manifests, Rust source files, and other scaffolded content.

use std::collections::HashMap;

use crate::error::{KubegenError, Result};

/// A context containing variables for template rendering
///
/// # Examples
/// ```
/// use kubegen::templates::TemplateContext;
///
/// let mut ctx = TemplateContext::new();
/// ctx.set("name", "my-operator");
/// ctx.set("version", "v1");
/// assert_eq!(ctx.get("name"), Some(&"my-operator".to_string()));
/// ```
#[derive(Debug, Clone, Default)]
pub struct TemplateContext {
    variables: HashMap<String, String>,
}

impl TemplateContext {
    /// Create a new empty template context
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a variable in the context
    pub fn set<K: Into<String>, V: Into<String>>(&mut self, key: K, value: V) -> &mut Self {
        self.variables.insert(key.into(), value.into());
        self
    }

    /// Get a variable from the context
    pub fn get(&self, key: &str) -> Option<&String> {
        self.variables.get(key)
    }

    /// Check if a variable exists in the context
    pub fn contains(&self, key: &str) -> bool {
        self.variables.contains_key(key)
    }

    /// Get all variables as a reference to the underlying HashMap
    pub fn variables(&self) -> &HashMap<String, String> {
        &self.variables
    }

    /// Merge another context into this one, overwriting existing keys
    pub fn merge(&mut self, other: &TemplateContext) -> &mut Self {
        for (key, value) in &other.variables {
            self.variables.insert(key.clone(), value.clone());
        }
        self
    }
}

impl<K, V> FromIterator<(K, V)> for TemplateContext
where
    K: Into<String>,
    V: Into<String>,
{
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        let mut ctx = Self::new();
        for (k, v) in iter {
            ctx.set(k, v);
        }
        ctx
    }
}

/// A trait for types that can be rendered as templates
pub trait Template {
    /// Get the template content as a string
    fn content(&self) -> &str;

    /// Get the template name (for error messages)
    fn name(&self) -> &str;
}

/// A simple string-based template
#[derive(Debug, Clone)]
pub struct StringTemplate {
    name: String,
    content: String,
}

impl StringTemplate {
    /// Create a new string template
    pub fn new<N: Into<String>, C: Into<String>>(name: N, content: C) -> Self {
        Self {
            name: name.into(),
            content: content.into(),
        }
    }
}

impl Template for StringTemplate {
    fn content(&self) -> &str {
        &self.content
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// A trait for rendering templates with variable substitution
pub trait TemplateRenderer {
    /// Render a template with the given context
    fn render(&self, template: &dyn Template, context: &TemplateContext) -> Result<String>;
}

/// A simple template renderer that performs basic variable substitution
///
/// Variables are denoted by `{{variable_name}}` syntax.
///
/// # Examples
/// ```
/// use kubegen::templates::{SimpleRenderer, TemplateRenderer, StringTemplate, TemplateContext, Template};
///
/// let renderer = SimpleRenderer::new();
/// let template = StringTemplate::new("test", "Hello, {{name}}!");
/// let mut ctx = TemplateContext::new();
/// ctx.set("name", "World");
///
/// let result = renderer.render(&template, &ctx).unwrap();
/// assert_eq!(result, "Hello, World!");
/// ```
#[derive(Debug, Clone, Default)]
pub struct SimpleRenderer {
    /// If true, missing variables cause an error. If false, they are left as-is.
    strict: bool,
}

impl SimpleRenderer {
    /// Create a new simple renderer with default settings (non-strict mode)
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new strict renderer that errors on missing variables
    pub fn strict() -> Self {
        Self { strict: true }
    }

    /// Set whether the renderer should be strict about missing variables
    pub fn with_strict(mut self, strict: bool) -> Self {
        self.strict = strict;
        self
    }
}

impl TemplateRenderer for SimpleRenderer {
    fn render(&self, template: &dyn Template, context: &TemplateContext) -> Result<String> {
        let content = template.content();
        let mut result = String::with_capacity(content.len());
        let mut chars = content.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '{' && chars.peek() == Some(&'{') {
                // Consume the second '{'
                chars.next();

                // Find the variable name
                let mut var_name = String::new();
                let mut found_end = false;

                while let Some(vc) = chars.next() {
                    if vc == '}' && chars.peek() == Some(&'}') {
                        chars.next(); // Consume the second '}'
                        found_end = true;
                        break;
                    }
                    var_name.push(vc);
                }

                if !found_end {
                    return Err(KubegenError::TemplateRender {
                        template_name: template.name().to_string(),
                        reason: format!("Unclosed variable tag '{{{{{}' ", var_name),
                    });
                }

                let var_name = var_name.trim();

                if var_name.is_empty() {
                    return Err(KubegenError::TemplateRender {
                        template_name: template.name().to_string(),
                        reason: "Empty variable name in template".to_string(),
                    });
                }

                match context.get(var_name) {
                    Some(value) => result.push_str(value),
                    None => {
                        if self.strict {
                            return Err(KubegenError::TemplateRender {
                                template_name: template.name().to_string(),
                                reason: format!("Missing variable '{}'", var_name),
                            });
                        } else {
                            // In non-strict mode, leave the variable as-is
                            result.push_str("{{");
                            result.push_str(var_name);
                            result.push_str("}}");
                        }
                    }
                }
            } else {
                result.push(c);
            }
        }

        Ok(result)
    }
}

/// Convenience function to render a string template with a context
pub fn render_string(template: &str, context: &TemplateContext) -> Result<String> {
    let renderer = SimpleRenderer::new();
    let tmpl = StringTemplate::new("inline", template);
    renderer.render(&tmpl, context)
}

/// Convenience function to render a string template strictly (error on missing vars)
pub fn render_string_strict(template: &str, context: &TemplateContext) -> Result<String> {
    let renderer = SimpleRenderer::strict();
    let tmpl = StringTemplate::new("inline", template);
    renderer.render(&tmpl, context)
}

#[cfg(test)]
mod tests {
    use super::*;

    // TemplateContext tests
    #[test]
    fn test_context_new() {
        let ctx = TemplateContext::new();
        assert!(ctx.variables().is_empty());
    }

    #[test]
    fn test_context_set_and_get() {
        let mut ctx = TemplateContext::new();
        ctx.set("key", "value");
        assert_eq!(ctx.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_context_get_missing() {
        let ctx = TemplateContext::new();
        assert_eq!(ctx.get("missing"), None);
    }

    #[test]
    fn test_context_contains() {
        let mut ctx = TemplateContext::new();
        ctx.set("exists", "yes");
        assert!(ctx.contains("exists"));
        assert!(!ctx.contains("missing"));
    }

    #[test]
    fn test_context_set_chaining() {
        let mut ctx = TemplateContext::new();
        ctx.set("a", "1").set("b", "2").set("c", "3");
        assert_eq!(ctx.get("a"), Some(&"1".to_string()));
        assert_eq!(ctx.get("b"), Some(&"2".to_string()));
        assert_eq!(ctx.get("c"), Some(&"3".to_string()));
    }

    #[test]
    fn test_context_overwrite() {
        let mut ctx = TemplateContext::new();
        ctx.set("key", "original");
        ctx.set("key", "updated");
        assert_eq!(ctx.get("key"), Some(&"updated".to_string()));
    }

    #[test]
    fn test_context_merge() {
        let mut ctx1 = TemplateContext::new();
        ctx1.set("a", "1").set("b", "2");

        let mut ctx2 = TemplateContext::new();
        ctx2.set("b", "overwritten").set("c", "3");

        ctx1.merge(&ctx2);
        assert_eq!(ctx1.get("a"), Some(&"1".to_string()));
        assert_eq!(ctx1.get("b"), Some(&"overwritten".to_string()));
        assert_eq!(ctx1.get("c"), Some(&"3".to_string()));
    }

    #[test]
    fn test_context_from_iter() {
        let ctx: TemplateContext = [("a", "1"), ("b", "2")].into_iter().collect();
        assert_eq!(ctx.get("a"), Some(&"1".to_string()));
        assert_eq!(ctx.get("b"), Some(&"2".to_string()));
    }

    #[test]
    fn test_context_clone() {
        let mut ctx = TemplateContext::new();
        ctx.set("key", "value");
        let cloned = ctx.clone();
        assert_eq!(cloned.get("key"), Some(&"value".to_string()));
    }

    // StringTemplate tests
    #[test]
    fn test_string_template_new() {
        let tmpl = StringTemplate::new("test", "content");
        assert_eq!(tmpl.name(), "test");
        assert_eq!(tmpl.content(), "content");
    }

    #[test]
    fn test_string_template_clone() {
        let tmpl = StringTemplate::new("test", "content");
        let cloned = tmpl.clone();
        assert_eq!(cloned.name(), "test");
        assert_eq!(cloned.content(), "content");
    }

    // SimpleRenderer tests
    #[test]
    fn test_render_no_variables() {
        let renderer = SimpleRenderer::new();
        let tmpl = StringTemplate::new("test", "Hello, World!");
        let ctx = TemplateContext::new();

        let result = renderer.render(&tmpl, &ctx).unwrap();
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_render_single_variable() {
        let renderer = SimpleRenderer::new();
        let tmpl = StringTemplate::new("test", "Hello, {{name}}!");
        let mut ctx = TemplateContext::new();
        ctx.set("name", "World");

        let result = renderer.render(&tmpl, &ctx).unwrap();
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_render_multiple_variables() {
        let renderer = SimpleRenderer::new();
        let tmpl = StringTemplate::new("test", "{{greeting}}, {{name}}!");
        let mut ctx = TemplateContext::new();
        ctx.set("greeting", "Hi");
        ctx.set("name", "User");

        let result = renderer.render(&tmpl, &ctx).unwrap();
        assert_eq!(result, "Hi, User!");
    }

    #[test]
    fn test_render_repeated_variable() {
        let renderer = SimpleRenderer::new();
        let tmpl = StringTemplate::new("test", "{{x}} + {{x}} = {{x}}{{x}}");
        let mut ctx = TemplateContext::new();
        ctx.set("x", "1");

        let result = renderer.render(&tmpl, &ctx).unwrap();
        assert_eq!(result, "1 + 1 = 11");
    }

    #[test]
    fn test_render_variable_with_spaces() {
        let renderer = SimpleRenderer::new();
        let tmpl = StringTemplate::new("test", "Hello, {{ name }}!");
        let mut ctx = TemplateContext::new();
        ctx.set("name", "World");

        let result = renderer.render(&tmpl, &ctx).unwrap();
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_render_missing_variable_non_strict() {
        let renderer = SimpleRenderer::new();
        let tmpl = StringTemplate::new("test", "Hello, {{name}}!");
        let ctx = TemplateContext::new();

        let result = renderer.render(&tmpl, &ctx).unwrap();
        assert_eq!(result, "Hello, {{name}}!");
    }

    #[test]
    fn test_render_missing_variable_strict() {
        let renderer = SimpleRenderer::strict();
        let tmpl = StringTemplate::new("test", "Hello, {{name}}!");
        let ctx = TemplateContext::new();

        let result = renderer.render(&tmpl, &ctx);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing variable"));
    }

    #[test]
    fn test_render_empty_variable_name() {
        let renderer = SimpleRenderer::new();
        let tmpl = StringTemplate::new("test", "Hello, {{}}!");
        let ctx = TemplateContext::new();

        let result = renderer.render(&tmpl, &ctx);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Empty variable"));
    }

    #[test]
    fn test_render_unclosed_variable() {
        let renderer = SimpleRenderer::new();
        let tmpl = StringTemplate::new("test", "Hello, {{name!");
        let ctx = TemplateContext::new();

        let result = renderer.render(&tmpl, &ctx);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unclosed"));
    }

    #[test]
    fn test_render_single_brace() {
        let renderer = SimpleRenderer::new();
        let tmpl = StringTemplate::new("test", "Use {braces} normally");
        let ctx = TemplateContext::new();

        let result = renderer.render(&tmpl, &ctx).unwrap();
        assert_eq!(result, "Use {braces} normally");
    }

    #[test]
    fn test_render_multiline() {
        let renderer = SimpleRenderer::new();
        let tmpl = StringTemplate::new(
            "test",
            "Line 1: {{var1}}\nLine 2: {{var2}}\nLine 3: {{var1}}",
        );
        let mut ctx = TemplateContext::new();
        ctx.set("var1", "A");
        ctx.set("var2", "B");

        let result = renderer.render(&tmpl, &ctx).unwrap();
        assert_eq!(result, "Line 1: A\nLine 2: B\nLine 3: A");
    }

    #[test]
    fn test_render_complex_content() {
        let renderer = SimpleRenderer::new();
        let tmpl = StringTemplate::new(
            "Cargo.toml",
            r#"[package]
name = "{{project_name}}"
version = "{{version}}"
edition = "2021"

[dependencies]
kube = { version = "{{kube_version}}", features = ["runtime", "derive"] }
"#,
        );

        let mut ctx = TemplateContext::new();
        ctx.set("project_name", "my-operator");
        ctx.set("version", "0.1.0");
        ctx.set("kube_version", "0.88");

        let result = renderer.render(&tmpl, &ctx).unwrap();
        assert!(result.contains("name = \"my-operator\""));
        assert!(result.contains("version = \"0.1.0\""));
        assert!(result.contains("kube = { version = \"0.88\""));
    }

    #[test]
    fn test_render_with_strict_mode() {
        let renderer = SimpleRenderer::new().with_strict(true);
        let tmpl = StringTemplate::new("test", "{{missing}}");
        let ctx = TemplateContext::new();

        let result = renderer.render(&tmpl, &ctx);
        assert!(result.is_err());
    }

    // Convenience function tests
    #[test]
    fn test_render_string() {
        let mut ctx = TemplateContext::new();
        ctx.set("name", "test");

        let result = render_string("Hello, {{name}}!", &ctx).unwrap();
        assert_eq!(result, "Hello, test!");
    }

    #[test]
    fn test_render_string_strict() {
        let ctx = TemplateContext::new();
        let result = render_string_strict("Hello, {{name}}!", &ctx);
        assert!(result.is_err());
    }

    // Edge cases
    #[test]
    fn test_render_empty_template() {
        let renderer = SimpleRenderer::new();
        let tmpl = StringTemplate::new("test", "");
        let ctx = TemplateContext::new();

        let result = renderer.render(&tmpl, &ctx).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_render_only_variable() {
        let renderer = SimpleRenderer::new();
        let tmpl = StringTemplate::new("test", "{{value}}");
        let mut ctx = TemplateContext::new();
        ctx.set("value", "content");

        let result = renderer.render(&tmpl, &ctx).unwrap();
        assert_eq!(result, "content");
    }

    #[test]
    fn test_render_adjacent_variables() {
        let renderer = SimpleRenderer::new();
        let tmpl = StringTemplate::new("test", "{{a}}{{b}}{{c}}");
        let mut ctx = TemplateContext::new();
        ctx.set("a", "1");
        ctx.set("b", "2");
        ctx.set("c", "3");

        let result = renderer.render(&tmpl, &ctx).unwrap();
        assert_eq!(result, "123");
    }

    #[test]
    fn test_render_variable_with_underscore() {
        let renderer = SimpleRenderer::new();
        let tmpl = StringTemplate::new("test", "{{my_var}}");
        let mut ctx = TemplateContext::new();
        ctx.set("my_var", "value");

        let result = renderer.render(&tmpl, &ctx).unwrap();
        assert_eq!(result, "value");
    }

    #[test]
    fn test_render_variable_with_hyphen() {
        let renderer = SimpleRenderer::new();
        let tmpl = StringTemplate::new("test", "{{my-var}}");
        let mut ctx = TemplateContext::new();
        ctx.set("my-var", "value");

        let result = renderer.render(&tmpl, &ctx).unwrap();
        assert_eq!(result, "value");
    }

    #[test]
    fn test_render_special_chars_in_value() {
        let renderer = SimpleRenderer::new();
        let tmpl = StringTemplate::new("test", "Value: {{val}}");
        let mut ctx = TemplateContext::new();
        ctx.set("val", "{{nested}} and <html> & \"quotes\"");

        let result = renderer.render(&tmpl, &ctx).unwrap();
        assert_eq!(result, "Value: {{nested}} and <html> & \"quotes\"");
    }

    #[test]
    fn test_context_debug() {
        let mut ctx = TemplateContext::new();
        ctx.set("key", "value");
        let debug_str = format!("{:?}", ctx);
        assert!(debug_str.contains("TemplateContext"));
    }

    #[test]
    fn test_renderer_debug() {
        let renderer = SimpleRenderer::new();
        let debug_str = format!("{:?}", renderer);
        assert!(debug_str.contains("SimpleRenderer"));
    }
}
