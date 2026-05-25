use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::io::{Read, Write};
use std::net::{IpAddr, SocketAddr, TcpListener as StdTcpListener, TcpStream, UdpSocket};
#[cfg(unix)]
use std::os::fd::{AsRawFd, FromRawFd};
use std::path::{Path, PathBuf};
#[cfg(not(unix))]
use std::process::{Child, Command, Stdio};
use std::ptr;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tracing::{error, info, warn};
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{connect, Message};
use url::Url;

#[cfg(not(feature = "agent-service-lite"))]
use crate::config::Config as WeylusConfig;
#[cfg(not(feature = "agent-service-lite"))]
use crate::web::Web2UiMessage;
#[cfg(not(feature = "agent-service-lite"))]
use crate::weylus::Weylus;

type BoxError = Box<dyn Error + Send + Sync + 'static>;

const DEFAULT_CONFIG_PATH: &str = "/etc/auroraops/agent-config.json";
const DEFAULT_AGENT_PORT: u16 = 18765;
const AUTO_WEYLUS_PORT: u16 = 0;
const WEYLUS_TUNNEL_CHUNK_SIZE: usize = 64 * 1024;
const WEYLUS_TUNNEL_FRAME_OPEN: u8 = 1;
const WEYLUS_TUNNEL_FRAME_DATA: u8 = 2;
const WEYLUS_TUNNEL_FRAME_CLOSE: u8 = 3;
const REGISTER_RETRY_MAX: Duration = Duration::from_secs(10);
const TCP_RETRY_MAX: Duration = Duration::from_secs(5);
#[cfg(all(not(feature = "agent-service-lite"), target_os = "linux"))]
const XAUTHORITY_CANDIDATES: &[&str] = &[
    "/run/lightdm/root/:0",
    "/var/lib/lightdm/.Xauthority",
    "/var/run/lightdm/root/:0",
    "/run/gdm/auth-for-gdm/database",
    "/var/lib/gdm/.local/share/xorg/Xauthority",
    "/var/lib/gdm3/.local/share/xorg/Xauthority",
    "/var/run/sddm/{display}",
];

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AgentConfig {
    pub server_host: String,
    pub device_name: String,
    pub http_base: String,
    #[serde(default)]
    pub device_id: u64,
    #[serde(default)]
    pub token: String,
    #[serde(default)]
    pub tcp_address: String,
    #[serde(default)]
    pub hostname: String,
    #[serde(default = "default_weylus_bind")]
    pub bind_address: String,
    #[serde(default = "default_weylus_port")]
    pub web_port: u16,
    #[serde(default)]
    pub access_code: Option<String>,
    #[serde(default)]
    pub try_vaapi: bool,
    #[serde(default)]
    pub try_nvenc: bool,
    #[serde(default)]
    pub wayland_support: bool,
    #[serde(default)]
    pub kms_support: bool,
    #[serde(default)]
    pub kms_device: Option<String>,
    #[serde(default = "default_control_display_manager")]
    pub control_display_manager: bool,
}

fn default_weylus_bind() -> String {
    "127.0.0.1".to_string()
}

fn default_weylus_port() -> u16 {
    AUTO_WEYLUS_PORT
}

fn default_control_display_manager() -> bool {
    true
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentStatus {
    pub state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tcp_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    pub updated_at: u128,
    pub desktop_url: String,
}

impl Default for AgentStatus {
    fn default() -> Self {
        Self {
            state: "idle".to_string(),
            device_id: None,
            tcp_address: None,
            message: None,
            updated_at: now_millis(),
            desktop_url: String::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetEntry {
    pub asset_type: String,
    pub unique_key: String,
    pub asset_name: String,
    #[serde(default)]
    pub brand: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub serial_no: String,
    #[serde(default)]
    pub specification: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub sync_hash: String,
    #[serde(default)]
    pub remark: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssetDiagnostic {
    pub name: String,
    pub ok: bool,
    pub count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ControlResponse {
    ok: bool,
    status: AgentStatus,
    config: AgentConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    assets: Option<Vec<AssetEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    diagnostics: Option<Vec<AssetDiagnostic>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SaveConfigPayload {
    server_host: String,
    device_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SaveDesktopConfigPayload {
    #[serde(default = "default_weylus_bind")]
    bind_address: String,
    #[serde(default = "default_weylus_port")]
    web_port: u16,
    #[serde(default)]
    access_code: Option<String>,
    #[serde(default)]
    try_vaapi: bool,
    #[serde(default)]
    try_nvenc: bool,
    #[serde(default)]
    wayland_support: bool,
    #[serde(default)]
    kms_support: bool,
    #[serde(default)]
    kms_device: Option<String>,
    #[serde(default = "default_control_display_manager")]
    control_display_manager: bool,
}

#[derive(Clone)]
pub struct AgentRuntime {
    inner: Arc<AgentInner>,
}

struct AgentInner {
    config_path: PathBuf,
    local_port: u16,
    config: RwLock<AgentConfig>,
    status: RwLock<AgentStatus>,
    connector_stop: Mutex<Option<oneshot::Sender<()>>>,
    run_id: AtomicU64,
    auto_web_port: AtomicBool,
}

impl AgentRuntime {
    pub fn new(config_path: PathBuf, local_port: u16) -> Self {
        Self {
            inner: Arc::new(AgentInner {
                config_path,
                local_port,
                config: RwLock::new(AgentConfig {
                    bind_address: default_weylus_bind(),
                    web_port: AUTO_WEYLUS_PORT,
                    ..AgentConfig::default()
                }),
                status: RwLock::new(AgentStatus::default()),
                connector_stop: Mutex::new(None),
                run_id: AtomicU64::new(0),
                auto_web_port: AtomicBool::new(true),
            }),
        }
    }

    fn load_config(&self) -> Result<(), BoxError> {
        match fs::read_to_string(&self.inner.config_path) {
            Ok(data) => {
                let mut cfg: AgentConfig = serde_json::from_str(&data)?;
                normalize_config(&mut cfg);
                self.inner
                    .auto_web_port
                    .store(cfg.web_port == AUTO_WEYLUS_PORT, Ordering::SeqCst);
                self.set_desktop_url(&cfg);
                *self.inner.config.write().unwrap() = cfg;
                Ok(())
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(Box::new(err)),
        }
    }

    fn save_config_file(&self, cfg: &AgentConfig) -> Result<(), BoxError> {
        if let Some(parent) = self.inner.config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut persisted = cfg.clone();
        if self.inner.auto_web_port.load(Ordering::SeqCst) {
            persisted.web_port = AUTO_WEYLUS_PORT;
        }
        fs::write(
            &self.inner.config_path,
            serde_json::to_string_pretty(&persisted)?,
        )?;
        Ok(())
    }

    fn get_config(&self) -> AgentConfig {
        self.inner.config.read().unwrap().clone()
    }

    fn get_display_config(&self) -> AgentConfig {
        let mut cfg = self.get_config();
        if self.inner.auto_web_port.load(Ordering::SeqCst) {
            cfg.web_port = AUTO_WEYLUS_PORT;
        }
        cfg
    }

    fn get_status(&self) -> AgentStatus {
        self.inner.status.read().unwrap().clone()
    }

    fn update_status(
        &self,
        state: impl Into<String>,
        device_id: Option<u64>,
        tcp_address: Option<String>,
        message: Option<String>,
    ) {
        let desktop_url = self.inner.status.read().unwrap().desktop_url.clone();
        *self.inner.status.write().unwrap() = AgentStatus {
            state: state.into(),
            device_id,
            tcp_address,
            message,
            updated_at: now_millis(),
            desktop_url,
        };
    }

    fn update_status_if_current(
        &self,
        run_id: u64,
        state: impl Into<String>,
        device_id: Option<u64>,
        tcp_address: Option<String>,
        message: Option<String>,
    ) {
        if self.inner.run_id.load(Ordering::SeqCst) == run_id {
            self.update_status(state, device_id, tcp_address, message);
        }
    }

    fn set_desktop_url(&self, cfg: &AgentConfig) {
        if cfg.web_port == AUTO_WEYLUS_PORT {
            self.inner.status.write().unwrap().desktop_url = String::new();
            return;
        }
        let bind = if cfg.bind_address == "0.0.0.0" || cfg.bind_address == "::" {
            "127.0.0.1"
        } else {
            cfg.bind_address.as_str()
        };
        self.inner.status.write().unwrap().desktop_url =
            format!("http://{}:{}/", bind, cfg.web_port);
    }

    fn save_config(
        &self,
        server_host: String,
        device_name: String,
    ) -> Result<AgentConfig, BoxError> {
        if server_host.trim().is_empty() || device_name.trim().is_empty() {
            return Err("serverHost and deviceName are required".into());
        }

        let mut cfg = self.get_config();
        let changed =
            cfg.server_host != server_host.trim() || cfg.device_name != device_name.trim();
        if changed {
            self.stop_connector();
            cfg.device_id = 0;
            cfg.token.clear();
            cfg.tcp_address.clear();
            cfg.hostname.clear();
        }
        cfg.server_host = server_host.trim().to_string();
        cfg.device_name = device_name.trim().to_string();
        normalize_config(&mut cfg);
        self.save_config_file(&cfg)?;
        self.set_desktop_url(&cfg);
        *self.inner.config.write().unwrap() = cfg.clone();
        if changed {
            self.update_status("idle", None, None, Some("config updated".to_string()));
        }
        Ok(cfg)
    }

    fn save_desktop_config(
        &self,
        payload: SaveDesktopConfigPayload,
    ) -> Result<AgentConfig, BoxError> {
        if payload.bind_address.trim().is_empty() {
            return Err("bindAddress is required".into());
        }
        let _: IpAddr = payload
            .bind_address
            .trim()
            .parse()
            .map_err(|_| "bindAddress must be a valid IP address")?;

        let mut cfg = self.get_config();
        let current_web_port = cfg.web_port;
        self.inner
            .auto_web_port
            .store(payload.web_port == AUTO_WEYLUS_PORT, Ordering::SeqCst);
        cfg.bind_address = payload.bind_address.trim().to_string();
        cfg.web_port =
            if payload.web_port == AUTO_WEYLUS_PORT && current_web_port != AUTO_WEYLUS_PORT {
                current_web_port
            } else {
                payload.web_port
            };
        cfg.access_code = payload
            .access_code
            .map(|code| code.trim().to_string())
            .filter(|code| !code.is_empty());
        cfg.try_vaapi = payload.try_vaapi;
        cfg.try_nvenc = payload.try_nvenc;
        cfg.wayland_support = payload.wayland_support;
        cfg.kms_support = payload.kms_support;
        cfg.kms_device = payload
            .kms_device
            .map(|device| device.trim().to_string())
            .filter(|device| !device.is_empty());
        cfg.control_display_manager = payload.control_display_manager;
        normalize_config(&mut cfg);
        self.save_config_file(&cfg)?;
        self.set_desktop_url(&cfg);
        *self.inner.config.write().unwrap() = cfg.clone();
        self.update_status(
            self.get_status().state,
            device_id_opt(cfg.device_id),
            optional_string(&cfg.tcp_address),
            Some("desktop config saved; restart service to apply".to_string()),
        );
        Ok(cfg)
    }

    fn prepare_desktop_runtime_config(&self) -> Result<AgentConfig, BoxError> {
        let mut cfg = self.get_config();
        normalize_config(&mut cfg);
        self.inner
            .auto_web_port
            .store(cfg.web_port == AUTO_WEYLUS_PORT, Ordering::SeqCst);
        if cfg.web_port == AUTO_WEYLUS_PORT {
            cfg.web_port = reserve_loopback_port()?;
            info!(
                "AuroraOps desktop service selected local port {}.",
                cfg.web_port
            );
        }
        self.set_desktop_url(&cfg);
        *self.inner.config.write().unwrap() = cfg.clone();
        Ok(cfg)
    }

    fn start_connector(&self) -> Result<AgentStatus, BoxError> {
        let cfg = self.get_config();
        if cfg.server_host.trim().is_empty() || cfg.device_name.trim().is_empty() {
            return Err("serverHost and deviceName are required".into());
        }
        if self.inner.connector_stop.lock().unwrap().is_some() {
            return Ok(self.get_status());
        }

        let run_id = self.inner.run_id.fetch_add(1, Ordering::SeqCst) + 1;
        let (stop_tx, stop_rx) = oneshot::channel();
        *self.inner.connector_stop.lock().unwrap() = Some(stop_tx);
        self.update_status(
            "starting",
            device_id_opt(cfg.device_id),
            optional_string(&cfg.tcp_address),
            Some("agent starting".to_string()),
        );

        let runtime = self.clone();
        thread::spawn(move || {
            runtime.run_connector(run_id, cfg, stop_rx);
            let mut guard = runtime.inner.connector_stop.lock().unwrap();
            guard.take();
        });
        Ok(self.get_status())
    }

    fn stop_connector(&self) {
        self.inner.run_id.fetch_add(1, Ordering::SeqCst);
        if let Some(stop) = self.inner.connector_stop.lock().unwrap().take() {
            let _ = stop.send(());
        }
        let cfg = self.get_config();
        self.update_status(
            "stopped",
            device_id_opt(cfg.device_id),
            optional_string(&cfg.tcp_address),
            Some("agent stopped".to_string()),
        );
    }

    fn run_connector(&self, run_id: u64, mut cfg: AgentConfig, mut stop_rx: oneshot::Receiver<()>) {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .unwrap();

        let hostname = hostname();
        let ip = detect_ip();
        let mut register_backoff = Duration::from_secs(2);
        loop {
            if self.inner.run_id.load(Ordering::SeqCst) != run_id {
                return;
            }
            match register_device(&client, &cfg, &hostname, &ip) {
                Ok(updated) => {
                    cfg = updated;
                    if self.inner.run_id.load(Ordering::SeqCst) != run_id {
                        return;
                    }
                    *self.inner.config.write().unwrap() = cfg.clone();
                    if let Err(err) = self.save_config_file(&cfg) {
                        self.update_status_if_current(
                            run_id,
                            "error",
                            device_id_opt(cfg.device_id),
                            optional_string(&cfg.tcp_address),
                            Some(err.to_string()),
                        );
                        return;
                    }
                    self.update_status_if_current(
                        run_id,
                        "registered",
                        device_id_opt(cfg.device_id),
                        optional_string(&cfg.tcp_address),
                        None,
                    );
                    break;
                }
                Err(err) => {
                    self.update_status_if_current(
                        run_id,
                        "registering",
                        device_id_opt(cfg.device_id),
                        optional_string(&cfg.tcp_address),
                        Some(err.to_string()),
                    );
                }
            }

            if wait_for_stop(&mut stop_rx, register_backoff) {
                return;
            }
            register_backoff = (register_backoff * 2).min(REGISTER_RETRY_MAX);
        }

        if let Err(err) = sync_assets(&client, &cfg) {
            warn!("asset sync failed: {err}");
        }

        start_weylus_tunnel(cfg.clone());

        let mut backoff = Duration::from_secs(2);
        loop {
            if self.inner.run_id.load(Ordering::SeqCst) != run_id {
                return;
            }
            self.update_status_if_current(
                run_id,
                "connecting",
                device_id_opt(cfg.device_id),
                optional_string(&cfg.tcp_address),
                None,
            );
            match connect_tcp(&cfg, &client, &mut stop_rx, || {
                self.update_status_if_current(
                    run_id,
                    "connected",
                    device_id_opt(cfg.device_id),
                    optional_string(&cfg.tcp_address),
                    None,
                );
            }) {
                Ok(()) => {
                    self.update_status_if_current(
                        run_id,
                        "stopped",
                        device_id_opt(cfg.device_id),
                        optional_string(&cfg.tcp_address),
                        Some("tcp stopped".to_string()),
                    );
                    return;
                }
                Err(err) => {
                    self.update_status_if_current(
                        run_id,
                        "reconnecting",
                        device_id_opt(cfg.device_id),
                        optional_string(&cfg.tcp_address),
                        Some(err.to_string()),
                    );
                }
            }
            match stop_rx.try_recv() {
                Ok(_) | Err(tokio::sync::oneshot::error::TryRecvError::Closed) => return,
                Err(tokio::sync::oneshot::error::TryRecvError::Empty) => {}
            }
            if wait_for_stop(&mut stop_rx, backoff) {
                return;
            }
            backoff = (backoff * 2).min(TCP_RETRY_MAX);
        }
    }
}

fn wait_for_stop(stop_rx: &mut oneshot::Receiver<()>, duration: Duration) -> bool {
    let deadline = std::time::Instant::now() + duration;
    loop {
        match stop_rx.try_recv() {
            Ok(_) | Err(tokio::sync::oneshot::error::TryRecvError::Closed) => return true,
            Err(tokio::sync::oneshot::error::TryRecvError::Empty) => {}
        }
        let now = std::time::Instant::now();
        if now >= deadline {
            return false;
        }
        thread::sleep((deadline - now).min(Duration::from_millis(200)));
    }
}

#[cfg(not(feature = "agent-service-lite"))]
pub fn run_service(conf: &WeylusConfig) -> Result<(), BoxError> {
    setup_uinput_once();
    let config_path = conf
        .agent_config
        .clone()
        .unwrap_or_else(|| PathBuf::from(DEFAULT_CONFIG_PATH));
    let runtime = AgentRuntime::new(config_path, conf.agent_port);
    runtime.load_config()?;
    if runtime.get_config().control_display_manager {
        setup_display_environment();
    }

    if let (Some(server), Some(name)) = (&conf.agent_server, &conf.agent_name) {
        runtime.save_config(server.clone(), name.clone())?;
    }

    runtime.prepare_desktop_runtime_config()?;
    start_weylus_service(conf, &runtime);
    let http_runtime = runtime.clone();
    let _http_thread = thread::spawn(move || {
        if let Err(err) = run_local_http(http_runtime) {
            error!("AuroraOps local HTTP failed: {err}");
        }
    });

    let _ = runtime.start_connector();

    #[cfg(unix)]
    {
        use signal_hook::consts::TERM_SIGNALS;
        use signal_hook::iterator::Signals;
        let mut signals = Signals::new(TERM_SIGNALS)?;
        let _ = signals.forever().next();
        runtime.stop_connector();
    }
    #[cfg(not(unix))]
    {
        loop {
            thread::park();
        }
    }

    Ok(())
}

#[cfg(feature = "agent-service-lite")]
pub fn run_service_lite(config_path: PathBuf, local_port: u16) -> Result<(), BoxError> {
    setup_uinput_once();
    let runtime = AgentRuntime::new(config_path, local_port);
    runtime.load_config()?;

    let http_runtime = runtime.clone();
    let _http_thread = thread::spawn(move || {
        if let Err(err) = run_local_http(http_runtime) {
            error!("AuroraOps local HTTP failed: {err}");
        }
    });

    let _ = runtime.start_connector();

    #[cfg(unix)]
    {
        use signal_hook::consts::TERM_SIGNALS;
        use signal_hook::iterator::Signals;
        let mut signals = Signals::new(TERM_SIGNALS)?;
        let _ = signals.forever().next();
        runtime.stop_connector();
    }
    #[cfg(not(unix))]
    {
        loop {
            thread::park();
        }
    }

    Ok(())
}

fn setup_uinput_once() {
    #[cfg(target_os = "linux")]
    {
        let helper = "/opt/auroraops/auroraops-uinput-setup";
        if Path::new(helper).is_file() {
            match std::process::Command::new(helper).output() {
                Ok(output) if output.status.success() => {
                    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                    if !stderr.is_empty() {
                        info!("{stderr}");
                    }
                }
                Ok(output) => {
                    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                    warn!(
                        "uinput setup helper exited with {}: {}",
                        output.status, stderr
                    );
                }
                Err(err) => warn!("Failed to run uinput setup helper: {err}"),
            }
        } else {
            try_modprobe_uinput();
        }

        match fs::metadata("/dev/uinput") {
            Ok(meta) => {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    info!(
                        "/dev/uinput detected with mode {:o}",
                        meta.permissions().mode() & 0o7777
                    );
                }
                #[cfg(not(unix))]
                {
                    info!("/dev/uinput detected");
                }
            }
            Err(err) => warn!("/dev/uinput is not available after setup attempt: {err}"),
        }
    }
}

#[cfg(not(feature = "agent-service-lite"))]
fn setup_display_environment() {
    #[cfg(target_os = "linux")]
    {
        if std::env::var_os("DISPLAY").is_none() {
            std::env::set_var("DISPLAY", ":0");
            info!("DISPLAY was unset; using DISPLAY=:0 for desktop/DM capture.");
        }

        if std::env::var_os("XAUTHORITY").is_none() {
            let display = std::env::var("DISPLAY").unwrap_or_else(|_| ":0".to_string());
            if let Some(path) = find_xauthority(&display) {
                std::env::set_var("XAUTHORITY", &path);
                info!("XAUTHORITY was unset; using {}", path.display());
            } else {
                warn!(
                    "XAUTHORITY was unset and no known display-manager auth file was found; X11 capture may be unavailable."
                );
            }
        }
    }
}

#[cfg(all(not(feature = "agent-service-lite"), target_os = "linux"))]
fn find_xauthority(display: &str) -> Option<PathBuf> {
    let display_name = display.trim_start_matches(':');
    for candidate in XAUTHORITY_CANDIDATES {
        let candidate = candidate.replace("{display}", display_name);
        let path = PathBuf::from(candidate);
        if path.is_file() {
            return Some(path);
        }
    }

    for pattern_dir in ["/run/user", "/var/run/sddm", "/tmp"] {
        if let Some(path) = find_first_xauthority_under(Path::new(pattern_dir), 3) {
            return Some(path);
        }
    }
    None
}

#[cfg(all(not(feature = "agent-service-lite"), target_os = "linux"))]
fn find_first_xauthority_under(root: &Path, depth: usize) -> Option<PathBuf> {
    if depth == 0 || !root.is_dir() {
        return None;
    }
    let entries = fs::read_dir(root).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if path.is_file()
            && (file_name == "Xauthority"
                || file_name == ".Xauthority"
                || file_name.contains("xauth")
                || file_name.contains("Xauth"))
        {
            return Some(path);
        }
        if path.is_dir() {
            if let Some(found) = find_first_xauthority_under(&path, depth - 1) {
                return Some(found);
            }
        }
    }
    None
}

#[cfg(target_os = "linux")]
fn try_modprobe_uinput() {
    match std::process::Command::new("modprobe")
        .arg("uinput")
        .output()
    {
        Ok(output) if output.status.success() => info!("Loaded uinput kernel module."),
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            warn!("modprobe uinput failed with {}: {}", output.status, stderr);
        }
        Err(err) => warn!("Failed to run modprobe uinput: {err}"),
    }
}

#[cfg(not(feature = "agent-service-lite"))]
fn start_weylus_service(conf: &WeylusConfig, runtime: &AgentRuntime) {
    let mut weylus_conf = conf.clone();
    let agent_cfg = runtime.get_config();
    if let Ok(bind) = agent_cfg.bind_address.parse::<IpAddr>() {
        weylus_conf.bind_address = bind;
    } else {
        weylus_conf.bind_address = IpAddr::from([127, 0, 0, 1]);
    }
    weylus_conf.web_port = agent_cfg.web_port;
    weylus_conf.access_code = agent_cfg.access_code.clone();
    #[cfg(target_os = "linux")]
    {
        weylus_conf.try_vaapi = agent_cfg.try_vaapi;
        weylus_conf.try_nvenc = agent_cfg.try_nvenc;
        weylus_conf.wayland_support = agent_cfg.wayland_support;
        weylus_conf.kms_support = agent_cfg.kms_support;
        weylus_conf.kms_device = agent_cfg.kms_device.clone();
    }
    #[cfg(all(not(target_os = "linux"), target_os = "windows"))]
    {
        weylus_conf.try_nvenc = agent_cfg.try_nvenc;
    }
    weylus_conf.no_gui = true;
    runtime.set_desktop_url(&agent_cfg);

    thread::spawn(move || {
        let mut weylus = Weylus::new();
        if !weylus.start(&weylus_conf, |msg| match msg {
            Web2UiMessage::UInputInaccessible => {
                warn!(std::include_str!("strings/uinput_error.txt"))
            }
        }) {
            error!("Failed to start Weylus desktop service.");
            return;
        }
        loop {
            thread::park();
        }
    });
}

#[tokio::main]
async fn run_local_http(runtime: AgentRuntime) -> Result<(), BoxError> {
    let addr = SocketAddr::new(
        IpAddr::from([127, 0, 0, 1]),
        runtime.inner.local_port.max(DEFAULT_AGENT_PORT),
    );
    let listener = TcpListener::bind(addr).await?;
    info!("AuroraOps local management listening on http://{addr}/");
    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let runtime = runtime.clone();
        tokio::task::spawn(async move {
            let service = service_fn(move |req| {
                let runtime = runtime.clone();
                async move { handle_local_request(runtime, req).await }
            });
            if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                warn!("local http connection failed: {err}");
            }
        });
    }
}

async fn handle_local_request(
    runtime: AgentRuntime,
    req: Request<Incoming>,
) -> Result<Response<Full<Bytes>>, hyper::Error> {
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let response = match (method, path.as_str()) {
        (Method::GET, "/") => html(INDEX_HTML),
        (Method::GET, "/api/status") => {
            json_response(StatusCode::OK, envelope(&runtime, true, None, None, None))
        }
        (Method::POST, "/api/config") => {
            let body = req.into_body().collect().await?.to_bytes();
            match serde_json::from_slice::<SaveConfigPayload>(&body)
                .map_err(|err| err.to_string())
                .and_then(|payload| {
                    runtime
                        .save_config(payload.server_host, payload.device_name)
                        .map(|_| ())
                        .map_err(|err| err.to_string())
                }) {
                Ok(()) => json_response(StatusCode::OK, envelope(&runtime, true, None, None, None)),
                Err(err) => json_response(
                    StatusCode::BAD_REQUEST,
                    envelope(&runtime, false, Some(err), None, None),
                ),
            }
        }
        (Method::POST, "/api/desktop-config") => {
            let body = req.into_body().collect().await?.to_bytes();
            match serde_json::from_slice::<SaveDesktopConfigPayload>(&body)
                .map_err(|err| err.to_string())
                .and_then(|payload| {
                    runtime
                        .save_desktop_config(payload)
                        .map(|_| ())
                        .map_err(|err| err.to_string())
                }) {
                Ok(()) => json_response(StatusCode::OK, envelope(&runtime, true, None, None, None)),
                Err(err) => json_response(
                    StatusCode::BAD_REQUEST,
                    envelope(&runtime, false, Some(err), None, None),
                ),
            }
        }
        (Method::POST, "/api/desktop/restart") => match run_systemctl(&["restart"]) {
            Ok(message) => {
                runtime.update_status(
                    runtime.get_status().state,
                    device_id_opt(runtime.get_config().device_id),
                    optional_string(&runtime.get_config().tcp_address),
                    Some(message),
                );
                json_response(StatusCode::OK, envelope(&runtime, true, None, None, None))
            }
            Err(err) => json_response(
                StatusCode::BAD_REQUEST,
                envelope(&runtime, false, Some(err.to_string()), None, None),
            ),
        },
        (Method::GET, "/api/service/status") => {
            let status = service_status_message();
            json_response(
                StatusCode::OK,
                envelope(&runtime, true, Some(status), None, None),
            )
        }
        (Method::POST, "/api/service/enable") => match run_systemctl(&["enable", "--now"]) {
            Ok(message) => json_response(
                StatusCode::OK,
                envelope(&runtime, true, Some(message), None, None),
            ),
            Err(err) => json_response(
                StatusCode::BAD_REQUEST,
                envelope(&runtime, false, Some(err.to_string()), None, None),
            ),
        },
        (Method::POST, "/api/service/disable") => match run_systemctl(&["disable", "--now"]) {
            Ok(message) => json_response(
                StatusCode::OK,
                envelope(&runtime, true, Some(message), None, None),
            ),
            Err(err) => json_response(
                StatusCode::BAD_REQUEST,
                envelope(&runtime, false, Some(err.to_string()), None, None),
            ),
        },
        (Method::POST, "/api/service/restart") => match run_systemctl(&["restart"]) {
            Ok(message) => json_response(
                StatusCode::OK,
                envelope(&runtime, true, Some(message), None, None),
            ),
            Err(err) => json_response(
                StatusCode::BAD_REQUEST,
                envelope(&runtime, false, Some(err.to_string()), None, None),
            ),
        },
        (Method::POST, "/api/start") => match runtime.start_connector() {
            Ok(_) => json_response(StatusCode::OK, envelope(&runtime, true, None, None, None)),
            Err(err) => json_response(
                StatusCode::BAD_REQUEST,
                envelope(&runtime, false, Some(err.to_string()), None, None),
            ),
        },
        (Method::POST, "/api/stop") => {
            runtime.stop_connector();
            json_response(StatusCode::OK, envelope(&runtime, true, None, None, None))
        }
        (Method::POST, "/api/reconnect") => {
            runtime.stop_connector();
            match runtime.start_connector() {
                Ok(_) => json_response(StatusCode::OK, envelope(&runtime, true, None, None, None)),
                Err(err) => json_response(
                    StatusCode::BAD_REQUEST,
                    envelope(&runtime, false, Some(err.to_string()), None, None),
                ),
            }
        }
        (Method::GET, "/api/assets/preview") => {
            let (assets, diagnostics) = collect_assets();
            json_response(
                StatusCode::OK,
                envelope(&runtime, true, None, Some(assets), Some(diagnostics)),
            )
        }
        _ => text(StatusCode::NOT_FOUND, "not found"),
    };
    Ok(response)
}

fn envelope(
    runtime: &AgentRuntime,
    ok: bool,
    message: Option<String>,
    assets: Option<Vec<AssetEntry>>,
    diagnostics: Option<Vec<AssetDiagnostic>>,
) -> ControlResponse {
    ControlResponse {
        ok,
        status: runtime.get_status(),
        config: runtime.get_display_config(),
        assets,
        diagnostics,
        message,
    }
}

fn json_response(status: StatusCode, body: ControlResponse) -> Response<Full<Bytes>> {
    Response::builder()
        .status(status)
        .header("content-type", "application/json; charset=utf-8")
        .body(serde_json::to_vec(&body).unwrap().into())
        .unwrap()
}

fn html(body: &str) -> Response<Full<Bytes>> {
    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "text/html; charset=utf-8")
        .body(body.to_string().into())
        .unwrap()
}

fn text(status: StatusCode, body: &str) -> Response<Full<Bytes>> {
    Response::builder()
        .status(status)
        .header("content-type", "text/plain; charset=utf-8")
        .body(body.to_string().into())
        .unwrap()
}

fn run_systemctl(args: &[&str]) -> Result<String, BoxError> {
    let mut command_args = args.to_vec();
    command_args.push("auroraops-agent.service");
    let output = std::process::Command::new("systemctl")
        .args(&command_args)
        .output()?;
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(if stdout.is_empty() {
            format!("systemctl {} auroraops-agent.service ok", args.join(" "))
        } else {
            stdout
        })
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(if stderr.is_empty() {
            format!(
                "systemctl {} auroraops-agent.service failed",
                args.join(" ")
            )
            .into()
        } else {
            stderr.into()
        })
    }
}

fn service_status_message() -> String {
    let active = std::process::Command::new("systemctl")
        .args(["is-active", "auroraops-agent.service"])
        .output()
        .ok()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unknown".to_string());
    let enabled = std::process::Command::new("systemctl")
        .args(["is-enabled", "auroraops-agent.service"])
        .output()
        .ok()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unknown".to_string());
    format!("active={active}, enabled={enabled}")
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RegisterRequest {
    name: String,
    hostname: String,
    ip: String,
    device_type: String,
    os_name: String,
    location: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegisterResponse {
    id: u64,
    token: String,
    #[serde(default)]
    tcp_address: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct HeartbeatRequest {
    id: u64,
    hostname: String,
    ip: String,
    os_name: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AssetSyncRequest {
    device_id: u64,
    assets: Vec<AssetEntry>,
}

fn register_device(
    client: &reqwest::blocking::Client,
    cfg: &AgentConfig,
    hostname: &str,
    ip: &str,
) -> Result<AgentConfig, BoxError> {
    let req = RegisterRequest {
        name: cfg.device_name.clone(),
        hostname: hostname.to_string(),
        ip: ip.to_string(),
        device_type: "physical".to_string(),
        os_name: std::env::consts::OS.to_string(),
        location: std::env::consts::ARCH.to_string(),
    };
    let reg: RegisterResponse = post_json(
        client,
        &format!("{}/admin/client/register", cfg.http_base),
        &req,
    )?;
    let mut updated = cfg.clone();
    updated.device_id = reg.id;
    updated.token = reg.token;
    if !reg.tcp_address.is_empty() {
        updated.tcp_address = reg.tcp_address;
    }
    updated.hostname = hostname.to_string();
    Ok(updated)
}

fn sync_assets(client: &reqwest::blocking::Client, cfg: &AgentConfig) -> Result<(), BoxError> {
    if cfg.device_id == 0 {
        return Ok(());
    }
    let (assets, _) = collect_assets();
    if assets.is_empty() {
        return Ok(());
    }
    let req = AssetSyncRequest {
        device_id: cfg.device_id,
        assets,
    };
    post_json::<_, Value>(
        client,
        &format!("{}/admin/client/assets/sync", cfg.http_base),
        &req,
    )?;
    Ok(())
}

fn post_heartbeat(client: &reqwest::blocking::Client, cfg: &AgentConfig) {
    if cfg.device_id == 0 {
        return;
    }
    let req = HeartbeatRequest {
        id: cfg.device_id,
        hostname: cfg.hostname.clone(),
        ip: detect_ip(),
        os_name: std::env::consts::OS.to_string(),
    };
    if let Err(err) = post_json::<_, Value>(
        client,
        &format!("{}/admin/client/heartbeat", cfg.http_base),
        &req,
    ) {
        warn!("http heartbeat failed: {err}");
    }
}

fn post_json<T, R>(
    client: &reqwest::blocking::Client,
    url: &str,
    payload: &T,
) -> Result<R, BoxError>
where
    T: Serialize + ?Sized,
    R: for<'de> Deserialize<'de>,
{
    let value: Value = client
        .post(url)
        .json(payload)
        .send()?
        .error_for_status()?
        .json()?;
    if value.get("code").is_some() && value.get("data").is_some() {
        let code = value.get("code").and_then(Value::as_i64).unwrap_or(0);
        if code != 0 && code != 2000 {
            let msg = value
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("server request failed");
            return Err(msg.to_string().into());
        }
        let data = value.get("data").cloned().unwrap_or(Value::Null);
        Ok(serde_json::from_value(data)?)
    } else {
        Ok(serde_json::from_value(value)?)
    }
}

#[derive(Debug, Deserialize)]
struct TcpEnvelope {
    router: String,
    #[serde(default)]
    data: Value,
}

struct TcpOutbound {
    router: &'static str,
    data: Value,
}

#[derive(Deserialize)]
struct TcpResponse {
    #[serde(default)]
    code: i64,
    #[serde(default)]
    message: String,
}

fn connect_tcp(
    cfg: &AgentConfig,
    client: &reqwest::blocking::Client,
    stop_rx: &mut oneshot::Receiver<()>,
    on_connected: impl FnOnce(),
) -> Result<(), BoxError> {
    if cfg.tcp_address.trim().is_empty() {
        return Err("missing tcp address".into());
    }
    let mut stream = TcpStream::connect(&cfg.tcp_address)?;
    stream.set_nodelay(true)?;
    stream.set_read_timeout(Some(Duration::from_secs(1)))?;
    stream.set_write_timeout(Some(Duration::from_secs(10)))?;
    let (out_tx, out_rx) = mpsc::channel::<TcpOutbound>();
    let mut terminals = TerminalManager::new(out_tx.clone());
    let desktop_url = format!("ws://127.0.0.1:{}/ws", cfg.web_port);
    let mut desktops = DesktopManager::new(out_tx.clone(), desktop_url);

    send_tcp(
        &mut stream,
        "DeviceLoginReq",
        json!({
            "deviceId": cfg.device_id,
            "name": cfg.device_name,
            "hostname": cfg.hostname,
            "token": cfg.token,
            "timestamp": now_secs(),
        }),
    )?;
    read_expected_response(&mut stream, "DeviceLoginRes", Duration::from_secs(20))?;
    info!("device tcp login succeeded");
    on_connected();
    let mut write_stream = stream.try_clone()?;
    thread::spawn(move || {
        while let Ok(out) = out_rx.recv() {
            if send_tcp(&mut write_stream, out.router, out.data).is_err() {
                break;
            }
        }
    });
    let mut last_tcp_heartbeat = std::time::Instant::now();
    let mut last_http_heartbeat = std::time::Instant::now();
    loop {
        match stop_rx.try_recv() {
            Ok(_) | Err(tokio::sync::oneshot::error::TryRecvError::Closed) => {
                terminals.close_all();
                desktops.close_all();
                return Ok(());
            }
            Err(tokio::sync::oneshot::error::TryRecvError::Empty) => {}
        }

        match recv_tcp(&mut stream) {
            Ok(Some(env)) => {
                if env.router == "DeviceHeartbeatRes" {
                    // Heartbeat responses are best-effort keepalive acknowledgements.
                } else if env.router.starts_with("DeviceDesktop") {
                    desktops.handle(env);
                } else {
                    terminals.handle(env)
                }
            }
            Ok(None) => {}
            Err(err) => {
                terminals.close_all();
                desktops.close_all();
                return Err(err);
            }
        }

        if last_tcp_heartbeat.elapsed() >= Duration::from_secs(30) {
            let _ = out_tx.send(TcpOutbound {
                router: "DeviceHeartbeatReq",
                data: json!({ "deviceId": cfg.device_id }),
            });
            last_tcp_heartbeat = std::time::Instant::now();
        }
        if last_http_heartbeat.elapsed() >= Duration::from_secs(60) {
            post_heartbeat(client, cfg);
            let _ = sync_assets(client, cfg);
            last_http_heartbeat = std::time::Instant::now();
        }
    }
}

fn send_tcp(stream: &mut TcpStream, router: &str, data: Value) -> Result<(), BoxError> {
    let body = serde_json::to_vec(&json!({ "router": router, "data": data }))?;
    if body.len() > u32::MAX as usize {
        return Err("tcp payload too large".into());
    }
    let mut packet = Vec::with_capacity(4 + body.len());
    packet.extend_from_slice(&(body.len() as u32).to_be_bytes());
    packet.extend_from_slice(&body);
    stream.write_all(&packet)?;
    Ok(())
}

fn recv_tcp(stream: &mut TcpStream) -> Result<Option<TcpEnvelope>, BoxError> {
    let mut header = [0u8; 4];
    match stream.read_exact(&mut header) {
        Ok(()) => {}
        Err(err)
            if err.kind() == std::io::ErrorKind::WouldBlock
                || err.kind() == std::io::ErrorKind::TimedOut =>
        {
            return Ok(None)
        }
        Err(err) => return Err(Box::new(err)),
    }
    let len = u32::from_be_bytes(header) as usize;
    if len == 0 {
        return Ok(None);
    }
    let mut body = vec![0u8; len];
    stream.read_exact(&mut body)?;
    Ok(Some(serde_json::from_slice(&body)?))
}

fn read_expected_response(
    stream: &mut TcpStream,
    expected: &str,
    timeout: Duration,
) -> Result<(), BoxError> {
    let deadline = std::time::Instant::now() + timeout;
    loop {
        if std::time::Instant::now() >= deadline {
            return Err(format!("tcp request timeout: {expected}").into());
        }
        if let Some(env) = recv_tcp(stream)? {
            if env.router == expected {
                let res: TcpResponse = serde_json::from_value(env.data)?;
                if res.code != 0 && res.code != 2000 {
                    return Err(res.message.into());
                }
                return Ok(());
            }
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct WeylusTunnelOpen {
    stream_id: u64,
}

fn start_weylus_tunnel(cfg: AgentConfig) {
    thread::spawn(move || loop {
        if let Err(err) = run_weylus_tunnel(&cfg) {
            warn!("weylus tunnel disconnected: {err}");
        }
        thread::sleep(Duration::from_secs(3));
    });
}

fn run_weylus_tunnel(cfg: &AgentConfig) -> Result<(), BoxError> {
    if cfg.device_id == 0 || cfg.token.is_empty() || cfg.hostname.is_empty() {
        return Err("missing tunnel identity".into());
    }
    let tunnel_url = build_weylus_tunnel_url(cfg)?;
    let (mut socket, _) = connect(tunnel_url.as_str())?;
    if let MaybeTlsStream::Plain(stream) = socket.get_mut() {
        let _ = stream.set_read_timeout(Some(Duration::from_millis(100)));
    }
    info!("weylus tunnel connected");

    let streams: Arc<Mutex<HashMap<u64, mpsc::Sender<Vec<u8>>>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let (out_tx, out_rx) = mpsc::channel::<Vec<u8>>();
    loop {
        while let Ok(frame) = out_rx.try_recv() {
            socket.send(Message::Binary(frame))?;
        }

        match socket.read() {
            Ok(message) => {
                let bytes = match message {
                    Message::Binary(bytes) => bytes,
                    Message::Close(_) => break,
                    _ => continue,
                };
                if bytes.len() < 9 {
                    continue;
                }
                let frame_type = bytes[0];
                let stream_id = u64::from_be_bytes(bytes[1..9].try_into().unwrap());
                let payload = bytes[9..].to_vec();
                match frame_type {
                    WEYLUS_TUNNEL_FRAME_OPEN => {
                        let open: WeylusTunnelOpen = serde_json::from_slice(&payload)?;
                        let stream_id = open.stream_id;
                        let (tx, rx) = mpsc::channel::<Vec<u8>>();
                        streams.lock().unwrap().insert(stream_id, tx);
                        let streams_for_thread = streams.clone();
                        let cfg_for_thread = cfg.clone();
                        let out_tx_for_thread = out_tx.clone();
                        thread::spawn(move || {
                            if let Err(err) = proxy_weylus_stream(
                                &cfg_for_thread,
                                stream_id,
                                rx,
                                out_tx_for_thread,
                            ) {
                                warn!("weylus stream {stream_id} closed: {err}");
                            }
                            streams_for_thread.lock().unwrap().remove(&stream_id);
                        });
                    }
                    WEYLUS_TUNNEL_FRAME_DATA => {
                        if let Some(tx) = streams.lock().unwrap().get(&stream_id).cloned() {
                            let _ = tx.send(payload);
                        }
                    }
                    WEYLUS_TUNNEL_FRAME_CLOSE => {
                        streams.lock().unwrap().remove(&stream_id);
                    }
                    _ => {}
                }
            }
            Err(tungstenite::Error::Io(err))
                if err.kind() == std::io::ErrorKind::WouldBlock
                    || err.kind() == std::io::ErrorKind::TimedOut => {}
            Err(err) => return Err(Box::new(err)),
        }
    }
    Ok(())
}

fn queue_weylus_tunnel_frame(
    out_tx: &mpsc::Sender<Vec<u8>>,
    frame_type: u8,
    stream_id: u64,
    payload: &[u8],
) -> Result<(), BoxError> {
    let mut frame = Vec::with_capacity(9 + payload.len());
    frame.push(frame_type);
    frame.extend_from_slice(&stream_id.to_be_bytes());
    frame.extend_from_slice(payload);
    out_tx.send(frame)?;
    Ok(())
}

fn build_weylus_tunnel_url(cfg: &AgentConfig) -> Result<String, BoxError> {
    let mut url = Url::parse(&cfg.http_base)?;
    url.set_scheme(match url.scheme() {
        "https" => "wss",
        _ => "ws",
    })
    .map_err(|_| "invalid tunnel scheme")?;
    url.set_path("/admin/opsDevice/weylusTunnel/ws");
    url.query_pairs_mut()
        .append_pair("deviceId", &cfg.device_id.to_string())
        .append_pair("hostname", &cfg.hostname)
        .append_pair("token", &cfg.token);
    Ok(url.to_string())
}

fn proxy_weylus_stream(
    cfg: &AgentConfig,
    stream_id: u64,
    rx: mpsc::Receiver<Vec<u8>>,
    out_tx: mpsc::Sender<Vec<u8>>,
) -> Result<(), BoxError> {
    let mut upstream = TcpStream::connect(("127.0.0.1", cfg.web_port))?;
    upstream.set_nodelay(true)?;
    upstream.set_read_timeout(Some(Duration::from_millis(100)))?;
    upstream.set_write_timeout(Some(Duration::from_secs(10)))?;

    let mut upstream_read = upstream.try_clone()?;
    let out_tx_for_reader = out_tx.clone();
    thread::spawn(move || {
        let mut buf = vec![0u8; WEYLUS_TUNNEL_CHUNK_SIZE];
        loop {
            match upstream_read.read(&mut buf) {
                Ok(0) => {
                    let _ = queue_weylus_tunnel_frame(
                        &out_tx_for_reader,
                        WEYLUS_TUNNEL_FRAME_CLOSE,
                        stream_id,
                        &[],
                    );
                    return;
                }
                Ok(n) => {
                    if queue_weylus_tunnel_frame(
                        &out_tx_for_reader,
                        WEYLUS_TUNNEL_FRAME_DATA,
                        stream_id,
                        &buf[..n],
                    )
                    .is_err()
                    {
                        return;
                    }
                }
                Err(err)
                    if err.kind() == std::io::ErrorKind::WouldBlock
                        || err.kind() == std::io::ErrorKind::TimedOut => {}
                Err(_) => {
                    let _ = queue_weylus_tunnel_frame(
                        &out_tx_for_reader,
                        WEYLUS_TUNNEL_FRAME_CLOSE,
                        stream_id,
                        &[],
                    );
                    return;
                }
            }
        }
    });

    while let Ok(data) = rx.recv() {
        upstream.write_all(&data)?;
    }
    Ok(())
}

struct TerminalManager {
    sessions: HashMap<String, TerminalSession>,
    tcp_sender: mpsc::Sender<TcpOutbound>,
}

struct TerminalSession {
    tty: std::fs::File,
    #[cfg(unix)]
    pid: libc::pid_t,
    #[cfg(not(unix))]
    child: Child,
    done: Arc<std::sync::atomic::AtomicBool>,
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        self.done.store(true, Ordering::Relaxed);
        #[cfg(unix)]
        unsafe {
            libc::kill(self.pid, libc::SIGHUP);
            libc::waitpid(self.pid, ptr::null_mut(), libc::WNOHANG);
        }
    }
}

impl TerminalManager {
    fn new(tcp_sender: mpsc::Sender<TcpOutbound>) -> Self {
        Self {
            sessions: HashMap::new(),
            tcp_sender,
        }
    }

    fn handle(&mut self, env: TcpEnvelope) {
        match env.router.as_str() {
            "DeviceTerminalOpenReq" => self.open(env.data),
            "DeviceTerminalInputReq" => self.input(env.data),
            "DeviceTerminalCloseReq" => self.close(env.data, "terminal closed by server"),
            "DeviceTerminalResizeReq" => self.resize(env.data),
            _ => {}
        }
    }

    fn open(&mut self, data: Value) {
        let session_id = data
            .get("sessionId")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        if session_id.is_empty() {
            return;
        }
        if self.sessions.contains_key(&session_id) {
            if let Some((cols, rows)) = terminal_size_from_value(&data) {
                if let Some(session) = self.sessions.get_mut(&session_id) {
                    let _ = resize_terminal_session(session, cols, rows);
                }
            }
            return;
        }
        let shell = data
            .get("shell")
            .and_then(Value::as_str)
            .filter(|s| !s.trim().is_empty())
            .map(ToOwned::to_owned)
            .unwrap_or_else(default_shell);
        match start_terminal_process(&shell) {
            Ok(session) => {
                if let Some((cols, rows)) = terminal_size_from_value(&data) {
                    let _ = resize_terminal_session(&session, cols, rows);
                }

                let session_id_clone = session_id.clone();
                let tcp_sender = self.tcp_sender.clone();
                let done = session.done.clone();
                if let Ok(mut tty) = session.tty.try_clone() {
                    thread::spawn(move || {
                        let mut buf = [0u8; 4096];
                        let (output_tx, output_rx) = mpsc::channel::<Vec<u8>>();
                        let output_session_id = session_id_clone.clone();
                        let output_sender = tcp_sender.clone();
                        let output_done = done.clone();
                        thread::spawn(move || {
                            let mut pending = Vec::with_capacity(16384);
                            loop {
                                match output_rx.recv_timeout(Duration::from_millis(8)) {
                                    Ok(bytes) => {
                                        pending.extend_from_slice(&bytes);
                                        if pending.len() < 16384 {
                                            continue;
                                        }
                                    }
                                    Err(mpsc::RecvTimeoutError::Timeout) => {
                                        if pending.is_empty() {
                                            if output_done.load(Ordering::Relaxed) {
                                                return;
                                            }
                                            continue;
                                        }
                                    }
                                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                                        if pending.is_empty() {
                                            return;
                                        }
                                    }
                                }

                                let output = String::from_utf8_lossy(&pending).to_string();
                                pending.clear();
                                let _ = output_sender.send(TcpOutbound {
                                    router: "DeviceTerminalOutputReq",
                                    data: json!({
                                        "sessionId": output_session_id,
                                        "output": output,
                                    }),
                                });
                            }
                        });
                        loop {
                            if done.load(Ordering::Relaxed) {
                                return;
                            }
                            match tty.read(&mut buf) {
                                Ok(0) => break,
                                Ok(n) => {
                                    let _ = output_tx.send(buf[..n].to_vec());
                                }
                                Err(_) => break,
                            }
                        }
                        drop(output_tx);
                        if !done.load(Ordering::Relaxed) {
                            let _ = tcp_sender.send(TcpOutbound {
                                router: "DeviceTerminalClosedReq",
                                data: json!({
                                    "sessionId": session_id_clone,
                                    "message": "terminal exited",
                                }),
                            });
                        }
                    });
                }
                self.sessions.insert(session_id, session);
            }
            Err(err) => {
                let _ = send_terminal_closed(&self.tcp_sender, &session_id, &err.to_string());
            }
        }
    }

    fn input(&mut self, data: Value) {
        let session_id = data
            .get("sessionId")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let input = data.get("input").and_then(Value::as_str).unwrap_or("");
        if let Some(session) = self.sessions.get_mut(&session_id) {
            let _ = session.tty.write_all(input.as_bytes());
            let _ = session.tty.flush();
        } else {
            let _ =
                send_terminal_closed(&self.tcp_sender, &session_id, "terminal session not found");
        }
    }

    fn resize(&mut self, data: Value) {
        let session_id = data
            .get("sessionId")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let cols = data.get("cols").and_then(Value::as_u64).unwrap_or(0) as u16;
        let rows = data.get("rows").and_then(Value::as_u64).unwrap_or(0) as u16;
        if let Some(session) = self.sessions.get_mut(&session_id) {
            let _ = resize_terminal_session(session, cols, rows);
        }
    }

    fn close(&mut self, data: Value, default_message: &str) {
        let session_id = data
            .get("sessionId")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        if let Some(session) = self.sessions.remove(&session_id) {
            let message = data
                .get("message")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
                .unwrap_or(default_message);
            stop_terminal_session(session);
            let _ = send_terminal_closed(&self.tcp_sender, &session_id, message);
        }
    }

    fn close_all(&mut self) {
        let ids: Vec<String> = self.sessions.keys().cloned().collect();
        for id in ids {
            self.close(
                json!({ "sessionId": id, "message": "agent connection closed" }),
                "agent connection closed",
            );
        }
    }
}

#[cfg(unix)]
fn start_terminal_process(shell: &str) -> Result<TerminalSession, BoxError> {
    use std::ffi::CString;
    use std::os::raw::c_int;

    let c_shell = CString::new(shell).map_err(|_| "invalid shell path")?;
    let c_term = CString::new("TERM").expect("TERM CString");
    let c_term_value = CString::new("xterm-256color").expect("TERM value CString");
    let c_colorterm = CString::new("COLORTERM").expect("COLORTERM CString");
    let c_colorterm_value = CString::new("truecolor").expect("COLORTERM value CString");
    let c_clicolor = CString::new("CLICOLOR").expect("CLICOLOR CString");
    let c_clicolor_value = CString::new("1").expect("CLICOLOR value CString");
    let c_clicolor_force = CString::new("CLICOLOR_FORCE").expect("CLICOLOR_FORCE CString");
    let c_clicolor_force_value = CString::new("1").expect("CLICOLOR_FORCE value CString");
    let c_force_color = CString::new("FORCE_COLOR").expect("FORCE_COLOR CString");
    let c_force_color_value = CString::new("1").expect("FORCE_COLOR value CString");
    let mut master: c_int = -1;
    let pid = unsafe { libc::forkpty(&mut master, ptr::null_mut(), ptr::null(), ptr::null()) };
    if pid < 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    if pid == 0 {
        let arg0 = c_shell.as_ptr();
        let argv = [arg0, ptr::null()];
        unsafe {
            libc::setenv(c_term.as_ptr(), c_term_value.as_ptr(), 1);
            libc::setenv(c_colorterm.as_ptr(), c_colorterm_value.as_ptr(), 1);
            libc::setenv(c_clicolor.as_ptr(), c_clicolor_value.as_ptr(), 1);
            libc::setenv(
                c_clicolor_force.as_ptr(),
                c_clicolor_force_value.as_ptr(),
                1,
            );
            libc::setenv(c_force_color.as_ptr(), c_force_color_value.as_ptr(), 1);
            libc::execvp(arg0, argv.as_ptr());
            libc::_exit(127);
        }
    }

    let tty = unsafe { std::fs::File::from_raw_fd(master) };
    Ok(TerminalSession {
        tty,
        pid,
        done: Arc::new(std::sync::atomic::AtomicBool::new(false)),
    })
}

#[cfg(not(unix))]
fn start_terminal_process(_shell: &str) -> Result<TerminalSession, BoxError> {
    Err("terminal pty is not implemented for this platform".into())
}

fn resize_terminal_session(
    session: &TerminalSession,
    cols: u16,
    rows: u16,
) -> Result<(), BoxError> {
    if cols == 0 || rows == 0 {
        return Ok(());
    }
    #[cfg(unix)]
    {
        let ws = libc::winsize {
            ws_row: rows,
            ws_col: cols,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        let rc = unsafe { libc::ioctl(session.tty.as_raw_fd(), libc::TIOCSWINSZ, &ws) };
        if rc < 0 {
            return Err(std::io::Error::last_os_error().into());
        }
    }
    Ok(())
}

fn stop_terminal_session(session: TerminalSession) {
    session.done.store(true, Ordering::Relaxed);
    #[cfg(unix)]
    unsafe {
        libc::kill(session.pid, libc::SIGHUP);
        libc::kill(session.pid, libc::SIGTERM);
        libc::waitpid(session.pid, ptr::null_mut(), 0);
    }
    #[cfg(not(unix))]
    {
        let mut child = session.child;
        let _ = child.kill();
        let _ = child.wait();
    }
}

fn terminal_size_from_value(data: &Value) -> Option<(u16, u16)> {
    let cols = data.get("cols").and_then(Value::as_u64).unwrap_or(0) as u16;
    let rows = data.get("rows").and_then(Value::as_u64).unwrap_or(0) as u16;
    if cols == 0 || rows == 0 {
        None
    } else {
        Some((cols, rows))
    }
}

fn send_terminal_closed(
    tcp_sender: &mpsc::Sender<TcpOutbound>,
    session_id: &str,
    message: &str,
) -> Result<(), BoxError> {
    tcp_sender
        .send(TcpOutbound {
            router: "DeviceTerminalClosedReq",
            data: json!({ "sessionId": session_id, "message": message }),
        })
        .map_err(|_| "failed to queue terminal close message".into())
}

struct DesktopSession {
    outbound: mpsc::Sender<Message>,
}

struct DesktopManager {
    sessions: HashMap<String, DesktopSession>,
    tcp_sender: mpsc::Sender<TcpOutbound>,
    desktop_url: String,
}

impl DesktopManager {
    fn new(tcp_sender: mpsc::Sender<TcpOutbound>, desktop_url: String) -> Self {
        Self {
            sessions: HashMap::new(),
            tcp_sender,
            desktop_url,
        }
    }

    fn handle(&mut self, env: TcpEnvelope) {
        match env.router.as_str() {
            "DeviceDesktopOpenReq" => self.open(env.data),
            "DeviceDesktopTextReq" => self.send_text(env.data),
            "DeviceDesktopBinaryReq" => self.send_binary(env.data),
            "DeviceDesktopCloseReq" => self.close(env.data, "desktop closed by server"),
            _ => {}
        }
    }

    fn open(&mut self, data: Value) {
        let session_id = data
            .get("sessionId")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        if session_id.is_empty() {
            return;
        }
        if self.sessions.contains_key(&session_id) {
            self.send_closed(&session_id, "desktop session already exists");
            return;
        }

        let (out_tx, out_rx) = mpsc::channel::<Message>();
        let tcp_sender = self.tcp_sender.clone();
        let desktop_url = self.desktop_url.clone();
        let thread_session_id = session_id.clone();

        thread::spawn(move || {
            let (mut socket, _) = match connect(&desktop_url) {
                Ok(result) => result,
                Err(err) => {
                    let _ = tcp_sender.send(TcpOutbound {
                        router: "DeviceDesktopClosedReq",
                        data: json!({ "sessionId": thread_session_id, "message": err.to_string() }),
                    });
                    return;
                }
            };

            match socket.get_mut() {
                MaybeTlsStream::Plain(stream) => {
                    let _ = stream.set_read_timeout(Some(Duration::from_millis(100)));
                }
                _ => {}
            }

            loop {
                while let Ok(msg) = out_rx.try_recv() {
                    if socket.send(msg).is_err() {
                        let _ = tcp_sender.send(TcpOutbound {
                            router: "DeviceDesktopClosedReq",
                            data: json!({ "sessionId": thread_session_id, "message": "desktop websocket write failed" }),
                        });
                        return;
                    }
                }

                match socket.read() {
                    Ok(Message::Text(text)) => {
                        let _ = tcp_sender.send(TcpOutbound {
                            router: "DeviceDesktopTextOutputReq",
                            data: json!({ "sessionId": thread_session_id, "payload": text }),
                        });
                    }
                    Ok(Message::Binary(bytes)) => {
                        let _ = tcp_sender.send(TcpOutbound {
                            router: "DeviceDesktopBinaryOutputReq",
                            data: json!({ "sessionId": thread_session_id, "payload": base64_encode(&bytes) }),
                        });
                    }
                    Ok(Message::Close(_)) => {
                        let _ = tcp_sender.send(TcpOutbound {
                            router: "DeviceDesktopClosedReq",
                            data: json!({ "sessionId": thread_session_id, "message": "desktop websocket closed" }),
                        });
                        return;
                    }
                    Ok(_) => {}
                    Err(tungstenite::Error::Io(err))
                        if err.kind() == std::io::ErrorKind::WouldBlock
                            || err.kind() == std::io::ErrorKind::TimedOut => {}
                    Err(err) => {
                        let _ = tcp_sender.send(TcpOutbound {
                            router: "DeviceDesktopClosedReq",
                            data: json!({ "sessionId": thread_session_id, "message": err.to_string() }),
                        });
                        return;
                    }
                }
            }
        });

        self.sessions
            .insert(session_id, DesktopSession { outbound: out_tx });
    }

    fn send_text(&self, data: Value) {
        let session_id = data.get("sessionId").and_then(Value::as_str).unwrap_or("");
        let payload = data.get("payload").and_then(Value::as_str).unwrap_or("");
        if let Some(session) = self.sessions.get(session_id) {
            let _ = session.outbound.send(Message::Text(payload.to_string()));
        }
    }

    fn send_binary(&self, data: Value) {
        let session_id = data.get("sessionId").and_then(Value::as_str).unwrap_or("");
        let payload = data.get("payload").and_then(Value::as_str).unwrap_or("");
        if let Some(session) = self.sessions.get(session_id) {
            if let Ok(bytes) = base64_decode(payload) {
                let _ = session.outbound.send(Message::Binary(bytes));
            }
        }
    }

    fn close(&mut self, data: Value, default_message: &str) {
        let session_id = data
            .get("sessionId")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        if let Some(session) = self.sessions.remove(&session_id) {
            let _ = session.outbound.send(Message::Close(None));
            let message = data
                .get("message")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
                .unwrap_or(default_message);
            self.send_closed(&session_id, message);
        }
    }

    fn close_all(&mut self) {
        let ids: Vec<String> = self.sessions.keys().cloned().collect();
        for id in ids {
            self.close(
                json!({ "sessionId": id, "message": "agent connection closed" }),
                "agent connection closed",
            );
        }
    }

    fn send_closed(&self, session_id: &str, message: &str) {
        let _ = self.tcp_sender.send(TcpOutbound {
            router: "DeviceDesktopClosedReq",
            data: json!({ "sessionId": session_id, "message": message }),
        });
    }
}

fn collect_assets() -> (Vec<AssetEntry>, Vec<AssetDiagnostic>) {
    let mut assets = Vec::new();
    let mut diagnostics = Vec::new();

    let hostname = hostname();
    assets.push(AssetEntry {
        asset_type: "host".to_string(),
        unique_key: format!("host-{hostname}"),
        asset_name: hostname.clone(),
        brand: std::env::consts::OS.to_string(),
        model: std::env::consts::ARCH.to_string(),
        serial_no: String::new(),
        specification: format!("{} {}", std::env::consts::OS, std::env::consts::ARCH),
        source: "auroraops-agent".to_string(),
        sync_hash: String::new(),
        remark: "auto:agent".to_string(),
    });
    diagnostics.push(AssetDiagnostic {
        name: "host".to_string(),
        ok: true,
        count: 1,
        message: None,
    });

    #[cfg(target_os = "linux")]
    {
        if let Ok(cpuinfo) = fs::read_to_string("/proc/cpuinfo") {
            let model = cpuinfo
                .lines()
                .find_map(|line| {
                    line.strip_prefix("model name")
                        .and_then(|v| v.split(':').nth(1))
                })
                .map(str::trim)
                .unwrap_or("CPU");
            assets.push(AssetEntry {
                asset_type: "cpu".to_string(),
                unique_key: sanitize(model),
                asset_name: model.to_string(),
                brand: String::new(),
                model: model.to_string(),
                serial_no: String::new(),
                specification: String::new(),
                source: "auroraops-agent".to_string(),
                sync_hash: String::new(),
                remark: "auto:agent".to_string(),
            });
            diagnostics.push(AssetDiagnostic {
                name: "cpu".to_string(),
                ok: true,
                count: 1,
                message: None,
            });
        }
        if let Ok(meminfo) = fs::read_to_string("/proc/meminfo") {
            if let Some(total) = meminfo.lines().find(|line| line.starts_with("MemTotal:")) {
                assets.push(AssetEntry {
                    asset_type: "memory".to_string(),
                    unique_key: sanitize(total),
                    asset_name: "System Memory".to_string(),
                    brand: String::new(),
                    model: "RAM".to_string(),
                    serial_no: String::new(),
                    specification: total.to_string(),
                    source: "auroraops-agent".to_string(),
                    sync_hash: String::new(),
                    remark: "auto:agent".to_string(),
                });
                diagnostics.push(AssetDiagnostic {
                    name: "memory".to_string(),
                    ok: true,
                    count: 1,
                    message: None,
                });
            }
        }
    }

    (assets, diagnostics)
}

fn normalize_config(cfg: &mut AgentConfig) {
    cfg.server_host = cfg.server_host.trim().trim_end_matches('/').to_string();
    cfg.device_name = cfg.device_name.trim().to_string();
    if cfg.http_base.trim().is_empty() || cfg.http_base != normalize_http_base(&cfg.server_host) {
        cfg.http_base = normalize_http_base(&cfg.server_host);
    }
    cfg.bind_address = cfg.bind_address.trim().to_string();
    if cfg.bind_address.is_empty() || cfg.bind_address == "0.0.0.0" || cfg.bind_address == "::" {
        cfg.bind_address = default_weylus_bind();
    }
    if cfg
        .kms_device
        .as_ref()
        .is_some_and(|value| value.trim().is_empty())
    {
        cfg.kms_device = None;
    }
}

fn reserve_loopback_port() -> Result<u16, BoxError> {
    let listener = StdTcpListener::bind((default_weylus_bind().as_str(), 0))?;
    Ok(listener.local_addr()?.port())
}

fn normalize_http_base(host: &str) -> String {
    let host = host.trim().trim_end_matches('/');
    if host.starts_with("http://") || host.starts_with("https://") {
        host.to_string()
    } else {
        format!("http://{host}")
    }
}

fn default_shell() -> String {
    std::env::var("SHELL").unwrap_or_else(|_| {
        if cfg!(target_os = "windows") {
            "cmd.exe".to_string()
        } else {
            "/bin/sh".to_string()
        }
    })
}

fn hostname() -> String {
    fs::read_to_string("/etc/hostname")
        .map(|s| s.trim().to_string())
        .ok()
        .filter(|s| !s.is_empty())
        .or_else(|| std::env::var("HOSTNAME").ok())
        .unwrap_or_else(|| "auroraops-agent".to_string())
}

fn detect_ip() -> String {
    UdpSocket::bind("0.0.0.0:0")
        .and_then(|socket| {
            let _ = socket.connect("8.8.8.8:80");
            socket.local_addr()
        })
        .map(|addr| addr.ip().to_string())
        .unwrap_or_default()
}

fn sanitize(value: &str) -> String {
    value
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

fn base64_encode(bytes: &[u8]) -> String {
    BASE64.encode(bytes)
}

fn base64_decode(value: &str) -> Result<Vec<u8>, base64::DecodeError> {
    BASE64.decode(value)
}

fn optional_string(value: &str) -> Option<String> {
    if value.trim().is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

fn device_id_opt(value: u64) -> Option<u64> {
    if value == 0 {
        None
    } else {
        Some(value)
    }
}

fn now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[allow(dead_code)]
fn ensure_parent(path: &Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

const INDEX_HTML: &str = r##"<!doctype html>
<html lang="zh-CN">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>AuroraOps 客户端</title>
  <style>
    :root { color-scheme: light; --bg: #f5f7fa; --panel: #fff; --line: #d8dee4; --text: #17202a; --muted: #66717d; --primary: #1768ac; --danger: #b42318; }
    * { box-sizing: border-box; }
    body { margin: 0; font-family: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; background: var(--bg); color: var(--text); }
    main { max-width: 1120px; margin: 24px auto; padding: 0 20px; }
    header { display: flex; align-items: center; justify-content: space-between; gap: 16px; margin-bottom: 18px; }
    h1 { font-size: 24px; margin: 0; }
    h2 { font-size: 16px; margin: 0 0 14px; }
    section { background: var(--panel); border: 1px solid var(--line); border-radius: 8px; padding: 18px; margin-bottom: 14px; }
    label { display: block; font-size: 13px; margin-bottom: 6px; color: #46515c; }
    input { width: 100%; height: 38px; border: 1px solid #c6ccd2; border-radius: 6px; padding: 0 10px; font-size: 14px; background: #fff; color: var(--text); }
    input[type="checkbox"] { width: 18px; height: 18px; margin: 0; }
    .grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 14px; }
    .switches { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 10px; }
    .switch { border: 1px solid var(--line); border-radius: 8px; padding: 12px; display: flex; gap: 10px; align-items: flex-start; min-height: 76px; }
    .switch strong { display: block; font-size: 14px; margin-bottom: 4px; }
    .switch span { display: block; color: var(--muted); font-size: 12px; line-height: 1.35; }
    .actions { display: flex; flex-wrap: wrap; gap: 10px; margin-top: 16px; }
    button, a.button { border: 1px solid var(--primary); background: var(--primary); color: white; border-radius: 6px; min-height: 36px; padding: 0 14px; font-size: 14px; cursor: pointer; text-decoration: none; display: inline-flex; align-items: center; justify-content: center; }
    button.secondary, a.secondary { background: white; color: var(--primary); }
    button.danger { border-color: var(--danger); background: var(--danger); }
    button:disabled { opacity: .55; cursor: not-allowed; }
    dl { display: grid; grid-template-columns: 150px 1fr; gap: 8px 14px; margin: 0; font-size: 14px; }
    dt { color: var(--muted); }
    dd { margin: 0; overflow-wrap: anywhere; }
    pre { white-space: pre-wrap; background: #111827; color: #e5e7eb; padding: 12px; border-radius: 6px; min-height: 72px; max-height: 180px; overflow: auto; }
    .pill { display: inline-flex; align-items: center; height: 28px; border-radius: 999px; border: 1px solid var(--line); padding: 0 10px; color: var(--muted); background: #fff; font-size: 13px; }
    @media (max-width: 800px) { .grid, .switches, dl { grid-template-columns: 1fr; } header { align-items: flex-start; flex-direction: column; } }
  </style>
</head>
<body>
  <main>
    <header>
      <h1>AuroraOps 客户端</h1>
      <span id="serviceStatus" class="pill">service: unknown</span>
    </header>
    <section>
      <h2>连接配置</h2>
      <div class="grid">
        <div><label for="serverHost">服务端地址</label><input id="serverHost" placeholder="127.0.0.1:8000" /></div>
        <div><label for="deviceName">设备名称</label><input id="deviceName" placeholder="linux-node-01" /></div>
      </div>
      <div class="actions">
        <button onclick="saveConfig()">保存连接配置</button>
        <button onclick="startAgent()">启动连接</button>
        <button class="secondary" onclick="stopAgent()">停止连接</button>
        <button class="secondary" onclick="loadStatus()">刷新</button>
        <a id="desktopLink" class="button secondary" href="#" target="_blank">打开远程桌面端口</a>
      </div>
    </section>
    <section>
      <h2>远程桌面</h2>
      <div class="grid">
        <div><label for="bindAddress">绑定地址</label><input id="bindAddress" placeholder="127.0.0.1" /></div>
        <div><label for="webPort">Web 端口</label><input id="webPort" type="number" min="0" max="65535" placeholder="0 表示随机本地端口" /></div>
        <div><label for="accessCode">访问码</label><input id="accessCode" placeholder="留空则不限制" /></div>
        <div><label for="kmsDevice">KMS 设备</label><input id="kmsDevice" placeholder="/dev/dri/card0" /></div>
      </div>
      <div class="switches" style="margin-top: 14px;">
        <label class="switch"><input id="waylandSupport" type="checkbox" /><span><strong>Wayland / PipeWire</strong>启用 Wayland 捕获和自定义输入区域支持。</span></label>
        <label class="switch"><input id="kmsSupport" type="checkbox" /><span><strong>KMS / DRM</strong>直接通过 DRM/KMS 捕获帧缓冲。</span></label>
        <label class="switch"><input id="tryVaapi" type="checkbox" /><span><strong>VAAPI</strong>尝试使用 Linux VAAPI 硬件编码。</span></label>
        <label class="switch"><input id="tryNvenc" type="checkbox" /><span><strong>NVENC</strong>尝试使用 NVIDIA NVENC 硬件编码。</span></label>
        <label class="switch"><input id="controlDisplayManager" type="checkbox" /><span><strong>登录界面控制</strong>root 服务启动时自动探测 DISPLAY / XAUTHORITY。</span></label>
      </div>
      <div class="actions">
        <button onclick="saveDesktopConfig()">保存桌面配置</button>
        <button class="secondary" onclick="restartDesktop()">重启桌面服务</button>
      </div>
    </section>
    <section>
      <h2>服务管理</h2>
      <div class="actions" style="margin-top: 0;">
        <button onclick="enableService()">启用并启动自启服务</button>
        <button class="secondary" onclick="restartService()">重启 systemd 服务</button>
        <button class="danger" onclick="disableService()">停止并禁用自启</button>
      </div>
    </section>
    <section>
      <h2>状态</h2>
      <dl>
        <dt>连接状态</dt><dd id="state">-</dd>
        <dt>设备 ID</dt><dd id="deviceId">-</dd>
        <dt>TCP</dt><dd id="tcpAddress">-</dd>
        <dt>桌面服务</dt><dd id="desktopUrl">-</dd>
        <dt>消息</dt><dd id="message">-</dd>
      </dl>
    </section>
    <section>
      <h2>日志</h2>
      <pre id="log"></pre>
    </section>
  </main>
  <script>
    const ids = ['serverHost','deviceName','bindAddress','webPort','accessCode','kmsDevice','waylandSupport','kmsSupport','tryVaapi','tryNvenc','controlDisplayManager'];
    const $ = (id) => document.getElementById(id);
    const log = (text) => $('log').textContent = `${new Date().toLocaleTimeString()} ${text}\n` + $('log').textContent;
    async function request(path, options) {
      const response = await fetch(path, options);
      const data = await response.json();
      if (!response.ok || !data.ok) throw new Error(data.message || 'request failed');
      return data;
    }
    function setBusy(busy) {
      document.querySelectorAll('button').forEach((button) => button.disabled = busy);
    }
    function render(data) {
      const cfg = data.config || {};
      const status = data.status || {};
      $('serverHost').value = cfg.serverHost || '';
      $('deviceName').value = cfg.deviceName || '';
      $('bindAddress').value = cfg.bindAddress || '127.0.0.1';
      $('webPort').value = cfg.webPort ?? 0;
      $('accessCode').value = cfg.accessCode || '';
      $('kmsDevice').value = cfg.kmsDevice || '';
      $('waylandSupport').checked = !!cfg.waylandSupport;
      $('kmsSupport').checked = !!cfg.kmsSupport;
      $('tryVaapi').checked = !!cfg.tryVaapi;
      $('tryNvenc').checked = !!cfg.tryNvenc;
      $('controlDisplayManager').checked = cfg.controlDisplayManager !== false;
      $('state').textContent = status.state || '-';
      $('deviceId').textContent = status.deviceId || cfg.deviceId || '-';
      $('tcpAddress').textContent = status.tcpAddress || cfg.tcpAddress || '-';
      $('desktopUrl').textContent = status.desktopUrl || '-';
      $('desktopLink').href = status.desktopUrl || '#';
      $('message').textContent = data.message || status.message || '-';
      if (data.message && data.message.startsWith('active=')) $('serviceStatus').textContent = data.message;
    }
    async function call(path, options, okText) {
      setBusy(true);
      try { const data = await request(path, options); render(data); if (okText) log(okText); return data; }
      catch (e) { log(e.message); }
      finally { setBusy(false); }
    }
    async function loadStatus() { await call('/api/status'); await call('/api/service/status'); }
    async function saveConfig() {
      await call('/api/config', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ serverHost: $('serverHost').value, deviceName: $('deviceName').value }) }, '连接配置已保存');
    }
    async function saveDesktopConfig() {
      await call('/api/desktop-config', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({
        bindAddress: $('bindAddress').value, webPort: Number($('webPort').value || 0), accessCode: $('accessCode').value || null,
        kmsDevice: $('kmsDevice').value || null, waylandSupport: $('waylandSupport').checked, kmsSupport: $('kmsSupport').checked,
        tryVaapi: $('tryVaapi').checked, tryNvenc: $('tryNvenc').checked, controlDisplayManager: $('controlDisplayManager').checked
      }) }, '桌面配置已保存，重启桌面服务后生效');
    }
    async function startAgent() { await call('/api/start', { method: 'POST' }, '连接已启动'); }
    async function stopAgent() { await call('/api/stop', { method: 'POST' }, '连接已停止'); }
    async function restartDesktop() { await call('/api/desktop/restart', { method: 'POST' }, '桌面服务重启请求已发送'); }
    async function enableService() { await call('/api/service/enable', { method: 'POST' }, '自启服务已启用'); await loadStatus(); }
    async function disableService() { await call('/api/service/disable', { method: 'POST' }, '自启服务已禁用'); await loadStatus(); }
    async function restartService() { await call('/api/service/restart', { method: 'POST' }, 'systemd 服务重启请求已发送'); }
    loadStatus(); setInterval(loadStatus, 5000);
  </script>
</body>
</html>
"##;
