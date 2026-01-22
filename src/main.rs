//! kubegen CLI binary entry point

use clap::Parser;
use kubegen::cli::{AddCommands, Cli, Commands};
use kubegen::commands::{
    execute_add_crd, execute_add_metrics, execute_add_webhook, execute_new, execute_upgrade,
};
use tracing::{debug, error};

fn main() {
    let cli = Cli::parse();

    // Initialize logging based on verbose flag
    kubegen::logging::init(cli.verbose);

    debug!("CLI arguments parsed successfully");

    let template_dir = cli.template_dir.as_deref();

    let result = match cli.command {
        Commands::New(args) => execute_new(&args, template_dir),
        Commands::Add(add_cmd) => match add_cmd {
            AddCommands::Crd(args) => execute_add_crd(&args, template_dir),
            AddCommands::Metrics(args) => execute_add_metrics(&args, template_dir),
            AddCommands::Webhook(args) => execute_add_webhook(&args, template_dir),
        },
        Commands::Upgrade(args) => execute_upgrade(&args, template_dir),
    };

    if let Err(e) = result {
        error!("{}", e);
        std::process::exit(1);
    }
}
