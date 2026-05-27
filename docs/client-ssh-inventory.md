# AuroraOps Client SSH Inventory

Updated: 2026-05-28

This file tracks the client machines registered to the local AuroraOps server and the SSH details currently known for maintenance updates.

## Registered Clients

| Device ID | Name | Hostname | IP | OS | Arch | SSH | Status / Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | linux-测试 | fit-pc | 192.168.2.52 | Kylin V10 SP1 | aarch64 | `fit@192.168.2.52:22` | SSH works; password provided by user and sudo verified. Service runs as root from `/opt/auroraops/auroraops-agent`. Rebuilt locally on the remote host with `ffmpeg-system,vaapi` and installed on 2026-05-27; service active; SHA256 `6c0f7c1af0b27ae6ece368baf86ae1b7bde4051cc2b3a597a8c6b164a595bc5b`. Backup: `/opt/auroraops/auroraops-agent.bak-20260527` SHA256 `e3246e8d099281aca70046d5c5d2deecc26885b77f6bd91e17c30e5cf3120453`. System FFmpeg currently shows VAAPI only, no NVENC encoder. |
| 2 | linux-node-01 | gg | 192.168.2.161 | Kylin V10 SP1 | x86_64 | unknown | `root`, `gg`, and `icepie` failed with publickey/password auth on port 22. |
| 3 | win-192.168.2.72 | VM | 192.168.2.72 | Windows | x86_64 | `icepie@192.168.2.72:22` | SSH works. Service `auroraops-agent` runs from `C:\AuroraOpsTest\auroraops-agent.exe`. Rebuilt and updated Windows x86_64 build on 2026-05-28 with embedded frontend capture/rotation controls, Windows cursor compositing with Win10 fallback, optional virtual cursor using an inline macOS-style cursor image, pointer-lock drag fixes, keyboard release hardening, and explicit DXGI selection preserved; service Running; SHA256 `18370828EC23BFA716F13A4C812A47C74591C9660E6FC4D3740B5857D6559D49`. Latest backup: `C:\AuroraOpsTest\auroraops-agent.exe.bak-20260528-maccursor-inline`. Previous backup: `C:\AuroraOpsTest\auroraops-agent.exe.bak-20260528-dxgi`. |
| 4 | codex-local | next | 192.168.2.222 | Arch Linux | x86_64 | local machine | Local `/opt/auroraops/auroraops-agent` built with `ffmpeg-system,vaapi`; SHA256 `78ff11fe93d17e8fbe9d86ddebb43ddca8327e5ca394963ed7f3f15217d50c01`; DB heartbeat online after update. |
| 5 | win-192.168.2.36 | DESKTOP-5E5QU52 | 192.168.2.36 | Windows | x86_64 | `Administrator@192.168.2.36:22` | SSH works; password provided by user. Service `auroraops-agent` runs from `C:\Program Files\AuroraOps\auroraops-agent.exe`. Rebuilt and updated Windows x86_64 build on 2026-05-28 with embedded frontend capture/rotation controls, Windows cursor compositing with Win10 fallback, optional virtual cursor using an inline macOS-style cursor image, pointer-lock drag fixes, keyboard release hardening, and explicit DXGI selection preserved; service Running; SHA256 `18370828EC23BFA716F13A4C812A47C74591C9660E6FC4D3740B5857D6559D49`. Latest backup: `C:\Program Files\AuroraOps\auroraops-agent.exe.bak-20260528-maccursor-inline`. Previous backup: `C:\Program Files\AuroraOps\auroraops-agent.exe.bak-20260528-dxgi`. |
| 6 | linux-192.168.2.186 | archlinux | 192.168.2.186 | Arch Linux | x86_64 | `icepie@192.168.2.186:22` | SSH works and sudo was verified. Service runs as root from `/usr/local/bin/auroraops-agent`. Rebuilt locally on the remote host with `ffmpeg-system,vaapi` on 2026-05-27; system FFmpeg has `h264_nvenc`, `hevc_nvenc`, and `h264_vaapi`; service active; SHA256 `164b7253dad44c9e226ee9fb99d5d96424a5e255e0f279d950d438f80547fb25`. Backup: `/usr/local/bin/auroraops-agent.bak-remote-build-20260527`. |
| 7 | linux-192.168.1.141 | quiet | 192.168.1.141 | Arch Linux | x86_64 | `zks@192.168.1.141:22` | SSH works; password provided by user and sudo verified. Remote shell is fish; use `bash -lc` for maintenance commands. Existing ssh config entry `root@192.168.1.141:10022` is not valid now and can override this host; use `ssh -F /dev/null -p 22 zks@192.168.1.141` until config is fixed. Service runs as root from `/usr/local/bin/auroraops-agent`. Rebuilt locally on the remote host with `ffmpeg-system,vaapi`; system FFmpeg has `h264_nvenc`, `hevc_nvenc`, and `h264_vaapi`; installed on 2026-05-27; service active; SHA256 `88c7f2b3f38e74f7cddbabee475970c2d5c98ef0eb4e36bbd9d583d74276c94a`. Backup: `/usr/local/bin/auroraops-agent.bak-20260527` SHA256 `b24e5d1993685752ea0d7e2f166abca1bd8710631f3f75c99181e7cab169dc26`. |
| 8 | linux-192.168.2.101 | fitos-mjj | 192.168.2.101 | Arch Linux | x86_64 | `gg@192.168.2.101:22` | SSH works; password provided by user and sudo verified. Service runs as root from `/usr/local/bin/auroraops-agent` with `--try-vaapi`; system FFmpeg has `h264_nvenc`, `hevc_nvenc`, and `h264_vaapi`. Reused the Arch x86_64 remote-local build from `192.168.2.186` and installed on 2026-05-27; service active; SHA256 `164b7253dad44c9e226ee9fb99d5d96424a5e255e0f279d950d438f80547fb25`. Backup: `/usr/local/bin/auroraops-agent.bak-20260527` SHA256 `0a4a0493b4971e1594c50c026f87055680e102586bd32b9487e2ad9ed04fc5b1`. |
| 9 | gl | gl-H610M-H-V3-DDR4 | 192.168.2.7 | Ubuntu 26.04 LTS | x86_64 | `gl@192.168.2.7:22` | SSH works; password provided by user and sudo verified. Remote shell is fish; use `bash -lc` for maintenance commands. Service runs as root from `/opt/auroraops/auroraops-agent`; system FFmpeg has `h264_nvenc`, `hevc_nvenc`, and `h264_vaapi`. Installed matching Ubuntu x86_64 VAAPI build on 2026-05-27; service active; SHA256 `78ff11fe93d17e8fbe9d86ddebb43ddca8327e5ca394963ed7f3f15217d50c01`. Backup: `/opt/auroraops/auroraops-agent.bak-20260527` SHA256 `4acf0c20d991a403e089c3a2c0b649ab7196d2b9476fd1f77c01200b094a7a07`. Remote-local build attempted with proxy `192.168.2.222:12333`; dependency fetch succeeded, but build needs GStreamer development pkg-config files (`gstreamer-1.0.pc`). |

## Probe Notes

- Device data came from local MySQL database `hotgo.hg_ops_device`.
- Successful SSH probes were read-only: `id`, `hostname`, `uname`, service/path checks.
- Linux x86_64 local build SHA256 on `192.168.2.222`: `78ff11fe93d17e8fbe9d86ddebb43ddca8327e5ca394963ed7f3f15217d50c01`.
- Updated successfully: local `192.168.2.222`, remote-local build on `192.168.2.52`, remote-local build on `192.168.2.186`, remote-local build on `192.168.1.141`, reused Arch x86_64 VAAPI build on `192.168.2.101`, matching Ubuntu x86_64 VAAPI build on `192.168.2.7`, Windows x86_64 build on `192.168.2.72` and `192.168.2.36`.
- Windows build SHA256 with embedded frontend capture/rotation controls, Windows cursor compositing with Win10 fallback, optional inline macOS-style virtual cursor, pointer-lock drag fixes, keyboard release hardening, and explicit DXGI selection preserved: `18370828ec23bfa716f13a4c812a47c74591c9660e6fc4d3740b5857d6559d49`.
- aarch64 and Windows clients require matching binaries/packages and should not receive the local Linux x86_64 binary.
- Plaintext SSH passwords are not stored in this repository file.

## Need Confirmation

Please confirm SSH username, port, and whether password/sudo is needed for:

- `192.168.2.161` (`linux-node-01`, Kylin x86_64)
