//! Error types for kubegen
//!
//! This module provides structured error handling using thiserror.
//! All errors include context and user-friendly messages.

use std::path::PathBuf;
use thiserror::Error;

/// Result type alias for kubegen operations
pub type Result<T> = std::result::Result<T, KubegenError>;

/// Main error type for kubegen
#[derive(Error, Debug)]
pub enum KubegenError {
    /// Invalid project name
    #[error("Invalid project name '{name}': {reason}")]
    InvalidProjectName { name: String, reason: String },

    /// Invalid CRD name
    #[error("Invalid CRD name - group: '{group}', version: '{version}', kind: '{kind}': {reason}")]
    InvalidCrdName {
        group: String,
        version: String,
        kind: String,
        reason: String,
    },

    /// Directory already exists
    #[error("Directory already exists: {path}")]
    DirectoryExists { path: PathBuf },

    /// File already exists
    #[error("File already exists: {path}")]
    FileExists { path: PathBuf },

    /// Failed to create directory
    #[error("Failed to create directory {path}: {source}")]
    DirectoryCreate {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Failed to write file
    #[error("Failed to write file {path}: {source}")]
    FileWrite {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Failed to read file
    #[error("Failed to read file {path}: {source}")]
    FileRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Template not found
    #[error("Template not found: {template_name}")]
    TemplateNotFound { template_name: String },

    /// Template rendering error
    #[error("Failed to render template '{template_name}': {reason}")]
    TemplateRender {
        template_name: String,
        reason: String,
    },

    /// Invalid path
    #[error("Invalid path: {path}")]
    InvalidPath { path: PathBuf },

    /// Project not found
    #[error("Project not initialized in current directory")]
    ProjectNotFound,

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Validation error (generic)
    #[error("{0}")]
    ValidationError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_project_name_error() {
        let err = KubegenError::InvalidProjectName {
            name: "My-Operator".to_string(),
            reason: "must be lowercase".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Invalid project name 'My-Operator': must be lowercase"
        );
    }

    #[test]
    fn test_directory_exists_error() {
        let err = KubegenError::DirectoryExists {
            path: PathBuf::from("/tmp/test"),
        };
        assert_eq!(err.to_string(), "Directory already exists: /tmp/test");
    }

    #[test]
    fn test_template_not_found_error() {
        let err = KubegenError::TemplateNotFound {
            template_name: "crd.yaml".to_string(),
        };
        assert_eq!(err.to_string(), "Template not found: crd.yaml");
    }

    #[test]
    fn test_invalid_crd_name_error() {
        let err = KubegenError::InvalidCrdName {
            group: "example.com".to_string(),
            version: "v1".to_string(),
            kind: "my-resource".to_string(),
            reason: "kind must be PascalCase".to_string(),
        };
        assert!(err
            .to_string()
            .contains("Invalid CRD name - group: 'example.com'"));
        assert!(err.to_string().contains("kind must be PascalCase"));
    }

    #[test]
    fn test_result_type_alias() {
        fn returns_error() -> Result<()> {
            Err(KubegenError::ProjectNotFound)
        }

        let result = returns_error();
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(
                e.to_string(),
                "Project not initialized in current directory"
            );
        }
    }

    #[test]
    fn test_validation_error() {
        let err = KubegenError::ValidationError("Custom validation message".to_string());
        assert_eq!(err.to_string(), "Custom validation message");
    }
}
