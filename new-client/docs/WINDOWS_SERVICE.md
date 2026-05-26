# Windows 服务与桌面会话

本文说明 AuroraOps 客户端在 Windows 上的服务安装、自动提权和桌面会话代理机制。

## 目标

Windows 服务运行在 Session 0，不能直接稳定访问当前登录用户的桌面。AuroraOps 客户端采用两层架构：

- `auroraops-agent` Windows 服务：由 Service Control Manager 常驻，负责自启、服务状态和拉起会话代理。
- `auroraops-agent --session-agent`：由服务在当前活动用户会话中启动，负责远程桌面、远程终端、本机管理页和服务端连接。

这样可以同时满足：

- 开机自启和服务常驻。
- 用户登录后自动进入可捕获桌面的会话。
- 用户切换会话或 agent 异常退出后自动重拉。

## 安装

在 PowerShell 或 CMD 中执行：

```powershell
.\auroraops-agent.exe --install-service
```

如果当前不是管理员权限，客户端会通过 UAC 请求提权。确认后会执行：

- 创建 Windows 服务 `auroraops-agent`。
- 设置显示名 `AuroraOps 客户端`。
- 设置启动类型为 `auto`。
- 启动服务。

默认服务命令类似：

```text
"C:\path\auroraops-agent.exe" --windows-service --service --config "C:\ProgramData\AuroraOps\agent-config.json" --port 18765
```

## 服务管理

```powershell
.\auroraops-agent.exe --start-service
.\auroraops-agent.exe --stop-service
.\auroraops-agent.exe --restart-service
.\auroraops-agent.exe --uninstall-service
```

这些命令在非管理员权限下也会自动请求 UAC。

也可以使用系统工具查看：

```powershell
sc query auroraops-agent
sc qc auroraops-agent
Get-Service auroraops-agent
```

## 运行机制

服务启动后：

1. 注册 Windows SCM service handler。
2. 状态上报为 `SERVICE_RUNNING`。
3. 每隔数秒调用 `WTSGetActiveConsoleSessionId` 获取当前活动会话。
4. 使用 `WTSQueryUserToken` 获取该会话用户 token。
5. 使用 `DuplicateTokenEx` 复制 primary token。
6. 使用 `CreateEnvironmentBlock` 构建用户环境变量。
7. 使用 `CreateProcessAsUserW` 启动：

```text
auroraops-agent.exe --session-agent --service --config ... --port ...
```

会话代理启动在 `winsta0\default` 桌面，并隐藏控制台窗口。

## 进程检查

用户登录后，通常可以看到两个进程：

```powershell
Get-Process auroraops-agent | Select-Object Id,SessionId,Path
```

示例：

```text
Id    SessionId Path
--    --------- ----
1234  0         C:\AuroraOps\auroraops-agent.exe
5678  1         C:\AuroraOps\auroraops-agent.exe
```

- `SessionId = 0`：Windows 服务进程。
- 非 0 Session：当前登录用户桌面会话中的 session agent。

## 配置路径

默认配置文件：

```text
C:\ProgramData\AuroraOps\agent-config.json
```

安装服务时可指定：

```powershell
.\auroraops-agent.exe --install-service --config "D:\AuroraOps\agent-config.json" --port 18765
```

本机管理页：

```text
http://127.0.0.1:18765/
```

## 排查

### 服务启动失败

```powershell
sc query auroraops-agent
sc qc auroraops-agent
```

确认 `BINARY_PATH_NAME` 指向的 exe 存在，且路径没有被移动。

### 没有 session agent

```powershell
Get-Process auroraops-agent | Select-Object Id,SessionId,Path
```

如果只有 Session 0 进程：

- 确认当前机器有用户登录到图形桌面。
- 确认服务账户是 LocalSystem。
- 检查服务日志中是否有 `WTSQueryUserToken failed` 或 `CreateProcessAsUserW failed`。

### 有 session agent 但不能远程桌面

打开本机管理页检查连接和桌面状态：

```text
http://127.0.0.1:18765/
```

确认：

- 服务端地址和设备名已保存。
- 客户端已连接 AuroraOps 服务端。
- 桌面编码选项符合当前显卡和驱动能力。

### MediaFoundation / NVENC

Windows 下可在本机管理页开启：

- `NVENC`：需要 NVIDIA 显卡和驱动支持。
- `MediaFoundation`：使用 Windows Media Foundation 编码路径，是否启用取决于系统和显卡驱动能力。

硬件编码失败时会回退到软件编码或当前可用路径。
