//! Integration tests for `kubegen add webhook` command

use std::process::Command;

use tempfile::TempDir;

/// Helper to create a minimal project structure for webhook tests
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

/// Test that `kubegen add webhook --validating` generates valid webhook files
#[test]
fn test_kubegen_add_webhook_validating_generates_files() {
    let temp = TempDir::new().expect("Failed to create temp directory");
    setup_project(&temp);

    let output = Command::new(env!("CARGO_BIN_EXE_kubegen"))
        .args(["add", "webhook", "MyResource", "--validating"])
        .current_dir(temp.path())
        .output()
        .expect("Failed to execute kubegen");

    assert!(
        output.status.success(),
        "kubegen add webhook failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let webhook_dir = temp.path().join("src/webhook");

    // Verify webhook directory exists
    assert!(webhook_dir.exists(), "Webhook directory not created");

    // Verify mod.rs exists
    assert!(webhook_dir.join("mod.rs").exists(), "mod.rs not created");

    // Verify validating.rs exists
    assert!(
        webhook_dir.join("validating.rs").exists(),
        "validating.rs not created"
    );

    // Verify mutating.rs does NOT exist
    assert!(
        !webhook_dir.join("mutating.rs").exists(),
        "mutating.rs should not be created for --validating only"
    );
}

/// Test that `kubegen add webhook --mutating` generates valid webhook files
#[test]
fn test_kubegen_add_webhook_mutating_generates_files() {
    let temp = TempDir::new().expect("Failed to create temp directory");
    setup_project(&temp);

    let output = Command::new(env!("CARGO_BIN_EXE_kubegen"))
        .args(["add", "webhook", "MyResource", "--mutating"])
        .current_dir(temp.path())
        .output()
        .expect("Failed to execute kubegen");

    assert!(
        output.status.success(),
        "kubegen add webhook failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let webhook_dir = temp.path().join("src/webhook");

    // Verify webhook directory exists
    assert!(webhook_dir.exists(), "Webhook directory not created");

    // Verify mod.rs exists
    assert!(webhook_dir.join("mod.rs").exists(), "mod.rs not created");

    // Verify mutating.rs exists
    assert!(
        webhook_dir.join("mutating.rs").exists(),
        "mutating.rs not created"
    );

    // Verify validating.rs does NOT exist
    assert!(
        !webhook_dir.join("validating.rs").exists(),
        "validating.rs should not be created for --mutating only"
    );
}

/// Test that `kubegen add webhook --validating --mutating` generates both files
#[test]
fn test_kubegen_add_webhook_both_generates_files() {
    let temp = TempDir::new().expect("Failed to create temp directory");
    setup_project(&temp);

    let output = Command::new(env!("CARGO_BIN_EXE_kubegen"))
        .args(["add", "webhook", "MyResource", "--validating", "--mutating"])
        .current_dir(temp.path())
        .output()
        .expect("Failed to execute kubegen");

    assert!(
        output.status.success(),
        "kubegen add webhook failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let webhook_dir = temp.path().join("src/webhook");

    // Verify all files exist
    assert!(webhook_dir.join("mod.rs").exists(), "mod.rs not created");
    assert!(
        webhook_dir.join("validating.rs").exists(),
        "validating.rs not created"
    );
    assert!(
        webhook_dir.join("mutating.rs").exists(),
        "mutating.rs not created"
    );
}

/// Test that generated webhook files have correct content
#[test]
fn test_kubegen_add_webhook_file_content() {
    let temp = TempDir::new().expect("Failed to create temp directory");
    setup_project(&temp);

    let output = Command::new(env!("CARGO_BIN_EXE_kubegen"))
        .args(["add", "webhook", "MyResource", "--validating", "--mutating"])
        .current_dir(temp.path())
        .output()
        .expect("Failed to execute kubegen");

    assert!(output.status.success());

    let webhook_dir = temp.path().join("src/webhook");

    // Check mod.rs content
    let mod_content =
        std::fs::read_to_string(webhook_dir.join("mod.rs")).expect("Failed to read mod.rs");
    assert!(
        mod_content.contains("MyResource"),
        "mod.rs should reference MyResource"
    );
    assert!(
        mod_content.contains("my_resource"),
        "mod.rs should use snake_case kind"
    );
    assert!(mod_content.contains("warp"), "mod.rs should import warp");
    assert!(
        mod_content.contains("AdmissionRequest"),
        "mod.rs should use AdmissionRequest"
    );
    assert!(
        mod_content.contains("healthz"),
        "mod.rs should have health endpoint"
    );

    // Check validating.rs content
    let validating_content = std::fs::read_to_string(webhook_dir.join("validating.rs"))
        .expect("Failed to read validating.rs");
    assert!(
        validating_content.contains("MyResource"),
        "validating.rs should reference MyResource"
    );
    assert!(
        validating_content.contains("validate"),
        "validating.rs should have validate function"
    );
    assert!(
        validating_content.contains("AdmissionResponse"),
        "validating.rs should use AdmissionResponse"
    );

    // Check mutating.rs content
    let mutating_content = std::fs::read_to_string(webhook_dir.join("mutating.rs"))
        .expect("Failed to read mutating.rs");
    assert!(
        mutating_content.contains("MyResource"),
        "mutating.rs should reference MyResource"
    );
    assert!(
        mutating_content.contains("mutate"),
        "mutating.rs should have mutate function"
    );
    assert!(
        mutating_content.contains("with_patch"),
        "mutating.rs should use with_patch for mutations"
    );
}

/// Test that `kubegen add webhook` with --dry-run doesn't create files
#[test]
fn test_kubegen_add_webhook_dry_run() {
    let temp = TempDir::new().expect("Failed to create temp directory");
    setup_project(&temp);

    let output = Command::new(env!("CARGO_BIN_EXE_kubegen"))
        .args(["add", "webhook", "MyResource", "--validating", "--dry-run"])
        .current_dir(temp.path())
        .output()
        .expect("Failed to execute kubegen");

    assert!(
        output.status.success(),
        "kubegen add webhook --dry-run failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Webhook directory should NOT exist
    assert!(
        !temp.path().join("src/webhook").exists(),
        "Webhook was created despite --dry-run"
    );
}

/// Test that `kubegen add webhook` fails when not in a project
#[test]
fn test_kubegen_add_webhook_not_in_project() {
    let temp = TempDir::new().expect("Failed to create temp directory");
    // Don't create project structure

    let output = Command::new(env!("CARGO_BIN_EXE_kubegen"))
        .args(["add", "webhook", "MyResource", "--validating"])
        .current_dir(temp.path())
        .output()
        .expect("Failed to execute kubegen");

    assert!(
        !output.status.success(),
        "kubegen add webhook should fail when not in a project"
    );
}

/// Test that `kubegen add webhook` fails with invalid kind
#[test]
fn test_kubegen_add_webhook_invalid_kind() {
    let temp = TempDir::new().expect("Failed to create temp directory");
    setup_project(&temp);

    let output = Command::new(env!("CARGO_BIN_EXE_kubegen"))
        .args(["add", "webhook", "invalid-kind", "--validating"])
        .current_dir(temp.path())
        .output()
        .expect("Failed to execute kubegen");

    assert!(
        !output.status.success(),
        "kubegen add webhook should fail with invalid kind"
    );
}

/// Test that `kubegen add webhook` fails without --validating or --mutating
#[test]
fn test_kubegen_add_webhook_no_type_fails() {
    let temp = TempDir::new().expect("Failed to create temp directory");
    setup_project(&temp);

    let output = Command::new(env!("CARGO_BIN_EXE_kubegen"))
        .args(["add", "webhook", "MyResource"])
        .current_dir(temp.path())
        .output()
        .expect("Failed to execute kubegen");

    assert!(
        !output.status.success(),
        "kubegen add webhook should fail without --validating or --mutating"
    );
}

/// Test that `kubegen add webhook` fails when webhook already exists
#[test]
fn test_kubegen_add_webhook_existing_fails() {
    let temp = TempDir::new().expect("Failed to create temp directory");
    setup_project(&temp);

    // Create existing webhook directory
    let webhook_dir = temp.path().join("src/webhook");
    std::fs::create_dir_all(&webhook_dir).expect("Failed to create webhook directory");
    std::fs::write(webhook_dir.join("mod.rs"), "existing content").expect("Failed to write file");

    let output = Command::new(env!("CARGO_BIN_EXE_kubegen"))
        .args(["add", "webhook", "MyResource", "--validating"])
        .current_dir(temp.path())
        .output()
        .expect("Failed to execute kubegen");

    assert!(
        !output.status.success(),
        "kubegen add webhook should fail when webhook exists"
    );
}

/// Test that `kubegen add webhook --force` overwrites existing webhook
#[test]
fn test_kubegen_add_webhook_force_overwrites() {
    let temp = TempDir::new().expect("Failed to create temp directory");
    setup_project(&temp);

    // Create existing webhook directory
    let webhook_dir = temp.path().join("src/webhook");
    std::fs::create_dir_all(&webhook_dir).expect("Failed to create webhook directory");
    std::fs::write(webhook_dir.join("mod.rs"), "old content").expect("Failed to write file");

    let output = Command::new(env!("CARGO_BIN_EXE_kubegen"))
        .args(["add", "webhook", "MyResource", "--validating", "--force"])
        .current_dir(temp.path())
        .output()
        .expect("Failed to execute kubegen");

    assert!(
        output.status.success(),
        "kubegen add webhook --force failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify mod.rs was overwritten
    let content = std::fs::read_to_string(webhook_dir.join("mod.rs")).expect("Failed to read file");
    assert!(content.contains("MyResource"), "mod.rs was not overwritten");
}
