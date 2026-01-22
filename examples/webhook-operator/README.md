# Webhook Operator Example

A Kubernetes operator with admission webhooks for validation and mutation.

## Features

All features from basic-operator, plus:
- Validating webhook to enforce spec constraints
- Mutating webhook to set default values
- TLS support with cert-manager integration
- Webhook configuration manifests

## Project Structure

```
webhook-operator/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── error.rs
│   ├── my_resource/
│   │   └── ...
│   └── webhook/
│       ├── mod.rs            # Webhook server setup
│       ├── validating.rs     # Validation logic
│       └── mutating.rs       # Mutation logic
├── manifests/
│   ├── my_resource-crd.yaml
│   ├── validating-webhook-config.yaml
│   ├── mutating-webhook-config.yaml
│   ├── certificate.yaml      # cert-manager Certificate
│   └── issuer.yaml           # cert-manager Issuer
└── examples/
    └── example-my_resource.yaml
```

## Prerequisites

- cert-manager installed in the cluster for TLS certificates

```bash
# Install cert-manager
kubectl apply -f https://github.com/cert-manager/cert-manager/releases/download/v1.14.0/cert-manager.yaml
```

## Deployment

```bash
# Apply cert-manager resources first
kubectl apply -f manifests/issuer.yaml
kubectl apply -f manifests/certificate.yaml

# Apply the CRD
kubectl apply -f manifests/my_resource-crd.yaml

# Apply webhook configurations
kubectl apply -f manifests/validating-webhook-config.yaml
kubectl apply -f manifests/mutating-webhook-config.yaml

# Deploy the operator (as a Deployment in-cluster)
# The webhook server listens on :8443 for HTTPS requests
```

## Webhook Behavior

### Validating Webhook

Rejects resources that don't meet validation rules:

```yaml
# This will be rejected if replicas < 0
apiVersion: example.com/v1alpha1
kind: MyResource
metadata:
  name: invalid-resource
spec:
  replicas: -1  # Invalid!
```

### Mutating Webhook

Sets default values for unspecified fields:

```yaml
# Before mutation
apiVersion: example.com/v1alpha1
kind: MyResource
metadata:
  name: my-resource
spec: {}

# After mutation (defaults applied)
apiVersion: example.com/v1alpha1
kind: MyResource
metadata:
  name: my-resource
spec:
  replicas: 1  # Default value set
```

## Generated with

```bash
kubegen new webhook-operator --group example.com --version v1alpha1 --kind MyResource
cd webhook-operator
kubegen add webhook MyResource --validating --mutating
```
