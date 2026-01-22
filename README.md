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

## Installation

### From crates.io (Recommended)

```bash
cargo install kubegen
```

### Homebrew (macOS/Linux)

```bash
brew tap bobbyiliev/kubegen
brew install kubegen
```

### Download Binary

Download the latest release for your platform from the [releases page](https://github.com/bobbyiliev/kubegen-cli/releases).

**Linux (x86_64)**
```bash
curl -LO https://github.com/bobbyiliev/kubegen-cli/releases/latest/download/kubegen-x86_64-unknown-linux-gnu.tar.gz
tar xzf kubegen-x86_64-unknown-linux-gnu.tar.gz
sudo mv kubegen /usr/local/bin/
```

**macOS (Apple Silicon)**
```bash
curl -LO https://github.com/bobbyiliev/kubegen-cli/releases/latest/download/kubegen-aarch64-apple-darwin.tar.gz
tar xzf kubegen-aarch64-apple-darwin.tar.gz
sudo mv kubegen /usr/local/bin/
```

**macOS (Intel)**
```bash
curl -LO https://github.com/bobbyiliev/kubegen-cli/releases/latest/download/kubegen-x86_64-apple-darwin.tar.gz
tar xzf kubegen-x86_64-apple-darwin.tar.gz
sudo mv kubegen /usr/local/bin/
```

**Windows**

Download `kubegen-x86_64-pc-windows-msvc.zip` from the [releases page](https://github.com/bobbyiliev/kubegen-cli/releases) and add to your PATH.

### Build from Source

```bash
git clone https://github.com/bobbyiliev/kubegen-cli.git
cd kubegen-cli
cargo build --release
# Binary will be at target/release/kubegen
```

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
