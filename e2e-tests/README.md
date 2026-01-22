# End-to-End Tests

This directory contains e2e tests for kubegen that verify generated operators work correctly in a real Kubernetes cluster.

## Prerequisites

- [kind](https://kind.sigs.k8s.io/) - Kubernetes in Docker
- [kubectl](https://kubernetes.io/docs/tasks/tools/) - Kubernetes CLI
- [Docker](https://docs.docker.com/get-docker/) - Container runtime

## Quick Start

```bash
# Run all e2e tests
./run-tests.sh

# Run specific test
./run-tests.sh test_crd_deployment
```

## Test Structure

```
e2e-tests/
├── README.md           # This file
├── run-tests.sh        # Main test runner
├── lib/
│   ├── cluster.sh      # kind cluster management
│   └── kubectl.sh      # kubectl helpers
└── tests/
    ├── 00_cluster_health.sh      # Verify cluster is healthy
    ├── 01_crd_deployment.sh      # Test CRD deployment
    ├── 02_operator_deployment.sh # Full operator deployment test
    └── 03_metrics_endpoint.sh    # Test metrics endpoint accessibility
```

## Available Tests

| Test | Description |
|------|-------------|
| `00_cluster_health` | Verifies the kind cluster is healthy and accessible |
| `01_crd_deployment` | Tests CRD generation and deployment |
| `02_operator_deployment` | Full flow: generate operator, build Docker image, deploy to cluster, verify pods running |
| `03_metrics_endpoint` | Generate operator with metrics, deploy, verify /metrics endpoint returns Prometheus data |

## Writing Tests

Tests are shell scripts that use the helper functions from `lib/`.

```bash
#!/bin/bash
source "$(dirname "$0")/../lib/cluster.sh"
source "$(dirname "$0")/../lib/kubectl.sh"

test_my_feature() {
    # Setup
    ensure_cluster

    # Test logic
    kubectl_apply "manifests/my-crd.yaml"
    kubectl_wait_for "crd" "myresources.example.com"

    # Assertions
    assert_resource_exists "crd" "myresources.example.com"
}

test_my_feature
```

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `CLUSTER_NAME` | `kubegen-e2e` | kind cluster name |
| `KUBECONFIG` | auto | Path to kubeconfig |
| `SKIP_CLEANUP` | `false` | Keep cluster after tests |
| `SKIP_CLUSTER_CREATE` | `false` | Skip cluster creation (use existing) |
