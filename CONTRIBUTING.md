# Contributing to kubegen

Thank you for your interest in contributing to kubegen! This guide will help you get started.

## Table of Contents

- [Project Overview](#project-overview)
- [Getting Started](#getting-started)
- [Project Structure](#project-structure)
- [Development Workflow](#development-workflow)
- [Testing](#testing)
- [Pull Request Process](#pull-request-process)
- [Code Style](#code-style)
- [Issue Labels](#issue-labels)

## Project Overview

kubegen is a CLI tool for scaffolding Kubernetes operators in Rust using kube-rs. Our goal is to make operator development as approachable as Kubebuilder does for Go.

**Core Principles:**
- Small, focused, atomic PRs only
- Every feature includes implementation, tests, and documentation
- Generated code is explicit and owned by the user
- Follow kube-rs and Rust best practices

**For detailed architecture and design decisions, see [AGENT.md](AGENT.md).**

## Getting Started

### Prerequisites

**Required:**
- Rust 1.75+ (install via [rustup](https://rustup.rs/))
- Git

**For end-to-end testing:**
- Docker
- kind (Kubernetes in Docker)
- kubectl

### Setup

```bash
# Clone the repository
git clone https://github.com/your-org/kubegen-cli
cd kubegen-cli

# Build the project
cargo build

# Run tests
cargo test

# Run the CLI
cargo run -- --help
```

### First-Time Contributors

Look for issues labeled `good-first-issue`. These are small, well-defined tasks perfect for getting familiar with the codebase.

## Project Structure

```
kubegen-cli/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── lib.rs               # Public API exports
│   ├── cli/                 # Command implementations
│   │   ├── mod.rs
│   │   ├── new.rs           # kubegen new command
│   │   └── add/
│   │       ├── mod.rs
│   │       ├── crd.rs       # kubegen add crd
│   │       ├── metrics.rs   # kubegen add metrics
│   │       └── webhook.rs   # kubegen add webhook
│   ├── template/            # Template rendering
│   │   ├── mod.rs
│   │   ├── renderer.rs      # {{variable}} substitution
│   │   ├── context.rs       # Template context types
│   │   └── embed.rs         # rust-embed integration
│   ├── validation/          # Input validation
│   │   ├── mod.rs
│   │   ├── names.rs         # Name validation (DNS-1123, etc.)
│   │   └── paths.rs         # Path validation
│   ├── fs/                  # Filesystem operations
│   │   ├── mod.rs
│   │   ├── writer.rs        # File writing with dry-run
│   │   └── checker.rs       # Overwrite protection
│   ├── error.rs             # Error types
│   └── types.rs             # Common types
├── templates/               # Embedded templates
│   ├── project/             # kubegen new templates
│   ├── crd/                 # kubegen add crd templates
│   ├── metrics/             # kubegen add metrics templates
│   └── webhook/             # kubegen add webhook templates
├── tests/
│   ├── integration/         # Integration tests
│   │   ├── test_new_command.rs
│   │   ├── test_add_crd.rs
│   │   ├── test_add_metrics.rs
│   │   └── test_add_webhook.rs
│   └── e2e/                 # End-to-end tests with kind
│       ├── helpers/
│       │   ├── kind.rs
│       │   ├── kubectl.rs
│       │   └── docker.rs
│       └── test_*.rs
├── examples/                # Example generated operators
├── docs/                    # Documentation
├── scripts/                 # Helper scripts
│   └── e2e-test-local.sh
├── Makefile                 # Development tasks
└── AGENT.md                 # Architecture & design docs
```

### Module Responsibilities

| Module | Responsibility |
|--------|----------------|
| `cli/` | Parse commands, validate inputs, orchestrate operations |
| `template/` | Render templates with variable substitution |
| `validation/` | Validate names, paths, and formats |
| `fs/` | Safe filesystem operations with dry-run support |
| `error.rs` | Error types with context and user-friendly messages |

## Development Workflow

### Making Changes

1. **Find or create an issue**
   - Check existing issues first
   - For new features, discuss in an issue before starting work
   - Reference the issue number in your PR

2. **Create a branch**
   ```bash
   git checkout -b feature/add-metrics-command
   ```

3. **Make your changes**
   - Keep changes focused on one issue
   - Follow the code style guide
   - Add tests for new functionality
   - Update documentation if needed

4. **Test your changes**
   ```bash
   # Run unit tests
   cargo test

   # Run integration tests
   cargo test --test '*'

   # Test locally by building and running
   cargo build
   ./target/debug/kubegen new test-operator
   cd test-operator && cargo build
   ```

5. **Format and lint**
   ```bash
   cargo fmt
   cargo clippy --all-targets -- -D warnings
   ```

6. **Commit your changes**
   ```bash
   git add .
   git commit -m "[cli] Add metrics command scaffolding"
   ```

7. **Push and create a PR**
   ```bash
   git push origin feature/add-metrics-command
   ```

### Testing Changes Locally

**Quick test (generate and compile):**
```bash
# Build kubegen
cargo build

# Generate a test project
./target/debug/kubegen new test-operator --domain example.com

# Verify it compiles
cd test-operator
cargo build
```

**Full e2e test (deploy to kind):**
```bash
# Run automated e2e tests
make test-e2e-local

# Or manually:
kind create cluster --name kubegen-test
./target/release/kubegen new my-op
cd my-op
docker build -t my-op:test .
kind load docker-image my-op:test --name kubegen-test
kubectl apply -f manifests/
kubectl logs -l app=my-op
```

## Testing

We maintain high test coverage across three levels:

### Unit Tests

**Location:** `src/*/tests.rs` or `#[cfg(test)]` modules

**What to test:**
- CLI argument parsing
- Validation logic (names, paths, formats)
- Template rendering
- Filesystem operations (use `tempfile::TempDir`)
- Error handling

**Example:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_project_name() {
        assert!(validate_project_name("my-operator").is_ok());
        assert!(validate_project_name("My-Operator").is_err());
        assert!(validate_project_name("my_operator").is_err());
    }
}
```

**Run:** `cargo test`

### Integration Tests

**Location:** `tests/integration/`

**What to test:**
- Full command execution
- Generated files exist and contain expected content
- Generated projects compile
- Error cases (invalid inputs, missing directories)

**Example:**
```rust
#[test]
fn test_kubegen_new_generates_project() {
    let temp = TempDir::new().unwrap();
    let output = Command::new("cargo")
        .args(&["run", "--", "new", "test-op"])
        .current_dir(&temp)
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(temp.path().join("test-op/Cargo.toml").exists());
    assert!(temp.path().join("test-op/src/main.rs").exists());
}
```

**Run:** `cargo test --test '*'`

### End-to-End Tests

**Location:** `tests/e2e/`

**What to test:**
- Generate operator with `kubegen new`
- Add features with `kubegen add crd`, etc.
- Build Docker image
- Deploy to kind cluster
- Verify operator runs
- Test functionality (reconciliation, metrics, webhooks)

**Example:**
```rust
#[test]
#[ignore] // Only run with --ignored flag
fn test_operator_deploys_to_kind() {
    let cluster = KindCluster::create("kubegen-test").unwrap();

    // Generate operator
    Command::new("./target/release/kubegen")
        .args(&["new", "test-op"])
        .status()
        .unwrap();

    // Build and deploy
    docker_build("test-op", "test-op:e2e");
    cluster.load_image("test-op:e2e");
    cluster.kubectl(&["apply", "-f", "test-op/manifests/"]);

    // Verify
    let pods = cluster.kubectl(&["get", "pods", "-l", "app=test-op"]);
    assert!(pods.contains("Running"));

    cluster.delete();
}
```

**Run:** `make test-e2e-local` or `cargo test --test e2e -- --ignored`

### Test Requirements by Change Type

| Change Type | Unit Tests | Integration Tests | E2E Tests |
|-------------|-----------|-------------------|-----------|
| New validation logic | ✅ Required | - | - |
| New CLI command | ✅ Required | ✅ Required | - |
| New template | ✅ Rendering | ✅ Compilation | Recommended |
| Template changes | ✅ Rendering | ✅ Compilation | Recommended |
| New feature (metrics, webhook) | ✅ Required | ✅ Required | ✅ Required |

## Pull Request Process

### Before Submitting

- [ ] Code compiles: `cargo build`
- [ ] Tests pass: `cargo test`
- [ ] Linting passes: `cargo clippy --all-targets -- -D warnings`
- [ ] Formatting passes: `cargo fmt -- --check`
- [ ] Integration tests pass: `cargo test --test '*'`
- [ ] Documentation updated (if user-facing change)
- [ ] CHANGELOG.md updated (under Unreleased section)

### PR Guidelines

**Size:**
- Keep PRs small and focused (one issue per PR)
- Large features should be split into phases
- Each PR should be independently reviewable

**Title format:** `[area] Brief description`
- Examples: `[cli] Add --dry-run flag`, `[template] Add CRD YAML generation`

**Description:**
- Reference the issue: "Fixes #123"
- Explain what changed and why
- Note any breaking changes
- Include testing notes

**Example PR template:**
```markdown
## Summary
Implements the `kubegen add metrics` command to scaffold Prometheus metrics support.

Fixes #45

## Changes
- Added metrics command in `src/cli/add/metrics.rs`
- Created metrics templates in `templates/metrics/`
- Added integration tests
- Updated documentation

## Testing
- Unit tests for metrics command parsing
- Integration test verifies file generation
- Manually tested: generated project compiles and exposes /metrics endpoint

## Breaking Changes
None
```

### Review Process

1. **CI checks must pass** (build, test, lint, format)
2. **At least one maintainer approval required**
3. **Address review feedback** with new commits (don't force-push)
4. **Squashing:** Maintainers will squash-merge when merging

### After Merge

- Delete your branch
- Close the associated issue (if not auto-closed)
- Update your local main: `git pull origin main`

## Code Style

### Rust Style

Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/):

- Use `rustfmt` for formatting (enforced by CI)
- Use `clippy` for linting (enforced by CI)
- Write idiomatic Rust
- Add doc comments (`///`) for public APIs
- Prefer explicit error handling over `unwrap()`

### Error Handling

Use `thiserror` for structured errors with context:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KubersError {
    #[error("Invalid project name '{name}': {reason}")]
    InvalidProjectName { name: String, reason: String },

    #[error("Failed to write file {path}: {source}")]
    FileWrite {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}
```

### Naming Conventions

- **Functions:** `snake_case`
- **Types:** `PascalCase`
- **Constants:** `SCREAMING_SNAKE_CASE`
- **Modules:** `snake_case`

### Documentation

- Add doc comments for public items
- Include examples in doc comments when helpful
- Explain *why*, not just *what*

```rust
/// Validates a project name according to DNS-1123 subdomain rules.
///
/// Project names must:
/// - Be lowercase
/// - Contain only alphanumeric characters and hyphens
/// - Start and end with an alphanumeric character
/// - Be between 1 and 63 characters
///
/// # Examples
///
/// ```
/// assert!(validate_project_name("my-operator").is_ok());
/// assert!(validate_project_name("My-Operator").is_err());
/// ```
pub fn validate_project_name(name: &str) -> Result<(), ValidationError> {
    // ...
}
```

### Commit Messages

**Format:** `[area] Brief description`

**Areas:**
- `cli` - CLI commands and parsing
- `template` - Template rendering and templates
- `validation` - Validation logic
- `test` - Tests
- `docs` - Documentation
- `ci` - CI/CD workflows
- `deps` - Dependency updates

**Examples:**
- `[cli] Add --dry-run flag to new command`
- `[template] Add CRD YAML generation template`
- `[test] Add integration tests for add crd command`
- `[docs] Update architecture documentation`
- `[ci] Add e2e test workflow`

### Template Style

Templates use `{{variable}}` syntax:

```rust
// In template file (templates/project/main.rs.tmpl)
fn main() {
    let project_name = "{{project_name}}";
    println!("Starting {}", project_name);
}
```

**Template guidelines:**
- Use descriptive variable names
- Document required variables in template comments
- Handle edge cases (empty strings, special characters)
- Validate templates compile in integration tests

## Issue Labels

We use labels to categorize issues:

### Type
- `type/feature` - New feature
- `type/bug` - Bug fix
- `type/docs` - Documentation
- `type/test` - Testing improvements

### Area
- `area/cli` - CLI commands and parsing
- `area/templates` - Template system
- `area/crd` - CRD generation
- `area/metrics` - Metrics scaffolding
- `area/webhook` - Webhook scaffolding
- `area/ci` - CI/CD
- `area/e2e` - End-to-end testing

### Phase
- `phase/1` - Foundation (project setup, basic structure)
- `phase/2` - Core CLI (filesystem, validation)
- `phase/3` - Templates (rendering engine)
- `phase/4` - Project scaffolding (kubegen new)
- `phase/5` - CRD scaffolding (kubegen add crd)
- `phase/6` - Metrics (kubegen add metrics)
- `phase/7` - Webhooks (kubegen add webhook)
- `phase/8` - E2E testing
- `phase/9` - Documentation and polish
- `phase/future` - Future enhancements

### Status
- `good-first-issue` - Good for newcomers
- `help-wanted` - Community contributions welcome
- `blocked` - Blocked by another issue or decision

## Development Environment

### Recommended Tools

- **Editor:** VSCode with rust-analyzer extension
- **CLI testing:** Use `cargo run -- <args>` for quick iteration
- **Debugging:** Use `RUST_LOG=debug` for verbose output

### Environment Variables

```bash
# Enable debug logging
export RUST_LOG=debug

# Run kubegen with debug output
cargo run -- new my-op

# Run specific test with output
cargo test test_name -- --nocapture
```

### Makefile Targets

```bash
make build              # Build release binary
make test               # Run all tests
make test-integration   # Run integration tests only
make test-e2e-local     # Run e2e tests with kind
make lint               # Run clippy
make format             # Run rustfmt
make check-format       # Check formatting without modifying
make clean              # Clean build artifacts and kind clusters
```

## Getting Help

- **Questions about contributing?** Open a discussion or ask in an issue
- **Found a bug?** Open an issue with reproduction steps
- **Want to add a feature?** Open an issue to discuss first
- **Need architecture clarification?** Check [AGENT.md](AGENT.md)

## Code of Conduct

This project follows the [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code.

## License

By contributing, you agree that your contributions will be licensed under the same license as the project (see LICENSE file).

---

Thank you for contributing to kubegen! Your efforts help make Kubernetes operator development in Rust more accessible to everyone.
