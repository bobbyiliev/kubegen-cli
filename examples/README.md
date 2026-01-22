# kubegen Examples

This directory contains example operators generated with kubegen, demonstrating different features and configurations.

## Examples

### [basic-operator](./basic-operator/)

A minimal Kubernetes operator with:
- Single CRD (MyResource)
- Basic reconciliation loop
- Status conditions support
- Finalizer for cleanup

Generate with:
```bash
kubegen new basic-operator --group example.com --version v1alpha1 --kind MyResource
```

### [metrics-operator](./metrics-operator/)

An operator with Prometheus metrics integration:
- All features from basic-operator
- Prometheus metrics endpoint on :8080/metrics
- Reconciliation counters and histograms
- ServiceMonitor for Prometheus Operator

Generate with:
```bash
kubegen new metrics-operator --group example.com --version v1alpha1 --kind MyResource
# Metrics are included by default in generated projects
```

### [webhook-operator](./webhook-operator/)

An operator with admission webhooks:
- All features from basic-operator
- Validating webhook for spec validation
- Mutating webhook for defaults
- TLS certificate management

Generate with:
```bash
kubegen new webhook-operator --group example.com --version v1alpha1 --kind MyResource
cd webhook-operator
kubegen add webhook MyResource --validating --mutating
```

## Running the Examples

Each example includes:
- `Cargo.toml` - Rust dependencies
- `src/` - Operator source code
- `manifests/` - Kubernetes manifests (CRD, RBAC, etc.)
- `examples/` - Sample CR instances

To run an example:

```bash
cd examples/basic-operator

# Build the operator
cargo build

# Apply the CRD to your cluster
kubectl apply -f manifests/

# Run the operator locally
cargo run

# In another terminal, create a sample resource
kubectl apply -f examples/
```

## Prerequisites

- Rust 1.70+
- kubectl configured with cluster access
- For webhook-operator: cert-manager installed in cluster
