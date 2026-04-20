# AuroraOps Go Agent

## Modes

- Electron mode: Electron starts the Go agent locally and loads the agent HTTP UI.
- Headless mode: run only the Go agent, and it still serves the web UI.

## Headless

```bash
./auroraops-agent --headless --server 192.168.1.20:8000 --name node-01 --port 18765
```

Open:

```bash
http://127.0.0.1:18765/
```

## Linux systemd

For packaged Linux installs, the `deb` and `rpm` post-install scripts copy:

- `auroraops-agent` to `/opt/auroraops/auroraops-agent`
- `auroraops-agent.service` to `/etc/systemd/system/auroraops-agent.service`

and then run:

```bash
systemctl daemon-reload
systemctl enable auroraops-agent
systemctl restart auroraops-agent
```

Electron on Linux also prefers `/opt/auroraops/auroraops-agent` at runtime, so the desktop client and `systemd` use the same agent binary.

## Build Matrix

Go agent binaries are now organized by platform and architecture:

- `go-client/bin/linux/x64/auroraops-agent`
- `go-client/bin/linux/arm64/auroraops-agent`
- `go-client/bin/win32/x64/auroraops-agent.exe`
- `go-client/bin/win32/arm64/auroraops-agent.exe`

Useful package commands:

```bash
npm run build:linux:x64
npm run build:linux:arm64
npm run build:linux
```

## Asset Sync Behavior

- Client-collected hardware fields are treated as machine facts and will be refreshed during sync.
- Server-side manual fields such as asset `status`, `sort`, and non-empty `remark` are preserved during sync.
- The local UI supports previewing collected assets before pushing them to the server.
- Synced assets are marked with `source=agent`, include a `sync_hash`, and update `last_seen_at` on each successful observation.

Files:

- [`auroraops-agent.service`](/data/Projects/AuroraOps/client/go-client/auroraops-agent.service)
- [`install-systemd.sh`](/data/Projects/AuroraOps/client/go-client/install-systemd.sh)

Install:

```bash
chmod +x install-systemd.sh
sudo ./install-systemd.sh ./bin/linux/auroraops-agent
```

Manage:

```bash
sudo systemctl status auroraops-agent
sudo systemctl restart auroraops-agent
sudo systemctl stop auroraops-agent
```

## Windows Service

Install service:

```powershell
auroraops-agent.exe --service install --config C:\ProgramData\AuroraOps\agent-config.json --port 18765 --server 192.168.1.20:8000 --name node-01
```

Uninstall service:

```powershell
auroraops-agent.exe --service uninstall
```

The Windows service runs the same local HTTP UI and connector logic as headless mode.

## Bundled With Electron Package

Electron packaging copies the following files into the packaged `go-agent/` directory:

- Linux binary
- Windows binary
- `auroraops-agent.service`
- `install-systemd.sh`
- `install-windows-service.ps1`
- `uninstall-windows-service.ps1`
- this README
