//! Integration tests for `kubegen add crd` command

use std::process::Command;

use tempfile::TempDir;

/// Helper to create a minimal project structure for CRD tests
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
kube = { version = "0.87", features = ["runtime", "derive"] }
"#,
    )
    .expect("Failed to write Cargo.toml");
}

/// Test that `kubegen add crd` generates valid CRD files
#[test]
fn test_kubegen_add_crd_generates_files() {
    let temp = TempDir::new().expect("Failed to create temp directory");
    setup_project(&temp);

    let output = Command::new(env!("CARGO_BIN_EXE_kubegen"))
        .args([
            "add",
            "crd",
            "MyResource",
            "--group",
            "example.com",
            "--api-version",
            "v1alpha1",
        ])
        .current_dir(temp.path())
        .output()
        .expect("Failed to execute kubegen");

    assert!(
        output.status.success(),
        "kubegen add crd failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let crd_dir = temp.path().join("src/my_resource");

    // Verify CRD directory exists
    assert!(crd_dir.exists(), "CRD directory not created");

    // Verify all expected files exist
    assert!(crd_dir.join("mod.rs").exists(), "mod.rs not created");
    assert!(crd_dir.join("types.rs").exists(), "types.rs not created");
    assert!(
        crd_dir.join("controller.rs").exists(),
        "controller.rs not created"
    );
}

/// Test that generated CRD files have correct content
#[test]
fn test_kubegen_add_crd_file_content() {
    let temp = TempDir::new().expect("Failed to create temp directory");
    setup_project(&temp);

    let output = Command::new(env!("CARGO_BIN_EXE_kubegen"))
        .args([
            "add",
            "crd",
            "DatabaseCluster",
            "--group",
            "db.example.com",
            "--api-version",
            "v1beta1",
        ])
        .current_dir(temp.path())
        .output()
        .expect("Failed to execute kubegen");

    assert!(output.status.success());

    let crd_dir = temp.path().join("src/database_cluster");

    // Check types.rs content
    let types_content =
        std::fs::read_to_string(crd_dir.join("types.rs")).expect("Failed to read types.rs");
    assert!(
        types_content.contains("DatabaseCluster"),
        "types.rs should contain kind name"
    );
    assert!(
        types_content.contains("db.example.com"),
        "types.rs should contain group"
    );
    assert!(
        types_content.contains("v1beta1"),
        "types.rs should contain version"
    );

    // Check controller.rs content
    let controller_content = std::fs::read_to_string(crd_dir.join("controller.rs"))
        .expect("Failed to read controller.rs");
    assert!(
        controller_content.contains("DatabaseCluster"),
        "controller.rs should contain kind name"
    );
    assert!(
        controller_content.contains("reconcile"),
        "controller.rs should contain reconcile function"
    );

    // Check mod.rs content
    let mod_content =
        std::fs::read_to_string(crd_dir.join("mod.rs")).expect("Failed to read mod.rs");
    assert!(
        mod_content.contains("mod types"),
        "mod.rs should export types module"
    );
    assert!(
        mod_content.contains("mod controller"),
        "mod.rs should export controller module"
    );
}

/// Test that `kubegen add crd` with --dry-run doesn't create files
#[test]
fn test_kubegen_add_crd_dry_run() {
    let temp = TempDir::new().expect("Failed to create temp directory");
    setup_project(&temp);

    let output = Command::new(env!("CARGO_BIN_EXE_kubegen"))
        .args(["add", "crd", "DryRunResource", "--dry-run"])
        .current_dir(temp.path())
        .output()
        .expect("Failed to execute kubegen");

    assert!(
        output.status.success(),
        "kubegen add crd --dry-run failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // CRD directory should NOT exist
    assert!(
        !temp.path().join("src/dry_run_resource").exists(),
        "CRD was created despite --dry-run"
    );
}

/// Test that `kubegen add crd` fails with invalid kind name
#[test]
fn test_kubegen_add_crd_invalid_kind() {
    let temp = TempDir::new().expect("Failed to create temp directory");
    setup_project(&temp);

    let output = Command::new(env!("CARGO_BIN_EXE_kubegen"))
        .args(["add", "crd", "invalid-kind"])
        .current_dir(temp.path())
        .output()
        .expect("Failed to execute kubegen");

    assert!(
        !output.status.success(),
        "kubegen add crd should fail with invalid kind"
    );
}

/// Test that `kubegen add crd` fails when not in a project
#[test]
fn test_kubegen_add_crd_not_in_project() {
    let temp = TempDir::new().expect("Failed to create temp directory");
    // Don't create project structure

    let output = Command::new(env!("CARGO_BIN_EXE_kubegen"))
        .args(["add", "crd", "MyResource"])
        .current_dir(temp.path())
        .output()
        .expect("Failed to execute kubegen");

    assert!(
        !output.status.success(),
        "kubegen add crd should fail when not in a project"
    );
}

/// Test that `kubegen add crd` fails when CRD already exists
#[test]
fn test_kubegen_add_crd_existing_fails() {
    let temp = TempDir::new().expect("Failed to create temp directory");
    setup_project(&temp);

    // Create existing CRD directory
    let crd_dir = temp.path().join("src/my_resource");
    std::fs::create_dir_all(&crd_dir).expect("Failed to create CRD directory");
    std::fs::write(crd_dir.join("mod.rs"), "existing content").expect("Failed to write file");

    let output = Command::new(env!("CARGO_BIN_EXE_kubegen"))
        .args(["add", "crd", "MyResource"])
        .current_dir(temp.path())
        .output()
        .expect("Failed to execute kubegen");

    assert!(
        !output.status.success(),
        "kubegen add crd should fail when CRD exists"
    );
}

/// Test that `kubegen add crd --force` overwrites existing CRD
#[test]
fn test_kubegen_add_crd_force_overwrites() {
    let temp = TempDir::new().expect("Failed to create temp directory");
    setup_project(&temp);

    // Create existing CRD directory
    let crd_dir = temp.path().join("src/my_resource");
    std::fs::create_dir_all(&crd_dir).expect("Failed to create CRD directory");
    std::fs::write(crd_dir.join("mod.rs"), "old content").expect("Failed to write file");

    let output = Command::new(env!("CARGO_BIN_EXE_kubegen"))
        .args(["add", "crd", "MyResource", "--force"])
        .current_dir(temp.path())
        .output()
        .expect("Failed to execute kubegen");

    assert!(
        output.status.success(),
        "kubegen add crd --force failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify mod.rs was overwritten
    let content = std::fs::read_to_string(crd_dir.join("mod.rs")).expect("Failed to read file");
    assert!(content.contains("MyResource"), "mod.rs was not overwritten");
}

/// Test that `kubegen add crd` uses default group when not specified
#[test]
fn test_kubegen_add_crd_default_group() {
    let temp = TempDir::new().expect("Failed to create temp directory");
    setup_project(&temp);

    let output = Command::new(env!("CARGO_BIN_EXE_kubegen"))
        .args(["add", "crd", "DefaultGroup", "--api-version", "v1"])
        .current_dir(temp.path())
        .output()
        .expect("Failed to execute kubegen");

    assert!(
        output.status.success(),
        "kubegen add crd failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Check that default group (example.com) is used
    let types_content = std::fs::read_to_string(temp.path().join("src/default_group/types.rs"))
        .expect("Failed to read types.rs");
    assert!(
        types_content.contains("example.com"),
        "types.rs should use default group example.com"
    );
}

/// Test that a generated project with CRD compiles successfully
///
/// This test is marked as ignored by default because it requires network access
/// to download dependencies and takes longer to run. Run with:
/// `cargo test --test integration_crd -- --ignored`
#[test]
#[ignore]
fn test_project_with_crd_compiles() {
    let temp = TempDir::new().expect("Failed to create temp directory");

    // Step 1: Generate project with kubegen new
    let new_output = Command::new(env!("CARGO_BIN_EXE_kubegen"))
        .args(["new", "crd-compile-test", "--domain", "example.com"])
        .current_dir(temp.path())
        .output()
        .expect("Failed to execute kubegen new");

    assert!(
        new_output.status.success(),
        "kubegen new failed: {}",
        String::from_utf8_lossy(&new_output.stderr)
    );

    let project_dir = temp.path().join("crd-compile-test");

    // Step 2: Add CRD to the project
    let crd_output = Command::new(env!("CARGO_BIN_EXE_kubegen"))
        .args([
            "add",
            "crd",
            "MyResource",
            "--group",
            "example.com",
            "--api-version",
            "v1alpha1",
        ])
        .current_dir(&project_dir)
        .output()
        .expect("Failed to execute kubegen add crd");

    assert!(
        crd_output.status.success(),
        "kubegen add crd failed: {}",
        String::from_utf8_lossy(&crd_output.stderr)
    );

    // Step 3: Update lib.rs to include the CRD module
    // Insert the module declaration after the doc comment to avoid rustc error E0753
    let lib_path = project_dir.join("src/lib.rs");
    let lib_content = std::fs::read_to_string(&lib_path).expect("Failed to read lib.rs");
    let updated_lib =
        lib_content.replacen("pub mod error;", "pub mod error;\npub mod my_resource;", 1);
    std::fs::write(&lib_path, updated_lib).expect("Failed to write lib.rs");

    // Step 4: Run cargo check to verify compilation
    let check_output = Command::new("cargo")
        .args(["check"])
        .current_dir(&project_dir)
        .output()
        .expect("Failed to run cargo check");

    assert!(
        check_output.status.success(),
        "Project with CRD failed to compile:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check_output.stdout),
        String::from_utf8_lossy(&check_output.stderr)
    );
}
