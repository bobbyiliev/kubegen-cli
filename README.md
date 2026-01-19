# kubegen

kubegen is a Rust based CLI for scaffolding Kubernetes operators using kube-rs.

Think Kubebuilder, but for Rust.

## Goals

- Make writing Kubernetes operators in Rust approachable
- Generate production ready scaffolding
- Follow kube-rs best practices
- Keep generated code explicit and owned by the user

## What kubegen does

- Scaffold a new Rust operator project
- Generate CRDs and Rust types
- Create reconciliation boilerplate
- Optionally add metrics and webhooks

## Non goals

- Providing a runtime framework
- Abstracting Kubernetes concepts
- Managing operator lifecycle after generation

## Project status

Early development. APIs and templates may change.

## Usage

```bash
kubegen new my-operator
kubegen add crd MyResource
kubegen add metrics
kubegen add webhook MyResource
```

## Contributing

Focused and atomic PRs only.

Every feature should include:

* Implementation
* Tests
* Documentation if user facing

See CONTRIBUTING.md for details.
