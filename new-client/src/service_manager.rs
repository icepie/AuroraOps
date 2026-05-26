use std::error::Error;
use std::path::PathBuf;
use std::process::Command;

pub type BoxError = Box<dyn Error + Send + Sync + 'static>;

#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
pub const SERVICE_NAME: &str = "auroraops-agent";
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
pub const SERVICE_DISPLAY_NAME: &str = "AuroraOps 客户端";

#[derive(Clone, Copy, Debug)]
pub enum ServiceAction {
    Install,
    Uninstall,
    Start,
    Stop,
    Restart,
}

pub fn default_config_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        let base = std::env::var_os("PROGRAMDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(r"C:\ProgramData"));
        return base.join("AuroraOps").join("agent-config.json");
    }

    #[cfg(not(target_os = "windows"))]
    {
        PathBuf::from("/etc/auroraops/agent-config.json")
    }
}

pub fn handle_cli_action(
    action: ServiceAction,
    config_path: Option<PathBuf>,
    port: u16,
) -> Result<String, BoxError> {
    match action {
        ServiceAction::Install => install(config_path, port),
        ServiceAction::Uninstall => uninstall(),
        ServiceAction::Start => start(),
        ServiceAction::Stop => stop(),
        ServiceAction::Restart => restart(),
    }
}

pub fn install(config_path: Option<PathBuf>, port: u16) -> Result<String, BoxError> {
    #[cfg(target_os = "windows")]
    {
        let elevated_config_path = config_path.clone().unwrap_or_else(default_config_path);
        return windows::with_elevation(
            vec![
                "--install-service".to_string(),
                "--config".to_string(),
                elevated_config_path.display().to_string(),
                "--port".to_string(),
                port.to_string(),
            ],
            || windows::install(config_path, port),
        );
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = config_path;
        let _ = port;
        run_systemctl(&["enable", "--now"])
    }
}

pub fn uninstall() -> Result<String, BoxError> {
    #[cfg(target_os = "windows")]
    {
        return windows::with_elevation(
            vec!["--uninstall-service".to_string()],
            windows::uninstall,
        );
    }

    #[cfg(not(target_os = "windows"))]
    {
        run_systemctl(&["disable", "--now"])
    }
}

pub fn start() -> Result<String, BoxError> {
    #[cfg(target_os = "windows")]
    {
        return windows::with_elevation(vec!["--start-service".to_string()], windows::start);
    }

    #[cfg(not(target_os = "windows"))]
    {
        run_systemctl(&["start"])
    }
}

pub fn stop() -> Result<String, BoxError> {
    #[cfg(target_os = "windows")]
    {
        return windows::with_elevation(vec!["--stop-service".to_string()], windows::stop);
    }

    #[cfg(not(target_os = "windows"))]
    {
        run_systemctl(&["stop"])
    }
}

pub fn restart() -> Result<String, BoxError> {
    #[cfg(target_os = "windows")]
    {
        return windows::with_elevation(vec!["--restart-service".to_string()], windows::restart);
    }

    #[cfg(not(target_os = "windows"))]
    {
        run_systemctl(&["restart"])
    }
}

pub fn status_message() -> String {
    #[cfg(target_os = "windows")]
    {
        return windows::status_message();
    }

    #[cfg(not(target_os = "windows"))]
    {
        linux_status_message()
    }
}

#[cfg(not(target_os = "windows"))]
fn run_systemctl(args: &[&str]) -> Result<String, BoxError> {
    let mut command_args = args.to_vec();
    command_args.push("auroraops-agent.service");
    let output = Command::new("systemctl").args(&command_args).output()?;
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

#[cfg(not(target_os = "windows"))]
fn linux_status_message() -> String {
    let active = Command::new("systemctl")
        .args(["is-active", "auroraops-agent.service"])
        .output()
        .ok()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unknown".to_string());
    let enabled = Command::new("systemctl")
        .args(["is-enabled", "auroraops-agent.service"])
        .output()
        .ok()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unknown".to_string());
    format!("active={active}, enabled={enabled}")
}

#[cfg(target_os = "windows")]
mod windows {
    use super::{default_config_path, BoxError, SERVICE_DISPLAY_NAME, SERVICE_NAME};
    use std::ffi::OsStr;
    use std::iter;
    use std::os::windows::ffi::OsStrExt;
    use std::path::PathBuf;
    use std::process::Command;
    use std::ptr;
    use winapi::shared::minwindef::{DWORD, FALSE, LPVOID};
    use winapi::um::handleapi::CloseHandle;
    use winapi::um::processthreadsapi::{GetCurrentProcess, OpenProcessToken};
    use winapi::um::securitybaseapi::GetTokenInformation;
    use winapi::um::shellapi::ShellExecuteW;
    use winapi::um::synchapi::Sleep;
    use winapi::um::winnt::{TokenElevation, HANDLE, TOKEN_ELEVATION, TOKEN_QUERY};
    use winapi::um::winuser::SW_SHOWNORMAL;

    pub fn with_elevation<F>(args: Vec<String>, action: F) -> Result<String, BoxError>
    where
        F: FnOnce() -> Result<String, BoxError>,
    {
        if is_elevated() {
            return action();
        }
        relaunch_elevated(&args)?;
        Ok("已请求管理员权限，请在 UAC 窗口确认后继续。".to_string())
    }

    pub fn install(config_path: Option<PathBuf>, port: u16) -> Result<String, BoxError> {
        let exe = std::env::current_exe()?;
        let config_path = config_path.unwrap_or_else(default_config_path);
        let bin_path = format!(
            "\"{}\" --windows-service --service --config \"{}\" --port {}",
            exe.display(),
            config_path.display(),
            port
        );

        let _ = run_sc(&["stop", SERVICE_NAME]);
        let _ = run_sc(&["delete", SERVICE_NAME]);
        run_sc(&[
            "create",
            SERVICE_NAME,
            "binPath=",
            &bin_path,
            "start=",
            "auto",
            "DisplayName=",
            SERVICE_DISPLAY_NAME,
        ])?;
        run_sc(&[
            "description",
            SERVICE_NAME,
            "AuroraOps remote desktop and terminal client agent",
        ])?;
        let started = run_sc(&["start", SERVICE_NAME]).unwrap_or_else(|err| err.to_string());
        Ok(format!("Windows 服务已安装: {SERVICE_NAME}. {started}"))
    }

    pub fn uninstall() -> Result<String, BoxError> {
        let _ = run_sc(&["stop", SERVICE_NAME]);
        run_sc(&["delete", SERVICE_NAME])?;
        Ok(format!("Windows 服务已删除: {SERVICE_NAME}"))
    }

    pub fn start() -> Result<String, BoxError> {
        match run_sc(&["start", SERVICE_NAME]) {
            Ok(message) => Ok(message),
            Err(err) if err.to_string().contains("1056") => {
                Ok(format!("Windows 服务已在运行: {SERVICE_NAME}"))
            }
            Err(err) => Err(err),
        }
    }

    pub fn stop() -> Result<String, BoxError> {
        match run_sc(&["stop", SERVICE_NAME]) {
            Ok(message) => Ok(message),
            Err(err) if err.to_string().contains("1062") => {
                Ok(format!("Windows 服务已停止: {SERVICE_NAME}"))
            }
            Err(err) => Err(err),
        }
    }

    pub fn restart() -> Result<String, BoxError> {
        let _ = run_sc(&["stop", SERVICE_NAME]);
        wait_for_state("STOPPED", 15)?;
        run_sc(&["start", SERVICE_NAME])
    }

    pub fn status_message() -> String {
        let query = run_sc(&["query", SERVICE_NAME]).unwrap_or_else(|err| err.to_string());
        let qc = run_sc(&["qc", SERVICE_NAME]).unwrap_or_else(|err| err.to_string());
        let state = query
            .lines()
            .find(|line| line.trim_start().starts_with("STATE"))
            .map(str::trim)
            .unwrap_or("STATE: unknown");
        let start = qc
            .lines()
            .find(|line| line.trim_start().starts_with("START_TYPE"))
            .map(str::trim)
            .unwrap_or("START_TYPE: unknown");
        format!("{state}, {start}")
    }

    fn run_sc(args: &[&str]) -> Result<String, BoxError> {
        let output = Command::new("sc.exe").args(args).output()?;
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if output.status.success() {
            Ok(if stdout.is_empty() {
                format!("sc.exe {} ok", args.join(" "))
            } else {
                stdout
            })
        } else {
            Err(if stderr.is_empty() {
                if stdout.is_empty() {
                    format!("sc.exe {} failed", args.join(" ")).into()
                } else {
                    stdout.into()
                }
            } else {
                stderr.into()
            })
        }
    }

    fn wait_for_state(expected: &str, timeout_seconds: u32) -> Result<(), BoxError> {
        for _ in 0..timeout_seconds {
            let query = run_sc(&["query", SERVICE_NAME])?;
            if query.contains(expected) {
                return Ok(());
            }
            unsafe {
                Sleep(1000);
            }
        }
        Err(format!("等待 Windows 服务进入 {expected} 状态超时: {SERVICE_NAME}").into())
    }

    fn is_elevated() -> bool {
        unsafe {
            let mut token: HANDLE = ptr::null_mut();
            if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == FALSE {
                return false;
            }
            let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
            let mut returned: DWORD = 0;
            let ok = GetTokenInformation(
                token,
                TokenElevation,
                &mut elevation as *mut _ as LPVOID,
                std::mem::size_of::<TOKEN_ELEVATION>() as DWORD,
                &mut returned,
            );
            CloseHandle(token);
            ok != FALSE && elevation.TokenIsElevated != 0
        }
    }

    fn relaunch_elevated(args: &[String]) -> Result<(), BoxError> {
        let exe = std::env::current_exe()?;
        let params = args
            .iter()
            .map(|arg| quote_windows_arg(arg))
            .collect::<Vec<_>>()
            .join(" ");
        let operation = wide("runas");
        let file = wide(exe.as_os_str());
        let parameters = wide(params.as_str());
        let result = unsafe {
            ShellExecuteW(
                ptr::null_mut(),
                operation.as_ptr(),
                file.as_ptr(),
                parameters.as_ptr(),
                ptr::null(),
                SW_SHOWNORMAL,
            )
        };
        if (result as usize) <= 32 {
            Err(format!("请求管理员权限失败，ShellExecuteW 返回 {}", result as usize).into())
        } else {
            Ok(())
        }
    }

    fn quote_windows_arg(arg: &str) -> String {
        if arg.is_empty() || arg.chars().any(|ch| ch.is_whitespace() || ch == '"') {
            format!("\"{}\"", arg.replace('\\', r"\\").replace('"', "\\\""))
        } else {
            arg.to_string()
        }
    }

    fn wide<T: AsRef<OsStr>>(value: T) -> Vec<u16> {
        value.as_ref().encode_wide().chain(iter::once(0)).collect()
    }
}
