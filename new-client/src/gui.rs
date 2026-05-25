use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use tracing::{error, info, warn};

use crate::config::Config;
use crate::protocol::CustomInputAreas;

const INSTALLED_LAUNCHER: &str = "/opt/auroraops/auroraops-client-launcher";

pub fn run(config: &Config, _log_receiver: mpsc::Receiver<String>) {
    if try_installed_launcher() {
        return;
    }

    open_local_management_page(config.agent_port);
    info!("Starting AuroraOps client service in the foreground.");
    if let Err(err) = crate::aurora::run_service(config) {
        error!("AuroraOps service failed: {err}");
        std::process::exit(1);
    }
}

pub fn get_input_area(_no_gui: bool, sender: mpsc::Sender<CustomInputAreas>) {
    if let Err(err) = sender.send(CustomInputAreas::default()) {
        warn!("Failed to send default custom input areas: {err}");
    }
}

fn try_installed_launcher() -> bool {
    if !std::path::Path::new(INSTALLED_LAUNCHER).is_file() {
        return false;
    }

    match Command::new(INSTALLED_LAUNCHER)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(_) => {
            info!("Started AuroraOps client launcher: {INSTALLED_LAUNCHER}");
            true
        }
        Err(err) => {
            warn!("Failed to start AuroraOps client launcher {INSTALLED_LAUNCHER}: {err}");
            false
        }
    }
}

fn open_local_management_page(port: u16) {
    let url = format!("http://127.0.0.1:{port}/");
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(800));
        if let Err(err) = open_url(&url) {
            warn!("Failed to open AuroraOps local management page {url}: {err}");
            info!("Open this URL manually: {url}");
        }
    });
}

fn open_url(url: &str) -> std::io::Result<()> {
    #[cfg(target_os = "linux")]
    {
        return spawn_detached("xdg-open", &[url]).or_else(|_| {
            for browser in [
                "qaxbrowser-safe-stable",
                "qaxbrowser",
                "firefox",
                "google-chrome",
                "chromium",
                "chromium-browser",
            ] {
                if spawn_detached(browser, &[url]).is_ok() {
                    return Ok(());
                }
            }
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "no browser command found",
            ))
        });
    }

    #[cfg(target_os = "macos")]
    {
        return spawn_detached("open", &[url]);
    }

    #[cfg(target_os = "windows")]
    {
        return Command::new("cmd")
            .args(["/C", "start", "", url])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map(|_| ());
    }

    #[allow(unreachable_code)]
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "opening a browser is unsupported on this platform",
    ))
}

fn spawn_detached(program: &str, args: &[&str]) -> std::io::Result<()> {
    Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
}
