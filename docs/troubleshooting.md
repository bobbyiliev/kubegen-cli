# Troubleshooting Guide

Common issues and solutions when using kubegen.

## Installation Issues

### `cargo install` fails with compilation errors

**Symptom:** Build errors when running `cargo install kubegen`

**Solutions:**
1. Ensure you have a recent Rust version (1.70+):
   ```bash
   rustup update stable
   ```

2. Clear the cargo cache and retry:
   ```bash
   cargo clean
   cargo install kubegen
   ```

### Binary not found after installation

**Symptom:** `kubegen: command not found`

**Solutions:**
1. Ensure `~/.cargo/bin` is in your PATH:
   ```bash
   export PATH="$HOME/.cargo/bin:$PATH"
   ```

2. Add to your shell profile (`~/.bashrc`, `~/.zshrc`, etc.):
   ```bash
   echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.zshrc
   source ~/.zshrc
   ```

## Project Creation Issues

### "Invalid project name" error

**Symptom:** `Invalid project name 'My-Operator': must be lowercase`

**Cause:** Project names must follow DNS-1123 subdomain rules.

**Solution:** Use lowercase names with hyphens:
```bash
# Wrong
kubegen new My-Operator
kubegen new my_operator

# Correct
kubegen new my-operator
```

**Rules:**
- Lowercase only
- Alphanumeric and hyphens only
- Must start and end with alphanumeric
- Maximum 63 characters

### "Directory already exists" error

**Symptom:** `Directory already exists: my-operator`

**Solutions:**

1. Use a different name:
   ```bash
   kubegen new my-operator-v2
   ```

2. Force overwrite (use with caution):
   ```bash
   kubegen new my-operator --force
   ```

3. Remove existing directory:
   ```bash
   rm -rf my-operator
   kubegen new my-operator
   ```

## CRD Issues

### "Invalid CRD name" - kind must be PascalCase

**Symptom:** `Invalid CRD name: kind must start with an uppercase letter (PascalCase)`

**Solution:** Use PascalCase for kind names:
```bash
# Wrong
kubegen add crd myresource
kubegen add crd my-resource
kubegen add crd my_resource

# Correct
kubegen add crd MyResource
kubegen add crd DatabaseCluster
```

### "Invalid CRD name" - version format

**Symptom:** `version must match pattern v[0-9]+(alpha|beta)?[0-9]*`

**Solution:** Use valid Kubernetes API versions:
```bash
# Wrong
kubegen add crd MyResource --api-version 1.0
kubegen add crd MyResource --api-version version1

# Correct
kubegen add crd MyResource --api-version v1
kubegen add crd MyResource --api-version v1alpha1
kubegen add crd MyResource --api-version v1beta1
```

### "Invalid CRD name" - group format

**Symptom:** `group must be lowercase`

**Solution:** Use lowercase DNS subdomain format:
```bash
# Wrong
kubegen add crd MyResource --group Example.com
kubegen add crd MyResource --group my_group.example.com

# Correct
kubegen add crd MyResource --group example.com
kubegen add crd MyResource --group apps.example.com
```

## Build Issues

### Generated project fails to compile

**Symptom:** `cargo build` fails in generated project

**Solutions:**

1. Ensure all dependencies are available:
   ```bash
   cargo update
   cargo build
   ```

2. Check Rust version compatibility:
   ```bash
   rustc --version
   # Should be 1.70 or higher
   ```

3. Verify the Cargo.toml has correct dependencies:
   ```bash
   cat Cargo.toml
   ```

### Missing kube-rs features

**Symptom:** Compilation errors related to kube-rs features

**Solution:** Ensure the generated Cargo.toml includes required features:
```toml
[dependencies]
kube = { version = "0.88", features = ["runtime", "derive"] }
```

## Runtime Issues

### Operator can't connect to cluster

**Symptom:** `Error: Failed to infer config` or similar

**Solutions:**

1. Ensure kubeconfig is available:
   ```bash
   kubectl cluster-info
   ```

2. Check KUBECONFIG environment variable:
   ```bash
   echo $KUBECONFIG
   # Should point to valid config or be unset (uses ~/.kube/config)
   ```

3. For in-cluster deployment, ensure ServiceAccount has proper RBAC.

### CRD not found in cluster

**Symptom:** `the server doesn't have a resource type "myresources"`

**Solution:** Apply the CRD manifest first:
```bash
kubectl apply -f manifests/my_resource-crd.yaml
kubectl get crd
```

### Webhook certificate errors

**Symptom:** `x509: certificate signed by unknown authority`

**Solutions:**

1. Ensure cert-manager is installed:
   ```bash
   kubectl get pods -n cert-manager
   ```

2. Wait for certificate to be ready:
   ```bash
   kubectl get certificate -n <namespace>
   ```

3. Check certificate secret exists:
   ```bash
   kubectl get secret <service-name>-tls
   ```

## Metrics Issues

### Metrics endpoint not accessible

**Symptom:** `curl http://localhost:8080/metrics` fails

**Solutions:**

1. Verify the operator is running:
   ```bash
   cargo run
   # In another terminal:
   curl http://localhost:8080/metrics
   ```

2. Check the port configuration:
   ```bash
   kubegen add metrics --port 9090
   # Then use port 9090
   ```

3. For in-cluster, ensure Service is created and port-forward:
   ```bash
   kubectl port-forward svc/<operator>-metrics 8080:8080
   ```

### ServiceMonitor not being scraped

**Symptom:** Metrics not appearing in Prometheus

**Solutions:**

1. Verify ServiceMonitor is applied:
   ```bash
   kubectl get servicemonitor
   ```

2. Check Prometheus Operator is watching the namespace:
   ```bash
   kubectl get prometheus -o yaml | grep serviceMonitorSelector
   ```

3. Verify labels match between Service and ServiceMonitor.

## Development Issues

### Template changes not reflected

**Symptom:** Modified templates don't appear in generated code

**Cause:** Templates are embedded at compile time.

**Solution:** Rebuild kubegen after template changes:
```bash
cargo build --release
```

### Tests failing

**Symptom:** `cargo test` fails

**Solutions:**

1. Run with verbose output:
   ```bash
   cargo test -- --nocapture
   ```

2. Run specific failing test:
   ```bash
   cargo test test_name -- --nocapture
   ```

3. Check for environment-specific issues (file permissions, paths).

## Getting Help

If you encounter an issue not covered here:

1. **Check existing issues:** [GitHub Issues](https://github.com/bobbyiliev/kubegen-cli/issues)

2. **Enable verbose output:**
   ```bash
   kubegen --verbose new my-operator
   ```

3. **Open a new issue** with:
   - kubegen version (`kubegen --version`)
   - Rust version (`rustc --version`)
   - Operating system
   - Full command used
   - Complete error message

## See Also

- [User Guide](user-guide.md) - Command reference
- [Architecture](architecture.md) - Internal design
