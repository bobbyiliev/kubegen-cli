#!/bin/bash
# E2E Test: CRD Deployment
#
# Tests that a generated CRD can be deployed to a cluster and
# custom resources can be created successfully.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../lib/cluster.sh"
source "$SCRIPT_DIR/../lib/kubectl.sh"

# Test configuration
TEST_PROJECT="test-operator"
TEST_KIND="TestResource"
TEST_GROUP="test.kubegen.io"
TEST_VERSION="v1alpha1"
TEST_NAMESPACE="e2e-crd-test"

# Paths
PROJECT_DIR=""
KUBEGEN_BIN=""

setup() {
    log_info "Setting up CRD deployment test..."

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

    # Delete test resources
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

    # Check CRD manifest exists
    local crd_file="manifests/test_resource-crd.yaml"
    if [ ! -f "$crd_file" ]; then
        log_error "CRD manifest not found: $crd_file"
        return 1
    fi

    log_info "✓ CRD generated"
}

test_deploy_crd() {
    log_info "Deploying CRD to cluster..."

    cd "$PROJECT_DIR/$TEST_PROJECT"

    # Apply CRD
    kubectl_apply "manifests/test_resource-crd.yaml"

    # Wait for CRD to be established
    local crd_name="testresourcess.${TEST_GROUP}"
    kubectl_wait_for "crd" "$crd_name"

    # Verify CRD is established
    kubectl wait --for=condition=established "crd/$crd_name" --timeout=30s

    log_info "✓ CRD deployed and established"
}

test_create_cr() {
    log_info "Creating custom resource..."

    cd "$PROJECT_DIR/$TEST_PROJECT"

    # Check if example CR exists
    local example_file="examples/example-test_resource.yaml"
    if [ ! -f "$example_file" ]; then
        log_error "Example CR not found: $example_file"
        return 1
    fi

    # Apply example CR to test namespace
    kubectl apply -f "$example_file" -n "$TEST_NAMESPACE"

    # Verify CR was created
    kubectl_wait_for "testresources" "example-test-resource" "$TEST_NAMESPACE"

    # Get the CR and verify it exists
    local cr_json
    cr_json=$(kubectl get testresources example-test-resource -n "$TEST_NAMESPACE" -o json)

    assert_json_field "$cr_json" ".kind" "TestResource"
    assert_json_field "$cr_json" ".apiVersion" "${TEST_GROUP}/${TEST_VERSION}"

    log_info "✓ Custom resource created successfully"
}

test_cr_status() {
    log_info "Verifying CR structure..."

    cd "$PROJECT_DIR/$TEST_PROJECT"

    # Get CR
    local cr_json
    cr_json=$(kubectl get testresources example-test-resource -n "$TEST_NAMESPACE" -o json)

    # Verify spec exists
    local spec
    spec=$(echo "$cr_json" | jq -r '.spec // empty')
    if [ -z "$spec" ]; then
        log_error "CR spec is missing"
        return 1
    fi

    log_info "✓ CR structure is valid"
}

test_delete_cr() {
    log_info "Testing CR deletion..."

    kubectl delete testresources example-test-resource -n "$TEST_NAMESPACE"

    # Verify CR is deleted
    sleep 2
    if kubectl get testresources example-test-resource -n "$TEST_NAMESPACE" &>/dev/null; then
        log_error "CR still exists after deletion"
        return 1
    fi

    log_info "✓ CR deleted successfully"
}

# Main test function
main() {
    local exit_code=0

    # Setup
    setup || { log_error "Setup failed"; return 1; }

    # Run tests
    test_generate_operator || exit_code=1

    if [ $exit_code -eq 0 ]; then
        test_deploy_crd || exit_code=1
    fi

    if [ $exit_code -eq 0 ]; then
        test_create_cr || exit_code=1
    fi

    if [ $exit_code -eq 0 ]; then
        test_cr_status || exit_code=1
    fi

    if [ $exit_code -eq 0 ]; then
        test_delete_cr || exit_code=1
    fi

    # Cleanup
    cleanup

    if [ $exit_code -eq 0 ]; then
        log_info "✓ All CRD deployment tests passed"
    else
        log_error "✗ Some tests failed"
    fi

    return $exit_code
}

main
