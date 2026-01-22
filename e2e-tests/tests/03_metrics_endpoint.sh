#!/bin/bash
# E2E Test: Metrics Endpoint
#
# Tests that a generated operator with metrics exposes a working /metrics endpoint:
# 1. Generate a new operator with kubegen new
# 2. Add metrics with kubegen add metrics
# 3. Build and deploy to kind cluster
# 4. Verify /metrics endpoint returns Prometheus metrics

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../lib/cluster.sh"
source "$SCRIPT_DIR/../lib/kubectl.sh"

# Test configuration
TEST_PROJECT="e2e-metrics"
TEST_KIND="MetricResource"
TEST_GROUP="metrics.kubegen.io"
TEST_VERSION="v1alpha1"
TEST_NAMESPACE="e2e-metrics-test"
TEST_IMAGE="e2e-metrics:test"
METRICS_PORT=8080

# Paths
PROJECT_DIR=""
KUBEGEN_BIN=""

setup() {
    log_info "Setting up metrics endpoint test..."

    # Find kubegen binary
    KUBEGEN_BIN="$(cd "$SCRIPT_DIR/../.." && pwd)/target/release/kubegen"
    if [ ! -f "$KUBEGEN_BIN" ]; then
        KUBEGEN_BIN="$(cd "$SCRIPT_DIR/../.." && pwd)/target/debug/kubegen"
    fi

    if [ ! -f "$KUBEGEN_BIN" ]; then
        log_error "kubegen binary not found. Run 'cargo build' first."
        return 1
    fi

    log_info "Using kubegen: ${KUBEGEN_BIN}"

    # Create temp directory for test project
    PROJECT_DIR=$(mktemp -d)
    log_info "Test project directory: ${PROJECT_DIR}"

    # Create test namespace
    ensure_namespace "$TEST_NAMESPACE"
}

cleanup() {
    log_info "Cleaning up..."

    # Delete service
    kubectl delete service "$TEST_PROJECT" -n "$TEST_NAMESPACE" --ignore-not-found=true 2>/dev/null || true

    # Delete deployment
    kubectl delete deployment "$TEST_PROJECT" -n "$TEST_NAMESPACE" --ignore-not-found=true 2>/dev/null || true

    # Delete CRD resources
    if [ -n "$PROJECT_DIR" ] && [ -d "$PROJECT_DIR/$TEST_PROJECT" ]; then
        kubectl delete -f "$PROJECT_DIR/$TEST_PROJECT/manifests/" --ignore-not-found=true 2>/dev/null || true
    fi

    # Delete namespace
    delete_namespace "$TEST_NAMESPACE" 2>/dev/null || true

    # Remove temp directory
    if [ -n "$PROJECT_DIR" ] && [ -d "$PROJECT_DIR" ]; then
        rm -rf "$PROJECT_DIR"
    fi
}

test_generate_operator_with_metrics() {
    log_info "Generating test operator with metrics..."

    cd "$PROJECT_DIR"
    "$KUBEGEN_BIN" new "$TEST_PROJECT" --domain "$TEST_GROUP" --non-interactive

    if [ ! -d "$TEST_PROJECT" ]; then
        log_error "Failed to generate operator project"
        return 1
    fi

    log_info "Operator project generated"

    # Add CRD
    cd "$TEST_PROJECT"
    "$KUBEGEN_BIN" add crd "$TEST_KIND" --group "$TEST_GROUP" --api-version "$TEST_VERSION" --force

    log_info "CRD added to project"

    # Add metrics support
    "$KUBEGEN_BIN" add metrics --port "$METRICS_PORT" --force

    # Verify metrics module was added (it's in src/metrics/mod.rs)
    if [ ! -f "src/metrics/mod.rs" ]; then
        log_error "src/metrics/mod.rs not found after 'kubegen add metrics'"
        return 1
    fi

    log_info "Metrics module added"

    # Wire up metrics in main.rs - add mod metrics and call the metrics server
    cat > src/main.rs <<EOF
//! ${TEST_PROJECT} - A Kubernetes operator built with kube-rs

use std::sync::Arc;

use kube::Client;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod error;
mod metrics;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting ${TEST_PROJECT}");

    // Create Kubernetes client
    let client = Client::try_default().await?;
    let _client = Arc::new(client);

    // Initialize and start metrics server
    let metrics = metrics::init();
    tokio::spawn(metrics::run(metrics.clone()));

    info!("${TEST_PROJECT} initialized successfully");
    info!("Metrics available at http://0.0.0.0:${METRICS_PORT}/metrics");

    // Keep the main task running
    tokio::signal::ctrl_c().await?;
    info!("Shutting down ${TEST_PROJECT}");

    Ok(())
}
EOF

    log_info "main.rs updated with metrics integration"
}

test_build_operator() {
    log_info "Building operator with metrics..."

    cd "$PROJECT_DIR/$TEST_PROJECT"

    # Create a Dockerfile for the operator
    cat > Dockerfile <<'EOF'
# Build stage - use latest stable Rust
FROM rust:slim-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy source
COPY . .

# Build release binary
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/e2e-metrics /usr/local/bin/operator

EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/operator"]
EOF

    log_info "Dockerfile created"

    # Build the Docker image
    log_info "Building Docker image (this may take a while)..."
    docker build -t "$TEST_IMAGE" . 2>&1 | tail -20

    if [ $? -ne 0 ]; then
        log_error "Failed to build Docker image"
        return 1
    fi

    log_info "Docker image built: ${TEST_IMAGE}"

    # Load image into kind cluster
    log_info "Loading image into kind cluster..."
    kind load docker-image "$TEST_IMAGE" --name "$CLUSTER_NAME"

    log_info "Image loaded into cluster"
}

test_deploy_crd() {
    log_info "Deploying CRD..."

    cd "$PROJECT_DIR/$TEST_PROJECT"

    # Apply CRD
    kubectl_apply "manifests/metric_resource-crd.yaml"

    # Wait for CRD to be established
    local crd_name="metricresources.${TEST_GROUP}"
    kubectl_wait_for "crd" "$crd_name"

    kubectl wait --for=condition=established "crd/$crd_name" --timeout=30s

    log_info "CRD deployed and established"
}

test_deploy_operator() {
    log_info "Deploying operator with metrics..."

    cd "$PROJECT_DIR/$TEST_PROJECT"

    # Create deployment manifest with service for metrics
    cat > manifests/deployment.yaml <<EOF
apiVersion: apps/v1
kind: Deployment
metadata:
  name: ${TEST_PROJECT}
  namespace: ${TEST_NAMESPACE}
  labels:
    app: ${TEST_PROJECT}
spec:
  replicas: 1
  selector:
    matchLabels:
      app: ${TEST_PROJECT}
  template:
    metadata:
      labels:
        app: ${TEST_PROJECT}
    spec:
      serviceAccountName: ${TEST_PROJECT}
      containers:
      - name: operator
        image: ${TEST_IMAGE}
        imagePullPolicy: Never
        ports:
        - containerPort: ${METRICS_PORT}
          name: metrics
        env:
        - name: RUST_LOG
          value: info
        resources:
          limits:
            memory: "128Mi"
            cpu: "500m"
          requests:
            memory: "64Mi"
            cpu: "100m"
---
apiVersion: v1
kind: Service
metadata:
  name: ${TEST_PROJECT}
  namespace: ${TEST_NAMESPACE}
  labels:
    app: ${TEST_PROJECT}
spec:
  selector:
    app: ${TEST_PROJECT}
  ports:
  - port: ${METRICS_PORT}
    targetPort: ${METRICS_PORT}
    name: metrics
---
apiVersion: v1
kind: ServiceAccount
metadata:
  name: ${TEST_PROJECT}
  namespace: ${TEST_NAMESPACE}
---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: ${TEST_PROJECT}
rules:
- apiGroups: ["${TEST_GROUP}"]
  resources: ["metricresources"]
  verbs: ["get", "list", "watch", "create", "update", "patch", "delete"]
- apiGroups: ["${TEST_GROUP}"]
  resources: ["metricresources/status"]
  verbs: ["get", "update", "patch"]
---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRoleBinding
metadata:
  name: ${TEST_PROJECT}
roleRef:
  apiGroup: rbac.authorization.k8s.io
  kind: ClusterRole
  name: ${TEST_PROJECT}
subjects:
- kind: ServiceAccount
  name: ${TEST_PROJECT}
  namespace: ${TEST_NAMESPACE}
EOF

    # Apply deployment
    kubectl_apply "manifests/deployment.yaml"

    log_info "Operator deployment created"
}

test_verify_pods_running() {
    log_info "Verifying operator pods are running..."

    # Wait for deployment to be available
    kubectl_wait_deployment "$TEST_PROJECT" "$TEST_NAMESPACE" "120s"

    # Verify pod is running
    local pod_status
    pod_status=$(kubectl get pods -l "app=$TEST_PROJECT" -n "$TEST_NAMESPACE" -o jsonpath='{.items[0].status.phase}')

    if [ "$pod_status" != "Running" ]; then
        log_error "Pod is not running. Status: ${pod_status}"
        kubectl describe pods -l "app=$TEST_PROJECT" -n "$TEST_NAMESPACE"
        return 1
    fi

    log_info "Operator pod is running"

    # Get pod name
    local pod_name
    pod_name=$(kubectl get pods -l "app=$TEST_PROJECT" -n "$TEST_NAMESPACE" -o jsonpath='{.items[0].metadata.name}')
    log_info "Pod name: ${pod_name}"

    # Show recent logs
    log_info "Recent pod logs:"
    kubectl logs "$pod_name" -n "$TEST_NAMESPACE" --tail=10 || true
}

test_metrics_endpoint() {
    log_info "Testing metrics endpoint..."

    # Get pod IP directly - more reliable than port-forward
    local pod_ip
    pod_ip=$(kubectl get pods -l "app=$TEST_PROJECT" -n "$TEST_NAMESPACE" -o jsonpath='{.items[0].status.podIP}')
    log_info "Pod IP: ${pod_ip}"

    # Use a curl pod to fetch metrics from within the cluster
    log_info "Fetching metrics from http://${pod_ip}:${METRICS_PORT}/metrics"

    # Run curl in a pod - use --quiet to suppress pod lifecycle messages
    # Store output in a configmap to reliably retrieve it
    local metrics_response

    # Create a job that writes output to a configmap
    kubectl delete configmap metrics-output -n "$TEST_NAMESPACE" --ignore-not-found=true 2>/dev/null || true

    # Run curl and store result
    metrics_response=$(kubectl run curl-metrics --rm -i --restart=Never --quiet \
        --image=curlimages/curl:latest \
        -n "$TEST_NAMESPACE" \
        -- curl -s --connect-timeout 10 --max-time 15 "http://${pod_ip}:${METRICS_PORT}/metrics" 2>/dev/null)

    log_info "Metrics response received (${#metrics_response} bytes)"

    # Verify response contains Prometheus metrics
    if echo "$metrics_response" | grep -q "# HELP\|# TYPE"; then
        log_info "Metrics endpoint returned valid Prometheus metrics"
        log_info "Sample metrics output:"
        echo "$metrics_response" | head -20
    else
        log_error "Metrics endpoint did not return valid Prometheus format"
        log_error "Response length: ${#metrics_response}"
        log_error "First 200 chars: ${metrics_response:0:200}"

        # Debug: try wget as fallback
        log_info "Trying wget as fallback..."
        local wget_response
        wget_response=$(kubectl run wget-metrics --rm -i --restart=Never --quiet \
            --image=busybox:latest \
            -n "$TEST_NAMESPACE" \
            -- wget -qO- --timeout=10 "http://${pod_ip}:${METRICS_PORT}/metrics" 2>/dev/null) || true

        if [ -n "$wget_response" ]; then
            log_info "wget response (${#wget_response} bytes):"
            echo "$wget_response" | head -20
            metrics_response="$wget_response"
        fi

        # Check again after wget
        if echo "$metrics_response" | grep -q "# HELP\|# TYPE"; then
            log_info "Metrics endpoint returned valid Prometheus metrics (via wget)"
        else
            return 1
        fi
    fi

    # Check for expected operator metrics
    if echo "$metrics_response" | grep -qE "reconciliations_total|reconciliation"; then
        log_info "Found expected operator metrics"
    else
        log_warn "Expected operator metrics not found (reconciliations_total)"
        log_info "This may be okay if no reconciliations have occurred yet"
    fi
}

test_health_endpoint() {
    log_info "Testing health endpoint..."

    # Get pod IP
    local pod_ip
    pod_ip=$(kubectl get pods -l "app=$TEST_PROJECT" -n "$TEST_NAMESPACE" -o jsonpath='{.items[0].status.podIP}')

    # Use wget to check health endpoint (simpler output handling)
    local health_response
    health_response=$(kubectl run wget-health --rm -i --restart=Never --quiet \
        --image=busybox:latest \
        -n "$TEST_NAMESPACE" \
        -- wget -qO- --timeout=10 "http://${pod_ip}:${METRICS_PORT}/healthz" 2>/dev/null) || true

    if [ "$health_response" = "ok" ]; then
        log_info "Health endpoint returned 'ok'"
    else
        log_warn "Health endpoint response: $health_response (may not be implemented)"
    fi
}

# Main test function
main() {
    local exit_code=0

    # Setup
    setup || { log_error "Setup failed"; return 1; }

    # Run tests in sequence
    test_generate_operator_with_metrics || exit_code=1

    if [ $exit_code -eq 0 ]; then
        test_build_operator || exit_code=1
    fi

    if [ $exit_code -eq 0 ]; then
        test_deploy_crd || exit_code=1
    fi

    if [ $exit_code -eq 0 ]; then
        test_deploy_operator || exit_code=1
    fi

    if [ $exit_code -eq 0 ]; then
        test_verify_pods_running || exit_code=1
    fi

    if [ $exit_code -eq 0 ]; then
        test_metrics_endpoint || exit_code=1
    fi

    if [ $exit_code -eq 0 ]; then
        test_health_endpoint || exit_code=1
    fi

    # Cleanup
    cleanup

    if [ $exit_code -eq 0 ]; then
        log_info "All metrics endpoint tests passed"
    else
        log_error "Some tests failed"
    fi

    return $exit_code
}

main
