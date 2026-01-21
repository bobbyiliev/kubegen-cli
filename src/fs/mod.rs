//! Filesystem operations for kubegen
//!
//! Provides safe filesystem helpers with error context and dry-run support.

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{KubegenError, Result};

/// A planned filesystem operation for dry-run mode
#[derive(Debug, Clone, PartialEq)]
pub enum PlannedOperation {
    /// Create a file with the given content
    CreateFile { path: PathBuf, content: String },
    /// Create a directory
    CreateDir { path: PathBuf },
}

impl PlannedOperation {
    /// Get the path affected by this operation
    pub fn path(&self) -> &Path {
        match self {
            PlannedOperation::CreateFile { path, .. } => path,
            PlannedOperation::CreateDir { path } => path,
        }
    }
}

/// Context for dry-run mode that collects planned operations
#[derive(Debug, Default)]
pub struct DryRunContext {
    operations: Vec<PlannedOperation>,
}

impl DryRunContext {
    /// Create a new dry-run context
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a planned file creation
    pub fn plan_file<P: AsRef<Path>>(&mut self, path: P, content: &str) {
        self.operations.push(PlannedOperation::CreateFile {
            path: path.as_ref().to_path_buf(),
            content: content.to_string(),
        });
    }

    /// Record a planned directory creation
    pub fn plan_dir<P: AsRef<Path>>(&mut self, path: P) {
        self.operations.push(PlannedOperation::CreateDir {
            path: path.as_ref().to_path_buf(),
        });
    }

    /// Get all planned operations
    pub fn operations(&self) -> &[PlannedOperation] {
        &self.operations
    }

    /// Format the dry-run output for display
    pub fn format_preview(&self) -> String {
        if self.operations.is_empty() {
            return "No changes planned.".to_string();
        }

        let mut output = String::from("Dry run - the following changes would be made:\n\n");

        for op in &self.operations {
            match op {
                PlannedOperation::CreateDir { path } => {
                    output.push_str(&format!("📁 CREATE DIR: {}\n", path.display()));
                }
                PlannedOperation::CreateFile { path, content } => {
                    output.push_str(&format!("📄 CREATE FILE: {}\n", path.display()));
                    // Show preview of content (first few lines)
                    let preview_lines: Vec<&str> = content.lines().take(10).collect();
                    if !preview_lines.is_empty() {
                        output.push_str("   Content preview:\n");
                        for line in &preview_lines {
                            output.push_str(&format!("   │ {}\n", line));
                        }
                        let total_lines = content.lines().count();
                        if total_lines > 10 {
                            output
                                .push_str(&format!("   │ ... ({} more lines)\n", total_lines - 10));
                        }
                    }
                    output.push('\n');
                }
            }
        }

        output
    }
}

/// Write content to a file, creating parent directories if needed
///
/// # Arguments
/// * `path` - The file path to write to
/// * `content` - The content to write
///
/// # Errors
/// Returns an error if the file cannot be written or parent directories cannot be created
pub fn write_file<P: AsRef<Path>>(path: P, content: &str) -> Result<()> {
    let path = path.as_ref();

    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            create_dir_all(parent)?;
        }
    }

    fs::write(path, content).map_err(|source| KubegenError::FileWrite {
        path: path.to_path_buf(),
        source,
    })
}

/// Create a directory and all parent directories
///
/// # Arguments
/// * `path` - The directory path to create
///
/// # Errors
/// Returns an error if the directory cannot be created
pub fn create_dir_all<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();
    fs::create_dir_all(path).map_err(|source| KubegenError::DirectoryCreate {
        path: path.to_path_buf(),
        source,
    })
}

/// Check if a file exists
pub fn file_exists<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().is_file()
}

/// Check if a directory exists
pub fn dir_exists<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().is_dir()
}

/// Read file contents to string
///
/// # Arguments
/// * `path` - The file path to read
///
/// # Errors
/// Returns an error if the file cannot be read
pub fn read_to_string<P: AsRef<Path>>(path: P) -> Result<String> {
    let path = path.as_ref();
    fs::read_to_string(path).map_err(|source| KubegenError::FileRead {
        path: path.to_path_buf(),
        source,
    })
}

/// Check if path exists and fail if it does (overwrite protection)
///
/// # Arguments
/// * `path` - The path to check
///
/// # Errors
/// Returns an error if the path already exists
pub fn ensure_not_exists<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();
    if path.is_dir() {
        return Err(KubegenError::DirectoryExists {
            path: path.to_path_buf(),
        });
    }
    if path.is_file() {
        return Err(KubegenError::FileExists {
            path: path.to_path_buf(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_write_and_read_file() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("test.txt");

        write_file(&file_path, "hello world").unwrap();
        assert!(file_exists(&file_path));

        let content = read_to_string(&file_path).unwrap();
        assert_eq!(content, "hello world");
    }

    #[test]
    fn test_write_file_creates_parent_dirs() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("nested/dir/test.txt");

        write_file(&file_path, "content").unwrap();
        assert!(file_exists(&file_path));
    }

    #[test]
    fn test_create_dir_all() {
        let temp = TempDir::new().unwrap();
        let dir_path = temp.path().join("a/b/c");

        create_dir_all(&dir_path).unwrap();
        assert!(dir_exists(&dir_path));
    }

    #[test]
    fn test_file_exists_returns_false_for_missing() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("nonexistent.txt");

        assert!(!file_exists(&file_path));
    }

    #[test]
    fn test_dir_exists_returns_false_for_missing() {
        let temp = TempDir::new().unwrap();
        let dir_path = temp.path().join("nonexistent");

        assert!(!dir_exists(&dir_path));
    }

    #[test]
    fn test_ensure_not_exists_passes_for_missing() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("nonexistent");

        assert!(ensure_not_exists(&path).is_ok());
    }

    #[test]
    fn test_ensure_not_exists_fails_for_existing_file() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("test.txt");

        write_file(&file_path, "content").unwrap();

        let result = ensure_not_exists(&file_path);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("File already exists"));
    }

    #[test]
    fn test_ensure_not_exists_fails_for_existing_dir() {
        let temp = TempDir::new().unwrap();
        let dir_path = temp.path().join("testdir");

        create_dir_all(&dir_path).unwrap();

        let result = ensure_not_exists(&dir_path);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Directory already exists"));
    }

    #[test]
    fn test_read_nonexistent_file_fails() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("nonexistent.txt");

        let result = read_to_string(&file_path);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to read file"));
    }

    // Dry-run context tests
    #[test]
    fn test_dry_run_context_new() {
        let ctx = DryRunContext::new();
        assert!(ctx.operations().is_empty());
    }

    #[test]
    fn test_dry_run_plan_file() {
        let mut ctx = DryRunContext::new();
        ctx.plan_file("/tmp/test.txt", "hello world");

        assert_eq!(ctx.operations().len(), 1);
        match &ctx.operations()[0] {
            PlannedOperation::CreateFile { path, content } => {
                assert_eq!(path.to_str().unwrap(), "/tmp/test.txt");
                assert_eq!(content, "hello world");
            }
            _ => panic!("Expected CreateFile operation"),
        }
    }

    #[test]
    fn test_dry_run_plan_dir() {
        let mut ctx = DryRunContext::new();
        ctx.plan_dir("/tmp/mydir");

        assert_eq!(ctx.operations().len(), 1);
        match &ctx.operations()[0] {
            PlannedOperation::CreateDir { path } => {
                assert_eq!(path.to_str().unwrap(), "/tmp/mydir");
            }
            _ => panic!("Expected CreateDir operation"),
        }
    }

    #[test]
    fn test_dry_run_multiple_operations() {
        let mut ctx = DryRunContext::new();
        ctx.plan_dir("/tmp/project");
        ctx.plan_file("/tmp/project/Cargo.toml", "[package]\nname = \"test\"");
        ctx.plan_file("/tmp/project/src/main.rs", "fn main() {}");

        assert_eq!(ctx.operations().len(), 3);
    }

    #[test]
    fn test_planned_operation_path() {
        let file_op = PlannedOperation::CreateFile {
            path: PathBuf::from("/tmp/test.txt"),
            content: "content".to_string(),
        };
        let dir_op = PlannedOperation::CreateDir {
            path: PathBuf::from("/tmp/mydir"),
        };

        assert_eq!(file_op.path().to_str().unwrap(), "/tmp/test.txt");
        assert_eq!(dir_op.path().to_str().unwrap(), "/tmp/mydir");
    }

    #[test]
    fn test_format_preview_empty() {
        let ctx = DryRunContext::new();
        let preview = ctx.format_preview();
        assert_eq!(preview, "No changes planned.");
    }

    #[test]
    fn test_format_preview_with_operations() {
        let mut ctx = DryRunContext::new();
        ctx.plan_dir("/tmp/project");
        ctx.plan_file("/tmp/project/test.txt", "line 1\nline 2");

        let preview = ctx.format_preview();
        assert!(preview.contains("Dry run"));
        assert!(preview.contains("CREATE DIR"));
        assert!(preview.contains("CREATE FILE"));
        assert!(preview.contains("/tmp/project"));
        assert!(preview.contains("line 1"));
    }

    #[test]
    fn test_format_preview_truncates_long_content() {
        let mut ctx = DryRunContext::new();
        let long_content: String = (1..=20)
            .map(|i| format!("line {}", i))
            .collect::<Vec<_>>()
            .join("\n");
        ctx.plan_file("/tmp/test.txt", &long_content);

        let preview = ctx.format_preview();
        assert!(preview.contains("line 1"));
        assert!(preview.contains("line 10"));
        assert!(preview.contains("more lines"));
        assert!(!preview.contains("line 11"));
    }
}
