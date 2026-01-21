//! Validation utilities for kubegen
//!
//! Provides validation for project names, CRD names, and Kubernetes resource names.

use crate::error::{KubegenError, Result};

/// Validate a project name according to DNS-1123 subdomain rules.
///
/// Project names must:
/// - Be lowercase
/// - Contain only alphanumeric characters and hyphens
/// - Start and end with an alphanumeric character
/// - Be between 1 and 63 characters
///
/// # Examples
/// ```
/// use kubegen::validation::validate_project_name;
///
/// assert!(validate_project_name("my-operator").is_ok());
/// assert!(validate_project_name("My-Operator").is_err()); // uppercase
/// assert!(validate_project_name("-operator").is_err());   // starts with hyphen
/// ```
pub fn validate_project_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(KubegenError::InvalidProjectName {
            name: name.to_string(),
            reason: "name cannot be empty".to_string(),
        });
    }

    if name.len() > 63 {
        return Err(KubegenError::InvalidProjectName {
            name: name.to_string(),
            reason: "name must be 63 characters or less".to_string(),
        });
    }

    if name != name.to_lowercase() {
        return Err(KubegenError::InvalidProjectName {
            name: name.to_string(),
            reason: "name must be lowercase".to_string(),
        });
    }

    if !name.chars().next().unwrap().is_ascii_alphanumeric() {
        return Err(KubegenError::InvalidProjectName {
            name: name.to_string(),
            reason: "name must start with an alphanumeric character".to_string(),
        });
    }

    if !name.chars().last().unwrap().is_ascii_alphanumeric() {
        return Err(KubegenError::InvalidProjectName {
            name: name.to_string(),
            reason: "name must end with an alphanumeric character".to_string(),
        });
    }

    for c in name.chars() {
        if !c.is_ascii_alphanumeric() && c != '-' {
            return Err(KubegenError::InvalidProjectName {
                name: name.to_string(),
                reason: format!(
                    "name contains invalid character '{}', only lowercase alphanumeric and hyphens allowed",
                    c
                ),
            });
        }
    }

    Ok(())
}

/// Validate a CRD kind name (must be PascalCase).
///
/// Kind names must:
/// - Start with an uppercase letter
/// - Contain only alphanumeric characters
/// - Be a valid Rust identifier
///
/// # Examples
/// ```
/// use kubegen::validation::validate_crd_kind;
///
/// assert!(validate_crd_kind("MyResource").is_ok());
/// assert!(validate_crd_kind("myresource").is_err());  // not PascalCase
/// assert!(validate_crd_kind("My-Resource").is_err()); // contains hyphen
/// ```
pub fn validate_crd_kind(kind: &str) -> Result<()> {
    if kind.is_empty() {
        return Err(KubegenError::InvalidCrdName {
            group: String::new(),
            version: String::new(),
            kind: kind.to_string(),
            reason: "kind cannot be empty".to_string(),
        });
    }

    let first_char = kind.chars().next().unwrap();
    if !first_char.is_ascii_uppercase() {
        return Err(KubegenError::InvalidCrdName {
            group: String::new(),
            version: String::new(),
            kind: kind.to_string(),
            reason: "kind must start with an uppercase letter (PascalCase)".to_string(),
        });
    }

    for c in kind.chars() {
        if !c.is_ascii_alphanumeric() {
            return Err(KubegenError::InvalidCrdName {
                group: String::new(),
                version: String::new(),
                kind: kind.to_string(),
                reason: format!(
                    "kind contains invalid character '{}', only alphanumeric characters allowed",
                    c
                ),
            });
        }
    }

    Ok(())
}

/// Validate a CRD API version.
///
/// API versions must match the pattern: v[0-9]+(alpha|beta)?[0-9]*
/// Examples: v1, v1alpha1, v1beta1, v2beta2
///
/// # Examples
/// ```
/// use kubegen::validation::validate_crd_version;
///
/// assert!(validate_crd_version("v1").is_ok());
/// assert!(validate_crd_version("v1alpha1").is_ok());
/// assert!(validate_crd_version("v1beta2").is_ok());
/// assert!(validate_crd_version("1.0").is_err());
/// ```
pub fn validate_crd_version(version: &str) -> Result<()> {
    if version.is_empty() {
        return Err(KubegenError::InvalidCrdName {
            group: String::new(),
            version: version.to_string(),
            kind: String::new(),
            reason: "version cannot be empty".to_string(),
        });
    }

    if !version.starts_with('v') {
        return Err(KubegenError::InvalidCrdName {
            group: String::new(),
            version: version.to_string(),
            kind: String::new(),
            reason: "version must start with 'v' (e.g., v1, v1alpha1)".to_string(),
        });
    }

    let after_v = &version[1..];
    if after_v.is_empty() {
        return Err(KubegenError::InvalidCrdName {
            group: String::new(),
            version: version.to_string(),
            kind: String::new(),
            reason: "version must have a number after 'v' (e.g., v1, v1alpha1)".to_string(),
        });
    }

    // Check for valid pattern: number, optional (alpha|beta), optional number
    let valid_patterns = [
        regex_lite_match(after_v, r"^\d+$"),         // v1, v2
        regex_lite_match(after_v, r"^\d+alpha\d*$"), // v1alpha1
        regex_lite_match(after_v, r"^\d+beta\d*$"),  // v1beta1
    ];

    if !valid_patterns.iter().any(|&x| x) {
        return Err(KubegenError::InvalidCrdName {
            group: String::new(),
            version: version.to_string(),
            kind: String::new(),
            reason: "version must match pattern v[0-9]+(alpha|beta)?[0-9]* (e.g., v1, v1alpha1, v1beta2)".to_string(),
        });
    }

    Ok(())
}

/// Simple pattern matching without regex dependency
fn regex_lite_match(s: &str, pattern: &str) -> bool {
    match pattern {
        r"^\d+$" => s.chars().all(|c| c.is_ascii_digit()) && !s.is_empty(),
        r"^\d+alpha\d*$" => {
            if let Some(idx) = s.find("alpha") {
                let before = &s[..idx];
                let after = &s[idx + 5..];
                !before.is_empty()
                    && before.chars().all(|c| c.is_ascii_digit())
                    && after.chars().all(|c| c.is_ascii_digit())
            } else {
                false
            }
        }
        r"^\d+beta\d*$" => {
            if let Some(idx) = s.find("beta") {
                let before = &s[..idx];
                let after = &s[idx + 4..];
                !before.is_empty()
                    && before.chars().all(|c| c.is_ascii_digit())
                    && after.chars().all(|c| c.is_ascii_digit())
            } else {
                false
            }
        }
        _ => false,
    }
}

/// Validate a CRD API group (DNS subdomain).
///
/// API groups must be valid DNS subdomains:
/// - Lowercase
/// - Contain only alphanumeric characters, hyphens, and dots
/// - Each segment must start and end with alphanumeric
///
/// # Examples
/// ```
/// use kubegen::validation::validate_crd_group;
///
/// assert!(validate_crd_group("example.com").is_ok());
/// assert!(validate_crd_group("apps.kubernetes.io").is_ok());
/// assert!(validate_crd_group("Example.com").is_err()); // uppercase
/// ```
pub fn validate_crd_group(group: &str) -> Result<()> {
    if group.is_empty() {
        return Err(KubegenError::InvalidCrdName {
            group: group.to_string(),
            version: String::new(),
            kind: String::new(),
            reason: "group cannot be empty".to_string(),
        });
    }

    if group != group.to_lowercase() {
        return Err(KubegenError::InvalidCrdName {
            group: group.to_string(),
            version: String::new(),
            kind: String::new(),
            reason: "group must be lowercase".to_string(),
        });
    }

    for segment in group.split('.') {
        if segment.is_empty() {
            return Err(KubegenError::InvalidCrdName {
                group: group.to_string(),
                version: String::new(),
                kind: String::new(),
                reason: "group segments cannot be empty".to_string(),
            });
        }

        if !segment.chars().next().unwrap().is_ascii_alphanumeric() {
            return Err(KubegenError::InvalidCrdName {
                group: group.to_string(),
                version: String::new(),
                kind: String::new(),
                reason: "each group segment must start with alphanumeric character".to_string(),
            });
        }

        if !segment.chars().last().unwrap().is_ascii_alphanumeric() {
            return Err(KubegenError::InvalidCrdName {
                group: group.to_string(),
                version: String::new(),
                kind: String::new(),
                reason: "each group segment must end with alphanumeric character".to_string(),
            });
        }

        for c in segment.chars() {
            if !c.is_ascii_alphanumeric() && c != '-' {
                return Err(KubegenError::InvalidCrdName {
                    group: group.to_string(),
                    version: String::new(),
                    kind: String::new(),
                    reason: format!("group contains invalid character '{}'", c),
                });
            }
        }
    }

    Ok(())
}

/// Convert a PascalCase kind to snake_case
///
/// # Examples
/// ```
/// use kubegen::validation::to_snake_case;
///
/// assert_eq!(to_snake_case("MyResource"), "my_resource");
/// assert_eq!(to_snake_case("Pod"), "pod");
/// ```
pub fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}

/// Generate plural form of a kind name (simple English rules)
///
/// # Examples
/// ```
/// use kubegen::validation::pluralize;
///
/// assert_eq!(pluralize("Pod"), "pods");
/// assert_eq!(pluralize("Policy"), "policies");
/// assert_eq!(pluralize("Ingress"), "ingresses");
/// ```
pub fn pluralize(s: &str) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    // Project name validation tests
    #[test]
    fn test_valid_project_names() {
        assert!(validate_project_name("my-operator").is_ok());
        assert!(validate_project_name("operator").is_ok());
        assert!(validate_project_name("my-cool-operator").is_ok());
        assert!(validate_project_name("op123").is_ok());
        assert!(validate_project_name("a").is_ok());
    }

    #[test]
    fn test_invalid_project_name_uppercase() {
        let result = validate_project_name("My-Operator");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("lowercase"));
    }

    #[test]
    fn test_invalid_project_name_starts_with_hyphen() {
        let result = validate_project_name("-operator");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("start with"));
    }

    #[test]
    fn test_invalid_project_name_ends_with_hyphen() {
        let result = validate_project_name("operator-");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("end with"));
    }

    #[test]
    fn test_invalid_project_name_empty() {
        let result = validate_project_name("");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }

    #[test]
    fn test_invalid_project_name_underscore() {
        let result = validate_project_name("my_operator");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("invalid character"));
    }

    // CRD kind validation tests
    #[test]
    fn test_valid_crd_kinds() {
        assert!(validate_crd_kind("MyResource").is_ok());
        assert!(validate_crd_kind("Pod").is_ok());
        assert!(validate_crd_kind("HTTPServer").is_ok());
        assert!(validate_crd_kind("A").is_ok());
    }

    #[test]
    fn test_invalid_crd_kind_lowercase_start() {
        let result = validate_crd_kind("myResource");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("uppercase"));
    }

    #[test]
    fn test_invalid_crd_kind_hyphen() {
        let result = validate_crd_kind("My-Resource");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("invalid character"));
    }

    // CRD version validation tests
    #[test]
    fn test_valid_crd_versions() {
        assert!(validate_crd_version("v1").is_ok());
        assert!(validate_crd_version("v2").is_ok());
        assert!(validate_crd_version("v1alpha1").is_ok());
        assert!(validate_crd_version("v1beta1").is_ok());
        assert!(validate_crd_version("v2beta2").is_ok());
        assert!(validate_crd_version("v10alpha").is_ok());
    }

    #[test]
    fn test_invalid_crd_version_no_v() {
        let result = validate_crd_version("1.0");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("start with 'v'"));
    }

    #[test]
    fn test_invalid_crd_version_just_v() {
        let result = validate_crd_version("v");
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_crd_version_bad_pattern() {
        let result = validate_crd_version("vfoo");
        assert!(result.is_err());
    }

    // CRD group validation tests
    #[test]
    fn test_valid_crd_groups() {
        assert!(validate_crd_group("example.com").is_ok());
        assert!(validate_crd_group("apps.kubernetes.io").is_ok());
        assert!(validate_crd_group("my-group.example.com").is_ok());
        assert!(validate_crd_group("example").is_ok());
    }

    #[test]
    fn test_invalid_crd_group_uppercase() {
        let result = validate_crd_group("Example.com");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("lowercase"));
    }

    #[test]
    fn test_invalid_crd_group_empty_segment() {
        let result = validate_crd_group("example..com");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }

    // Helper function tests
    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("MyResource"), "my_resource");
        assert_eq!(to_snake_case("Pod"), "pod");
        assert_eq!(to_snake_case("A"), "a");
        assert_eq!(to_snake_case("MyCustomResource"), "my_custom_resource");
    }

    #[test]
    fn test_pluralize() {
        assert_eq!(pluralize("Pod"), "pods");
        assert_eq!(pluralize("Policy"), "policies");
        assert_eq!(pluralize("Ingress"), "ingresses");
        assert_eq!(pluralize("Deployment"), "deployments");
        assert_eq!(pluralize("Gateway"), "gateways");
    }
}
