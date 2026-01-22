#!/bin/bash
# Basic cluster health test
# Verifies the kind cluster is running and healthy

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../lib/cluster.sh"
source "$SCRIPT_DIR/../lib/kubectl.sh"

test_cluster_health() {
    log_info "Testing cluster health..."

    # Check nodes are ready
    local ready_nodes
    ready_nodes=$(kubectl get nodes --no-headers | grep -c " Ready " || echo "0")

    if [ "$ready_nodes" -lt 1 ]; then
        log_error "No ready nodes found"
        return 1
    fi

    log_info "✓ Found ${ready_nodes} ready node(s)"

    # Check core pods are running
    log_info "Checking kube-system pods..."
    kubectl get pods -n kube-system

    # Verify coredns is running
    if kubectl get deployment coredns -n kube-system &> /dev/null; then
        kubectl_wait_deployment "coredns" "kube-system" "60s"
        log_info "✓ CoreDNS is running"
    fi

    log_info "✓ Cluster health check passed"
    return 0
}

test_cluster_health
