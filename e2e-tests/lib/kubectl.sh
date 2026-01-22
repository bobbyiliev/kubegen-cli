#!/bin/bash
# kubectl helper functions for e2e tests

set -euo pipefail

# Source cluster helpers for logging
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/cluster.sh"

# Default timeout for wait operations
DEFAULT_TIMEOUT="${DEFAULT_TIMEOUT:-120s}"

# Apply a manifest file
kubectl_apply() {
    local file="$1"
    log_info "Applying: ${file}"
    kubectl apply -f "$file"
}

# Delete a manifest file
kubectl_delete() {
    local file="$1"
    log_info "Deleting: ${file}"
    kubectl delete -f "$file" --ignore-not-found=true
}

# Wait for a resource to exist
kubectl_wait_for() {
    local resource_type="$1"
    local resource_name="$2"
    local timeout="${3:-$DEFAULT_TIMEOUT}"

    log_info "Waiting for ${resource_type}/${resource_name} to exist..."

    local retries=60
    local wait_seconds=2

    for ((i=1; i<=retries; i++)); do
        if kubectl get "$resource_type" "$resource_name" &> /dev/null; then
            log_info "${resource_type}/${resource_name} exists"
            return 0
        fi

        if [ "$i" -lt "$retries" ]; then
            sleep "$wait_seconds"
        fi
    done

    log_error "${resource_type}/${resource_name} did not appear in time"
    return 1
}

# Wait for a resource condition
kubectl_wait_condition() {
    local condition="$1"
    local resource_type="$2"
    local resource_name="$3"
    local timeout="${4:-$DEFAULT_TIMEOUT}"

    log_info "Waiting for ${resource_type}/${resource_name} condition=${condition}..."
    kubectl wait --for="condition=${condition}" "$resource_type" "$resource_name" --timeout="$timeout"
}

# Wait for deployment to be ready
kubectl_wait_deployment() {
    local name="$1"
    local namespace="${2:-default}"
    local timeout="${3:-$DEFAULT_TIMEOUT}"

    log_info "Waiting for deployment ${name} in ${namespace} to be ready..."
    kubectl wait --for=condition=available deployment/"$name" -n "$namespace" --timeout="$timeout"
}

# Wait for pod to be ready
kubectl_wait_pod() {
    local selector="$1"
    local namespace="${2:-default}"
    local timeout="${3:-$DEFAULT_TIMEOUT}"

    log_info "Waiting for pod with selector ${selector} in ${namespace} to be ready..."
    kubectl wait --for=condition=ready pod -l "$selector" -n "$namespace" --timeout="$timeout"
}

# Get resource as JSON
kubectl_get_json() {
    local resource_type="$1"
    local resource_name="$2"
    local namespace="${3:-}"

    if [ -n "$namespace" ]; then
        kubectl get "$resource_type" "$resource_name" -n "$namespace" -o json
    else
        kubectl get "$resource_type" "$resource_name" -o json
    fi
}

# Check if a resource exists
resource_exists() {
    local resource_type="$1"
    local resource_name="$2"
    local namespace="${3:-}"

    if [ -n "$namespace" ]; then
        kubectl get "$resource_type" "$resource_name" -n "$namespace" &> /dev/null
    else
        kubectl get "$resource_type" "$resource_name" &> /dev/null
    fi
}

# Assert resource exists (fails test if not)
assert_resource_exists() {
    local resource_type="$1"
    local resource_name="$2"
    local namespace="${3:-}"

    if resource_exists "$resource_type" "$resource_name" "$namespace"; then
        log_info "✓ ${resource_type}/${resource_name} exists"
        return 0
    else
        log_error "✗ ${resource_type}/${resource_name} does not exist"
        return 1
    fi
}

# Assert resource does not exist
assert_resource_not_exists() {
    local resource_type="$1"
    local resource_name="$2"
    local namespace="${3:-}"

    if ! resource_exists "$resource_type" "$resource_name" "$namespace"; then
        log_info "✓ ${resource_type}/${resource_name} does not exist (as expected)"
        return 0
    else
        log_error "✗ ${resource_type}/${resource_name} exists but should not"
        return 1
    fi
}

# Assert JSON field equals value
assert_json_field() {
    local json="$1"
    local field="$2"
    local expected="$3"

    local actual
    actual=$(echo "$json" | jq -r "$field")

    if [ "$actual" = "$expected" ]; then
        log_info "✓ ${field} = ${expected}"
        return 0
    else
        log_error "✗ ${field}: expected '${expected}', got '${actual}'"
        return 1
    fi
}

# Create namespace if it doesn't exist
ensure_namespace() {
    local namespace="$1"

    if ! kubectl get namespace "$namespace" &> /dev/null; then
        log_info "Creating namespace: ${namespace}"
        kubectl create namespace "$namespace"
    fi
}

# Delete namespace
delete_namespace() {
    local namespace="$1"

    if kubectl get namespace "$namespace" &> /dev/null; then
        log_info "Deleting namespace: ${namespace}"
        kubectl delete namespace "$namespace" --timeout=60s || true
    fi
}

# Port forward to a service
port_forward() {
    local service="$1"
    local local_port="$2"
    local remote_port="$3"
    local namespace="${4:-default}"

    log_info "Port forwarding ${service}:${remote_port} to localhost:${local_port}"
    kubectl port-forward "svc/${service}" "${local_port}:${remote_port}" -n "$namespace" &
    local pf_pid=$!

    # Give it a moment to establish
    sleep 2

    # Return the PID so caller can kill it later
    echo "$pf_pid"
}

# Check if endpoint returns expected status code
check_endpoint() {
    local url="$1"
    local expected_status="${2:-200}"

    local status
    status=$(curl -s -o /dev/null -w "%{http_code}" "$url" || echo "000")

    if [ "$status" = "$expected_status" ]; then
        log_info "✓ ${url} returned ${status}"
        return 0
    else
        log_error "✗ ${url}: expected ${expected_status}, got ${status}"
        return 1
    fi
}

# Get logs from a pod
get_pod_logs() {
    local selector="$1"
    local namespace="${2:-default}"
    local tail="${3:-100}"

    kubectl logs -l "$selector" -n "$namespace" --tail="$tail"
}
