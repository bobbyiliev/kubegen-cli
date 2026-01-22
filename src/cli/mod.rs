//! CLI command definitions and parsing
//!
//! This module defines the command-line interface using clap.

use std::path::PathBuf;

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

    /// Custom template directory to override embedded templates
    #[arg(long, global = true, value_name = "DIR")]
    pub template_dir: Option<PathBuf>,

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

    /// API group for the webhook (e.g., mygroup.example.com)
    #[arg(short, long)]
    pub group: Option<String>,

    /// Kubernetes service name for the webhook (defaults to `<project>-webhook`)
    #[arg(long)]
    pub service_name: Option<String>,

    /// Namespace where the webhook service runs (defaults to system namespace)
    #[arg(long, default_value = "default")]
    pub namespace: String,

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

    // Flag combination tests
    #[test]
    fn test_new_all_flags_combined() {
        let cli = Cli::parse_from([
            "kubegen",
            "--verbose",
            "new",
            "test-operator",
            "--domain",
            "test.io",
            "--non-interactive",
            "--dry-run",
            "--force",
        ]);
        assert!(cli.verbose);
        if let Commands::New(args) = cli.command {
            assert_eq!(args.name, "test-operator");
            assert_eq!(args.domain, "test.io");
            assert!(args.non_interactive);
            assert!(args.dry_run);
            assert!(args.force);
        } else {
            panic!("Expected New command");
        }
    }

    #[test]
    fn test_add_crd_all_flags_combined() {
        let cli = Cli::parse_from([
            "kubegen",
            "add",
            "crd",
            "TestResource",
            "--group",
            "test.example.com",
            "--api-version",
            "v2beta1",
            "--dry-run",
            "--force",
        ]);
        if let Commands::Add(AddCommands::Crd(args)) = cli.command {
            assert_eq!(args.kind, "TestResource");
            assert_eq!(args.group, Some("test.example.com".to_string()));
            assert_eq!(args.api_version, "v2beta1");
            assert!(args.dry_run);
            assert!(args.force);
        } else {
            panic!("Expected Add Crd command");
        }
    }

    #[test]
    fn test_add_metrics_all_flags() {
        let cli = Cli::parse_from([
            "kubegen",
            "add",
            "metrics",
            "--port",
            "9090",
            "--dry-run",
            "--force",
        ]);
        if let Commands::Add(AddCommands::Metrics(args)) = cli.command {
            assert_eq!(args.port, 9090);
            assert!(args.dry_run);
            assert!(args.force);
        } else {
            panic!("Expected Add Metrics command");
        }
    }

    #[test]
    fn test_add_webhook_all_flags() {
        let cli = Cli::parse_from([
            "kubegen",
            "add",
            "webhook",
            "TestResource",
            "--validating",
            "--mutating",
            "--group",
            "mygroup.example.com",
            "--service-name",
            "my-webhook",
            "--namespace",
            "my-namespace",
            "--dry-run",
            "--force",
        ]);
        if let Commands::Add(AddCommands::Webhook(args)) = cli.command {
            assert_eq!(args.kind, "TestResource");
            assert!(args.validating);
            assert!(args.mutating);
            assert_eq!(args.group, Some("mygroup.example.com".to_string()));
            assert_eq!(args.service_name, Some("my-webhook".to_string()));
            assert_eq!(args.namespace, "my-namespace");
            assert!(args.dry_run);
            assert!(args.force);
        } else {
            panic!("Expected Add Webhook command");
        }
    }

    // Short flag tests
    #[test]
    fn test_verbose_short_flag() {
        let cli = Cli::parse_from(["kubegen", "-v", "new", "test"]);
        assert!(cli.verbose);
    }

    #[test]
    fn test_domain_short_flag() {
        let cli = Cli::parse_from(["kubegen", "new", "test", "-d", "short.io"]);
        if let Commands::New(args) = cli.command {
            assert_eq!(args.domain, "short.io");
        } else {
            panic!("Expected New command");
        }
    }

    #[test]
    fn test_group_short_flag() {
        let cli = Cli::parse_from(["kubegen", "add", "crd", "Test", "-g", "short.io"]);
        if let Commands::Add(AddCommands::Crd(args)) = cli.command {
            assert_eq!(args.group, Some("short.io".to_string()));
        } else {
            panic!("Expected Add Crd command");
        }
    }

    #[test]
    fn test_port_short_flag() {
        let cli = Cli::parse_from(["kubegen", "add", "metrics", "-p", "3000"]);
        if let Commands::Add(AddCommands::Metrics(args)) = cli.command {
            assert_eq!(args.port, 3000);
        } else {
            panic!("Expected Add Metrics command");
        }
    }

    // Error case tests
    #[test]
    fn test_missing_required_name() {
        let result = Cli::try_parse_from(["kubegen", "new"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_required_kind() {
        let result = Cli::try_parse_from(["kubegen", "add", "crd"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_required_webhook_kind() {
        let result = Cli::try_parse_from(["kubegen", "add", "webhook"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_subcommand() {
        let result = Cli::try_parse_from(["kubegen", "invalid"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_add_subcommand() {
        let result = Cli::try_parse_from(["kubegen", "add", "invalid"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_port_value() {
        let result = Cli::try_parse_from(["kubegen", "add", "metrics", "--port", "not-a-number"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_unknown_flag() {
        let result = Cli::try_parse_from(["kubegen", "new", "test", "--unknown-flag"]);
        assert!(result.is_err());
    }

    // Global verbose flag tests
    #[test]
    fn test_verbose_before_subcommand() {
        let cli = Cli::parse_from(["kubegen", "--verbose", "new", "test"]);
        assert!(cli.verbose);
    }

    #[test]
    fn test_verbose_after_subcommand() {
        let cli = Cli::parse_from(["kubegen", "new", "--verbose", "test"]);
        assert!(cli.verbose);
    }

    #[test]
    fn test_verbose_at_end() {
        let cli = Cli::parse_from(["kubegen", "new", "test", "--verbose"]);
        assert!(cli.verbose);
    }

    // Default value tests
    #[test]
    fn test_default_domain() {
        let cli = Cli::parse_from(["kubegen", "new", "test"]);
        if let Commands::New(args) = cli.command {
            assert_eq!(args.domain, "example.com");
        } else {
            panic!("Expected New command");
        }
    }

    #[test]
    fn test_default_api_version() {
        let cli = Cli::parse_from(["kubegen", "add", "crd", "Test"]);
        if let Commands::Add(AddCommands::Crd(args)) = cli.command {
            assert_eq!(args.api_version, "v1alpha1");
        } else {
            panic!("Expected Add Crd command");
        }
    }

    #[test]
    fn test_default_port() {
        let cli = Cli::parse_from(["kubegen", "add", "metrics"]);
        if let Commands::Add(AddCommands::Metrics(args)) = cli.command {
            assert_eq!(args.port, 8080);
        } else {
            panic!("Expected Add Metrics command");
        }
    }

    // Boolean flag defaults
    #[test]
    fn test_boolean_flags_default_false() {
        let cli = Cli::parse_from(["kubegen", "new", "test"]);
        assert!(!cli.verbose);
        if let Commands::New(args) = cli.command {
            assert!(!args.non_interactive);
            assert!(!args.dry_run);
            assert!(!args.force);
        } else {
            panic!("Expected New command");
        }
    }

    #[test]
    fn test_webhook_boolean_flags_default_false() {
        let cli = Cli::parse_from(["kubegen", "add", "webhook", "Test"]);
        if let Commands::Add(AddCommands::Webhook(args)) = cli.command {
            assert!(!args.validating);
            assert!(!args.mutating);
            assert!(args.group.is_none());
            assert!(args.service_name.is_none());
            assert_eq!(args.namespace, "default");
            assert!(!args.dry_run);
            assert!(!args.force);
        } else {
            panic!("Expected Add Webhook command");
        }
    }

    // Edge cases
    #[test]
    fn test_name_with_hyphens() {
        let cli = Cli::parse_from(["kubegen", "new", "my-cool-operator-name"]);
        if let Commands::New(args) = cli.command {
            assert_eq!(args.name, "my-cool-operator-name");
        } else {
            panic!("Expected New command");
        }
    }

    #[test]
    fn test_kind_pascal_case() {
        let cli = Cli::parse_from(["kubegen", "add", "crd", "MyCustomResourceDefinition"]);
        if let Commands::Add(AddCommands::Crd(args)) = cli.command {
            assert_eq!(args.kind, "MyCustomResourceDefinition");
        } else {
            panic!("Expected Add Crd command");
        }
    }

    #[test]
    fn test_single_char_name() {
        let cli = Cli::parse_from(["kubegen", "new", "a"]);
        if let Commands::New(args) = cli.command {
            assert_eq!(args.name, "a");
        } else {
            panic!("Expected New command");
        }
    }

    #[test]
    fn test_empty_string_arg_error() {
        // clap requires at least something for positional args
        let result = Cli::try_parse_from(["kubegen", "new", ""]);
        // Empty string is technically valid for clap, app-level validation handles this
        assert!(result.is_ok());
    }

    // Template directory tests
    #[test]
    fn test_parse_template_dir() {
        let cli = Cli::parse_from([
            "kubegen",
            "--template-dir",
            "/path/to/templates",
            "new",
            "test",
        ]);
        assert_eq!(cli.template_dir, Some(PathBuf::from("/path/to/templates")));
    }

    #[test]
    fn test_parse_template_dir_with_add_command() {
        let cli = Cli::parse_from([
            "kubegen",
            "--template-dir",
            "./my-templates",
            "add",
            "crd",
            "MyResource",
        ]);
        assert_eq!(cli.template_dir, Some(PathBuf::from("./my-templates")));
        if let Commands::Add(AddCommands::Crd(args)) = cli.command {
            assert_eq!(args.kind, "MyResource");
        } else {
            panic!("Expected Add Crd command");
        }
    }

    #[test]
    fn test_parse_no_template_dir() {
        let cli = Cli::parse_from(["kubegen", "new", "test"]);
        assert!(cli.template_dir.is_none());
    }

    #[test]
    fn test_template_dir_global_flag() {
        // Template dir can appear before or after subcommand
        let cli1 = Cli::parse_from(["kubegen", "--template-dir", "/path", "new", "test"]);
        let cli2 = Cli::parse_from(["kubegen", "new", "--template-dir", "/path", "test"]);
        let cli3 = Cli::parse_from(["kubegen", "new", "test", "--template-dir", "/path"]);

        assert_eq!(cli1.template_dir, Some(PathBuf::from("/path")));
        assert_eq!(cli2.template_dir, Some(PathBuf::from("/path")));
        assert_eq!(cli3.template_dir, Some(PathBuf::from("/path")));
    }
}
