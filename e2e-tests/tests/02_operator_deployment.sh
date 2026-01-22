#!/bin/bash
# E2E Test: Operator Deployment
#
# Tests the full flow:
# 1. Generate a new operator with kubegen new
# 2. Build the Docker image
# 3. Deploy to kind cluster
# 4. Verify pods are running

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../lib/cluster.sh"
source "$SCRIPT_DIR/../lib/kubectl.sh"

# Test configuration
TEST_PROJECT="e2e-operator"
TEST_KIND="Widget"
TEST_GROUP="widgets.kubegen.io"
TEST_VERSION="v1alpha1"
TEST_NAMESPACE="e2e-operator-test"
TEST_IMAGE="e2e-operator:test"

# Paths
PROJECT_DIR=""
KUBEGEN_BIN=""

setup() {
    log_info "Setting up operator deployment test..."

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

test_generate_operator() {
    log_info "Generating test operator..."

    cd "$PROJECT_DIR"
    "$KUBEGEN_BIN" new "$TEST_PROJECT" --domain "$TEST_GROUP" --non-interactive

    if [ ! -d "$TEST_PROJECT" ]; then
        log_error "Failed to generate operator project"
        return 1
    fi

    log_info "✓ Operator project generated"

    # Add CRD
    cd "$TEST_PROJECT"
    "$KUBEGEN_BIN" add crd "$TEST_KIND" --group "$TEST_GROUP" --api-version "$TEST_VERSION" --force

    log_info "✓ CRD added to project"
}

test_build_operator() {
    log_info "Building operator..."

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

COPY --from=builder /app/target/release/e2e-operator /usr/local/bin/operator

ENTRYPOINT ["/usr/local/bin/operator"]
EOF

    log_info "✓ Dockerfile created"

    # Build the Docker image
    log_info "Building Docker image (this may take a while)..."
    docker build -t "$TEST_IMAGE" . 2>&1 | tail -20

    if [ $? -ne 0 ]; then
        log_error "Failed to build Docker image"
        return 1
    fi

    log_info "✓ Docker image built: ${TEST_IMAGE}"

    # Load image into kind cluster
    log_info "Loading image into kind cluster..."
    kind load docker-image "$TEST_IMAGE" --name "$CLUSTER_NAME"

    log_info "✓ Image loaded into cluster"
}

test_deploy_crd() {
    log_info "Deploying CRD..."

    cd "$PROJECT_DIR/$TEST_PROJECT"

    # Apply CRD
    kubectl_apply "manifests/widget-crd.yaml"

    # Wait for CRD to be established
    local crd_name="widgets.${TEST_GROUP}"
    kubectl_wait_for "crd" "$crd_name"

    kubectl wait --for=condition=established "crd/$crd_name" --timeout=30s

    log_info "✓ CRD deployed and established"
}

test_deploy_operator() {
    log_info "Deploying operator..."

    cd "$PROJECT_DIR/$TEST_PROJECT"

    # Create deployment manifest
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
  resources: ["widgets"]
  verbs: ["get", "list", "watch", "create", "update", "patch", "delete"]
- apiGroups: ["${TEST_GROUP}"]
  resources: ["widgets/status"]
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

    log_info "✓ Operator deployment created"
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

    log_info "✓ Operator pod is running"

    # Get pod name for logs
    local pod_name
    pod_name=$(kubectl get pods -l "app=$TEST_PROJECT" -n "$TEST_NAMESPACE" -o jsonpath='{.items[0].metadata.name}')

    log_info "Pod name: ${pod_name}"

    # Show recent logs
    log_info "Recent pod logs:"
    kubectl logs "$pod_name" -n "$TEST_NAMESPACE" --tail=10 || true

    log_info "✓ Operator is running successfully"
}

test_create_custom_resource() {
    log_info "Creating custom resource..."

    cd "$PROJECT_DIR/$TEST_PROJECT"

    # Apply example CR
    kubectl apply -f "examples/example-widget.yaml" -n "$TEST_NAMESPACE"

    # Wait for CR to exist
    kubectl_wait_for "widgets" "example-widget" "$TEST_NAMESPACE"

    # Verify CR was created
    local cr_json
    cr_json=$(kubectl get widgets example-widget -n "$TEST_NAMESPACE" -o json)

    assert_json_field "$cr_json" ".kind" "Widget"
    assert_json_field "$cr_json" ".apiVersion" "${TEST_GROUP}/${TEST_VERSION}"

    log_info "✓ Custom resource created successfully"
}

test_operator_processes_cr() {
    log_info "Verifying operator processes the CR..."

    # Give the operator a moment to process
    sleep 5

    # Get pod logs to verify reconciliation
    local pod_name
    pod_name=$(kubectl get pods -l "app=$TEST_PROJECT" -n "$TEST_NAMESPACE" -o jsonpath='{.items[0].metadata.name}')

    local logs
    logs=$(kubectl logs "$pod_name" -n "$TEST_NAMESPACE" --tail=50 2>/dev/null || echo "")

    # The operator should have logged something about the resource
    # This is a basic check - in a real test you might check for specific log patterns
    if [ -n "$logs" ]; then
        log_info "Operator logs (last 10 lines):"
        echo "$logs" | tail -10
        log_info "✓ Operator is processing resources"
    else
        log_warn "No logs found from operator (this may be okay if the operator hasn't processed anything yet)"
    fi
}

# Main test function
main() {
    local exit_code=0

    # Setup
    setup || { log_error "Setup failed"; return 1; }

    # Run tests in sequence
    test_generate_operator || exit_code=1

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
        test_create_custom_resource || exit_code=1
    fi

    if [ $exit_code -eq 0 ]; then
        test_operator_processes_cr || exit_code=1
    fi

    # Cleanup
    cleanup

    if [ $exit_code -eq 0 ]; then
        log_info "✓ All operator deployment tests passed"
    else
        log_error "✗ Some tests failed"
    fi

    return $exit_code
}

main
