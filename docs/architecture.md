# kubegen Architecture

This document describes the internal architecture of kubegen.

## Overview

kubegen is a CLI tool that generates Kubernetes operator scaffolding. It uses a template-based code generation approach with embedded templates compiled into the binary.

```
┌─────────────────────────────────────────────────────────────┐
│                         CLI Layer                           │
│                     (src/cli/mod.rs)                        │
│              Parses commands using clap v4                  │
└─────────────────────────┬───────────────────────────────────┘
                          │
┌─────────────────────────▼───────────────────────────────────┐
│                     Command Layer                           │
│                   (src/commands/*.rs)                       │
│           Orchestrates validation and generation            │
└─────────────────────────┬───────────────────────────────────┘
                          │
        ┌─────────────────┼─────────────────┐
        │                 │                 │
┌───────▼───────┐ ┌───────▼───────┐ ┌───────▼───────┐
│  Validation   │ │   Templates   │ │  Filesystem   │
│  (validation/)│ │  (templates/) │ │    (fs/)      │
│               │ │               │ │               │
│ Name checks   │ │ Embedded via  │ │ File writing  │
│ Format rules  │ │ rust-embed    │ │ Dir creation  │
└───────────────┘ └───────────────┘ └───────────────┘
```

## Module Structure

### Core Modules

| Module | Path | Purpose |
|--------|------|---------|
| `cli` | `src/cli/mod.rs` | Command-line argument parsing |
| `commands` | `src/commands/*.rs` | Command implementations |
| `templates` | `src/templates/*.rs` | Template rendering system |
| `validation` | `src/validation/mod.rs` | Input validation |
| `fs` | `src/fs/mod.rs` | File system operations |
| `error` | `src/error.rs` | Error types |
| `logging` | `src/logging.rs` | Tracing setup |
| `prompts` | `src/prompts.rs` | Interactive prompts |

## Template System

### Embedded Templates

Templates are compiled into the binary using `rust-embed`:

```rust
#[derive(Embed)]
#[folder = "templates/"]
pub struct TemplateAssets;
```

This approach:
- Eliminates runtime template file dependencies
- Ensures templates are always available
- Simplifies distribution (single binary)

### Template Categories

Templates are organized by category:

| Category | Directory | Purpose |
|----------|-----------|---------|
| `Project` | `templates/project/` | Initial project scaffolding |
| `Crd` | `templates/crd/` | CRD types and controllers |
| `Webhook` | `templates/webhook/` | Admission webhooks |
| `Metrics` | `templates/metrics/` | Prometheus metrics |

### Variable Substitution

Templates use `{{variable}}` syntax for substitution:

```rust
let template = "name = \"{{project_name}}\"";
let mut ctx = TemplateContext::new();
ctx.set("project_name", "my-operator");
let result = renderer.render(&template, &ctx)?;
// Result: name = "my-operator"
```

The `SimpleRenderer` performs basic string substitution:
- Parses `{{variable}}` patterns
- Replaces with values from `TemplateContext`
- Supports strict mode (error on missing variables)
- Supports non-strict mode (preserve missing variables)

### Template Context Builders

Typed context builders ensure required variables are set:

```rust
// Project context with builder pattern
let project = ProjectContext::builder()
    .name("my-operator")
    .group("example.com")
    .version("v1")
    .kind("MyResource")
    .build()?;

// CRD context
let crd = CrdContext::builder()
    .group("apps.example.com")
    .version("v1alpha1")
    .kind("Database")
    .build()?;
```

## Validation Layer

### Validation Rules

| Input | Rules | Example |
|-------|-------|---------|
| Project name | DNS-1123 subdomain | `my-operator` |
| CRD kind | PascalCase, alphanumeric | `MyResource` |
| API version | `v[0-9]+(alpha\|beta)?[0-9]*` | `v1alpha1` |
| API group | DNS subdomain, lowercase | `apps.example.com` |

### Validation Flow

```
User Input
    │
    ▼
┌─────────────────┐
│ validate_*()    │
│ functions       │
└────────┬────────┘
         │
    ┌────┴────┐
    │         │
    ▼         ▼
  Ok(())    Err(KubegenError)
```

All validation functions return `Result<()>` with descriptive error messages.

## Code Generation Flow

### `kubegen new` Command

```
1. Parse arguments (name, domain, flags)
         │
         ▼
2. Validate project name
         │
         ▼
3. Check directory doesn't exist (unless --force)
         │
         ▼
4. Build ProjectContext with all variables
         │
         ▼
5. For each project template:
   - Load embedded template
   - Render with context
   - Write to file
         │
         ▼
6. Output success message
```

### `kubegen add crd` Command

```
1. Parse arguments (kind, group, version)
         │
         ▼
2. Validate all inputs (kind, group, version)
         │
         ▼
3. Build CrdContext
         │
         ▼
4. Generate derived values:
   - kind_snake (MyResource → my_resource)
   - plural (MyResource → myresources)
   - short_name (MyResource → mr)
         │
         ▼
5. For each CRD template:
   - types.rs.tmpl → src/<kind_snake>/types.rs
   - controller.rs.tmpl → src/<kind_snake>/controller.rs
   - finalizer.rs.tmpl → src/<kind_snake>/finalizer.rs
   - status.rs.tmpl → src/<kind_snake>/status.rs
   - mod.rs.tmpl → src/<kind_snake>/mod.rs
   - crd.yaml.tmpl → manifests/<kind_snake>-crd.yaml
   - example.yaml.tmpl → examples/example-<kind_snake>.yaml
```

## Error Handling

### Error Types

All errors are defined in `src/error.rs` using `thiserror`:

```rust
#[derive(Error, Debug)]
pub enum KubegenError {
    #[error("Invalid project name '{name}': {reason}")]
    InvalidProjectName { name: String, reason: String },

    #[error("Invalid CRD name - group: '{group}', version: '{version}', kind: '{kind}': {reason}")]
    InvalidCrdName { group: String, version: String, kind: String, reason: String },

    #[error("Template not found: {template_name}")]
    TemplateNotFound { template_name: String },

    // ... more error types
}
```

### Error Propagation

Commands use the `?` operator for clean error propagation:

```rust
pub fn execute_add_crd(args: &CrdArgs) -> Result<()> {
    validate_crd_kind(&args.kind)?;
    validate_crd_version(&args.api_version)?;
    // ...
}
```

## Helper Functions

### Name Transformations

| Function | Input | Output |
|----------|-------|--------|
| `to_snake_case()` | `MyResource` | `my_resource` |
| `pluralize()` | `Policy` | `policies` |
| `generate_short_name()` | `MyResource` | `mr` |

### Pluralization Rules

The `pluralize()` function handles common English patterns:
- Words ending in s, x, ch, sh → add "es"
- Words ending in consonant + y → change to "ies"
- Other words → add "s"

## Testing

### Unit Tests

Each module has comprehensive unit tests:

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_validate_project_name() {
        assert!(validate_project_name("my-operator").is_ok());
        assert!(validate_project_name("My-Operator").is_err());
    }
}
```

### Integration Tests

Template rendering is tested against actual embedded templates:

```rust
#[test]
fn test_render_embedded_project_cargo() {
    let content = get_template("project/Cargo.toml.tmpl").unwrap();
    let renderer = SimpleRenderer::new();
    // ...
}
```

## Extension Points

### Adding New Templates

1. Create template in `templates/<category>/`
2. Use `{{variable}}` syntax for substitution
3. Template is automatically embedded via `rust-embed`
4. Update command to render new template

### Adding New Commands

1. Define args struct in `src/cli/mod.rs`
2. Create command implementation in `src/commands/`
3. Add command variant to `Commands` enum
4. Wire up in `src/main.rs`

### Adding New Validation

1. Add validation function in `src/validation/mod.rs`
2. Return `Result<()>` with descriptive errors
3. Call from command implementation

## Dependencies

| Crate | Purpose |
|-------|---------|
| `clap` | CLI argument parsing |
| `rust-embed` | Embed templates in binary |
| `thiserror` | Error type derivation |
| `tracing` | Logging and diagnostics |
| `dialoguer` | Interactive prompts |

## See Also

- [User Guide](user-guide.md) - How to use kubegen
- [Troubleshooting](troubleshooting.md) - Common issues
