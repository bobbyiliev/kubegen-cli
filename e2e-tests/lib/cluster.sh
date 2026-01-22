#!/bin/bash
# Cluster management helpers for e2e tests

set -euo pipefail

CLUSTER_NAME="${CLUSTER_NAME:-kubegen-e2e}"
KIND_CONFIG="${KIND_CONFIG:-}"
SKIP_CLUSTER_CREATE="${SKIP_CLUSTER_CREATE:-false}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $*"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $*"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $*"
}

# Check if kind is installed
check_kind() {
    if ! command -v kind &> /dev/null; then
        log_error "kind is not installed. Install from https://kind.sigs.k8s.io/"
        exit 1
    fi
}

# Check if kubectl is installed
check_kubectl() {
    if ! command -v kubectl &> /dev/null; then
        log_error "kubectl is not installed. Install from https://kubernetes.io/docs/tasks/tools/"
        exit 1
    fi
}

# Check if docker is running
check_docker() {
    if ! docker info &> /dev/null; then
        log_error "Docker is not running. Please start Docker."
        exit 1
    fi
}

# Check all prerequisites
check_prerequisites() {
    check_kind
    check_kubectl
    check_docker
}

# Check if cluster exists
cluster_exists() {
    kind get clusters 2>/dev/null | grep -q "^${CLUSTER_NAME}$"
}

# Create kind cluster
create_cluster() {
    log_info "Creating kind cluster: ${CLUSTER_NAME}"

    if [ -n "$KIND_CONFIG" ] && [ -f "$KIND_CONFIG" ]; then
        kind create cluster --name "$CLUSTER_NAME" --config "$KIND_CONFIG"
    else
        kind create cluster --name "$CLUSTER_NAME"
    fi

    log_info "Cluster created successfully"
}

# Delete kind cluster
delete_cluster() {
    if cluster_exists; then
        log_info "Deleting kind cluster: ${CLUSTER_NAME}"
        kind delete cluster --name "$CLUSTER_NAME"
        log_info "Cluster deleted"
    else
        log_warn "Cluster ${CLUSTER_NAME} does not exist"
    fi
}

# Ensure cluster is running, create if needed
ensure_cluster() {
    check_prerequisites

    if [ "$SKIP_CLUSTER_CREATE" = "true" ]; then
        log_info "Using pre-existing cluster (SKIP_CLUSTER_CREATE=true)"
    elif cluster_exists; then
        log_info "Using existing cluster: ${CLUSTER_NAME}"
    else
        create_cluster
    fi

    # Set kubeconfig
    export KUBECONFIG="${KUBECONFIG:-$(kind get kubeconfig-path --name="$CLUSTER_NAME" 2>/dev/null || echo "$HOME/.kube/config")}"

    # Wait for cluster to be ready
    wait_for_cluster_ready
}

# Wait for cluster nodes to be ready
wait_for_cluster_ready() {
    log_info "Waiting for cluster to be ready..."

    local retries=30
    local wait_seconds=2

    for ((i=1; i<=retries; i++)); do
        if kubectl get nodes &> /dev/null; then
            local ready_nodes
            ready_nodes=$(kubectl get nodes --no-headers 2>/dev/null | grep -c " Ready " || true)
            ready_nodes="${ready_nodes:-0}"
            ready_nodes=$(echo "$ready_nodes" | tr -d '[:space:]')
            if [[ "$ready_nodes" =~ ^[0-9]+$ ]] && [ "$ready_nodes" -gt 0 ]; then
                log_info "Cluster is ready ($ready_nodes node(s))"
                return 0
            fi
        fi

        if [ "$i" -lt "$retries" ]; then
            log_info "Waiting for nodes to be ready... ($i/$retries)"
            sleep "$wait_seconds"
        fi
    done

    log_error "Cluster did not become ready in time"
    return 1
}

# Get cluster info
cluster_info() {
    log_info "Cluster: ${CLUSTER_NAME}"
    kubectl cluster-info
    echo ""
    kubectl get nodes
}

# Load a local Docker image into the kind cluster
load_image() {
    local image="$1"
    log_info "Loading image into cluster: ${image}"
    kind load docker-image "$image" --name "$CLUSTER_NAME"
}

# Build and load image
build_and_load_image() {
    local dockerfile="$1"
    local image="$2"
    local context="${3:-.}"

    log_info "Building image: ${image}"
    docker build -t "$image" -f "$dockerfile" "$context"
    load_image "$image"
}
