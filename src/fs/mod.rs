//! Filesystem operations for kubegen
//!
//! Provides safe filesystem helpers with error context and dry-run support.

use std::fs;
use std::path::Path;

use crate::error::{KubegenError, Result};

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
}
