#[path = "../../src/aurora.rs"]
mod aurora;
#[path = "../../src/cerror.rs"]
mod cerror;
#[path = "../../src/log.rs"]
mod log;
#[path = "../../src/service_manager.rs"]
mod service_manager;

use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about = "AuroraOps service agent", long_about = None)]
struct Args {
    #[arg(long, default_value = "/etc/auroraops/agent-config.json")]
    config: PathBuf,
    #[arg(long, default_value_t = 18765)]
    port: u16,
}

fn main() {
    let (sender, _receiver) = std::sync::mpsc::sync_channel::<String>(100);
    log::setup_logging(sender);

    let args = Args::parse();
    if let Err(err) = aurora::run_service_lite(args.config, args.port) {
        tracing::error!("AuroraOps service failed: {err}");
        std::process::exit(1);
    }
}
