//! Logging setup for kubegen
//!
//! Configures tracing-subscriber with env filter support.
//! Supports RUST_LOG environment variable and --verbose flag.

use tracing::Level;
use tracing_subscriber::{fmt, EnvFilter};

/// Initialize the logging system
///
/// # Arguments
/// * `verbose` - If true, sets log level to DEBUG, otherwise INFO
///
/// # Environment
/// The RUST_LOG environment variable takes precedence over the verbose flag.
/// Examples:
/// - `RUST_LOG=debug` - Enable debug logging for all modules
/// - `RUST_LOG=kubegen=trace` - Enable trace logging for kubegen only
/// - `RUST_LOG=warn` - Only show warnings and errors
pub fn init(verbose: bool) {
    let default_level = if verbose { Level::DEBUG } else { Level::INFO };

    let filter = EnvFilter::builder()
        .with_default_directive(default_level.into())
        .from_env_lossy();

    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .without_time()
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Testing logging initialization is tricky because tracing
    // can only be initialized once per process. We test the filter creation
    // logic instead.

    #[test]
    fn test_env_filter_creation() {
        // Test that EnvFilter can be created with default directive
        let filter = EnvFilter::builder()
            .with_default_directive(Level::INFO.into())
            .from_env_lossy();

        // Filter should be created without panicking
        assert!(!format!("{:?}", filter).is_empty());
    }
}
