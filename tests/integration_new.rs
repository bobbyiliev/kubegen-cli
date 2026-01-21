//! Integration tests for `kubegen new` command

use std::process::Command;

use tempfile::TempDir;

/// Test that `kubegen new` generates a valid project structure
#[test]
fn test_kubegen_new_generates_valid_project() {
    let temp = TempDir::new().expect("Failed to create temp directory");

    // Run kubegen new
    let output = Command::new(env!("CARGO_BIN_EXE_kubegen"))
        .args(["new", "test-operator", "--domain", "example.com"])
        .current_dir(temp.path())
        .output()
        .expect("Failed to execute kubegen");

    assert!(
        output.status.success(),
        "kubegen new failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let project_dir = temp.path().join("test-operator");

    // Verify project directory exists
    assert!(project_dir.exists(), "Project directory not created");

    // Verify all expected files exist
    assert!(
        project_dir.join("Cargo.toml").exists(),
        "Cargo.toml not created"
    );
    assert!(
        project_dir.join("README.md").exists(),
        "README.md not created"
    );
    assert!(
        project_dir.join("Makefile").exists(),
        "Makefile not created"
    );
    assert!(
        project_dir.join(".gitignore").exists(),
        ".gitignore not created"
    );
    assert!(
        project_dir.join("src/main.rs").exists(),
        "src/main.rs not created"
    );
    assert!(
        project_dir.join("src/lib.rs").exists(),
        "src/lib.rs not created"
    );
    assert!(
        project_dir.join("src/error.rs").exists(),
        "src/error.rs not created"
    );
}

/// Test that `kubegen new` with --dry-run doesn't create files
#[test]
fn test_kubegen_new_dry_run() {
    let temp = TempDir::new().expect("Failed to create temp directory");

    let output = Command::new(env!("CARGO_BIN_EXE_kubegen"))
        .args(["new", "dry-run-test", "--dry-run"])
        .current_dir(temp.path())
        .output()
        .expect("Failed to execute kubegen");

    assert!(
        output.status.success(),
        "kubegen new --dry-run failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Project directory should NOT exist
    assert!(
        !temp.path().join("dry-run-test").exists(),
        "Project was created despite --dry-run"
    );
}

/// Test that `kubegen new` fails with invalid project name
#[test]
fn test_kubegen_new_invalid_name() {
    let temp = TempDir::new().expect("Failed to create temp directory");

    let output = Command::new(env!("CARGO_BIN_EXE_kubegen"))
        .args(["new", "Invalid Name"])
        .current_dir(temp.path())
        .output()
        .expect("Failed to execute kubegen");

    assert!(
        !output.status.success(),
        "kubegen new should fail with invalid name"
    );
}

/// Test that `kubegen new` fails when project already exists
#[test]
fn test_kubegen_new_existing_project_fails() {
    let temp = TempDir::new().expect("Failed to create temp directory");
    let project_dir = temp.path().join("existing-project");

    // Create existing project
    std::fs::create_dir_all(&project_dir).expect("Failed to create directory");
    std::fs::write(project_dir.join("Cargo.toml"), "existing").expect("Failed to write file");

    let output = Command::new(env!("CARGO_BIN_EXE_kubegen"))
        .args(["new", "existing-project"])
        .current_dir(temp.path())
        .output()
        .expect("Failed to execute kubegen");

    assert!(
        !output.status.success(),
        "kubegen new should fail when project exists"
    );
}

/// Test that `kubegen new --force` overwrites existing project
#[test]
fn test_kubegen_new_force_overwrites() {
    let temp = TempDir::new().expect("Failed to create temp directory");
    let project_dir = temp.path().join("force-test");

    // Create existing project
    std::fs::create_dir_all(project_dir.join("src")).expect("Failed to create directory");
    std::fs::write(project_dir.join("Cargo.toml"), "old content").expect("Failed to write file");

    let output = Command::new(env!("CARGO_BIN_EXE_kubegen"))
        .args(["new", "force-test", "--force"])
        .current_dir(temp.path())
        .output()
        .expect("Failed to execute kubegen");

    assert!(
        output.status.success(),
        "kubegen new --force failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify Cargo.toml was overwritten
    let content =
        std::fs::read_to_string(project_dir.join("Cargo.toml")).expect("Failed to read file");
    assert!(
        content.contains("[package]"),
        "Cargo.toml was not overwritten"
    );
}

/// Test that generated Cargo.toml has correct content
#[test]
fn test_kubegen_new_cargo_toml_content() {
    let temp = TempDir::new().expect("Failed to create temp directory");

    let output = Command::new(env!("CARGO_BIN_EXE_kubegen"))
        .args(["new", "content-test", "--domain", "mycompany.io"])
        .current_dir(temp.path())
        .output()
        .expect("Failed to execute kubegen");

    assert!(output.status.success());

    let cargo_content = std::fs::read_to_string(temp.path().join("content-test/Cargo.toml"))
        .expect("Failed to read Cargo.toml");

    assert!(cargo_content.contains("name = \"content-test\""));
    assert!(cargo_content.contains("kube"));
    assert!(cargo_content.contains("tokio"));
}

/// Test that generated project compiles successfully with cargo build
///
/// This test is marked as ignored by default because it requires network access
/// to download dependencies and takes longer to run. Run with:
/// `cargo test --test integration_new -- --ignored`
#[test]
#[ignore]
fn test_generated_project_compiles() {
    let temp = TempDir::new().expect("Failed to create temp directory");

    // Generate project
    let output = Command::new(env!("CARGO_BIN_EXE_kubegen"))
        .args(["new", "compile-test", "--domain", "example.com"])
        .current_dir(temp.path())
        .output()
        .expect("Failed to execute kubegen");

    assert!(
        output.status.success(),
        "kubegen new failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let project_dir = temp.path().join("compile-test");

    // Run cargo check (faster than cargo build, still validates compilation)
    let check_output = Command::new("cargo")
        .args(["check"])
        .current_dir(&project_dir)
        .output()
        .expect("Failed to run cargo check");

    assert!(
        check_output.status.success(),
        "Generated project failed to compile:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check_output.stdout),
        String::from_utf8_lossy(&check_output.stderr)
    );
}
