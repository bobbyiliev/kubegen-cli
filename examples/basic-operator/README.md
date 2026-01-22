# Basic Operator Example

A minimal Kubernetes operator demonstrating core kubegen features.

## Features

- Custom Resource Definition (CRD) for `MyResource`
- Reconciliation loop with status updates
- Finalizer support for cleanup on deletion
- Status conditions following Kubernetes conventions

## Project Structure

```
basic-operator/
├── Cargo.toml              # Rust dependencies
├── src/
│   ├── main.rs             # Entry point with tracing and metrics
│   ├── lib.rs              # Library exports
│   ├── error.rs            # Error types
│   └── my_resource/        # CRD module
│       ├── mod.rs          # Module exports
│       ├── types.rs        # CRD spec and status types
│       ├── controller.rs   # Reconciliation logic
│       ├── finalizer.rs    # Finalizer helpers
│       └── status.rs       # Status condition helpers
├── manifests/
│   └── my_resource-crd.yaml  # CRD manifest
└── examples/
    └── example-my_resource.yaml  # Sample CR
```

## Quick Start

```bash
# Build the operator
cargo build

# Apply the CRD
kubectl apply -f manifests/my_resource-crd.yaml

# Run the operator
cargo run

# Create a sample resource
kubectl apply -f examples/example-my_resource.yaml

# Check the resource status
kubectl get myresources
kubectl describe myresource example-my_resource
```

## Generated with

```bash
kubegen new basic-operator --group example.com --version v1alpha1 --kind MyResource
```
