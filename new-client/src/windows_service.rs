#[cfg(target_os = "windows")]
mod imp {
    use std::ffi::OsStr;
    use std::iter;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Duration;

    use tracing::{error, info};
    use winapi::shared::minwindef::{DWORD, FALSE, LPVOID, TRUE};
    use winapi::um::handleapi::CloseHandle;
    use winapi::um::minwinbase::STILL_ACTIVE;
    use winapi::um::processthreadsapi::{
        CreateProcessAsUserW, GetCurrentProcess, GetExitCodeProcess, OpenProcessToken,
        TerminateProcess, PROCESS_INFORMATION, STARTUPINFOW,
    };
    use winapi::um::securitybaseapi::{DuplicateTokenEx, SetTokenInformation};
    use winapi::um::userenv::{CreateEnvironmentBlock, DestroyEnvironmentBlock};
    use winapi::um::winbase::{
        WTSGetActiveConsoleSessionId, CREATE_NO_WINDOW, CREATE_UNICODE_ENVIRONMENT,
    };
    use winapi::um::winnt::{
        SecurityImpersonation, TokenPrimary, TokenSessionId, HANDLE, MAXIMUM_ALLOWED,
        SERVICE_WIN32_OWN_PROCESS, TOKEN_ALL_ACCESS,
    };
    use winapi::um::winsvc::{
        RegisterServiceCtrlHandlerW, SetServiceStatus, StartServiceCtrlDispatcherW,
        SERVICE_ACCEPT_SHUTDOWN, SERVICE_ACCEPT_STOP, SERVICE_CONTROL_INTERROGATE,
        SERVICE_CONTROL_SHUTDOWN, SERVICE_CONTROL_STOP, SERVICE_RUNNING, SERVICE_START_PENDING,
        SERVICE_STATUS, SERVICE_STATUS_HANDLE, SERVICE_STOPPED, SERVICE_STOP_PENDING,
        SERVICE_TABLE_ENTRYW,
    };
    use winapi::um::wtsapi32::WTSQueryUserToken;

    use crate::service_manager::SERVICE_NAME;

    static mut STATUS_HANDLE: SERVICE_STATUS_HANDLE = ptr::null_mut();
    static STOP_REQUESTED: AtomicBool = AtomicBool::new(false);
    const NO_ACTIVE_SESSION: DWORD = 0xFFFF_FFFF;

    pub fn dispatch() -> Result<(), String> {
        let mut service_name = wide(SERVICE_NAME);
        let mut table = [
            SERVICE_TABLE_ENTRYW {
                lpServiceName: service_name.as_mut_ptr(),
                lpServiceProc: Some(service_main),
            },
            SERVICE_TABLE_ENTRYW {
                lpServiceName: ptr::null_mut(),
                lpServiceProc: None,
            },
        ];

        let ok = unsafe { StartServiceCtrlDispatcherW(table.as_mut_ptr()) };
        if ok == 0 {
            return Err(last_os_error());
        }
        Ok(())
    }

    unsafe extern "system" fn service_main(_argc: DWORD, _argv: *mut *mut u16) {
        let mut service_name = wide(SERVICE_NAME);
        STATUS_HANDLE =
            RegisterServiceCtrlHandlerW(service_name.as_mut_ptr(), Some(control_handler));
        if STATUS_HANDLE.is_null() {
            error!(
                "Failed to register Windows service control handler: {}",
                last_os_error()
            );
            return;
        }

        set_status(SERVICE_START_PENDING, 0);
        set_status(SERVICE_RUNNING, service_accept_stop_shutdown());
        run_session_agent_supervisor();
        set_status(SERVICE_STOPPED, 0);
    }

    unsafe extern "system" fn control_handler(control: DWORD) {
        match control {
            SERVICE_CONTROL_STOP | SERVICE_CONTROL_SHUTDOWN => {
                info!("Windows service stop requested.");
                STOP_REQUESTED.store(true, Ordering::SeqCst);
                set_status(SERVICE_STOP_PENDING, 0);
            }
            SERVICE_CONTROL_INTERROGATE => {
                set_status(SERVICE_RUNNING, service_accept_stop_shutdown());
            }
            _ => {}
        }
    }

    fn set_status(current_state: DWORD, controls_accepted: DWORD) {
        unsafe {
            if STATUS_HANDLE.is_null() {
                return;
            }
            let mut status = SERVICE_STATUS {
                dwServiceType: SERVICE_WIN32_OWN_PROCESS,
                dwCurrentState: current_state,
                dwControlsAccepted: controls_accepted,
                dwWin32ExitCode: 0,
                dwServiceSpecificExitCode: 0,
                dwCheckPoint: 0,
                dwWaitHint: 0,
            };
            let _ = SetServiceStatus(STATUS_HANDLE, &mut status);
        }
    }

    fn service_accept_stop_shutdown() -> DWORD {
        SERVICE_ACCEPT_STOP | SERVICE_ACCEPT_SHUTDOWN
    }

    fn run_session_agent_supervisor() {
        let mut child: Option<SessionAgentProcess> = None;
        let mut last_session_id = NO_ACTIVE_SESSION;

        while !STOP_REQUESTED.load(Ordering::SeqCst) {
            if child.as_ref().is_some_and(|process| !process.is_running()) {
                child = None;
                last_session_id = NO_ACTIVE_SESSION;
            }

            let session_id = unsafe { WTSGetActiveConsoleSessionId() };
            if session_id != NO_ACTIVE_SESSION {
                let should_start = child.is_none() || session_id != last_session_id;
                if should_start {
                    if let Some(process) = child.take() {
                        process.terminate();
                    }
                    match start_session_agent(session_id) {
                        Ok(process) => {
                            info!(
                                "Started AuroraOps session agent in Windows session {session_id}."
                            );
                            child = Some(process);
                            last_session_id = session_id;
                        }
                        Err(err) => {
                            error!(
                                "Failed to start AuroraOps session agent in Windows session {session_id}: {err}"
                            );
                        }
                    }
                }
            }

            std::thread::sleep(Duration::from_secs(3));
        }

        if let Some(process) = child {
            process.terminate();
        }
    }

    fn start_session_agent(session_id: DWORD) -> Result<SessionAgentProcess, String> {
        match start_session_agent_as_active_user(session_id) {
            Ok(process) => {
                info!(
                    "Started AuroraOps session agent as active user in Windows session {session_id}."
                );
                return Ok(process);
            }
            Err(err) => {
                error!(
                    "Failed to start active-user session agent in Windows session {session_id}: {err}; falling back to service account token."
                );
            }
        }
        start_session_agent_as_service_account(session_id)
    }

    fn start_session_agent_as_service_account(
        session_id: DWORD,
    ) -> Result<SessionAgentProcess, String> {
        unsafe {
            let mut service_token: HANDLE = ptr::null_mut();
            if OpenProcessToken(GetCurrentProcess(), TOKEN_ALL_ACCESS, &mut service_token) == FALSE
            {
                return Err(format!(
                    "OpenProcessToken failed: {}",
                    std::io::Error::last_os_error()
                ));
            }

            let mut primary_token: HANDLE = ptr::null_mut();
            let duplicated = DuplicateTokenEx(
                service_token,
                TOKEN_ALL_ACCESS | MAXIMUM_ALLOWED,
                ptr::null_mut(),
                SecurityImpersonation,
                TokenPrimary,
                &mut primary_token,
            );
            CloseHandle(service_token);
            if duplicated == FALSE {
                return Err(format!(
                    "DuplicateTokenEx(service) failed: {}",
                    std::io::Error::last_os_error()
                ));
            }

            let mut target_session = session_id;
            let session_set = SetTokenInformation(
                primary_token,
                TokenSessionId,
                (&mut target_session as *mut DWORD).cast(),
                std::mem::size_of::<DWORD>() as DWORD,
            );
            if session_set == FALSE {
                let err = std::io::Error::last_os_error();
                CloseHandle(primary_token);
                return Err(format!("SetTokenInformation(TokenSessionId) failed: {err}"));
            }

            start_session_agent_with_primary_token(primary_token, false)
        }
    }

    fn start_session_agent_as_active_user(
        session_id: DWORD,
    ) -> Result<SessionAgentProcess, String> {
        unsafe {
            let mut user_token: HANDLE = ptr::null_mut();
            if WTSQueryUserToken(session_id, &mut user_token) == FALSE {
                return Err(format!(
                    "WTSQueryUserToken failed: {}",
                    std::io::Error::last_os_error()
                ));
            }

            let mut primary_token: HANDLE = ptr::null_mut();
            let duplicated = DuplicateTokenEx(
                user_token,
                TOKEN_ALL_ACCESS | MAXIMUM_ALLOWED,
                ptr::null_mut(),
                SecurityImpersonation,
                TokenPrimary,
                &mut primary_token,
            );
            CloseHandle(user_token);
            if duplicated == FALSE {
                return Err(format!(
                    "DuplicateTokenEx failed: {}",
                    std::io::Error::last_os_error()
                ));
            }

            start_session_agent_with_primary_token(primary_token, true)
        }
    }

    unsafe fn start_session_agent_with_primary_token(
        primary_token: HANDLE,
        inherit_handles: bool,
    ) -> Result<SessionAgentProcess, String> {
        let mut environment: LPVOID = ptr::null_mut();
        let has_environment =
            CreateEnvironmentBlock(&mut environment, primary_token, FALSE) != FALSE;
        let mut command_line = wide(command_line()?);
        let mut desktop = wide("winsta0\\default");
        let mut startup: STARTUPINFOW = std::mem::zeroed();
        startup.cb = std::mem::size_of::<STARTUPINFOW>() as DWORD;
        startup.lpDesktop = desktop.as_mut_ptr();

        let mut process_info: PROCESS_INFORMATION = std::mem::zeroed();
        let created = CreateProcessAsUserW(
            primary_token,
            ptr::null(),
            command_line.as_mut_ptr(),
            ptr::null_mut(),
            ptr::null_mut(),
            if inherit_handles { TRUE } else { FALSE },
            CREATE_UNICODE_ENVIRONMENT | CREATE_NO_WINDOW,
            if has_environment {
                environment
            } else {
                ptr::null_mut()
            },
            ptr::null(),
            &mut startup,
            &mut process_info,
        );

        if has_environment {
            DestroyEnvironmentBlock(environment);
        }
        CloseHandle(primary_token);

        if created == FALSE {
            return Err(format!(
                "CreateProcessAsUserW failed: {}",
                std::io::Error::last_os_error()
            ));
        }
        CloseHandle(process_info.hThread);
        Ok(SessionAgentProcess {
            process: process_info.hProcess,
            pid: process_info.dwProcessId,
        })
    }

    fn command_line() -> Result<String, String> {
        let exe = std::env::current_exe().map_err(|err| err.to_string())?;
        let mut args = Vec::new();
        args.push(quote_windows_arg(&exe.display().to_string()));
        args.push("--session-agent".to_string());
        args.push("--service".to_string());
        for arg in std::env::args().skip(1) {
            if arg == "--windows-service" || arg == "--service" || arg == "--session-agent" {
                continue;
            }
            args.push(quote_windows_arg(&arg));
        }
        Ok(args.join(" "))
    }

    struct SessionAgentProcess {
        process: HANDLE,
        pid: DWORD,
    }

    impl SessionAgentProcess {
        fn is_running(&self) -> bool {
            unsafe {
                let mut exit_code = 0;
                if GetExitCodeProcess(self.process, &mut exit_code) == FALSE {
                    return false;
                }
                exit_code == STILL_ACTIVE
            }
        }

        fn terminate(self) {
            unsafe {
                info!("Stopping AuroraOps session agent pid {}.", self.pid);
                let _ = TerminateProcess(self.process, 0);
            }
        }
    }

    impl Drop for SessionAgentProcess {
        fn drop(&mut self) {
            unsafe {
                CloseHandle(self.process);
            }
        }
    }

    fn wide<T: AsRef<OsStr>>(value: T) -> Vec<u16> {
        value.as_ref().encode_wide().chain(iter::once(0)).collect()
    }

    fn quote_windows_arg(arg: &str) -> String {
        if arg.is_empty() || arg.chars().any(|ch| ch.is_whitespace() || ch == '"') {
            format!("\"{}\"", arg.replace('\\', r"\\").replace('"', "\\\""))
        } else {
            arg.to_string()
        }
    }

    fn last_os_error() -> String {
        std::io::Error::last_os_error().to_string()
    }
}

#[cfg(target_os = "windows")]
pub use imp::dispatch;
