use std::path::PathBuf;

use clap::Parser;

mod aurora;
mod cerror;
mod log;
mod service_manager;

#[derive(Parser, Debug)]
#[command(version, about = "AuroraOps service agent", long_about = None)]
struct Args {
    #[arg(long)]
    config: Option<PathBuf>,
    #[arg(long, default_value_t = 18765)]
    port: u16,
}

fn main() {
    let (sender, _receiver) = std::sync::mpsc::sync_channel::<String>(100);
    log::setup_logging(sender);

    let args = Args::parse();
    let config = args
        .config
        .unwrap_or_else(service_manager::default_config_path);
    if let Err(err) = aurora::run_service_lite(config, args.port) {
        tracing::error!("AuroraOps service failed: {err}");
        std::process::exit(1);
    }
}
