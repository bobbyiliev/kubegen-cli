//! CLI command definitions and parsing
//!
//! This module defines the command-line interface using clap.

use clap::{Parser, Subcommand};

/// Build version string with git hash and build date
fn version_string() -> &'static str {
    concat!(
        env!("CARGO_PKG_VERSION"),
        " (git: ",
        env!("KUBEGEN_GIT_HASH"),
        ", built: ",
        env!("KUBEGEN_BUILD_DATE"),
        ")"
    )
}

/// kubegen - Kubernetes operator scaffolding tool for Rust
#[derive(Parser, Debug)]
#[command(name = "kubegen")]
#[command(author, version = version_string(), about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

/// Available commands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Create a new Kubernetes operator project
    New(NewArgs),

    /// Add components to an existing operator project
    #[command(subcommand)]
    Add(AddCommands),
}

/// Arguments for the `new` command
#[derive(Parser, Debug)]
pub struct NewArgs {
    /// Name of the operator project
    #[arg(value_name = "NAME")]
    pub name: String,

    /// Domain for the operator (e.g., example.com)
    #[arg(short, long, default_value = "example.com")]
    pub domain: String,

    /// Skip interactive prompts
    #[arg(long)]
    pub non_interactive: bool,

    /// Show what would be created without actually creating files
    #[arg(long)]
    pub dry_run: bool,

    /// Overwrite existing files without prompting
    #[arg(short, long)]
    pub force: bool,
}

/// Subcommands for the `add` command
#[derive(Subcommand, Debug)]
pub enum AddCommands {
    /// Add a Custom Resource Definition (CRD) to the project
    Crd(CrdArgs),

    /// Add Prometheus metrics support
    Metrics(MetricsArgs),

    /// Add admission webhook support
    Webhook(WebhookArgs),
}

/// Arguments for the `add crd` command
#[derive(Parser, Debug)]
pub struct CrdArgs {
    /// Kind name for the CRD (PascalCase, e.g., MyResource)
    #[arg(value_name = "KIND")]
    pub kind: String,

    /// API group for the CRD (e.g., mygroup.example.com)
    #[arg(short, long)]
    pub group: Option<String>,

    /// API version for the CRD (e.g., v1alpha1)
    #[arg(long = "api-version", default_value = "v1alpha1")]
    pub api_version: String,

    /// Show what would be created without actually creating files
    #[arg(long)]
    pub dry_run: bool,

    /// Overwrite existing files without prompting
    #[arg(short, long)]
    pub force: bool,
}

/// Arguments for the `add metrics` command
#[derive(Parser, Debug)]
pub struct MetricsArgs {
    /// Port for the metrics endpoint
    #[arg(short, long, default_value = "8080")]
    pub port: u16,

    /// Show what would be created without actually creating files
    #[arg(long)]
    pub dry_run: bool,

    /// Overwrite existing files without prompting
    #[arg(short, long)]
    pub force: bool,
}

/// Arguments for the `add webhook` command
#[derive(Parser, Debug)]
pub struct WebhookArgs {
    /// Kind name for the webhook (must match an existing CRD)
    #[arg(value_name = "KIND")]
    pub kind: String,

    /// Create a validating webhook
    #[arg(long)]
    pub validating: bool,

    /// Create a mutating webhook
    #[arg(long)]
    pub mutating: bool,

    /// Show what would be created without actually creating files
    #[arg(long)]
    pub dry_run: bool,

    /// Overwrite existing files without prompting
    #[arg(short, long)]
    pub force: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn test_cli_debug_assert() {
        Cli::command().debug_assert();
    }

    #[test]
    fn test_parse_new_command() {
        let cli = Cli::parse_from(["kubegen", "new", "my-operator"]);
        assert!(!cli.verbose);
        if let Commands::New(args) = cli.command {
            assert_eq!(args.name, "my-operator");
            assert_eq!(args.domain, "example.com");
            assert!(!args.dry_run);
        } else {
            panic!("Expected New command");
        }
    }

    #[test]
    fn test_parse_new_with_options() {
        let cli = Cli::parse_from([
            "kubegen",
            "--verbose",
            "new",
            "my-operator",
            "--domain",
            "mycompany.io",
            "--dry-run",
        ]);
        assert!(cli.verbose);
        if let Commands::New(args) = cli.command {
            assert_eq!(args.name, "my-operator");
            assert_eq!(args.domain, "mycompany.io");
            assert!(args.dry_run);
        } else {
            panic!("Expected New command");
        }
    }

    #[test]
    fn test_parse_add_crd_command() {
        let cli = Cli::parse_from(["kubegen", "add", "crd", "MyResource"]);
        if let Commands::Add(AddCommands::Crd(args)) = cli.command {
            assert_eq!(args.kind, "MyResource");
            assert_eq!(args.api_version, "v1alpha1");
            assert!(args.group.is_none());
        } else {
            panic!("Expected Add Crd command");
        }
    }

    #[test]
    fn test_parse_add_crd_with_options() {
        let cli = Cli::parse_from([
            "kubegen",
            "add",
            "crd",
            "MyResource",
            "--group",
            "apps.example.com",
            "--api-version",
            "v1beta1",
        ]);
        if let Commands::Add(AddCommands::Crd(args)) = cli.command {
            assert_eq!(args.kind, "MyResource");
            assert_eq!(args.group, Some("apps.example.com".to_string()));
            assert_eq!(args.api_version, "v1beta1");
        } else {
            panic!("Expected Add Crd command");
        }
    }

    #[test]
    fn test_parse_add_metrics_command() {
        let cli = Cli::parse_from(["kubegen", "add", "metrics"]);
        if let Commands::Add(AddCommands::Metrics(args)) = cli.command {
            assert_eq!(args.port, 8080);
            assert!(!args.dry_run);
        } else {
            panic!("Expected Add Metrics command");
        }
    }

    #[test]
    fn test_parse_add_webhook_command() {
        let cli = Cli::parse_from([
            "kubegen",
            "add",
            "webhook",
            "MyResource",
            "--validating",
            "--mutating",
        ]);
        if let Commands::Add(AddCommands::Webhook(args)) = cli.command {
            assert_eq!(args.kind, "MyResource");
            assert!(args.validating);
            assert!(args.mutating);
        } else {
            panic!("Expected Add Webhook command");
        }
    }

    #[test]
    fn test_parse_new_with_force() {
        let cli = Cli::parse_from(["kubegen", "new", "my-operator", "--force"]);
        if let Commands::New(args) = cli.command {
            assert!(args.force);
            assert!(!args.dry_run);
        } else {
            panic!("Expected New command");
        }
    }

    #[test]
    fn test_parse_new_with_force_short() {
        let cli = Cli::parse_from(["kubegen", "new", "my-operator", "-f"]);
        if let Commands::New(args) = cli.command {
            assert!(args.force);
        } else {
            panic!("Expected New command");
        }
    }

    #[test]
    fn test_parse_add_crd_with_force() {
        let cli = Cli::parse_from(["kubegen", "add", "crd", "MyResource", "--force"]);
        if let Commands::Add(AddCommands::Crd(args)) = cli.command {
            assert!(args.force);
        } else {
            panic!("Expected Add Crd command");
        }
    }
}
