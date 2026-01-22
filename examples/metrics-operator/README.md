# Metrics Operator Example

A Kubernetes operator with Prometheus metrics integration.

## Features

All features from basic-operator, plus:
- Prometheus metrics endpoint on `:8080/metrics`
- `reconcile_total` - Counter for reconciliation attempts
- `reconcile_duration_seconds` - Histogram for reconciliation time
- `reconcile_errors_total` - Counter for errors
- ServiceMonitor for Prometheus Operator integration

## Project Structure

```
metrics-operator/
├── Cargo.toml
├── src/
│   ├── main.rs             # Starts metrics server on :8080
│   ├── lib.rs
│   ├── error.rs
│   ├── metrics.rs          # Prometheus metrics setup
│   └── my_resource/
│       └── ...
├── manifests/
│   ├── my_resource-crd.yaml
│   └── servicemonitor.yaml   # Prometheus Operator config
└── examples/
    └── example-my_resource.yaml
```

## Metrics Endpoint

The operator exposes metrics at `http://localhost:8080/metrics`:

```bash
# Run the operator
cargo run

# In another terminal, check metrics
curl http://localhost:8080/metrics
```

Example output:
```
# HELP metrics_operator_reconcile_total Total number of reconciliations
# TYPE metrics_operator_reconcile_total counter
metrics_operator_reconcile_total 42

# HELP metrics_operator_reconcile_duration_seconds Time spent in reconciliation
# TYPE metrics_operator_reconcile_duration_seconds histogram
metrics_operator_reconcile_duration_seconds_bucket{le="0.005"} 30
...
```

## Prometheus Integration

If you have Prometheus Operator installed:

```bash
# Apply the ServiceMonitor
kubectl apply -f manifests/servicemonitor.yaml

# Prometheus will automatically discover and scrape the operator
```

## Generated with

```bash
kubegen new metrics-operator --group example.com --version v1alpha1 --kind MyResource
# Metrics are included by default
```
