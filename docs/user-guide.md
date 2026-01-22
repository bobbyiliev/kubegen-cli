# kubegen User Guide

This guide covers everything you need to know to use kubegen effectively.

## Quick Start

```bash
# Create a new operator project
kubegen new my-operator

# Navigate to the project
cd my-operator

# Add a Custom Resource Definition
kubegen add crd MyResource --group apps.example.com

# Add Prometheus metrics support
kubegen add metrics

# Add admission webhooks
kubegen add webhook MyResource --validating --mutating
```

## Commands

### `kubegen new`

Creates a new Kubernetes operator project with the standard directory structure.

```bash
kubegen new <NAME> [OPTIONS]
```

**Arguments:**
- `<NAME>` - Name of the operator project (required)

**Options:**
- `-d, --domain <DOMAIN>` - Domain for the operator (default: `example.com`)
- `--non-interactive` - Skip interactive prompts
- `--dry-run` - Show what would be created without creating files
- `-f, --force` - Overwrite existing files without prompting
- `-v, --verbose` - Enable verbose output

**Examples:**

```bash
# Basic usage
kubegen new my-operator

# With custom domain
kubegen new my-operator --domain mycompany.io

# Preview without creating files
kubegen new my-operator --dry-run

# Overwrite existing project
kubegen new my-operator --force
```

**Generated Structure:**

```
my-operator/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── lib.rs
│   └── error.rs
└── manifests/
```

### `kubegen add crd`

Adds a Custom Resource Definition (CRD) to an existing project.

```bash
kubegen add crd <KIND> [OPTIONS]
```

**Arguments:**
- `<KIND>` - Kind name in PascalCase (e.g., `MyResource`)

**Options:**
- `-g, --group <GROUP>` - API group (e.g., `apps.example.com`)
- `--api-version <VERSION>` - API version (default: `v1alpha1`)
- `--dry-run` - Show what would be created without creating files
- `-f, --force` - Overwrite existing files without prompting

**Examples:**

```bash
# Basic CRD
kubegen add crd MyResource

# With custom group and version
kubegen add crd Database --group databases.example.com --api-version v1beta1
```

**Generated Files:**

```
src/
└── my_resource/
    ├── mod.rs
    ├── types.rs       # Spec and Status structs
    ├── controller.rs  # Reconciliation logic
    ├── finalizer.rs   # Cleanup handlers
    └── status.rs      # Status condition helpers
manifests/
└── my_resource-crd.yaml
```

### `kubegen add metrics`

Adds Prometheus metrics support to the operator.

```bash
kubegen add metrics [OPTIONS]
```

**Options:**
- `-p, --port <PORT>` - Port for metrics endpoint (default: `8080`)
- `--dry-run` - Show what would be created without creating files
- `-f, --force` - Overwrite existing files without prompting

**Examples:**

```bash
# Default port 8080
kubegen add metrics

# Custom port
kubegen add metrics --port 9090
```

**Generated Files:**

```
src/
└── metrics.rs
manifests/
└── servicemonitor.yaml
```

**Metrics Exposed:**

- `<operator>_reconcile_total` - Total reconciliation attempts
- `<operator>_reconcile_duration_seconds` - Reconciliation duration histogram
- `<operator>_reconcile_errors_total` - Error counter

### `kubegen add webhook`

Adds admission webhook support for validating and/or mutating resources.

```bash
kubegen add webhook <KIND> [OPTIONS]
```

**Arguments:**
- `<KIND>` - Kind name (must match an existing CRD)

**Options:**
- `--validating` - Create a validating webhook
- `--mutating` - Create a mutating webhook
- `-g, --group <GROUP>` - API group for the webhook
- `--service-name <NAME>` - Kubernetes service name (default: `<project>-webhook`)
- `--namespace <NS>` - Namespace for the webhook service (default: `default`)
- `--dry-run` - Show what would be created without creating files
- `-f, --force` - Overwrite existing files without prompting

**Examples:**

```bash
# Validating webhook only
kubegen add webhook MyResource --validating

# Both validating and mutating
kubegen add webhook MyResource --validating --mutating

# With custom service configuration
kubegen add webhook MyResource --validating --service-name my-webhook --namespace operators
```

**Generated Files:**

```
src/
└── webhook/
    ├── mod.rs
    ├── validating.rs
    └── mutating.rs
manifests/
├── validating-webhook-config.yaml
├── mutating-webhook-config.yaml
├── certificate.yaml
└── issuer.yaml
```

## Global Options

These options work with any command:

- `-v, --verbose` - Enable verbose/debug output
- `--help` - Show help information
- `--version` - Show version information

The `--verbose` flag can be placed anywhere:

```bash
kubegen --verbose new my-operator
kubegen new --verbose my-operator
kubegen new my-operator --verbose
```

## Workflow Example

Here's a complete workflow for creating a database operator:

```bash
# 1. Create the project
kubegen new database-operator --domain databases.mycompany.io

# 2. Navigate to project
cd database-operator

# 3. Add a CRD for PostgreSQL databases
kubegen add crd PostgresDatabase --group postgres.databases.mycompany.io

# 4. Add a CRD for MySQL databases
kubegen add crd MySQLDatabase --group mysql.databases.mycompany.io

# 5. Add metrics
kubegen add metrics

# 6. Add validation webhook for PostgresDatabase
kubegen add webhook PostgresDatabase --validating

# 7. Build and test
cargo build
cargo test

# 8. Apply CRDs to cluster
kubectl apply -f manifests/
```

## Best Practices

### Naming Conventions

- **Project names**: Use lowercase with hyphens (`my-operator`)
- **Kind names**: Use PascalCase (`MyResource`, `DatabaseCluster`)
- **Groups**: Use reverse domain notation (`apps.example.com`)

### API Versions

- Use `v1alpha1` for experimental APIs
- Use `v1beta1` for APIs with stability guarantees
- Use `v1` for stable, production APIs

### Project Structure

kubegen follows kube-rs conventions:

- One module per CRD under `src/`
- CRD manifests in `manifests/`
- Main entry point handles controller setup

## Next Steps

After generating your operator:

1. **Customize the CRD spec** - Edit `types.rs` to add your fields
2. **Implement reconciliation** - Add logic in `controller.rs`
3. **Add tests** - Write unit and integration tests
4. **Build container image** - Create a Dockerfile for deployment
5. **Deploy to cluster** - Use Helm or Kustomize for production

## See Also

- [Examples](../examples/) - Working operator examples
- [Architecture](architecture.md) - kubegen internals
- [Troubleshooting](troubleshooting.md) - Common issues and solutions
