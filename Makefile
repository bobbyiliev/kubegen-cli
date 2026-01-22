.PHONY: build test lint fmt check clean install \
        test-e2e-local test-e2e-cleanup \
        help

# Default target
.DEFAULT_GOAL := help

# Build variables
CARGO := cargo
CLUSTER_NAME := kubegen-e2e

##@ Build

build: ## Build the project
	$(CARGO) build

build-release: ## Build release binary
	$(CARGO) build --release

install: ## Install kubegen locally
	$(CARGO) install --path .

##@ Testing

test: ## Run unit tests
	$(CARGO) test

test-verbose: ## Run unit tests with verbose output
	$(CARGO) test -- --nocapture

test-e2e-local: ## Run e2e tests locally (requires kind, kubectl, docker)
	@echo "Running e2e tests..."
	@./e2e-tests/run-tests.sh

test-e2e-keep: ## Run e2e tests and keep cluster for debugging
	@echo "Running e2e tests (keeping cluster)..."
	@SKIP_CLEANUP=true ./e2e-tests/run-tests.sh

test-e2e-cleanup: ## Delete the e2e test cluster
	@echo "Deleting e2e test cluster..."
	@kind delete cluster --name $(CLUSTER_NAME) 2>/dev/null || true

test-all: test test-e2e-local ## Run all tests (unit + e2e)

##@ Code Quality

lint: ## Run clippy linter
	$(CARGO) clippy --all-targets -- -D warnings

fmt: ## Format code
	$(CARGO) fmt

fmt-check: ## Check code formatting
	$(CARGO) fmt --all -- --check

check: fmt-check lint test ## Run all checks (format, lint, test)

##@ Documentation

doc: ## Build documentation
	$(CARGO) doc --no-deps

doc-open: ## Build and open documentation
	$(CARGO) doc --no-deps --open

##@ Cleanup

clean: ## Clean build artifacts
	$(CARGO) clean

clean-all: clean test-e2e-cleanup ## Clean everything including e2e cluster

##@ Help

help: ## Display this help
	@awk 'BEGIN {FS = ":.*##"; printf "\nUsage:\n  make \033[36m<target>\033[0m\n"} /^[a-zA-Z_-]+:.*?##/ { printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2 } /^##@/ { printf "\n\033[1m%s\033[0m\n", substr($$0, 5) } ' $(MAKEFILE_LIST)
