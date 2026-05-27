use std::net::IpAddr;
use std::{fs, path::PathBuf};

use clap::Parser;
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

fn default_agent_port() -> u16 {
    18765
}

#[cfg(target_os = "windows")]
fn default_windows_capture_source() -> String {
    "auto".to_string()
}

#[derive(Serialize, Deserialize, Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Config {
    #[arg(long, help = "Access code")]
    pub access_code: Option<String>,
    #[arg(long, default_value = "127.0.0.1", help = "Bind address")]
    pub bind_address: IpAddr,
    #[arg(long, default_value = "1701", help = "Web port")]
    pub web_port: u16,
    #[cfg(target_os = "linux")]
    #[arg(
        long,
        help = "Try to use hardware acceleration through the Video Acceleration API."
    )]
    pub try_vaapi: bool,
    #[cfg(any(target_os = "linux", target_os = "windows"))]
    #[arg(long, help = "Try to use Nvidia's NVENC to encode the video via GPU.")]
    #[serde(default)]
    pub try_nvenc: bool,
    #[cfg(target_os = "macos")]
    #[arg(
        long,
        help = "Try to use hardware acceleration through the VideoToolbox API."
    )]
    #[serde(default)]
    pub try_videotoolbox: bool,
    #[cfg(target_os = "windows")]
    #[arg(
        long,
        help = "Try to use hardware acceleration through the MediaFoundation API."
    )]
    #[serde(default)]
    pub try_mediafoundation: bool,
    #[cfg(target_os = "windows")]
    #[arg(
        long,
        default_value = "auto",
        help = "Windows screen capture source: auto, dxgi, or gdi."
    )]
    #[serde(default = "default_windows_capture_source")]
    pub windows_capture_source: String,
    #[arg(long, help = "Start Weylus server immediately on program start.")]
    #[serde(default)]
    pub auto_start: bool,
    #[arg(long, help = "Run Weylus without gui and start immediately.")]
    #[serde(default)]
    pub no_gui: bool,
    #[arg(long, help = "Run AuroraOps agent as a long-running service.")]
    #[serde(default)]
    pub service: bool,
    #[arg(long, help = "Alias for --service, kept for old agent compatibility.")]
    #[serde(default)]
    pub headless: bool,
    #[arg(long = "config", help = "AuroraOps agent JSON config path.")]
    #[serde(skip)]
    pub agent_config: Option<PathBuf>,
    #[arg(long = "port", default_value_t = default_agent_port(), help = "AuroraOps local management port.")]
    #[serde(default = "default_agent_port")]
    pub agent_port: u16,
    #[arg(long = "server", help = "AuroraOps server host for service mode.")]
    #[serde(default)]
    pub agent_server: Option<String>,
    #[arg(long = "name", help = "AuroraOps device name for service mode.")]
    #[serde(default)]
    pub agent_name: Option<String>,
    #[arg(
        long,
        help = "Install AuroraOps agent as a system service and start it."
    )]
    #[serde(skip)]
    pub install_service: bool,
    #[arg(long, help = "Uninstall AuroraOps agent system service.")]
    #[serde(skip)]
    pub uninstall_service: bool,
    #[arg(long, help = "Start AuroraOps agent system service.")]
    #[serde(skip)]
    pub start_service: bool,
    #[arg(long, help = "Stop AuroraOps agent system service.")]
    #[serde(skip)]
    pub stop_service: bool,
    #[arg(long, help = "Restart AuroraOps agent system service.")]
    #[serde(skip)]
    pub restart_service: bool,
    #[arg(
        long,
        hide = true,
        help = "Internal flag used when launched by the Windows Service Control Manager."
    )]
    #[serde(skip)]
    pub windows_service: bool,
    #[arg(
        long,
        hide = true,
        help = "Internal flag used by the Windows service to run in the active desktop session."
    )]
    #[serde(skip)]
    pub session_agent: bool,
    #[cfg(target_os = "linux")]
    #[arg(long, help = "Wayland/PipeWire Support.")]
    #[serde(default)]
    pub wayland_support: bool,
    #[cfg(target_os = "linux")]
    #[arg(long, help = "Enable direct DRM/KMS framebuffer capture.")]
    #[serde(default)]
    pub kms_support: bool,
    #[cfg(target_os = "linux")]
    #[arg(
        long,
        help = "Limit KMS capture to a specific DRM device path, for example /dev/dri/card0."
    )]
    #[serde(default)]
    pub kms_device: Option<String>,

    #[arg(long, help = "Print template of index.html served by Weylus.")]
    #[serde(skip)]
    pub print_index_html: bool,
    #[arg(long, help = "Print access.html served by Weylus.")]
    #[serde(skip)]
    pub print_access_html: bool,
    #[arg(long, help = "Print style.css served by Weylus.")]
    #[serde(skip)]
    pub print_style_css: bool,
    #[arg(long, help = "Print lib.js served by Weylus.")]
    #[serde(skip)]
    pub print_lib_js: bool,

    #[arg(
        long,
        help = "Use custom template of index.html to be served by Weylus."
    )]
    #[serde(skip)]
    pub custom_index_html: Option<PathBuf>,
    #[arg(long, help = "Use custom access.html to be served by Weylus.")]
    #[serde(skip)]
    pub custom_access_html: Option<PathBuf>,
    #[arg(long, help = "Use custom style.css to be served by Weylus.")]
    #[serde(skip)]
    pub custom_style_css: Option<PathBuf>,
    #[arg(long, help = "Use custom lib.js to be served by Weylus.")]
    #[serde(skip)]
    pub custom_lib_js: Option<PathBuf>,

    #[arg(long, help = "Print shell completions for given shell.")]
    #[serde(skip)]
    pub completions: Option<clap_complete::Shell>,
}

pub fn read_config() -> Option<Config> {
    for config_path in user_config_candidates() {
        match fs::read_to_string(&config_path) {
            Ok(s) => match toml::from_str(&s) {
                Ok(c) => return Some(c),
                Err(e) => {
                    warn!("Failed to read configuration file: {}", e);
                    return None;
                }
            },
            Err(err) => {
                match err.kind() {
                    std::io::ErrorKind::NotFound => {
                        debug!("Failed to read configuration file: {}", err)
                    }
                    _ => warn!("Failed to read configuration file: {}", err),
                }
                continue;
            }
        }
    }
    None
}

pub fn write_config(conf: &Config) {
    match user_config_path() {
        Some(config_path) => {
            if let Some(parent) = config_path.parent() {
                if let Err(err) = fs::create_dir_all(parent) {
                    warn!("Failed create directory for configuration: {}", err);
                    return;
                }
            }
            if let Err(err) = fs::write(
                config_path,
                &toml::to_string_pretty(&conf).expect("Failed to encode config to toml."),
            ) {
                warn!("Failed to write configuration file: {}", err);
            }
        }
        None => {
            warn!("Failed to find configuration directory!");
        }
    }
}

fn user_config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|base| base.join("auroraops").join("config.toml"))
}

fn legacy_user_config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|base| base.join("weylus").join("weylus.toml"))
}

fn user_config_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(path) = user_config_path() {
        candidates.push(path);
    }
    if let Some(path) = legacy_user_config_path() {
        candidates.push(path);
    }
    candidates
}

pub fn get_config() -> Config {
    let args = std::env::args();
    if let Some(mut config) = read_config() {
        if args.len() > 1 {
            config.update_from(args);
        }
        config
    } else {
        Config::parse()
    }
}
