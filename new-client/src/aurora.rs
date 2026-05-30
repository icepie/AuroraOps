use std::collections::HashMap;
use std::error::Error;
#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
use std::ffi::CStr;
use std::fs;
use std::io::{Read, Write};
#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
use std::mem;
use std::net::{IpAddr, SocketAddr, TcpListener as StdTcpListener, TcpStream, UdpSocket};
#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
use std::os::raw::c_char;
use std::path::{Path, PathBuf};
#[cfg(target_os = "windows")]
use std::process::Command;
#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
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
#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
use libc::free;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sysinfo::{Components, Disks, Networks, ProcessesToUpdate, System};
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tracing::{error, info, warn};
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{connect, Message};
use url::Url;

#[cfg(not(feature = "agent-service-lite"))]
use crate::config::Config as WeylusConfig;
use crate::service_manager;
#[cfg(not(feature = "agent-service-lite"))]
use crate::web::Web2UiMessage;
#[cfg(not(feature = "agent-service-lite"))]
use crate::weylus::Weylus;

type BoxError = Box<dyn Error + Send + Sync + 'static>;

const DEFAULT_AGENT_PORT: u16 = 18765;
const AUTO_WEYLUS_PORT: u16 = 0;
const WEYLUS_TUNNEL_CHUNK_SIZE: usize = 64 * 1024;
const WEYLUS_TUNNEL_FRAME_OPEN: u8 = 1;
const WEYLUS_TUNNEL_FRAME_DATA: u8 = 2;
const WEYLUS_TUNNEL_FRAME_CLOSE: u8 = 3;
const REGISTER_RETRY_MAX: Duration = Duration::from_secs(10);
const TCP_RETRY_MAX: Duration = Duration::from_secs(5);
const MONITOR_REPORT_INTERVAL: Duration = Duration::from_secs(1);
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
#[cfg(all(not(feature = "agent-service-lite"), target_os = "linux"))]
const WAYLAND_SESSION_ENV_KEYS: &[&str] = &[
    "XDG_RUNTIME_DIR",
    "WAYLAND_DISPLAY",
    "DBUS_SESSION_BUS_ADDRESS",
    "XDG_SESSION_TYPE",
    "XDG_CURRENT_DESKTOP",
    "SWAYSOCK",
    "DISPLAY",
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub access_code: Option<String>,
    #[serde(default)]
    pub try_vaapi: bool,
    #[serde(default)]
    pub try_nvenc: bool,
    #[serde(default)]
    pub try_mediafoundation: bool,
    #[cfg(target_os = "windows")]
    #[serde(default = "default_windows_capture_source")]
    pub windows_capture_source: String,
    #[cfg(target_os = "linux")]
    #[serde(default)]
    pub nvfbc_support: bool,
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

#[cfg(target_os = "windows")]
fn default_windows_capture_source() -> String {
    "auto".to_string()
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
    #[cfg(target_os = "windows")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub windows_desktop: Option<crate::input::autopilot_device_win::WindowsDesktopStatus>,
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
            #[cfg(target_os = "windows")]
            windows_desktop: None,
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
    capabilities: AgentCapabilities,
    #[serde(skip_serializing_if = "Option::is_none")]
    assets: Option<Vec<AssetEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    diagnostics: Option<Vec<AssetDiagnostic>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AgentCapabilities {
    platform: &'static str,
    wayland: bool,
    kms: bool,
    nvfbc: bool,
    vaapi: bool,
    nvenc: bool,
    mediafoundation: bool,
    windows_capture_source: bool,
    display_manager: bool,
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
    try_vaapi: bool,
    #[serde(default)]
    try_nvenc: bool,
    #[serde(default)]
    try_mediafoundation: bool,
    #[cfg(target_os = "windows")]
    #[serde(default = "default_windows_capture_source")]
    windows_capture_source: String,
    #[cfg(target_os = "linux")]
    #[serde(default)]
    nvfbc_support: bool,
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
        #[cfg(target_os = "windows")]
        {
            let mut status = self.inner.status.read().unwrap().clone();
            status.windows_desktop = Some(crate::input::autopilot_device_win::desktop_status());
            status
        }
        #[cfg(not(target_os = "windows"))]
        {
            self.inner.status.read().unwrap().clone()
        }
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
            #[cfg(target_os = "windows")]
            windows_desktop: None,
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
        cfg.access_code = None;
        cfg.try_vaapi = payload.try_vaapi;
        cfg.try_nvenc = payload.try_nvenc;
        cfg.try_mediafoundation = payload.try_mediafoundation;
        #[cfg(target_os = "windows")]
        {
            cfg.windows_capture_source =
                normalize_windows_capture_source(&payload.windows_capture_source);
        }
        #[cfg(target_os = "linux")]
        {
            cfg.nvfbc_support = payload.nvfbc_support;
        }
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
        .unwrap_or_else(service_manager::default_config_path);
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
        if let Some(env) = find_wayland_session_environment() {
            let wayland_display = env
                .get("WAYLAND_DISPLAY")
                .map(String::as_str)
                .unwrap_or("-");
            let dbus = env
                .get("DBUS_SESSION_BUS_ADDRESS")
                .map(String::as_str)
                .unwrap_or("-");
            info!(
                "Detected active Wayland session environment: WAYLAND_DISPLAY={} DBUS_SESSION_BUS_ADDRESS={}",
                wayland_display, dbus
            );
            apply_wayland_session_environment(&env);
        } else {
            warn!(
                "No active Wayland session environment was found; Wayland/PipeWire capture may be unavailable."
            );
        }

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
fn apply_wayland_session_environment(env: &HashMap<String, String>) {
    for key in WAYLAND_SESSION_ENV_KEYS {
        let Some(value) = env
            .get(*key)
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
        else {
            continue;
        };

        if std::env::var(key).ok().as_deref() == Some(value) {
            continue;
        }

        std::env::set_var(key, value);
        info!("Wayland session environment injected: {key}={value}");
    }
}

#[cfg(all(not(feature = "agent-service-lite"), target_os = "linux"))]
fn find_wayland_session_environment() -> Option<HashMap<String, String>> {
    let entries = fs::read_dir("/proc").ok()?;
    let mut best: Option<(i32, u32, HashMap<String, String>)> = None;

    for entry in entries.flatten() {
        let pid = entry
            .file_name()
            .to_str()
            .and_then(|name| name.parse::<u32>().ok());
        let Some(pid) = pid else {
            continue;
        };

        let proc_dir = entry.path();
        let env = read_proc_environment(&proc_dir.join("environ"));
        if !is_wayland_environment_candidate(&env) {
            continue;
        }

        let cmdline = fs::read(proc_dir.join("cmdline"))
            .ok()
            .map(|bytes| String::from_utf8_lossy(&bytes).replace('\0', " "))
            .unwrap_or_default();
        let score = score_wayland_environment(&env, &cmdline);
        if score <= 0 {
            continue;
        }

        if best
            .as_ref()
            .map(|(best_score, _, _)| score > *best_score)
            .unwrap_or(true)
        {
            best = Some((score, pid, env));
        }
    }

    if let Some((score, pid, env)) = &best {
        info!("Selected Wayland session environment from pid {pid} with score {score}");
        return Some(env.clone());
    }

    None
}

#[cfg(all(not(feature = "agent-service-lite"), target_os = "linux"))]
fn read_proc_environment(path: &Path) -> HashMap<String, String> {
    let mut env = HashMap::new();
    let Ok(bytes) = fs::read(path) else {
        return env;
    };

    for item in bytes
        .split(|byte| *byte == 0)
        .filter(|item| !item.is_empty())
    {
        let text = String::from_utf8_lossy(item);
        let Some((key, value)) = text.split_once('=') else {
            continue;
        };
        env.insert(key.to_string(), value.to_string());
    }

    env
}

#[cfg(all(not(feature = "agent-service-lite"), target_os = "linux"))]
fn is_wayland_environment_candidate(env: &HashMap<String, String>) -> bool {
    env.get("XDG_RUNTIME_DIR")
        .is_some_and(|value| !value.is_empty())
        && env
            .get("WAYLAND_DISPLAY")
            .is_some_and(|value| !value.is_empty())
}

#[cfg(all(not(feature = "agent-service-lite"), target_os = "linux"))]
fn score_wayland_environment(env: &HashMap<String, String>, cmdline: &str) -> i32 {
    let mut score = 0;

    if wayland_socket_exists(env) {
        score += 100;
    }
    if dbus_socket_exists(env) {
        score += 80;
    }
    if env
        .get("XDG_SESSION_TYPE")
        .is_some_and(|value| value == "wayland")
    {
        score += 40;
    }
    if env
        .get("SWAYSOCK")
        .is_some_and(|value| Path::new(value).exists())
    {
        score += 30;
    }
    if env
        .get("XDG_CURRENT_DESKTOP")
        .is_some_and(|value| value.to_ascii_lowercase().contains("sway"))
    {
        score += 20;
    }

    let cmdline = cmdline.to_ascii_lowercase();
    if cmdline.contains("sway")
        || cmdline.contains("waybar")
        || cmdline.contains("hyprland")
        || cmdline.contains("river")
        || cmdline.contains("wayfire")
        || cmdline.contains("xdg-desktop-portal")
    {
        score += 20;
    }

    score
}

#[cfg(all(not(feature = "agent-service-lite"), target_os = "linux"))]
fn wayland_socket_exists(env: &HashMap<String, String>) -> bool {
    let Some(display) = env.get("WAYLAND_DISPLAY").filter(|value| !value.is_empty()) else {
        return false;
    };

    if display.starts_with('/') {
        return Path::new(display).exists();
    }

    env.get("XDG_RUNTIME_DIR")
        .map(|runtime| Path::new(runtime).join(display).exists())
        .unwrap_or(false)
}

#[cfg(all(not(feature = "agent-service-lite"), target_os = "linux"))]
fn dbus_socket_exists(env: &HashMap<String, String>) -> bool {
    let Some(address) = env.get("DBUS_SESSION_BUS_ADDRESS") else {
        return false;
    };
    let Some(path) = address
        .strip_prefix("unix:path=")
        .and_then(|value| value.split(',').next())
        .filter(|value| !value.is_empty())
    else {
        return false;
    };

    Path::new(path).exists()
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
    weylus_conf.access_code = None;
    #[cfg(target_os = "linux")]
    {
        weylus_conf.try_vaapi = agent_cfg.try_vaapi;
        weylus_conf.try_nvenc = agent_cfg.try_nvenc;
        weylus_conf.nvfbc_support = agent_cfg.nvfbc_support;
        weylus_conf.wayland_support = agent_cfg.wayland_support;
        weylus_conf.kms_support = agent_cfg.kms_support;
        weylus_conf.kms_device = agent_cfg.kms_device.clone();
    }
    #[cfg(all(not(target_os = "linux"), target_os = "windows"))]
    {
        weylus_conf.try_nvenc = agent_cfg.try_nvenc;
        weylus_conf.try_mediafoundation = agent_cfg.try_mediafoundation;
        weylus_conf.windows_capture_source =
            normalize_windows_capture_source(&agent_cfg.windows_capture_source);
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
        (Method::POST, "/api/desktop/restart") => match service_manager::restart() {
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
        #[cfg(target_os = "windows")]
        (Method::POST, "/api/input-test") => {
            let input_test = WindowsInputTestQuery::from_query(req.uri().query());
            json_response(
                StatusCode::OK,
                envelope(
                    &runtime,
                    true,
                    Some(windows_input_self_test_safe(input_test)),
                    None,
                    None,
                ),
            )
        }
        #[cfg(target_os = "windows")]
        (Method::POST, "/api/capture-test") => json_response(
            StatusCode::OK,
            envelope(
                &runtime,
                true,
                Some(windows_capture_self_test_safe()),
                None,
                None,
            ),
        ),
        (Method::GET, "/api/service/status") => {
            let status = service_manager::status_message();
            json_response(
                StatusCode::OK,
                envelope(&runtime, true, Some(status), None, None),
            )
        }
        (Method::POST, "/api/service/enable") => {
            match service_manager::install(
                Some(runtime.inner.config_path.clone()),
                runtime.inner.local_port,
            ) {
                Ok(message) => json_response(
                    StatusCode::OK,
                    envelope(&runtime, true, Some(message), None, None),
                ),
                Err(err) => json_response(
                    StatusCode::BAD_REQUEST,
                    envelope(&runtime, false, Some(err.to_string()), None, None),
                ),
            }
        }
        (Method::POST, "/api/service/disable") => match service_manager::uninstall() {
            Ok(message) => json_response(
                StatusCode::OK,
                envelope(&runtime, true, Some(message), None, None),
            ),
            Err(err) => json_response(
                StatusCode::BAD_REQUEST,
                envelope(&runtime, false, Some(err.to_string()), None, None),
            ),
        },
        (Method::POST, "/api/service/restart") => match service_manager::restart() {
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
        capabilities: AgentCapabilities {
            platform: std::env::consts::OS,
            wayland: cfg!(all(target_os = "linux", feature = "pipewire")),
            kms: cfg!(target_os = "linux"),
            nvfbc: cfg!(all(target_os = "linux", feature = "nvfbc")),
            vaapi: cfg!(all(target_os = "linux", feature = "vaapi")),
            nvenc: cfg!(any(target_os = "linux", target_os = "windows")),
            mediafoundation: cfg!(target_os = "windows"),
            windows_capture_source: cfg!(target_os = "windows"),
            display_manager: cfg!(target_os = "linux"),
        },
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

#[cfg(target_os = "windows")]
#[derive(Default)]
struct WindowsInputTestQuery {
    launch_notepad: bool,
    text: Option<String>,
    enter: bool,
    click: Option<(f64, f64)>,
}

#[cfg(target_os = "windows")]
impl WindowsInputTestQuery {
    fn from_query(query: Option<&str>) -> Self {
        let mut parsed = Self::default();
        let Some(query) = query else {
            return parsed;
        };
        for (key, value) in url::form_urlencoded::parse(query.as_bytes()) {
            match key.as_ref() {
                "notepad" => parsed.launch_notepad = query_bool(&value),
                "text" => parsed.text = Some(value.into_owned()),
                "enter" => parsed.enter = query_bool(&value),
                "click" => parsed.click = parse_click_probe(&value),
                "clickX" => {
                    let x = value.parse::<f64>().ok();
                    let y = parsed.click.map(|(_, y)| y).or(Some(0.5));
                    if let (Some(x), Some(y)) = (x, y) {
                        parsed.click = Some((x, y));
                    }
                }
                "clickY" => {
                    let y = value.parse::<f64>().ok();
                    let x = parsed.click.map(|(x, _)| x).or(Some(0.5));
                    if let (Some(x), Some(y)) = (x, y) {
                        parsed.click = Some((x, y));
                    }
                }
                _ => {}
            }
        }
        parsed
    }
}

#[cfg(target_os = "windows")]
fn query_bool(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

#[cfg(target_os = "windows")]
fn parse_click_probe(value: &str) -> Option<(f64, f64)> {
    let (x, y) = value.split_once(',')?;
    Some((x.trim().parse().ok()?, y.trim().parse().ok()?))
}

#[cfg(target_os = "windows")]
fn windows_input_self_test_safe(query: WindowsInputTestQuery) -> String {
    match std::panic::catch_unwind(|| windows_input_self_test(query)) {
        Ok(message) => message,
        Err(err) => {
            if let Some(message) = err.downcast_ref::<String>() {
                format!("panic={message}")
            } else if let Some(message) = err.downcast_ref::<&'static str>() {
                format!("panic={message}")
            } else {
                "panic=unknown".to_string()
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn windows_input_self_test(query: WindowsInputTestQuery) -> String {
    if query.launch_notepad {
        return crate::input::autopilot_device_win::diagnose_notepad_keyboard_input().join("\n");
    }
    let mut lines = if query.text.is_some() || query.enter || query.click.is_some() {
        crate::input::autopilot_device_win::diagnose_input_probe(
            query.text.as_deref(),
            query.enter,
            query.click,
        )
    } else {
        let mut lines = crate::input::autopilot_device_win::diagnose_keyboard_context();
        lines.extend(crate::input::autopilot_device_win::diagnose_keyboard_sendinput());
        lines
    };
    lines.push("usage=/api/input-test?text=1234&enter=1&click=0.5,0.55 or ?notepad=1".to_string());
    lines.join("\n")
}

#[cfg(target_os = "windows")]
fn windows_capture_self_test_safe() -> String {
    match std::panic::catch_unwind(windows_capture_self_test) {
        Ok(message) => message,
        Err(err) => {
            if let Some(message) = err.downcast_ref::<String>() {
                format!("panic={message}")
            } else if let Some(message) = err.downcast_ref::<&'static str>() {
                format!("panic={message}")
            } else {
                "panic=unknown".to_string()
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn windows_capture_self_test() -> String {
    let mut lines = Vec::new();
    lines.push(format!(
        "input_desktop={}",
        crate::input::autopilot_device_win::desktop_status().input_desktop
    ));
    let capturables = crate::capturable::get_capturables();
    lines.push(format!("capturables={}", capturables.len()));
    let Some(capturable) = capturables.first() else {
        return lines.join("\n");
    };
    lines.push(format!("capturable={}", capturable.name()));
    let mut recorder = match capturable.recorder(true) {
        Ok(recorder) => recorder,
        Err(err) => {
            lines.push(format!("recorder=error: {err}"));
            return lines.join("\n");
        }
    };
    lines.push(format!("backend={}", recorder.backend_name()));
    match recorder.capture() {
        Ok(frame) => {
            let (width, height) = frame.size();
            let sample_hash = pixel_sample_hash(&frame);
            lines.push(format!(
                "frame={width}x{height} sample_hash={sample_hash:016x}"
            ));
        }
        Err(err) => lines.push(format!("capture=error: {err}")),
    }
    lines.join("\n")
}

#[cfg(target_os = "windows")]
fn pixel_sample_hash(frame: &crate::video::PixelProvider<'_>) -> u64 {
    let bytes = match frame {
        crate::video::PixelProvider::RGB(_, _, bytes)
        | crate::video::PixelProvider::RGB0(_, _, bytes)
        | crate::video::PixelProvider::BGR0(_, _, bytes)
        | crate::video::PixelProvider::BGR0S(_, _, _, bytes) => *bytes,
    };
    let step = (bytes.len() / 4096).max(1);
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes.iter().step_by(step).take(4096) {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RegisterRequest {
    name: String,
    hostname: String,
    ip: String,
    mac_address: String,
    device_type: String,
    os_name: String,
    architecture: String,
    kernel_version: String,
    client_version: String,
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
    mac_address: String,
    os_name: String,
    architecture: String,
    kernel_version: String,
    client_version: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MonitorSnapshot {
    system: String,
    architecture: String,
    kernel_version: String,
    cpu_model: String,
    gpu_models: Vec<String>,
    cpu_percent: f64,
    memory_percent: f64,
    swap_percent: f64,
    swap_enabled: bool,
    disk_percent: f64,
    net_rx_rate_bytes: f64,
    net_tx_rate_bytes: f64,
    net_rx_bytes: u64,
    net_tx_bytes: u64,
    cpu_cores: usize,
    cpu_physical_cores: usize,
    memory_used_bytes: u64,
    memory_total_bytes: u64,
    swap_used_bytes: u64,
    swap_total_bytes: u64,
    disk_used_bytes: u64,
    disk_total_bytes: u64,
    load1: f64,
    load5: f64,
    load15: f64,
    process_count: usize,
    tcp_connection_count: usize,
    udp_connection_count: usize,
    temperatures: Vec<TemperatureReading>,
    boot_time_seconds: u64,
    uptime_seconds: u64,
    agent_version: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TemperatureReading {
    name: String,
    value: f64,
    kind: String,
    max: Option<f64>,
    critical: Option<f64>,
}

struct MonitorCollector {
    system: System,
    networks: Networks,
    disks: Disks,
    components: Components,
    last_sample_at: std::time::Instant,
    os_name: String,
    architecture: String,
    kernel_version: String,
    cpu_model: String,
    gpu_models: Vec<String>,
    cpu_physical_cores: usize,
    #[cfg(target_os = "windows")]
    load1: f64,
    #[cfg(target_os = "windows")]
    load5: f64,
    #[cfg(target_os = "windows")]
    load15: f64,
}

impl MonitorCollector {
    fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_cpu_usage();
        system.refresh_memory();
        let cpu_model = system
            .cpus()
            .iter()
            .map(|cpu| cpu.brand().trim())
            .find(|brand| !brand.is_empty())
            .unwrap_or("")
            .to_string();
        let cpu_physical_cores =
            System::physical_core_count().unwrap_or_else(|| system.cpus().len());
        Self {
            system,
            networks: Networks::new_with_refreshed_list(),
            disks: Disks::new_with_refreshed_list(),
            components: Components::new_with_refreshed_list(),
            last_sample_at: std::time::Instant::now(),
            os_name: detect_os_name(),
            architecture: std::env::consts::ARCH.to_string(),
            kernel_version: detect_kernel_version(),
            cpu_model,
            gpu_models: detect_gpu_models(),
            cpu_physical_cores,
            #[cfg(target_os = "windows")]
            load1: 0.0,
            #[cfg(target_os = "windows")]
            load5: 0.0,
            #[cfg(target_os = "windows")]
            load15: 0.0,
        }
    }

    fn sample(&mut self) -> MonitorSnapshot {
        let now = std::time::Instant::now();
        let elapsed = now
            .duration_since(self.last_sample_at)
            .as_secs_f64()
            .max(1.0);
        self.last_sample_at = now;

        self.system.refresh_cpu_usage();
        self.system.refresh_memory();
        self.system.refresh_processes(ProcessesToUpdate::All, true);
        self.networks.refresh(true);
        self.disks.refresh(true);
        self.components.refresh(true);

        let total_memory = self.system.total_memory();
        let used_memory = self.system.used_memory();
        let total_swap = self.system.total_swap();
        let used_swap = self.system.used_swap();
        let (disk_total, disk_used) =
            self.disks
                .list()
                .iter()
                .fold((0_u64, 0_u64), |(total, used), disk| {
                    let disk_total = disk.total_space();
                    let disk_used = disk_total.saturating_sub(disk.available_space());
                    (
                        total.saturating_add(disk_total),
                        used.saturating_add(disk_used),
                    )
                });
        let (net_rx_delta, net_tx_delta, net_rx_total, net_tx_total) = self.networks.iter().fold(
            (0_u64, 0_u64, 0_u64, 0_u64),
            |(rx_delta, tx_delta, rx_total, tx_total), (_, data)| {
                (
                    rx_delta.saturating_add(data.received()),
                    tx_delta.saturating_add(data.transmitted()),
                    rx_total.saturating_add(data.total_received()),
                    tx_total.saturating_add(data.total_transmitted()),
                )
            },
        );
        let (load1, load5, load15) = self.load_average(self.system.global_cpu_usage() as f64);

        let (tcp_connection_count, udp_connection_count) = network_connection_counts();
        let temperatures = collect_temperature_readings(&self.components);

        MonitorSnapshot {
            system: self.os_name.clone(),
            architecture: self.architecture.clone(),
            kernel_version: self.kernel_version.clone(),
            cpu_model: self.cpu_model.clone(),
            gpu_models: self.gpu_models.clone(),
            cpu_percent: round2(self.system.global_cpu_usage() as f64),
            memory_percent: percent(used_memory, total_memory),
            swap_percent: percent(used_swap, total_swap),
            swap_enabled: total_swap > 0,
            disk_percent: percent(disk_used, disk_total),
            net_rx_rate_bytes: round2(net_rx_delta as f64 / elapsed),
            net_tx_rate_bytes: round2(net_tx_delta as f64 / elapsed),
            net_rx_bytes: net_rx_total,
            net_tx_bytes: net_tx_total,
            cpu_cores: self.system.cpus().len(),
            cpu_physical_cores: self.cpu_physical_cores,
            memory_used_bytes: used_memory,
            memory_total_bytes: total_memory,
            swap_used_bytes: used_swap,
            swap_total_bytes: total_swap,
            disk_used_bytes: disk_used,
            disk_total_bytes: disk_total,
            load1,
            load5,
            load15,
            process_count: process_count(&self.system),
            tcp_connection_count,
            udp_connection_count,
            temperatures,
            boot_time_seconds: System::boot_time(),
            uptime_seconds: System::uptime(),
            agent_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    #[cfg(target_os = "windows")]
    fn load_average(&mut self, cpu_percent: f64) -> (f64, f64, f64) {
        let core_count = self.system.cpus().len().max(1) as f64;
        let current = (cpu_percent / 100.0) * core_count;
        self.load1 = smooth_load(self.load1, current, 60.0);
        self.load5 = smooth_load(self.load5, current, 300.0);
        self.load15 = smooth_load(self.load15, current, 900.0);
        (round2(self.load1), round2(self.load5), round2(self.load15))
    }

    #[cfg(not(target_os = "windows"))]
    fn load_average(&mut self, _cpu_percent: f64) -> (f64, f64, f64) {
        let load = System::load_average();
        (round2(load.one), round2(load.five), round2(load.fifteen))
    }
}

fn percent(used: u64, total: u64) -> f64 {
    if total == 0 {
        return 0.0;
    }
    round2((used as f64 / total as f64) * 100.0)
}

fn round2(value: f64) -> f64 {
    if !value.is_finite() {
        return 0.0;
    }
    (value * 100.0).round() / 100.0
}

fn collect_temperature_readings(components: &Components) -> Vec<TemperatureReading> {
    components
        .iter()
        .filter_map(|component| {
            let value = component.temperature().map(f64::from)?;
            if !value.is_finite() {
                return None;
            }
            let name = component.label().trim();
            if name.is_empty() {
                return None;
            }
            Some(TemperatureReading {
                name: name.to_string(),
                value: round2(value),
                kind: classify_temperature_kind(name).to_string(),
                max: sanitize_temperature(component.max().map(f64::from)),
                critical: sanitize_temperature(component.critical().map(f64::from)),
            })
        })
        .collect()
}

fn sanitize_temperature(value: Option<f64>) -> Option<f64> {
    value.filter(|value| value.is_finite()).map(round2)
}

fn classify_temperature_kind(name: &str) -> &'static str {
    let lower = name.to_ascii_lowercase();
    if lower.contains("gpu")
        || lower.contains("nvidia")
        || lower.contains("radeon")
        || lower.contains("amdgpu")
    {
        "gpu"
    } else if lower.contains("nvme")
        || lower.contains("ssd")
        || lower.contains("hdd")
        || lower.contains("disk")
        || lower.contains("drive")
    {
        "disk"
    } else if lower.contains("cpu")
        || lower.contains("core")
        || lower.contains("package")
        || lower.contains("tctl")
        || lower.contains("tdie")
    {
        "cpu"
    } else if lower.contains("board")
        || lower.contains("motherboard")
        || lower.contains("pch")
        || lower.contains("acpi")
    {
        "board"
    } else {
        "sensor"
    }
}

#[cfg(target_os = "windows")]
fn smooth_load(previous: f64, current: f64, window_secs: f64) -> f64 {
    if previous <= 0.0 {
        return current;
    }
    let alpha = 1.0 - (-1.0_f64 / window_secs).exp();
    previous + alpha * (current - previous)
}

fn network_connection_counts() -> (usize, usize) {
    #[cfg(target_os = "linux")]
    {
        (
            proc_net_connection_count("/proc/net/tcp")
                + proc_net_connection_count("/proc/net/tcp6"),
            proc_net_connection_count("/proc/net/udp")
                + proc_net_connection_count("/proc/net/udp6"),
        )
    }
    #[cfg(not(target_os = "linux"))]
    {
        platform_network_connection_counts()
    }
}

#[cfg(target_os = "linux")]
fn process_count(_system: &System) -> usize {
    fs::read_dir("/proc")
        .map(|entries| {
            entries
                .flatten()
                .filter(|entry| {
                    entry
                        .file_name()
                        .to_string_lossy()
                        .chars()
                        .all(|ch| ch.is_ascii_digit())
                })
                .count()
        })
        .unwrap_or(0)
}

#[cfg(not(target_os = "linux"))]
fn process_count(system: &System) -> usize {
    system.processes().len()
}

#[cfg(target_os = "linux")]
fn proc_net_connection_count(path: &str) -> usize {
    fs::read_to_string(path)
        .map(|content| {
            content
                .lines()
                .skip(1)
                .filter(|line| !line.trim().is_empty())
                .count()
        })
        .unwrap_or(0)
}

#[cfg(target_os = "windows")]
fn platform_network_connection_counts() -> (usize, usize) {
    let output = match Command::new("netstat").arg("-ano").output() {
        Ok(output) if output.status.success() => output,
        _ => return (0, 0),
    };
    count_netstat_protocol_lines(&String::from_utf8_lossy(&output.stdout))
}

#[cfg(target_os = "macos")]
fn platform_network_connection_counts() -> (usize, usize) {
    let tcp = std::process::Command::new("netstat")
        .args(["-an", "-p", "tcp"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| count_netstat_protocol_lines(&String::from_utf8_lossy(&output.stdout)).0)
        .unwrap_or(0);
    let udp = std::process::Command::new("netstat")
        .args(["-an", "-p", "udp"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| count_netstat_protocol_lines(&String::from_utf8_lossy(&output.stdout)).1)
        .unwrap_or(0);
    (tcp, udp)
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
fn count_netstat_protocol_lines(content: &str) -> (usize, usize) {
    let mut tcp = 0_usize;
    let mut udp = 0_usize;
    for line in content.lines() {
        let proto = line.split_whitespace().next().unwrap_or("");
        if proto.eq_ignore_ascii_case("tcp")
            || proto.eq_ignore_ascii_case("tcp4")
            || proto.eq_ignore_ascii_case("tcp6")
        {
            tcp += 1;
        } else if proto.eq_ignore_ascii_case("udp")
            || proto.eq_ignore_ascii_case("udp4")
            || proto.eq_ignore_ascii_case("udp6")
        {
            udp += 1;
        }
    }
    (tcp, udp)
}

#[cfg(all(
    not(target_os = "linux"),
    not(target_os = "windows"),
    not(target_os = "macos")
))]
fn platform_network_connection_counts() -> (usize, usize) {
    (0, 0)
}

fn detect_gpu_models() -> Vec<String> {
    #[cfg(target_os = "linux")]
    {
        let from_sysfs = detect_gpu_models_from_sysfs();
        if !from_sysfs.is_empty() {
            return from_sysfs;
        }
    }
    Vec::new()
}

#[cfg(target_os = "linux")]
fn detect_gpu_models_from_sysfs() -> Vec<String> {
    let mut models = Vec::new();
    let entries = match fs::read_dir("/sys/class/drm") {
        Ok(entries) => entries,
        Err(_) => return models,
    };

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with("card") || name.contains('-') {
            continue;
        }
        let device = entry.path().join("device");
        let vendor = read_trimmed(device.join("vendor"));
        let device_id = read_trimmed(device.join("device"));
        let model = pci_device_model(&vendor, &device_id)
            .or_else(|| read_trimmed(device.join("product")))
            .unwrap_or_else(|| {
                let id = [vendor.as_deref(), device_id.as_deref()]
                    .into_iter()
                    .flatten()
                    .filter(|value| !value.is_empty())
                    .collect::<Vec<_>>()
                    .join(":");
                if id.is_empty() {
                    name.clone()
                } else {
                    id
                }
            });
        if !model.is_empty() && !models.iter().any(|item| item == &model) {
            models.push(model);
        }
    }

    models
}

#[cfg(target_os = "linux")]
fn read_trimmed(path: impl AsRef<Path>) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[cfg(target_os = "linux")]
fn pci_device_model(vendor: &Option<String>, device: &Option<String>) -> Option<String> {
    let vendor = vendor
        .as_deref()?
        .trim_start_matches("0x")
        .to_ascii_lowercase();
    let device = device
        .as_deref()?
        .trim_start_matches("0x")
        .to_ascii_lowercase();
    let pci_ids = fs::read_to_string("/usr/share/misc/pci.ids")
        .or_else(|_| fs::read_to_string("/usr/share/hwdata/pci.ids"))
        .ok()?;
    let mut in_vendor = false;
    for line in pci_ids.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        if !line.starts_with('\t') {
            in_vendor = line
                .split_whitespace()
                .next()
                .map(|id| id.eq_ignore_ascii_case(&vendor))
                .unwrap_or(false);
            continue;
        }
        if !in_vendor || line.starts_with("\t\t") {
            continue;
        }
        let trimmed = line.trim();
        let Some((id, model)) = trimmed.split_once(char::is_whitespace) else {
            continue;
        };
        if id.eq_ignore_ascii_case(&device) {
            return Some(model.trim().to_string());
        }
    }
    None
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
    let os_name = detect_os_name();
    let kernel_version = detect_kernel_version();
    let mac_address = detect_primary_mac(ip);
    let req = RegisterRequest {
        name: cfg.device_name.clone(),
        hostname: hostname.to_string(),
        ip: ip.to_string(),
        mac_address,
        device_type: detect_device_type(),
        os_name,
        architecture: std::env::consts::ARCH.to_string(),
        kernel_version,
        client_version: env!("CARGO_PKG_VERSION").to_string(),
        location: String::new(),
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
    let ip = detect_ip();
    let req = HeartbeatRequest {
        id: cfg.device_id,
        hostname: cfg.hostname.clone(),
        ip: ip.clone(),
        mac_address: detect_primary_mac(&ip),
        os_name: detect_os_name(),
        architecture: std::env::consts::ARCH.to_string(),
        kernel_version: detect_kernel_version(),
        client_version: env!("CARGO_PKG_VERSION").to_string(),
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
    stream.set_read_timeout(Some(Duration::from_millis(100)))?;
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
    let mut last_monitor_report = std::time::Instant::now() - MONITOR_REPORT_INTERVAL;
    let mut monitor_collector = MonitorCollector::new();
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
                if env.router == "DeviceHeartbeatRes" || env.router == "DeviceMonitorReportRes" {
                    // Keepalive and monitor acknowledgements are best-effort.
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
        if last_monitor_report.elapsed() >= MONITOR_REPORT_INTERVAL {
            let snapshot = monitor_collector.sample();
            let _ = out_tx.send(TcpOutbound {
                router: "DeviceMonitorReportReq",
                data: json!({
                    "deviceId": cfg.device_id,
                    "snapshot": snapshot,
                }),
            });
            last_monitor_report = std::time::Instant::now();
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
    reader: Option<Box<dyn Read + Send>>,
    writer: Box<dyn Write + Send>,
    master: Box<dyn portable_pty::MasterPty + Send>,
    child: Option<Box<dyn portable_pty::Child + Send + Sync>>,
    done: Arc<std::sync::atomic::AtomicBool>,
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        self.done.store(true, Ordering::Relaxed);
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
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
            Ok(mut session) => {
                if let Some((cols, rows)) = terminal_size_from_value(&data) {
                    let _ = resize_terminal_session(&session, cols, rows);
                }

                let session_id_clone = session_id.clone();
                let tcp_sender = self.tcp_sender.clone();
                let done = session.done.clone();
                if let Some(mut reader) = session.reader.take() {
                    spawn_terminal_reader(session_id_clone, tcp_sender, done, move |buf| {
                        reader.read(buf)
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
            let _ = session.writer.write_all(input.as_bytes());
            let _ = session.writer.flush();
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

fn spawn_terminal_reader<F>(
    session_id_clone: String,
    tcp_sender: mpsc::Sender<TcpOutbound>,
    done: Arc<std::sync::atomic::AtomicBool>,
    mut read_fn: F,
) where
    F: FnMut(&mut [u8]) -> std::io::Result<usize> + Send + 'static,
{
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
            match read_fn(&mut buf) {
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

fn start_terminal_process(shell: &str) -> Result<TerminalSession, BoxError> {
    use portable_pty::{native_pty_system, CommandBuilder, PtySize};

    let pty_system = native_pty_system();
    let pair = pty_system.openpty(PtySize {
        rows: 24,
        cols: 80,
        pixel_width: 0,
        pixel_height: 0,
    })?;
    let mut cmd = CommandBuilder::new(shell);
    cmd.env("TERM", "xterm-256color");
    cmd.env("COLORTERM", "truecolor");
    cmd.env("CLICOLOR", "1");
    cmd.env("CLICOLOR_FORCE", "1");
    cmd.env("FORCE_COLOR", "1");
    let child = pair.slave.spawn_command(cmd)?;
    let reader = pair.master.try_clone_reader()?;
    let writer = pair.master.take_writer()?;
    Ok(TerminalSession {
        reader: Some(reader),
        writer,
        master: pair.master,
        child: Some(child),
        done: Arc::new(std::sync::atomic::AtomicBool::new(false)),
    })
}

fn resize_terminal_session(
    session: &TerminalSession,
    cols: u16,
    rows: u16,
) -> Result<(), BoxError> {
    if cols == 0 || rows == 0 {
        return Ok(());
    }
    session.master.resize(portable_pty::PtySize {
        rows,
        cols,
        pixel_width: 0,
        pixel_height: 0,
    })?;
    Ok(())
}

fn stop_terminal_session(mut session: TerminalSession) {
    session.done.store(true, Ordering::Relaxed);
    if let Some(mut child) = session.child.take() {
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
    let os_name = detect_os_name();
    assets.push(AssetEntry {
        asset_type: "host".to_string(),
        unique_key: format!("host-{hostname}"),
        asset_name: hostname.clone(),
        brand: os_name.clone(),
        model: std::env::consts::ARCH.to_string(),
        serial_no: String::new(),
        specification: format!("{} {}", os_name, std::env::consts::ARCH),
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

    #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
    {
        let (detected, detected_diagnostics) = collect_fastfetch_assets();
        assets.extend(detected);
        diagnostics.extend(detected_diagnostics);
    }

    #[cfg(target_os = "linux")]
    {
        ensure_linux_fallback_assets(&mut assets, &mut diagnostics);
    }

    #[cfg(target_os = "windows")]
    {
        if windows_needs_cim_fallback(&assets) {
            let (detected, detected_diagnostics) = collect_windows_assets();
            extend_missing_asset_types(&mut assets, detected);
            diagnostics.extend(detected_diagnostics);
        }
    }

    (assets, diagnostics)
}

#[cfg(target_os = "windows")]
fn extend_missing_asset_types(assets: &mut Vec<AssetEntry>, fallback: Vec<AssetEntry>) {
    for item in fallback {
        if !assets
            .iter()
            .any(|existing| existing.asset_type == item.asset_type)
        {
            assets.push(item);
        }
    }
}

#[cfg(target_os = "windows")]
fn windows_needs_cim_fallback(assets: &[AssetEntry]) -> bool {
    [
        "motherboard",
        "bios",
        "cpu",
        "memory",
        "gpu",
        "network",
        "disk",
    ]
    .into_iter()
    .any(|asset_type| !assets.iter().any(|item| item.asset_type == asset_type))
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
fn collect_fastfetch_assets() -> (Vec<AssetEntry>, Vec<AssetDiagnostic>) {
    let mut collector = FastfetchCollector {
        assets: Vec::new(),
        diagnostics: Vec::new(),
    };
    collector.collect_board();
    collector.collect_bios();
    collector.collect_cpu();
    collector.collect_memory();
    collector.collect_gpus();
    collector.collect_network();
    collector.collect_disks();
    (collector.assets, collector.diagnostics)
}

#[cfg(target_os = "windows")]
fn collect_windows_assets() -> (Vec<AssetEntry>, Vec<AssetDiagnostic>) {
    match run_windows_cim_snapshot() {
        Ok(root) => {
            let mut collector = WindowsCimCollector {
                root,
                assets: Vec::new(),
                diagnostics: Vec::new(),
            };
            collector.collect_board();
            collector.collect_bios();
            collector.collect_cpu();
            collector.collect_memory();
            collector.collect_gpus();
            collector.collect_network();
            collector.collect_disks();
            (collector.assets, collector.diagnostics)
        }
        Err(message) => (
            Vec::new(),
            vec![AssetDiagnostic {
                name: "windows-cim".to_string(),
                ok: false,
                count: 0,
                message: Some(message),
            }],
        ),
    }
}

#[cfg(target_os = "windows")]
fn run_windows_cim_snapshot() -> Result<Value, String> {
    let script = r#"
$ErrorActionPreference = 'Stop'
$data = [ordered]@{
  board = @(Get-CimInstance -ClassName Win32_BaseBoard | Select-Object Manufacturer,Product,Version,SerialNumber)
  bios = @(Get-CimInstance -ClassName Win32_BIOS | Select-Object Manufacturer,SMBIOSBIOSVersion,Version,SerialNumber,ReleaseDate)
  cpu = @(Get-CimInstance -ClassName Win32_Processor | Select-Object Name,Manufacturer,ProcessorId,NumberOfCores,NumberOfLogicalProcessors,MaxClockSpeed)
  memory = @(Get-CimInstance -ClassName Win32_PhysicalMemory | Select-Object Manufacturer,PartNumber,SerialNumber,Capacity,Speed,ConfiguredClockSpeed,DeviceLocator,BankLabel,MemoryType,SMBIOSMemoryType)
  gpu = @(Get-CimInstance -ClassName Win32_VideoController | Select-Object Name,AdapterCompatibility,DriverVersion,AdapterRAM,PNPDeviceID)
  network = @(Get-CimInstance -ClassName Win32_NetworkAdapter | Where-Object { $_.PhysicalAdapter -and $_.MACAddress } | Select-Object Name,Manufacturer,MACAddress,Speed,NetConnectionID,PNPDeviceID)
  disk = @(Get-CimInstance -ClassName Win32_DiskDrive | Select-Object Model,Manufacturer,SerialNumber,Size,InterfaceType,MediaType,FirmwareRevision,DeviceID,PNPDeviceID)
  system = @(Get-CimInstance -ClassName Win32_ComputerSystem | Select-Object Manufacturer,Model,TotalPhysicalMemory)
}
$data | ConvertTo-Json -Depth 6 -Compress
"#;
    let output = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .output()
        .map_err(|error| format!("failed to run powershell CIM snapshot: {error}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        return Err(first_non_empty_owned([
            stderr,
            stdout,
            format!("powershell exited with {}", output.status),
        ]));
    }
    serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("failed to parse powershell CIM JSON: {error}"))
}

#[cfg(target_os = "windows")]
struct WindowsCimCollector {
    root: Value,
    assets: Vec<AssetEntry>,
    diagnostics: Vec<AssetDiagnostic>,
}

#[cfg(target_os = "windows")]
impl WindowsCimCollector {
    fn collect_board(&mut self) {
        let mut count = 0usize;
        for item in windows_json_rows(&self.root, "board") {
            let product = windows_value_string(item, &["Product"]);
            let vendor = windows_value_string(item, &["Manufacturer"]);
            let version = windows_value_string(item, &["Version"]);
            let serial = windows_value_string(item, &["SerialNumber"]);
            let name = first_non_empty_owned([product.clone(), "Motherboard".to_string()]);
            let unique_seed = first_meaningful([
                serial.as_str(),
                product.as_str(),
                version.as_str(),
                vendor.as_str(),
            ])
            .to_string();
            self.assets.push(windows_asset_entry(
                "motherboard",
                unique_seed.as_str(),
                name,
                vendor,
                product,
                serial,
                version,
            ));
            count += 1;
        }
        self.push_diag("motherboard", count > 0, count, "no motherboard detected");
    }

    fn collect_bios(&mut self) {
        let mut count = 0usize;
        for item in windows_json_rows(&self.root, "bios") {
            let vendor = windows_value_string(item, &["Manufacturer"]);
            let smbios = windows_value_string(item, &["SMBIOSBIOSVersion"]);
            let version = windows_value_string(item, &["Version"]);
            let serial = windows_value_string(item, &["SerialNumber"]);
            let release = windows_value_string(item, &["ReleaseDate"]);
            let model = first_non_empty_owned([smbios.clone(), version.clone()]);
            let name = first_non_empty_owned([
                join_non_empty(" ", [vendor.as_str(), model.as_str()]),
                "BIOS".to_string(),
            ]);
            let specification = join_non_empty(" / ", [version.as_str(), release.as_str()]);
            let unique_seed = first_meaningful([
                serial.as_str(),
                model.as_str(),
                release.as_str(),
                name.as_str(),
            ])
            .to_string();
            self.assets.push(windows_asset_entry(
                "bios",
                unique_seed.as_str(),
                name,
                vendor,
                model,
                serial,
                specification,
            ));
            count += 1;
        }
        self.push_diag("bios", count > 0, count, "no bios detected");
    }

    fn collect_cpu(&mut self) {
        let mut count = 0usize;
        for item in windows_json_rows(&self.root, "cpu") {
            let name = windows_value_string(item, &["Name"]);
            let vendor = windows_value_string(item, &["Manufacturer"]);
            let processor_id = windows_value_string(item, &["ProcessorId"]);
            let cores = windows_value_u64(item, &["NumberOfCores"])
                .map(|value| format_count(value as u32, "physical cores"))
                .unwrap_or_default();
            let logical = windows_value_u64(item, &["NumberOfLogicalProcessors"])
                .map(|value| format_count(value as u32, "logical cores"))
                .unwrap_or_default();
            let frequency = windows_value_u64(item, &["MaxClockSpeed"])
                .map(|value| format!("max {value} MHz"))
                .unwrap_or_default();
            let specification = join_non_empty(
                " / ",
                [cores.as_str(), logical.as_str(), frequency.as_str()],
            );
            let asset_name = first_non_empty_owned([name.clone(), "CPU".to_string()]);
            let unique_seed =
                first_meaningful([processor_id.as_str(), name.as_str(), specification.as_str()])
                    .to_string();
            self.assets.push(windows_asset_entry(
                "cpu",
                unique_seed.as_str(),
                asset_name,
                vendor,
                name,
                processor_id,
                specification,
            ));
            count += 1;
        }
        self.push_diag("cpu", count > 0, count, "no cpu detected");
    }

    fn collect_memory(&mut self) {
        let mut count = 0usize;
        for item in windows_json_rows(&self.root, "memory") {
            let size = windows_value_u64(item, &["Capacity"])
                .map(format_bytes)
                .unwrap_or_default();
            let vendor = windows_value_string(item, &["Manufacturer"]);
            let part_number = windows_value_string(item, &["PartNumber"]);
            let serial = windows_value_string(item, &["SerialNumber"]);
            let locator = windows_value_string(item, &["DeviceLocator"]);
            let bank = windows_value_string(item, &["BankLabel"]);
            let speed = windows_memory_speed(
                windows_value_u64(item, &["Speed"]),
                windows_value_u64(item, &["ConfiguredClockSpeed"]),
            );
            let memory_type = windows_memory_type(
                windows_value_u64(item, &["SMBIOSMemoryType"])
                    .or_else(|| windows_value_u64(item, &["MemoryType"])),
            );
            let location = join_non_empty(" ", [bank.as_str(), locator.as_str()]);
            let specification = join_non_empty(
                " / ",
                [
                    size.as_str(),
                    memory_type.as_str(),
                    speed.as_str(),
                    location.as_str(),
                ],
            );
            let asset_name = first_non_empty_owned([
                join_non_empty(" ", [size.as_str(), memory_type.as_str()]),
                "Memory".to_string(),
            ]);
            let unique_seed = first_meaningful([
                serial.as_str(),
                locator.as_str(),
                part_number.as_str(),
                specification.as_str(),
            ])
            .to_string();
            self.assets.push(windows_asset_entry(
                "memory",
                unique_seed.as_str(),
                asset_name,
                vendor,
                first_non_empty_owned([part_number, memory_type]),
                serial,
                specification,
            ));
            count += 1;
        }
        if count == 0 {
            for item in windows_json_rows(&self.root, "system") {
                if let Some(total) = windows_value_u64(item, &["TotalPhysicalMemory"]) {
                    let size = format_bytes(total);
                    self.assets.push(windows_asset_entry(
                        "memory",
                        size.as_str(),
                        "System Memory".to_string(),
                        windows_value_string(item, &["Manufacturer"]),
                        "RAM".to_string(),
                        String::new(),
                        size.clone(),
                    ));
                    count += 1;
                    break;
                }
            }
        }
        self.push_diag("memory", count > 0, count, "no memory detected");
    }

    fn collect_gpus(&mut self) {
        let mut count = 0usize;
        for item in windows_json_rows(&self.root, "gpu") {
            let name = windows_value_string(item, &["Name"]);
            let vendor = windows_value_string(item, &["AdapterCompatibility"]);
            let driver = windows_value_string(item, &["DriverVersion"]);
            let pnp = windows_value_string(item, &["PNPDeviceID"]);
            let memory = windows_value_u64(item, &["AdapterRAM"])
                .map(|value| format!("VRAM {}", format_bytes(value)))
                .unwrap_or_default();
            let specification = join_non_empty(" / ", [memory.as_str(), driver.as_str()]);
            let asset_name = first_non_empty_owned([name.clone(), "GPU".to_string()]);
            let unique_seed =
                first_meaningful([pnp.as_str(), name.as_str(), vendor.as_str()]).to_string();
            self.assets.push(windows_asset_entry(
                "gpu",
                unique_seed.as_str(),
                asset_name,
                vendor,
                name,
                String::new(),
                specification,
            ));
            count += 1;
        }
        self.push_diag("gpu", count > 0, count, "no gpu detected");
    }

    fn collect_network(&mut self) {
        let mut count = 0usize;
        for item in windows_json_rows(&self.root, "network") {
            let name = windows_value_string(item, &["Name"]);
            let vendor = windows_value_string(item, &["Manufacturer"]);
            let mac = windows_value_string(item, &["MACAddress"]);
            let speed = windows_value_u64(item, &["Speed"])
                .map(windows_network_speed)
                .unwrap_or_default();
            let connection = windows_value_string(item, &["NetConnectionID"]);
            let pnp = windows_value_string(item, &["PNPDeviceID"]);
            let specification =
                join_non_empty(" / ", [mac.as_str(), speed.as_str(), connection.as_str()]);
            let unique_seed =
                first_meaningful([mac.as_str(), pnp.as_str(), name.as_str()]).to_string();
            self.assets.push(windows_asset_entry(
                "network",
                unique_seed.as_str(),
                first_non_empty_owned([name, "Network Interface".to_string()]),
                vendor,
                String::new(),
                mac,
                specification,
            ));
            count += 1;
        }
        self.push_diag("network", count > 0, count, "no network interface detected");
    }

    fn collect_disks(&mut self) {
        let mut count = 0usize;
        for item in windows_json_rows(&self.root, "disk") {
            let model = windows_value_string(item, &["Model"]);
            let vendor = windows_value_string(item, &["Manufacturer"]);
            let serial = windows_value_string(item, &["SerialNumber"]);
            let size = windows_value_u64(item, &["Size"])
                .map(format_bytes)
                .unwrap_or_default();
            let interface_type = windows_value_string(item, &["InterfaceType"]);
            let media_type = windows_value_string(item, &["MediaType"]);
            let firmware = windows_value_string(item, &["FirmwareRevision"]);
            let device_id = windows_value_string(item, &["DeviceID"]);
            let pnp = windows_value_string(item, &["PNPDeviceID"]);
            let specification = join_non_empty(
                " / ",
                [
                    size.as_str(),
                    media_type.as_str(),
                    interface_type.as_str(),
                    firmware.as_str(),
                    device_id.as_str(),
                ],
            );
            let unique_seed = first_meaningful([
                serial.as_str(),
                pnp.as_str(),
                device_id.as_str(),
                model.as_str(),
            ])
            .to_string();
            self.assets.push(windows_asset_entry(
                "disk",
                unique_seed.as_str(),
                first_non_empty_owned([model.clone(), device_id, "Disk".to_string()]),
                vendor,
                model,
                serial,
                specification,
            ));
            count += 1;
        }
        self.push_diag("disk", count > 0, count, "no disk detected");
    }

    fn push_diag(&mut self, name: &str, ok: bool, count: usize, fallback_message: &str) {
        self.diagnostics.push(AssetDiagnostic {
            name: name.to_string(),
            ok,
            count,
            message: if ok {
                None
            } else {
                Some(fallback_message.to_string())
            },
        });
    }
}

#[cfg(target_os = "windows")]
fn windows_asset_entry(
    asset_type: &str,
    unique_seed: &str,
    asset_name: String,
    brand: String,
    model: String,
    serial_no: String,
    specification: String,
) -> AssetEntry {
    asset_entry_with_source(
        asset_type,
        unique_seed,
        asset_name,
        brand,
        model,
        serial_no,
        specification,
        "windows-cim",
    )
}

#[cfg(target_os = "windows")]
fn windows_json_rows<'a>(root: &'a Value, key: &str) -> Vec<&'a Value> {
    match root.get(key) {
        Some(Value::Array(items)) => items.iter().collect(),
        Some(Value::Object(map)) if !map.is_empty() => vec![root.get(key).unwrap()],
        _ => Vec::new(),
    }
}

#[cfg(target_os = "windows")]
fn windows_value_string(item: &Value, keys: &[&str]) -> String {
    for key in keys {
        if let Some(value) = item.get(*key) {
            let text = match value {
                Value::String(value) => value.trim().to_string(),
                Value::Number(value) => value.to_string(),
                Value::Bool(value) => value.to_string(),
                _ => String::new(),
            };
            let text = normalize_windows_wmi_value(&text);
            if !first_meaningful([text.as_str()]).is_empty() {
                return text;
            }
        }
    }
    String::new()
}

#[cfg(target_os = "windows")]
fn windows_value_u64(item: &Value, keys: &[&str]) -> Option<u64> {
    for key in keys {
        if let Some(value) = item.get(*key) {
            match value {
                Value::Number(number) => {
                    if let Some(value) = number.as_u64() {
                        return Some(value);
                    }
                    if let Some(value) = number.as_i64().filter(|value| *value > 0) {
                        return Some(value as u64);
                    }
                }
                Value::String(text) => {
                    let digits = text.trim().replace(',', "");
                    if let Ok(value) = digits.parse::<u64>() {
                        return Some(value);
                    }
                }
                _ => {}
            }
        }
    }
    None
}

#[cfg(target_os = "windows")]
fn normalize_windows_wmi_value(value: &str) -> String {
    let value = value.trim();
    if value.eq_ignore_ascii_case("system.string[]") {
        return String::new();
    }
    value.to_string()
}

#[cfg(target_os = "windows")]
fn windows_memory_speed(speed: Option<u64>, configured_speed: Option<u64>) -> String {
    match (speed, configured_speed) {
        (Some(max), Some(running)) if max > 0 && running > 0 && max != running => {
            format!("{max} MT/s, running {running} MT/s")
        }
        (Some(max), _) if max > 0 => format!("{max} MT/s"),
        (_, Some(running)) if running > 0 => format!("{running} MT/s"),
        _ => String::new(),
    }
}

#[cfg(target_os = "windows")]
fn windows_network_speed(bits_per_second: u64) -> String {
    if bits_per_second >= 1_000_000_000 {
        format!("{:.1} Gb/s", bits_per_second as f64 / 1_000_000_000.0)
    } else if bits_per_second >= 1_000_000 {
        format!("{:.0} Mb/s", bits_per_second as f64 / 1_000_000.0)
    } else if bits_per_second > 0 {
        format!("{bits_per_second} b/s")
    } else {
        String::new()
    }
}

#[cfg(target_os = "windows")]
fn windows_memory_type(value: Option<u64>) -> String {
    match value {
        Some(20) => "DDR".to_string(),
        Some(21) => "DDR2".to_string(),
        Some(24) => "DDR3".to_string(),
        Some(26) => "DDR4".to_string(),
        Some(34) => "DDR5".to_string(),
        Some(0) | None => String::new(),
        Some(value) => format!("MemoryType {value}"),
    }
}

// fastfetch-sys links the native detection library, but its wrapper header does
// not expose these detection result structures to bindgen.
#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
#[repr(C)]
struct FastfetchCpuCore {
    freq: u32,
    count: u32,
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
#[repr(C)]
#[allow(non_snake_case)]
struct FastfetchCpuResult {
    name: fastfetch_sys::FFstrbuf,
    vendor: fastfetch_sys::FFstrbuf,
    packages: u16,
    coresPhysical: u16,
    coresLogical: u16,
    coresOnline: u16,
    frequencyBase: u32,
    frequencyMax: u32,
    coreTypes: [FastfetchCpuCore; 16],
    temperature: f64,
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
#[repr(C)]
struct FastfetchBoardResult {
    name: fastfetch_sys::FFstrbuf,
    vendor: fastfetch_sys::FFstrbuf,
    version: fastfetch_sys::FFstrbuf,
    serial: fastfetch_sys::FFstrbuf,
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
#[repr(C)]
struct FastfetchBiosResult {
    date: fastfetch_sys::FFstrbuf,
    release: fastfetch_sys::FFstrbuf,
    vendor: fastfetch_sys::FFstrbuf,
    version: fastfetch_sys::FFstrbuf,
    bios_type: fastfetch_sys::FFstrbuf,
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
#[repr(C)]
#[allow(non_snake_case)]
struct FastfetchPhysicalMemoryResult {
    size: u64,
    maxSpeed: u32,
    runningSpeed: u32,
    memory_type: fastfetch_sys::FFstrbuf,
    formFactor: fastfetch_sys::FFstrbuf,
    locator: fastfetch_sys::FFstrbuf,
    partNumber: fastfetch_sys::FFstrbuf,
    vendor: fastfetch_sys::FFstrbuf,
    serial: fastfetch_sys::FFstrbuf,
    ecc: bool,
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
#[repr(C)]
struct FastfetchGpuMemory {
    total: u64,
    used: u64,
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
#[repr(C)]
#[allow(non_snake_case)]
struct FastfetchGpuResult {
    index: u32,
    gpu_type: fastfetch_sys::FFGPUType,
    vendor: fastfetch_sys::FFstrbuf,
    name: fastfetch_sys::FFstrbuf,
    driver: fastfetch_sys::FFstrbuf,
    platformApi: fastfetch_sys::FFstrbuf,
    memoryType: fastfetch_sys::FFstrbuf,
    temperature: f64,
    coreUsage: f64,
    coreCount: i32,
    frequency: u32,
    dedicated: FastfetchGpuMemory,
    shared: FastfetchGpuMemory,
    deviceId: u64,
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
#[repr(C)]
#[allow(non_snake_case)]
struct FastfetchLocalIpResult {
    name: fastfetch_sys::FFstrbuf,
    ipv4: fastfetch_sys::FFstrbuf,
    ipv6: fastfetch_sys::FFstrbuf,
    mac: fastfetch_sys::FFstrbuf,
    flags: fastfetch_sys::FFstrbuf,
    mtu: i32,
    speed: i32,
    defaultRoute: bool,
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
type FastfetchPhysicalDiskType = u8;
#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
const FASTFETCH_PHYSICAL_DISK_HDD: FastfetchPhysicalDiskType = 1 << 0;
#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
const FASTFETCH_PHYSICAL_DISK_SSD: FastfetchPhysicalDiskType = 1 << 1;
#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
const FASTFETCH_PHYSICAL_DISK_FIXED: FastfetchPhysicalDiskType = 1 << 2;
#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
const FASTFETCH_PHYSICAL_DISK_REMOVABLE: FastfetchPhysicalDiskType = 1 << 3;
#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
const FASTFETCH_PHYSICAL_DISK_READONLY: FastfetchPhysicalDiskType = 1 << 5;

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
#[repr(C)]
#[allow(non_snake_case)]
struct FastfetchPhysicalDiskResult {
    name: fastfetch_sys::FFstrbuf,
    interconnect: fastfetch_sys::FFstrbuf,
    serial: fastfetch_sys::FFstrbuf,
    devPath: fastfetch_sys::FFstrbuf,
    revision: fastfetch_sys::FFstrbuf,
    disk_type: FastfetchPhysicalDiskType,
    size: u64,
    temperature: f64,
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
extern "C" {
    fn ffDetectBoard(board: *mut FastfetchBoardResult) -> *const c_char;
    fn ffDetectBios(bios: *mut FastfetchBiosResult) -> *const c_char;
    fn ffDetectCPU(
        options: *const fastfetch_sys::FFCPUOptions,
        cpu: *mut FastfetchCpuResult,
    ) -> *const c_char;
    fn ffDetectPhysicalMemory(result: *mut fastfetch_sys::FFlist) -> *const c_char;
    fn ffDetectGPU(
        options: *const fastfetch_sys::FFGPUOptions,
        result: *mut fastfetch_sys::FFlist,
    ) -> *const c_char;
    fn ffDetectLocalIps(
        options: *const fastfetch_sys::FFLocalIpOptions,
        result: *mut fastfetch_sys::FFlist,
    ) -> *const c_char;
    fn ffDetectPhysicalDisk(
        result: *mut fastfetch_sys::FFlist,
        options: *mut fastfetch_sys::FFPhysicalDiskOptions,
    ) -> *const c_char;
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
struct FastfetchCollector {
    assets: Vec<AssetEntry>,
    diagnostics: Vec<AssetDiagnostic>,
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
impl FastfetchCollector {
    fn collect_board(&mut self) {
        unsafe {
            let mut board = FastfetchBoard::new();
            let error = ffDetectBoard(&mut board.raw);
            if !error.is_null() {
                self.push_diag("motherboard", false, 0, Some(c_error(error)));
                return;
            }

            let name = board.name();
            if name.is_empty() {
                self.push_diag(
                    "motherboard",
                    false,
                    0,
                    Some("no motherboard detected".to_string()),
                );
                return;
            }

            let vendor = board.vendor();
            let version = board.version();
            let serial = board.serial();
            let unique_seed =
                first_meaningful([serial.as_str(), name.as_str(), version.as_str()]).to_string();
            self.assets.push(asset_entry(
                "motherboard",
                unique_seed.as_str(),
                name.clone(),
                vendor,
                version,
                serial,
                String::new(),
            ));
            self.push_diag("motherboard", true, 1, None);
        }
    }

    fn collect_bios(&mut self) {
        unsafe {
            let mut bios = FastfetchBios::new();
            let error = ffDetectBios(&mut bios.raw);
            if !error.is_null() {
                self.push_diag("bios", false, 0, Some(c_error(error)));
                return;
            }

            let version = bios.version();
            let release = bios.release();
            let bios_type = bios.bios_type();
            let name = join_non_empty(
                " ",
                [bios_type.as_str(), version.as_str(), release.as_str()],
            );
            if name.is_empty() {
                self.push_diag("bios", false, 0, Some("no bios detected".to_string()));
                return;
            }

            let date = bios.date();
            let unique_seed = first_meaningful([
                version.as_str(),
                release.as_str(),
                date.as_str(),
                name.as_str(),
            ])
            .to_string();
            let specification = join_non_empty(" / ", [release.as_str(), date.as_str()]);
            self.assets.push(asset_entry(
                "bios",
                unique_seed.as_str(),
                name,
                bios.vendor(),
                version,
                String::new(),
                specification,
            ));
            self.push_diag("bios", true, 1, None);
        }
    }

    fn collect_cpu(&mut self) {
        unsafe {
            let mut cpu = FastfetchCpu::new();
            let options = mem::zeroed::<fastfetch_sys::FFCPUOptions>();
            let error = ffDetectCPU(&options, &mut cpu.raw);
            if !error.is_null() {
                self.push_diag("cpu", false, 0, Some(c_error(error)));
                return;
            }

            let name = cpu.name();
            if name.is_empty() && cpu.raw.coresOnline <= 1 {
                self.push_diag("cpu", false, 0, Some("no cpu detected".to_string()));
                return;
            }

            let display_name = if name.is_empty() {
                "CPU".to_string()
            } else if cpu.raw.packages > 1 {
                format!("{} x {}", cpu.raw.packages, name)
            } else {
                name.clone()
            };
            let specification = cpu_specification(&cpu.raw);
            let unique_seed =
                first_meaningful([name.as_str(), display_name.as_str(), specification.as_str()])
                    .to_string();
            self.assets.push(asset_entry(
                "cpu",
                unique_seed.as_str(),
                display_name,
                cpu.vendor(),
                name,
                String::new(),
                specification,
            ));
            self.push_diag("cpu", true, 1, None);
        }
    }

    fn collect_memory(&mut self) {
        unsafe {
            let mut list =
                FastfetchList::new::<FastfetchPhysicalMemoryResult>(destroy_physical_memory);
            let error = ffDetectPhysicalMemory(&mut list.raw);
            if !error.is_null() {
                self.push_diag("memory", false, 0, Some(c_error(error)));
                return;
            }

            for i in 0..list.raw.length {
                let memory = list.get::<FastfetchPhysicalMemoryResult>(i);
                if memory.is_null() {
                    continue;
                }
                let memory = &*memory;
                let size = format_bytes(memory.size);
                let memory_type = ffstrbuf_to_string(&memory.memory_type);
                let locator = ffstrbuf_to_string(&memory.locator);
                let part_number = ffstrbuf_to_string(&memory.partNumber);
                let vendor = ffstrbuf_to_string(&memory.vendor);
                let serial = ffstrbuf_to_string(&memory.serial);
                let speed = memory_speed(memory.maxSpeed, memory.runningSpeed);
                let ecc = if memory.ecc { "ECC" } else { "" };
                let specification = join_non_empty(
                    " / ",
                    [
                        size.as_str(),
                        memory_type.as_str(),
                        speed.as_str(),
                        ecc,
                        locator.as_str(),
                    ],
                );
                let asset_name = first_non_empty_owned([
                    join_non_empty(" ", [size.as_str(), memory_type.as_str()]),
                    "Memory".to_string(),
                ]);
                let unique_seed = first_meaningful([
                    serial.as_str(),
                    locator.as_str(),
                    part_number.as_str(),
                    specification.as_str(),
                ])
                .to_string();
                self.assets.push(asset_entry(
                    "memory",
                    unique_seed.as_str(),
                    asset_name,
                    vendor,
                    first_non_empty_owned([part_number, memory_type]),
                    serial,
                    specification,
                ));
            }
            self.push_diag(
                "memory",
                list.raw.length > 0,
                list.raw.length as usize,
                None,
            );
        }
    }

    fn collect_gpus(&mut self) {
        unsafe {
            let mut list = FastfetchList::new::<FastfetchGpuResult>(destroy_gpu);
            let options = mem::zeroed::<fastfetch_sys::FFGPUOptions>();
            let error = ffDetectGPU(&options, &mut list.raw);
            if !error.is_null() {
                self.push_diag("gpu", false, 0, Some(c_error(error)));
                return;
            }

            for i in 0..list.raw.length {
                let gpu = list.get::<FastfetchGpuResult>(i);
                if gpu.is_null() {
                    continue;
                }
                let gpu = &*gpu;
                let vendor = ffstrbuf_to_string(&gpu.vendor);
                let name = ffstrbuf_to_string(&gpu.name);
                let driver = ffstrbuf_to_string(&gpu.driver);
                let api = ffstrbuf_to_string(&gpu.platformApi);
                let memory_type = ffstrbuf_to_string(&gpu.memoryType);
                let memory = gpu_memory(gpu);
                let device_id = if gpu.deviceId > 0 {
                    gpu.deviceId.to_string()
                } else {
                    String::new()
                };
                let specification = join_non_empty(
                    " / ",
                    [
                        gpu_type(gpu.raw_type()).as_str(),
                        memory.as_str(),
                        memory_type.as_str(),
                        driver.as_str(),
                        api.as_str(),
                    ],
                );
                let asset_name = first_non_empty_owned([name.clone(), "GPU".to_string()]);
                let unique_seed =
                    first_meaningful([device_id.as_str(), name.as_str(), vendor.as_str()])
                        .to_string();
                self.assets.push(asset_entry(
                    "gpu",
                    unique_seed.as_str(),
                    asset_name,
                    vendor,
                    name,
                    String::new(),
                    specification,
                ));
            }
            self.push_diag("gpu", list.raw.length > 0, list.raw.length as usize, None);
        }
    }

    fn collect_network(&mut self) {
        unsafe {
            let mut list = FastfetchList::new::<FastfetchLocalIpResult>(destroy_local_ip);
            let mut options = mem::zeroed::<fastfetch_sys::FFLocalIpOptions>();
            options.showType = (fastfetch_sys::FFLocalIpType_FF_LOCALIP_TYPE_IPV4_BIT
                | fastfetch_sys::FFLocalIpType_FF_LOCALIP_TYPE_IPV6_BIT
                | fastfetch_sys::FFLocalIpType_FF_LOCALIP_TYPE_MAC_BIT
                | fastfetch_sys::FFLocalIpType_FF_LOCALIP_TYPE_MTU_BIT
                | fastfetch_sys::FFLocalIpType_FF_LOCALIP_TYPE_SPEED_BIT
                | fastfetch_sys::FFLocalIpType_FF_LOCALIP_TYPE_FLAGS_BIT)
                as fastfetch_sys::FFLocalIpType;
            let error = ffDetectLocalIps(&options, &mut list.raw);
            if !error.is_null() {
                self.push_diag("network", false, 0, Some(c_error(error)));
                return;
            }

            for i in 0..list.raw.length {
                let nic = list.get::<FastfetchLocalIpResult>(i);
                if nic.is_null() {
                    continue;
                }
                let nic = &*nic;
                let name = ffstrbuf_to_string(&nic.name);
                let mac = ffstrbuf_to_string(&nic.mac);
                let ipv4 = ffstrbuf_to_string(&nic.ipv4);
                let ipv6 = ffstrbuf_to_string(&nic.ipv6);
                let flags = ffstrbuf_to_string(&nic.flags);
                let speed = if nic.speed > 0 {
                    format!("{} Mb/s", nic.speed)
                } else {
                    String::new()
                };
                let mtu = if nic.mtu > 0 {
                    format!("MTU {}", nic.mtu)
                } else {
                    String::new()
                };
                let default_route = if nic.defaultRoute {
                    "default route"
                } else {
                    ""
                };
                let specification = join_non_empty(
                    " / ",
                    [
                        ipv4.as_str(),
                        ipv6.as_str(),
                        mac.as_str(),
                        speed.as_str(),
                        mtu.as_str(),
                        flags.as_str(),
                        default_route,
                    ],
                );
                let unique_seed =
                    first_meaningful([mac.as_str(), name.as_str(), ipv4.as_str(), ipv6.as_str()])
                        .to_string();
                self.assets.push(asset_entry(
                    "network",
                    unique_seed.as_str(),
                    first_non_empty_owned([name, "Network Interface".to_string()]),
                    String::new(),
                    String::new(),
                    mac,
                    specification,
                ));
            }
            self.push_diag(
                "network",
                list.raw.length > 0,
                list.raw.length as usize,
                None,
            );
        }
    }

    fn collect_disks(&mut self) {
        unsafe {
            let mut list = FastfetchList::new::<FastfetchPhysicalDiskResult>(destroy_physical_disk);
            let mut options = mem::zeroed::<fastfetch_sys::FFPhysicalDiskOptions>();
            let error = ffDetectPhysicalDisk(&mut list.raw, &mut options);
            if !error.is_null() {
                self.push_diag("disk", false, 0, Some(c_error(error)));
                return;
            }

            for i in 0..list.raw.length {
                let disk = list.get::<FastfetchPhysicalDiskResult>(i);
                if disk.is_null() {
                    continue;
                }
                let disk = &*disk;
                let name = ffstrbuf_to_string(&disk.name);
                let interconnect = ffstrbuf_to_string(&disk.interconnect);
                let serial = ffstrbuf_to_string(&disk.serial);
                let dev_path = ffstrbuf_to_string(&disk.devPath);
                let revision = ffstrbuf_to_string(&disk.revision);
                let size = format_bytes(disk.size);
                let disk_type = physical_disk_type(disk.raw_type());
                let specification = join_non_empty(
                    " / ",
                    [
                        size.as_str(),
                        disk_type.as_str(),
                        interconnect.as_str(),
                        revision.as_str(),
                        dev_path.as_str(),
                    ],
                );
                let unique_seed =
                    first_meaningful([serial.as_str(), dev_path.as_str(), name.as_str()])
                        .to_string();
                self.assets.push(asset_entry(
                    "disk",
                    unique_seed.as_str(),
                    first_non_empty_owned([name.clone(), dev_path.clone(), "Disk".to_string()]),
                    String::new(),
                    name,
                    serial,
                    specification,
                ));
            }
            self.push_diag("disk", list.raw.length > 0, list.raw.length as usize, None);
        }
    }

    fn push_diag(&mut self, name: &str, ok: bool, count: usize, message: Option<String>) {
        self.diagnostics.push(AssetDiagnostic {
            name: name.to_string(),
            ok,
            count,
            message,
        });
    }
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
struct FastfetchCpu {
    raw: FastfetchCpuResult,
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
impl FastfetchCpu {
    unsafe fn new() -> Self {
        let mut raw = mem::zeroed::<FastfetchCpuResult>();
        init_strbuf(&mut raw.name);
        init_strbuf(&mut raw.vendor);
        raw.temperature = f64::NAN;
        Self { raw }
    }

    fn name(&self) -> String {
        ffstrbuf_to_string(&self.raw.name)
    }

    fn vendor(&self) -> String {
        ffstrbuf_to_string(&self.raw.vendor)
    }
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
impl Drop for FastfetchCpu {
    fn drop(&mut self) {
        unsafe {
            destroy_strbuf(&mut self.raw.name);
            destroy_strbuf(&mut self.raw.vendor);
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
struct FastfetchBoard {
    raw: FastfetchBoardResult,
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
impl FastfetchBoard {
    unsafe fn new() -> Self {
        let mut raw = mem::zeroed::<FastfetchBoardResult>();
        init_strbuf(&mut raw.name);
        init_strbuf(&mut raw.vendor);
        init_strbuf(&mut raw.version);
        init_strbuf(&mut raw.serial);
        Self { raw }
    }

    fn name(&self) -> String {
        ffstrbuf_to_string(&self.raw.name)
    }

    fn vendor(&self) -> String {
        ffstrbuf_to_string(&self.raw.vendor)
    }

    fn version(&self) -> String {
        ffstrbuf_to_string(&self.raw.version)
    }

    fn serial(&self) -> String {
        ffstrbuf_to_string(&self.raw.serial)
    }
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
impl Drop for FastfetchBoard {
    fn drop(&mut self) {
        unsafe {
            destroy_strbuf(&mut self.raw.name);
            destroy_strbuf(&mut self.raw.vendor);
            destroy_strbuf(&mut self.raw.version);
            destroy_strbuf(&mut self.raw.serial);
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
struct FastfetchBios {
    raw: FastfetchBiosResult,
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
impl FastfetchBios {
    unsafe fn new() -> Self {
        let mut raw = mem::zeroed::<FastfetchBiosResult>();
        init_strbuf(&mut raw.date);
        init_strbuf(&mut raw.release);
        init_strbuf(&mut raw.vendor);
        init_strbuf(&mut raw.version);
        init_strbuf(&mut raw.bios_type);
        Self { raw }
    }

    fn date(&self) -> String {
        ffstrbuf_to_string(&self.raw.date)
    }

    fn release(&self) -> String {
        ffstrbuf_to_string(&self.raw.release)
    }

    fn vendor(&self) -> String {
        ffstrbuf_to_string(&self.raw.vendor)
    }

    fn version(&self) -> String {
        ffstrbuf_to_string(&self.raw.version)
    }

    fn bios_type(&self) -> String {
        ffstrbuf_to_string(&self.raw.bios_type)
    }
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
impl Drop for FastfetchBios {
    fn drop(&mut self) {
        unsafe {
            destroy_strbuf(&mut self.raw.date);
            destroy_strbuf(&mut self.raw.release);
            destroy_strbuf(&mut self.raw.vendor);
            destroy_strbuf(&mut self.raw.version);
            destroy_strbuf(&mut self.raw.bios_type);
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
struct FastfetchList {
    raw: fastfetch_sys::FFlist,
    destroy_item: Option<unsafe fn(*mut u8)>,
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
impl FastfetchList {
    unsafe fn new<T>(destroy_item: unsafe fn(*mut u8)) -> Self {
        let mut raw = mem::zeroed::<fastfetch_sys::FFlist>();
        raw.elementSize = mem::size_of::<T>() as u32;
        Self {
            raw,
            destroy_item: Some(destroy_item),
        }
    }

    unsafe fn get<T>(&self, index: u32) -> *const T {
        self.raw
            .data
            .add(index as usize * self.raw.elementSize as usize) as *const T
    }
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
impl Drop for FastfetchList {
    fn drop(&mut self) {
        unsafe {
            if !self.raw.data.is_null() {
                if let Some(destroy_item) = self.destroy_item {
                    for index in 0..self.raw.length {
                        destroy_item(
                            self.raw
                                .data
                                .add(index as usize * self.raw.elementSize as usize),
                        );
                    }
                }
                free(self.raw.data.cast());
                self.raw.data = ptr::null_mut();
            }
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
unsafe fn init_strbuf(value: &mut fastfetch_sys::FFstrbuf) {
    fastfetch_sys::ffStrbufInitA(value, 0);
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
unsafe fn destroy_strbuf(value: &mut fastfetch_sys::FFstrbuf) {
    if value.allocated != 0 && !value.chars.is_null() {
        free(value.chars.cast());
    }
    value.allocated = 0;
    value.length = 0;
    value.chars = ptr::null_mut();
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
unsafe fn destroy_physical_memory(item: *mut u8) {
    let item = item.cast::<FastfetchPhysicalMemoryResult>();
    destroy_strbuf(&mut (*item).memory_type);
    destroy_strbuf(&mut (*item).formFactor);
    destroy_strbuf(&mut (*item).locator);
    destroy_strbuf(&mut (*item).partNumber);
    destroy_strbuf(&mut (*item).vendor);
    destroy_strbuf(&mut (*item).serial);
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
unsafe fn destroy_gpu(item: *mut u8) {
    let item = item.cast::<FastfetchGpuResult>();
    destroy_strbuf(&mut (*item).vendor);
    destroy_strbuf(&mut (*item).name);
    destroy_strbuf(&mut (*item).driver);
    destroy_strbuf(&mut (*item).platformApi);
    destroy_strbuf(&mut (*item).memoryType);
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
unsafe fn destroy_local_ip(item: *mut u8) {
    let item = item.cast::<FastfetchLocalIpResult>();
    destroy_strbuf(&mut (*item).name);
    destroy_strbuf(&mut (*item).ipv4);
    destroy_strbuf(&mut (*item).ipv6);
    destroy_strbuf(&mut (*item).mac);
    destroy_strbuf(&mut (*item).flags);
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
unsafe fn destroy_physical_disk(item: *mut u8) {
    let item = item.cast::<FastfetchPhysicalDiskResult>();
    destroy_strbuf(&mut (*item).name);
    destroy_strbuf(&mut (*item).interconnect);
    destroy_strbuf(&mut (*item).serial);
    destroy_strbuf(&mut (*item).devPath);
    destroy_strbuf(&mut (*item).revision);
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
trait FastfetchGpuExt {
    fn raw_type(&self) -> fastfetch_sys::FFGPUType;
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
impl FastfetchGpuExt for FastfetchGpuResult {
    fn raw_type(&self) -> fastfetch_sys::FFGPUType {
        self.gpu_type
    }
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
trait FastfetchDiskExt {
    fn raw_type(&self) -> FastfetchPhysicalDiskType;
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
impl FastfetchDiskExt for FastfetchPhysicalDiskResult {
    fn raw_type(&self) -> FastfetchPhysicalDiskType {
        self.disk_type
    }
}

#[cfg(target_os = "linux")]
fn ensure_linux_fallback_assets(
    assets: &mut Vec<AssetEntry>,
    diagnostics: &mut Vec<AssetDiagnostic>,
) {
    if !assets.iter().any(|item| item.asset_type == "cpu") {
        if let Ok(cpuinfo) = fs::read_to_string("/proc/cpuinfo") {
            let model = cpuinfo
                .lines()
                .find_map(|line| {
                    line.strip_prefix("model name")
                        .and_then(|v| v.split(':').nth(1))
                })
                .or_else(|| {
                    cpuinfo.lines().find_map(|line| {
                        line.strip_prefix("Hardware")
                            .and_then(|v| v.split(':').nth(1))
                    })
                })
                .map(str::trim)
                .unwrap_or("CPU");
            assets.push(asset_entry(
                "cpu",
                model,
                model.to_string(),
                String::new(),
                model.to_string(),
                String::new(),
                "fallback:/proc/cpuinfo".to_string(),
            ));
            diagnostics.push(AssetDiagnostic {
                name: "cpu-fallback".to_string(),
                ok: true,
                count: 1,
                message: None,
            });
        }
    }
    if !assets.iter().any(|item| item.asset_type == "memory") {
        if let Ok(meminfo) = fs::read_to_string("/proc/meminfo") {
            if let Some(total) = meminfo.lines().find(|line| line.starts_with("MemTotal:")) {
                assets.push(asset_entry(
                    "memory",
                    total,
                    "System Memory".to_string(),
                    String::new(),
                    "RAM".to_string(),
                    String::new(),
                    total.to_string(),
                ));
                diagnostics.push(AssetDiagnostic {
                    name: "memory-fallback".to_string(),
                    ok: true,
                    count: 1,
                    message: None,
                });
            }
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
fn asset_entry(
    asset_type: &str,
    unique_seed: &str,
    asset_name: String,
    brand: String,
    model: String,
    serial_no: String,
    specification: String,
) -> AssetEntry {
    asset_entry_with_source(
        asset_type,
        unique_seed,
        asset_name,
        brand,
        model,
        serial_no,
        specification,
        "fastfetch-sys",
    )
}

fn asset_entry_with_source(
    asset_type: &str,
    unique_seed: &str,
    asset_name: String,
    brand: String,
    model: String,
    serial_no: String,
    specification: String,
    source: &str,
) -> AssetEntry {
    let unique_key = if serial_no.trim().is_empty() {
        sanitize(first_meaningful([
            unique_seed,
            asset_name.as_str(),
            model.as_str(),
            specification.as_str(),
        ]))
    } else {
        sanitize(serial_no.as_str())
    };
    AssetEntry {
        asset_type: asset_type.to_string(),
        unique_key: if unique_key.is_empty() {
            sanitize(&format!(
                "{asset_type}-{asset_name}-{model}-{specification}"
            ))
        } else {
            unique_key
        },
        asset_name,
        brand,
        model,
        serial_no,
        specification,
        source: source.to_string(),
        sync_hash: String::new(),
        remark: format!("auto:{source}"),
    }
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
fn ffstrbuf_to_string(value: &fastfetch_sys::FFstrbuf) -> String {
    if value.chars.is_null() || value.length == 0 {
        return String::new();
    }
    unsafe {
        let bytes = std::slice::from_raw_parts(value.chars as *const u8, value.length as usize);
        String::from_utf8_lossy(bytes).trim().to_string()
    }
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
fn c_error(error: *const std::os::raw::c_char) -> String {
    if error.is_null() {
        String::new()
    } else {
        unsafe { CStr::from_ptr(error).to_string_lossy().into_owned() }
    }
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
fn cpu_specification(cpu: &FastfetchCpuResult) -> String {
    let cores = if cpu.coresPhysical > 0 || cpu.coresLogical > 0 || cpu.coresOnline > 0 {
        let packages = format_count(cpu.packages as u32, "package");
        let physical = format_count(cpu.coresPhysical as u32, "physical cores");
        let logical = format_count(cpu.coresLogical as u32, "logical cores");
        let online = format_count(cpu.coresOnline as u32, "online cores");
        join_non_empty(
            " / ",
            [
                packages.as_str(),
                physical.as_str(),
                logical.as_str(),
                online.as_str(),
            ],
        )
    } else {
        String::new()
    };
    let frequency = match (cpu.frequencyBase, cpu.frequencyMax) {
        (base, max) if base > 0 && max > 0 && base != max => {
            format!("base {} MHz / max {} MHz", base, max)
        }
        (_, max) if max > 0 => format!("max {} MHz", max),
        (base, _) if base > 0 => format!("base {} MHz", base),
        _ => String::new(),
    };
    join_non_empty(" / ", [cores.as_str(), frequency.as_str()])
}

fn format_count(count: u32, label: &str) -> String {
    if count == 0 {
        String::new()
    } else {
        format!("{count} {label}")
    }
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
fn memory_speed(max_speed: u32, running_speed: u32) -> String {
    match (max_speed, running_speed) {
        (max, running) if max > 0 && running > 0 && max != running => {
            format!("{max} MT/s, running {running} MT/s")
        }
        (max, _) if max > 0 => format!("{max} MT/s"),
        (_, running) if running > 0 => format!("{running} MT/s"),
        _ => String::new(),
    }
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
fn gpu_type(value: fastfetch_sys::FFGPUType) -> String {
    match value {
        fastfetch_sys::FFGPUType_FF_GPU_TYPE_INTEGRATED => "Integrated".to_string(),
        fastfetch_sys::FFGPUType_FF_GPU_TYPE_DISCRETE => "Discrete".to_string(),
        fastfetch_sys::FFGPUType_FF_GPU_TYPE_UNKNOWN => "Unknown".to_string(),
        _ => String::new(),
    }
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
fn gpu_memory(gpu: &FastfetchGpuResult) -> String {
    if gpu.dedicated.total != u64::MAX && gpu.dedicated.total > 0 {
        return format!("VRAM {}", format_bytes(gpu.dedicated.total));
    }
    if gpu.shared.total != u64::MAX && gpu.shared.total > 0 {
        return format!("shared {}", format_bytes(gpu.shared.total));
    }
    String::new()
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
fn physical_disk_type(value: FastfetchPhysicalDiskType) -> String {
    let raw = value as u32;
    let mut parts = Vec::new();
    if raw & FASTFETCH_PHYSICAL_DISK_SSD as u32 != 0 {
        parts.push("SSD");
    } else if raw & FASTFETCH_PHYSICAL_DISK_HDD as u32 != 0 {
        parts.push("HDD");
    }
    if raw & FASTFETCH_PHYSICAL_DISK_REMOVABLE as u32 != 0 {
        parts.push("removable");
    } else if raw & FASTFETCH_PHYSICAL_DISK_FIXED as u32 != 0 {
        parts.push("fixed");
    }
    if raw & FASTFETCH_PHYSICAL_DISK_READONLY as u32 != 0 {
        parts.push("readonly");
    }
    parts.join(" ")
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    if bytes == 0 {
        return String::new();
    }
    let mut value = bytes as f64;
    let mut unit = 0usize;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 || value >= 100.0 {
        format!("{value:.0} {}", UNITS[unit])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

fn first_meaningful<const N: usize>(values: [&str; N]) -> &str {
    values
        .into_iter()
        .find(|value| {
            let value = value.trim();
            !value.is_empty()
                && !matches!(
                    value.to_ascii_lowercase().as_str(),
                    "unknown" | "none" | "default string" | "to be filled by o.e.m."
                )
        })
        .unwrap_or("")
        .trim()
}

fn first_non_empty_owned<const N: usize>(values: [String; N]) -> String {
    values
        .into_iter()
        .find(|value| !value.trim().is_empty())
        .unwrap_or_default()
}

fn join_non_empty<const N: usize>(separator: &str, values: [&str; N]) -> String {
    values
        .into_iter()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>()
        .join(separator)
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
    cfg.access_code = None;
    if cfg
        .kms_device
        .as_ref()
        .is_some_and(|value| value.trim().is_empty())
    {
        cfg.kms_device = None;
    }
    #[cfg(target_os = "windows")]
    {
        cfg.windows_capture_source = normalize_windows_capture_source(&cfg.windows_capture_source);
    }
}

#[cfg(target_os = "windows")]
fn normalize_windows_capture_source(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "dxgi" => "dxgi".to_string(),
        "gdi" => "gdi".to_string(),
        _ => "auto".to_string(),
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
    if cfg!(target_os = "windows") {
        return "cmd.exe".to_string();
    }

    std::env::var("SHELL")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .filter(|value| !value.contains('/') || Path::new(value).exists())
        .or_else(|| {
            ["/bin/bash", "/usr/bin/bash", "/bin/sh", "/usr/bin/sh"]
                .iter()
                .find(|path| Path::new(path).exists())
                .map(|path| (*path).to_string())
        })
        .unwrap_or_else(|| "/bin/sh".to_string())
}

fn hostname() -> String {
    fs::read_to_string("/etc/hostname")
        .map(|s| s.trim().to_string())
        .ok()
        .filter(|s| !s.is_empty())
        .or_else(|| std::env::var("HOSTNAME").ok())
        .or_else(|| std::env::var("COMPUTERNAME").ok())
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

fn detect_primary_mac(ip: &str) -> String {
    #[cfg(target_os = "windows")]
    {
        let ps = format!(
            "$ip='{}'; \
             $cfg=Get-CimInstance Win32_NetworkAdapterConfiguration -Filter \"IPEnabled = True\" | \
             Where-Object {{ $_.IPAddress -contains $ip }} | Select-Object -First 1; \
             if (-not $cfg) {{ $cfg=Get-CimInstance Win32_NetworkAdapterConfiguration -Filter \"IPEnabled = True\" | Select-Object -First 1 }}; \
             if ($cfg) {{ $cfg.MACAddress }}",
            ip.replace('\'', "''")
        );
        if let Ok(output) = Command::new("powershell.exe")
            .args(["-NoProfile", "-Command", &ps])
            .output()
        {
            if output.status.success() {
                let mac = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !mac.is_empty() {
                    return mac;
                }
            }
        }
        return String::new();
    }

    #[cfg(not(target_os = "windows"))]
    {
        let target_ip = ip.parse::<IpAddr>().ok();
        let interfaces = pnet_datalink::interfaces();
        if let Some(target_ip) = target_ip {
            for interface in &interfaces {
                if interface.is_loopback() || !interface.is_up() {
                    continue;
                }
                if interface
                    .ips
                    .iter()
                    .any(|network| network.ip() == target_ip)
                {
                    if let Some(mac) = interface.mac {
                        return mac.to_string();
                    }
                }
            }
        }
        interfaces
            .into_iter()
            .filter(|interface| !interface.is_loopback() && interface.is_up())
            .find_map(|interface| interface.mac.map(|mac| mac.to_string()))
            .unwrap_or_default()
    }
}

fn detect_os_name() -> String {
    first_meaningful_string([
        System::long_os_version(),
        format_os_name_version(System::name(), System::os_version()),
        parse_os_release_name(&fs::read_to_string("/etc/os-release").unwrap_or_default()),
        std::env::var("PRETTY_NAME").ok(),
        Some(System::distribution_id()),
        Some(std::env::consts::OS.to_string()),
    ])
    .unwrap_or_else(|| std::env::consts::OS.to_string())
}

fn detect_kernel_version() -> String {
    first_meaningful_string([System::kernel_version()]).unwrap_or_default()
}

fn detect_device_type() -> String {
    if detect_virtual_machine() {
        "virtual".to_string()
    } else {
        "physical".to_string()
    }
}

#[cfg(target_os = "linux")]
fn detect_virtual_machine() -> bool {
    let values = [
        "/sys/class/dmi/id/product_name",
        "/sys/class/dmi/id/sys_vendor",
        "/sys/class/dmi/id/board_vendor",
        "/sys/class/dmi/id/bios_vendor",
        "/proc/device-tree/hypervisor/compatible",
    ]
    .iter()
    .filter_map(|path| fs::read_to_string(path).ok())
    .collect::<Vec<_>>()
    .join(" ");
    contains_virtualization_marker(&values)
}

#[cfg(target_os = "windows")]
fn detect_virtual_machine() -> bool {
    let registry_values = [
        r"HKLM\HARDWARE\DESCRIPTION\System\BIOS",
        r"HKLM\HARDWARE\DESCRIPTION\System",
    ]
    .iter()
    .filter_map(|key| Command::new("reg").args(["query", key]).output().ok())
    .filter(|output| output.status.success())
    .map(|output| String::from_utf8_lossy(&output.stdout).into_owned())
    .collect::<Vec<_>>()
    .join(" ");
    if contains_virtualization_marker(&registry_values) {
        return true;
    }
    if let Ok(output) = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-Command",
            "(Get-CimInstance Win32_ComputerSystem | Select-Object -ExpandProperty Manufacturer) + ' ' + (Get-CimInstance Win32_ComputerSystem | Select-Object -ExpandProperty Model)",
        ])
        .output()
    {
        if output.status.success()
            && contains_virtualization_marker(&String::from_utf8_lossy(&output.stdout))
        {
            return true;
        }
    }
    false
}

#[cfg(target_os = "macos")]
fn detect_virtual_machine() -> bool {
    if let Ok(output) = std::process::Command::new("sysctl")
        .args(["-n", "machdep.cpu.features"])
        .output()
    {
        if output.status.success()
            && contains_virtualization_marker(&String::from_utf8_lossy(&output.stdout))
        {
            return true;
        }
    }
    false
}

#[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
fn detect_virtual_machine() -> bool {
    false
}

fn contains_virtualization_marker(value: &str) -> bool {
    let value = value.to_ascii_lowercase();
    [
        "kvm",
        "qemu",
        "bochs",
        "xen",
        "vmware",
        "virtualbox",
        "virtual machine",
        "hyper-v",
        "microsoft corporation virtual",
        "parallels",
        "bhyve",
        "openstack",
        "cloudstack",
        "rhev",
        "ovirt",
        "amazon ec2",
        "google compute engine",
        "alibaba cloud",
        "tencent cloud",
        "azure",
    ]
    .iter()
    .any(|marker| value.contains(marker))
}

fn parse_os_release_name(content: &str) -> Option<String> {
    for key in ["PRETTY_NAME", "NAME"] {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('#') {
                continue;
            }
            let Some((line_key, value)) = line.split_once('=') else {
                continue;
            };
            if line_key.trim() != key {
                continue;
            }
            let value = value
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .trim()
                .to_string();
            if !value.is_empty() {
                return Some(value);
            }
        }
    }
    None
}

fn format_os_name_version(name: Option<String>, version: Option<String>) -> Option<String> {
    let name = name.and_then(|value| normalize_os_label_part(&value))?;
    match version.and_then(|value| normalize_os_label_part(&value)) {
        Some(version) if !label_contains_case_insensitive(&name, &version) => {
            Some(format!("{name} {version}"))
        }
        _ => Some(name),
    }
}

fn first_meaningful_string(values: impl IntoIterator<Item = Option<String>>) -> Option<String> {
    values
        .into_iter()
        .flatten()
        .filter_map(|value| normalize_os_label_part(&value))
        .find(|value| {
            let normalized = value.to_ascii_lowercase();
            !matches!(normalized.as_str(), "unknown" | "unknown os")
        })
}

fn normalize_os_label_part(value: &str) -> Option<String> {
    let value = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

fn label_contains_case_insensitive(label: &str, part: &str) -> bool {
    label
        .to_ascii_lowercase()
        .contains(&part.to_ascii_lowercase())
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

#[cfg(test)]
mod tests {
    use super::{first_meaningful_string, format_os_name_version, parse_os_release_name};

    #[test]
    fn parse_os_release_prefers_pretty_name() {
        let content = r#"
NAME="Kylin Linux Advanced Server"
PRETTY_NAME="Kylin Linux Advanced Server V10 (Lance)"
"#;
        assert_eq!(
            parse_os_release_name(content).as_deref(),
            Some("Kylin Linux Advanced Server V10 (Lance)")
        );
    }

    #[test]
    fn parse_os_release_falls_back_to_name() {
        let content = "NAME=\"UnionTech OS Desktop\"\nVERSION_ID=\"20\"\n";
        assert_eq!(
            parse_os_release_name(content).as_deref(),
            Some("UnionTech OS Desktop")
        );
    }

    #[test]
    fn os_name_version_combines_short_values() {
        assert_eq!(
            format_os_name_version(Some("Windows".to_string()), Some("11 (22631)".to_string()))
                .as_deref(),
            Some("Windows 11 (22631)")
        );
    }

    #[test]
    fn os_name_version_avoids_duplicate_version() {
        assert_eq!(
            format_os_name_version(
                Some("macOS 15.1.1 Sequoia".to_string()),
                Some("15.1.1".to_string())
            )
            .as_deref(),
            Some("macOS 15.1.1 Sequoia")
        );
    }

    #[test]
    fn first_meaningful_string_skips_empty_and_unknown() {
        assert_eq!(
            first_meaningful_string([
                Some("  ".to_string()),
                Some("Unknown".to_string()),
                Some("Linux (Ubuntu 24.04)".to_string()),
            ])
            .as_deref(),
            Some("Linux (Ubuntu 24.04)")
        );
    }
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
    input, select { width: 100%; height: 38px; border: 1px solid #c6ccd2; border-radius: 6px; padding: 0 10px; font-size: 14px; background: #fff; color: var(--text); }
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
    [data-cap] { display: none; }
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
        <a id="desktopLink" class="button secondary" href="#" target="_blank" hidden>打开远程桌面端口</a>
      </div>
    </section>
    <section>
      <h2>远程桌面</h2>
      <div class="grid">
        <div><label for="bindAddress">绑定地址</label><input id="bindAddress" placeholder="127.0.0.1" /></div>
        <div><label for="webPort">Web 端口</label><input id="webPort" type="number" min="0" max="65535" placeholder="0 表示随机本地端口" /></div>
        <div data-cap="kms"><label for="kmsDevice">KMS 设备</label><input id="kmsDevice" placeholder="/dev/dri/card0" /></div>
        <div data-cap="windowsCaptureSource"><label for="windowsCaptureSource">Windows 画面来源</label><select id="windowsCaptureSource"><option value="auto">AUTO（DXGI 优先）</option><option value="dxgi">DXGI Desktop Duplication</option><option value="gdi">GDI BitBlt</option></select></div>
      </div>
      <div class="switches" style="margin-top: 14px;">
        <label class="switch" data-cap="wayland"><input id="waylandSupport" type="checkbox" /><span><strong>Wayland / PipeWire</strong>启用 Wayland 捕获和自定义输入区域支持。</span></label>
        <label class="switch" data-cap="kms"><input id="kmsSupport" type="checkbox" /><span><strong>KMS / DRM</strong>直接通过 DRM/KMS 捕获帧缓冲。</span></label>
        <label class="switch" data-cap="nvfbc"><input id="nvfbcSupport" type="checkbox" /><span><strong>NvFBC</strong>启用 NVIDIA Frame Buffer Capture 捕获后端。</span></label>
        <label class="switch" data-cap="vaapi"><input id="tryVaapi" type="checkbox" /><span><strong>VAAPI</strong>尝试使用 Linux VAAPI 硬件编码。</span></label>
        <label class="switch" data-cap="nvenc"><input id="tryNvenc" type="checkbox" /><span><strong>NVENC</strong>尝试使用 NVIDIA NVENC 硬件编码。</span></label>
        <label class="switch" data-cap="mediafoundation"><input id="tryMediafoundation" type="checkbox" /><span><strong>MediaFoundation</strong>尝试使用 Windows 硬件编码。</span></label>
        <label class="switch" data-cap="displayManager"><input id="controlDisplayManager" type="checkbox" /><span><strong>登录界面控制</strong>root 服务启动时自动探测 DISPLAY / XAUTHORITY。</span></label>
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
        <button class="secondary" onclick="restartService()">重启系统服务</button>
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
        <dt data-cap="windowsCaptureSource">Windows 桌面</dt><dd data-cap="windowsCaptureSource" id="windowsDesktop">-</dd>
        <dt>消息</dt><dd id="message">-</dd>
      </dl>
    </section>
    <section>
      <h2>日志</h2>
      <pre id="log"></pre>
    </section>
  </main>
  <script>
    const ids = ['serverHost','deviceName','bindAddress','webPort','kmsDevice','windowsCaptureSource','waylandSupport','kmsSupport','nvfbcSupport','tryVaapi','tryNvenc','tryMediafoundation','controlDisplayManager'];
    const $ = (id) => document.getElementById(id);
    const dirty = new Set();
    ids.forEach((id) => {
      const el = $(id);
      if (!el) return;
      el.addEventListener('input', () => dirty.add(id));
      el.addEventListener('change', () => dirty.add(id));
    });
    const query = new URLSearchParams(window.location.search);
    const showDesktopLink = ['1', 'true', 'yes'].includes((query.get('showDesktopLink') || query.get('desktopLink') || query.get('debugDesktop') || '').toLowerCase());
    const log = (text) => $('log').textContent = `${new Date().toLocaleTimeString()} ${text}\n` + $('log').textContent;
    function setInput(id, value) {
      const el = $(id);
      if (document.activeElement === el || dirty.has(id)) return;
      el.value = value ?? '';
    }
    function setChecked(id, value) {
      const el = $(id);
      if (document.activeElement === el || dirty.has(id)) return;
      el.checked = !!value;
    }
    function clearDirty(...fields) {
      fields.forEach((field) => dirty.delete(field));
    }
    async function request(path, options) {
      const response = await fetch(path, options);
      const data = await response.json();
      if (!response.ok || !data.ok) throw new Error(data.message || 'request failed');
      return data;
    }
    function setBusy(busy) {
      document.querySelectorAll('button').forEach((button) => button.disabled = busy);
    }
    function renderCapabilities(caps) {
      const supported = caps || {};
      document.querySelectorAll('[data-cap]').forEach((el) => {
        const key = el.getAttribute('data-cap');
        el.style.display = supported[key] ? '' : 'none';
      });
    }
    function render(data) {
      const cfg = data.config || {};
      const status = data.status || {};
      renderCapabilities(data.capabilities || {});
      setInput('serverHost', cfg.serverHost || '');
      setInput('deviceName', cfg.deviceName || '');
      setInput('bindAddress', cfg.bindAddress || '127.0.0.1');
      setInput('webPort', cfg.webPort ?? 0);
      setInput('kmsDevice', cfg.kmsDevice || '');
      setInput('windowsCaptureSource', cfg.windowsCaptureSource || 'auto');
      setChecked('waylandSupport', !!cfg.waylandSupport);
      setChecked('kmsSupport', !!cfg.kmsSupport);
      setChecked('nvfbcSupport', !!cfg.nvfbcSupport);
      setChecked('tryVaapi', !!cfg.tryVaapi);
      setChecked('tryNvenc', !!cfg.tryNvenc);
      setChecked('tryMediafoundation', !!cfg.tryMediafoundation);
      setChecked('controlDisplayManager', cfg.controlDisplayManager !== false);
      $('state').textContent = status.state || '-';
      $('deviceId').textContent = status.deviceId || cfg.deviceId || '-';
      $('tcpAddress').textContent = status.tcpAddress || cfg.tcpAddress || '-';
      $('desktopUrl').textContent = status.desktopUrl || '-';
      $('desktopLink').href = status.desktopUrl || '#';
      $('desktopLink').hidden = !showDesktopLink;
      const winDesktop = status.windowsDesktop || {};
      $('windowsDesktop').textContent = winDesktop.inputDesktop ? `${winDesktop.inputDesktop}${winDesktop.isWinlogon ? ' (Winlogon)' : ''}` : '-';
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
      clearDirty('serverHost', 'deviceName');
      await loadStatus();
    }
    async function saveDesktopConfig() {
      await call('/api/desktop-config', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({
        bindAddress: $('bindAddress').value, webPort: Number($('webPort').value || 0),
        kmsDevice: $('kmsDevice').value || null, waylandSupport: $('waylandSupport').checked, kmsSupport: $('kmsSupport').checked, nvfbcSupport: $('nvfbcSupport').checked,
        windowsCaptureSource: $('windowsCaptureSource').value || 'auto',
        tryVaapi: $('tryVaapi').checked, tryNvenc: $('tryNvenc').checked, tryMediafoundation: $('tryMediafoundation').checked, controlDisplayManager: $('controlDisplayManager').checked
      }) }, '桌面配置已保存，重启桌面服务后生效');
      clearDirty('bindAddress','webPort','kmsDevice','windowsCaptureSource','waylandSupport','kmsSupport','nvfbcSupport','tryVaapi','tryNvenc','tryMediafoundation','controlDisplayManager');
      await loadStatus();
    }
    async function startAgent() { await call('/api/start', { method: 'POST' }, '连接已启动'); }
    async function stopAgent() { await call('/api/stop', { method: 'POST' }, '连接已停止'); }
    async function restartDesktop() { await call('/api/desktop/restart', { method: 'POST' }, '桌面服务重启请求已发送'); }
    async function enableService() { await call('/api/service/enable', { method: 'POST' }, '自启服务已启用'); await loadStatus(); }
    async function disableService() { await call('/api/service/disable', { method: 'POST' }, '自启服务已禁用'); await loadStatus(); }
    async function restartService() { await call('/api/service/restart', { method: 'POST' }, '系统服务重启请求已发送'); }
    loadStatus(); setInterval(loadStatus, 5000);
  </script>
</body>
</html>
"##;
