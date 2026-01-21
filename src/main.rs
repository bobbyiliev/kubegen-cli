//! kubegen CLI binary entry point

use clap::Parser;
use kubegen::cli::{AddCommands, Cli, Commands};
use tracing::{debug, info};

fn main() {
    let cli = Cli::parse();

    // Initialize logging based on verbose flag
    kubegen::logging::init(cli.verbose);

    debug!("CLI arguments parsed successfully");

    match cli.command {
        Commands::New(args) => {
            info!("Creating new operator project: {}", args.name);
            debug!(domain = %args.domain, dry_run = args.dry_run, "Project settings");
            if args.dry_run {
                info!("(dry-run mode - no files will be created)");
            }
            // TODO: Implement project generation
        }
        Commands::Add(add_cmd) => match add_cmd {
            AddCommands::Crd(args) => {
                info!("Adding CRD: {}", args.kind);
                debug!(
                    api_version = %args.api_version,
                    group = ?args.group,
                    dry_run = args.dry_run,
                    "CRD settings"
                );
                // TODO: Implement CRD generation
            }
            AddCommands::Metrics(args) => {
                info!("Adding metrics support");
                debug!(port = args.port, dry_run = args.dry_run, "Metrics settings");
                // TODO: Implement metrics generation
            }
            AddCommands::Webhook(args) => {
                info!("Adding webhook for: {}", args.kind);
                debug!(
                    validating = args.validating,
                    mutating = args.mutating,
                    dry_run = args.dry_run,
                    "Webhook settings"
                );
                // TODO: Implement webhook generation
            }
        },
    }
}
