//! Implementation of the `kubegen version` command
//!
//! Shows detailed version and build information.

/// Version information structure
pub struct VersionInfo {
    pub version: &'static str,
    pub git_commit: &'static str,
    pub build_date: &'static str,
    pub rust_version: &'static str,
}

impl VersionInfo {
    /// Get the version information compiled into the binary
    pub fn get() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION"),
            git_commit: env!("KUBEGEN_GIT_HASH"),
            build_date: env!("KUBEGEN_BUILD_DATE"),
            rust_version: env!("KUBEGEN_RUST_VERSION"),
        }
    }
}

/// Execute the `kubegen version` command
pub fn execute_version() {
    let info = VersionInfo::get();

    println!("kubegen {}", info.version);
    println!("git commit: {}", info.git_commit);
    println!("build date: {}", info.build_date);
    println!("rust version: {}", info.rust_version);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_info_not_empty() {
        let info = VersionInfo::get();
        assert!(!info.version.is_empty());
        assert!(!info.git_commit.is_empty());
        assert!(!info.build_date.is_empty());
        assert!(!info.rust_version.is_empty());
    }

    #[test]
    fn test_version_is_semver_like() {
        let info = VersionInfo::get();
        // Version should contain at least one dot (e.g., "0.1.0")
        assert!(info.version.contains('.'));
    }

    #[test]
    fn test_rust_version_contains_rustc() {
        let info = VersionInfo::get();
        assert!(info.rust_version.contains("rustc"));
    }
}
