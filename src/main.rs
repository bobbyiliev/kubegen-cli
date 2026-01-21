//! kubegen CLI binary entry point

use clap::Parser;
use kubegen::cli::{AddCommands, Cli, Commands};

fn main() {
    let cli = Cli::parse();

    if cli.verbose {
        println!("Verbose mode enabled");
    }

    match cli.command {
        Commands::New(args) => {
            println!("Creating new operator project: {}", args.name);
            println!("  Domain: {}", args.domain);
            if args.dry_run {
                println!("  (dry-run mode)");
            }
            // TODO: Implement project generation
        }
        Commands::Add(add_cmd) => match add_cmd {
            AddCommands::Crd(args) => {
                println!("Adding CRD: {}", args.kind);
                println!("  API Version: {}", args.api_version);
                if let Some(group) = &args.group {
                    println!("  Group: {}", group);
                }
                // TODO: Implement CRD generation
            }
            AddCommands::Metrics(args) => {
                println!("Adding metrics support");
                println!("  Port: {}", args.port);
                // TODO: Implement metrics generation
            }
            AddCommands::Webhook(args) => {
                println!("Adding webhook for: {}", args.kind);
                if args.validating {
                    println!("  Type: validating");
                }
                if args.mutating {
                    println!("  Type: mutating");
                }
                // TODO: Implement webhook generation
            }
        },
    }
}
