//! Integration tests for `kubegen add metrics` command

use std::process::Command;

use tempfile::TempDir;

/// Helper to create a minimal project structure for metrics tests
fn setup_project(temp: &TempDir) {
    let project_dir = temp.path();
    std::fs::create_dir_all(project_dir.join("src")).expect("Failed to create src directory");
    std::fs::write(
        project_dir.join("Cargo.toml"),
        r#"[package]
name = "test-operator"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["full"] }
"#,
    )
    .expect("Failed to write Cargo.toml");
}

/// Test that `kubegen add metrics` generates valid metrics files
#[test]
fn test_kubegen_add_metrics_generates_files() {
    let temp = TempDir::new().expect("Failed to create temp directory");
    setup_project(&temp);

    let output = Command::new(env!("CARGO_BIN_EXE_kubegen"))
        .args(["add", "metrics"])
        .current_dir(temp.path())
        .output()
        .expect("Failed to execute kubegen");

    assert!(
        output.status.success(),
        "kubegen add metrics failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let metrics_dir = temp.path().join("src/metrics");

    // Verify metrics directory exists
    assert!(metrics_dir.exists(), "Metrics directory not created");

    // Verify mod.rs exists
    assert!(metrics_dir.join("mod.rs").exists(), "mod.rs not created");
}

/// Test that generated metrics file has correct content
#[test]
fn test_kubegen_add_metrics_file_content() {
    let temp = TempDir::new().expect("Failed to create temp directory");
    setup_project(&temp);

    let output = Command::new(env!("CARGO_BIN_EXE_kubegen"))
        .args(["add", "metrics", "--port", "9090"])
        .current_dir(temp.path())
        .output()
        .expect("Failed to execute kubegen");

    assert!(output.status.success());

    let mod_content = std::fs::read_to_string(temp.path().join("src/metrics/mod.rs"))
        .expect("Failed to read mod.rs");

    // Check for key metrics components
    assert!(
        mod_content.contains("prometheus"),
        "mod.rs should import prometheus"
    );
    assert!(
        mod_content.contains("reconciliations_total"),
        "mod.rs should define reconciliations_total metric"
    );
    assert!(
        mod_content.contains("reconciliation_errors_total"),
        "mod.rs should define reconciliation_errors_total metric"
    );
    assert!(
        mod_content.contains("reconciliation_duration_seconds"),
        "mod.rs should define reconciliation_duration_seconds metric"
    );
    assert!(
        mod_content.contains("9090"),
        "mod.rs should use custom port 9090"
    );
    assert!(
        mod_content.contains("/metrics"),
        "mod.rs should handle /metrics endpoint"
    );
    assert!(
        mod_content.contains("/health"),
        "mod.rs should handle health endpoint"
    );
}

/// Test that `kubegen add metrics` with --dry-run doesn't create files
#[test]
fn test_kubegen_add_metrics_dry_run() {
    let temp = TempDir::new().expect("Failed to create temp directory");
    setup_project(&temp);

    let output = Command::new(env!("CARGO_BIN_EXE_kubegen"))
        .args(["add", "metrics", "--dry-run"])
        .current_dir(temp.path())
        .output()
        .expect("Failed to execute kubegen");

    assert!(
        output.status.success(),
        "kubegen add metrics --dry-run failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Metrics directory should NOT exist
    assert!(
        !temp.path().join("src/metrics").exists(),
        "Metrics was created despite --dry-run"
    );
}

/// Test that `kubegen add metrics` fails when not in a project
#[test]
fn test_kubegen_add_metrics_not_in_project() {
    let temp = TempDir::new().expect("Failed to create temp directory");
    // Don't create project structure

    let output = Command::new(env!("CARGO_BIN_EXE_kubegen"))
        .args(["add", "metrics"])
        .current_dir(temp.path())
        .output()
        .expect("Failed to execute kubegen");

    assert!(
        !output.status.success(),
        "kubegen add metrics should fail when not in a project"
    );
}

/// Test that `kubegen add metrics` fails when metrics already exists
#[test]
fn test_kubegen_add_metrics_existing_fails() {
    let temp = TempDir::new().expect("Failed to create temp directory");
    setup_project(&temp);

    // Create existing metrics directory
    let metrics_dir = temp.path().join("src/metrics");
    std::fs::create_dir_all(&metrics_dir).expect("Failed to create metrics directory");
    std::fs::write(metrics_dir.join("mod.rs"), "existing content").expect("Failed to write file");

    let output = Command::new(env!("CARGO_BIN_EXE_kubegen"))
        .args(["add", "metrics"])
        .current_dir(temp.path())
        .output()
        .expect("Failed to execute kubegen");

    assert!(
        !output.status.success(),
        "kubegen add metrics should fail when metrics exists"
    );
}

/// Test that `kubegen add metrics --force` overwrites existing metrics
#[test]
fn test_kubegen_add_metrics_force_overwrites() {
    let temp = TempDir::new().expect("Failed to create temp directory");
    setup_project(&temp);

    // Create existing metrics directory
    let metrics_dir = temp.path().join("src/metrics");
    std::fs::create_dir_all(&metrics_dir).expect("Failed to create metrics directory");
    std::fs::write(metrics_dir.join("mod.rs"), "old content").expect("Failed to write file");

    let output = Command::new(env!("CARGO_BIN_EXE_kubegen"))
        .args(["add", "metrics", "--force"])
        .current_dir(temp.path())
        .output()
        .expect("Failed to execute kubegen");

    assert!(
        output.status.success(),
        "kubegen add metrics --force failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify mod.rs was overwritten
    let content = std::fs::read_to_string(metrics_dir.join("mod.rs")).expect("Failed to read file");
    assert!(content.contains("prometheus"), "mod.rs was not overwritten");
}

/// Test that `kubegen add metrics` uses default port when not specified
#[test]
fn test_kubegen_add_metrics_default_port() {
    let temp = TempDir::new().expect("Failed to create temp directory");
    setup_project(&temp);

    let output = Command::new(env!("CARGO_BIN_EXE_kubegen"))
        .args(["add", "metrics"])
        .current_dir(temp.path())
        .output()
        .expect("Failed to execute kubegen");

    assert!(
        output.status.success(),
        "kubegen add metrics failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Check that default port (8080) is used
    let mod_content = std::fs::read_to_string(temp.path().join("src/metrics/mod.rs"))
        .expect("Failed to read mod.rs");
    assert!(
        mod_content.contains("8080"),
        "mod.rs should use default port 8080"
    );
}

/// Test that `kubegen add metrics` with custom port works
#[test]
fn test_kubegen_add_metrics_custom_port() {
    let temp = TempDir::new().expect("Failed to create temp directory");
    setup_project(&temp);

    let output = Command::new(env!("CARGO_BIN_EXE_kubegen"))
        .args(["add", "metrics", "--port", "3000"])
        .current_dir(temp.path())
        .output()
        .expect("Failed to execute kubegen");

    assert!(
        output.status.success(),
        "kubegen add metrics failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let mod_content = std::fs::read_to_string(temp.path().join("src/metrics/mod.rs"))
        .expect("Failed to read mod.rs");
    assert!(
        mod_content.contains("3000"),
        "mod.rs should use custom port 3000"
    );
}
