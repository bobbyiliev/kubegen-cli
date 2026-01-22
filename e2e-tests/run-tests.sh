#!/bin/bash
# Main e2e test runner

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/lib/cluster.sh"

# Test results tracking
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0
FAILED_TESTS=()

# Configuration
SKIP_CLEANUP="${SKIP_CLEANUP:-false}"
TEST_FILTER="${1:-}"

# Run a single test
run_test() {
    local test_script="$1"
    local test_name
    test_name=$(basename "$test_script" .sh)

    # Apply filter if provided
    if [ -n "$TEST_FILTER" ] && [[ "$test_name" != *"$TEST_FILTER"* ]]; then
        return 0
    fi

    TESTS_RUN=$((TESTS_RUN + 1))
    log_info "Running test: ${test_name}"
    echo "----------------------------------------"

    if bash "$test_script"; then
        TESTS_PASSED=$((TESTS_PASSED + 1))
        log_info "✓ Test passed: ${test_name}"
    else
        TESTS_FAILED=$((TESTS_FAILED + 1))
        FAILED_TESTS+=("$test_name")
        log_error "✗ Test failed: ${test_name}"
    fi

    echo "----------------------------------------"
    echo ""
}

# Print summary
print_summary() {
    echo ""
    echo "========================================"
    echo "Test Summary"
    echo "========================================"
    echo "Total:  ${TESTS_RUN}"
    echo -e "Passed: ${GREEN}${TESTS_PASSED}${NC}"
    echo -e "Failed: ${RED}${TESTS_FAILED}${NC}"

    if [ ${#FAILED_TESTS[@]} -gt 0 ]; then
        echo ""
        echo "Failed tests:"
        for test in "${FAILED_TESTS[@]}"; do
            echo "  - $test"
        done
    fi

    echo "========================================"
}

# Cleanup function
cleanup() {
    if [ "$SKIP_CLEANUP" = "true" ]; then
        log_warn "Skipping cleanup (SKIP_CLEANUP=true)"
        log_info "Cluster ${CLUSTER_NAME} is still running"
        return 0
    fi

    log_info "Cleaning up..."
    delete_cluster || true
}

# Main
main() {
    log_info "kubegen e2e test runner"
    echo ""

    # Check prerequisites
    check_prerequisites

    # Setup trap for cleanup
    trap cleanup EXIT

    # Ensure cluster is running
    ensure_cluster
    echo ""

    # Find and run all tests
    local test_dir="$SCRIPT_DIR/tests"
    if [ -d "$test_dir" ]; then
        for test_script in "$test_dir"/*.sh; do
            if [ -f "$test_script" ]; then
                run_test "$test_script"
            fi
        done
    fi

    # Check if any tests were found
    if [ "$TESTS_RUN" -eq 0 ]; then
        log_warn "No tests found in ${test_dir}"
        log_info "Create test scripts in e2e-tests/tests/ to run e2e tests"
    fi

    # Print summary
    print_summary

    # Exit with error if any tests failed
    if [ "$TESTS_FAILED" -gt 0 ]; then
        exit 1
    fi
}

main "$@"
