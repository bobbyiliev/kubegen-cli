//! kubegen - A CLI tool for scaffolding Kubernetes operators in Rust using kube-rs
//!
//! This library provides the core functionality for the kubegen CLI tool.
//! Think Kubebuilder, but for Rust.

pub mod cli;
pub mod error;

// Re-export commonly used types
pub use error::{KubegenError, Result};
