# AuroraOps Client SSH Inventory

Updated: 2026-06-09

This file tracks the client machines registered to the local AuroraOps server and the SSH details currently known for maintenance updates.

## Registered Clients

| Device ID | Name | Hostname | IP | OS | Arch | SSH | Status / Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | linux-测试 | fit-pc | 192.168.2.52 | Kylin V10 SP1 | aarch64 | `fit@192.168.2.52:22` | SSH works; service runs as root from `/opt/auroraops/auroraops-agent`. Rebuilt locally on the remote host with `ffmpeg-system,vaapi` and installed on 2026-05-28 with embedded frontend capture/rotation controls, optional inline macOS-style virtual cursor, pointer-lock drag fixes, and keyboard release hardening; service active; SHA256 `496914e1051d5c3330878374d3d49c3c1182ccbb582a9d4c0a819eb8ee0d0700`. Latest backup: `/opt/auroraops/auroraops-agent.bak-20260528-maccursor-inline`. Previous backup: `/opt/auroraops/auroraops-agent.bak-20260527`. System FFmpeg currently shows VAAPI only, no NVENC encoder. 2026-06-09 Weylus timeout build not deployed because current known sudo password failed; this aarch64 Kylin client should be rebuilt locally on the remote host once sudo is available. |
| 2 | linux-node-01 | gg | 192.168.2.161 | Kylin V10 SP1 | x86_64 | `gg@192.168.2.161:22` | SSH works; service runs as root from `/opt/auroraops/auroraops-agent`. Rebuilt locally on the remote host with `ffmpeg-system,vaapi` and installed on 2026-06-09 with Weylus tunnel TCP connect timeout; service active; SHA256 `e07e8da0f345160d93e347ec50ec6a7bf32026e9c8cc79fd621453dc12d6b263`. Latest backup: `/opt/auroraops/auroraops-agent.bak-20260609-weylus-timeout`. Previous backup: `/opt/auroraops/auroraops-agent.bak-20260528-maccursor-inline`. Server registered the Weylus tunnel at `2026-06-09 10:27:48`. |
| 3 | win-192.168.2.72 | VM | 192.168.2.72 | Windows | x86_64 | `icepie@192.168.2.72:22` | SSH works. Service `auroraops-agent` runs from `C:\AuroraOpsTest\auroraops-agent.exe`. Rebuilt and updated Windows x86_64 build on 2026-06-09 with Weylus tunnel TCP connect timeout; service Running; SHA256 `A22599A153B9A3B856452C3823897DF20C67A468DE0C03E5FB7C1E11761D24D1`. Latest backup: `C:\AuroraOpsTest\auroraops-agent.exe.bak-20260609-weylus-timeout`. Previous backup: `C:\AuroraOpsTest\auroraops-agent.exe.bak-20260528-maccursor-inline`. Server registered the Weylus tunnel at `2026-06-09 10:19:43`. |
| 4 | codex-local | next | 192.168.2.222 | Arch Linux | x86_64 | local machine | Local `/opt/auroraops/auroraops-agent` built with `ffmpeg-system,vaapi` and installed on 2026-06-09 with Weylus tunnel TCP connect timeout so failed/stale tunnel connects do not block reconnect loops; service active; SHA256 `608b712ee1404afe5a163bd56a0db75077f8ee1b4f02f569a6f2d8841a49f73e`. Latest backup: `/opt/auroraops/auroraops-agent.bak-20260609-weylus-timeout`. Previous backup: `/opt/auroraops/auroraops-agent.bak-20260528-maccursor-inline-linux`. Server registered the Weylus tunnel at `2026-06-09 09:58:57`. |
| 5 | win-192.168.2.36 | DESKTOP-5E5QU52 | 192.168.2.36 | Windows | x86_64 | `Administrator@192.168.2.36:22` | SSH works with the correct password. Service `auroraops-agent` runs from `C:\Program Files\AuroraOps\auroraops-agent.exe`. Rebuilt and updated Windows x86_64 build on 2026-05-28 with embedded frontend capture/rotation controls, Windows cursor compositing with Win10 fallback, optional virtual cursor using an inline macOS-style cursor image, pointer-lock drag fixes, keyboard release hardening, and explicit DXGI selection preserved; service Running; SHA256 `18370828EC23BFA716F13A4C812A47C74591C9660E6FC4D3740B5857D6559D49`. Latest backup: `C:\Program Files\AuroraOps\auroraops-agent.exe.bak-20260528-maccursor-inline`. Previous backup: `C:\Program Files\AuroraOps\auroraops-agent.exe.bak-20260528-dxgi`. 2026-06-09 Weylus timeout build not deployed because password authentication failed; normal Windows x86_64 build SHA256 prepared locally: `a22599a153b9a3b856452c3823897df20c67a468de0c03e5fb7c1e11761d24d1`. |
| 6 | linux-192.168.2.186 | archlinux | 192.168.2.186 | Arch Linux | x86_64 | `icepie@192.168.2.186:22` | SSH works; sudo verified with current maintenance password. Service runs as root from `/usr/local/bin/auroraops-agent` with `--kms-support --kms-device /dev/dri/card0 --try-nvenc --nvfbc-support`; config has KMS `/dev/dri/card0`, NvFBC, and NVENC enabled. System FFmpeg has `h264_nvenc`, `hevc_nvenc`, and `h264_vaapi`; `libnvidia-fbc.so.1` is available. Rebuilt on this host with `ffmpeg-system,vaapi,nvfbc` and installed on 2026-06-09 with Weylus tunnel TCP connect timeout; service active; SHA256 `412f573d725ad5d46a8d279cb2150e2d8ccd5007728a7cffa7ceb47dd25d7a47`. Latest backup: `/usr/local/bin/auroraops-agent.bak-20260609-weylus-timeout`; service unit backup: `/etc/systemd/system/auroraops-agent.service.bak-20260609-weylus-timeout`. Unit `After=` was changed from `network-online.target graphical.target display-manager.service` to `network-online.target display-manager.service` to remove the `multi-user.target` ordering cycle. Server registered the Weylus tunnel at `2026-06-09 10:04:36`. |
| 7 | linux-192.168.1.141 | quiet | 192.168.1.141 | Arch Linux | x86_64 | `zks@192.168.1.141:22` | SSH works with `ssh -F /dev/null -p 22 zks@192.168.1.141`, but sudo requires the correct password. Remote shell is fish; use `bash -lc` for maintenance commands. Existing ssh config entry `root@192.168.1.141:10022` is not valid now and can override this host. Service runs as root from `/usr/local/bin/auroraops-agent`; config has KMS `/dev/dri/card0`, NvFBC, and NVENC enabled. `libnvidia-fbc.so.1` is available. Installed Arch x86_64 `ffmpeg-system,vaapi,nvfbc` build from `192.168.2.186` on 2026-06-03; service active; SHA256 `d25d9d30f72a8001c11d0092787bb0d817668a5aacd7733df48daf579f16b51e`. Latest backup: `/usr/local/bin/auroraops-agent.bak-20260603-nvfbc`; config backup: `/etc/auroraops/agent-config.json.bak-20260603-nvfbc`. Previous backup: `/usr/local/bin/auroraops-agent.bak-20260528-maccursor-inline`. 2026-06-09 Weylus timeout NVFBC build SHA256 available from device 6: `412f573d725ad5d46a8d279cb2150e2d8ccd5007728a7cffa7ceb47dd25d7a47`; not deployed because sudo authentication failed. |
| 8 | linux-192.168.2.101 | fitos-mjj | 192.168.2.101 | Arch Linux | x86_64 | `gg@192.168.2.101:22` | SSH works with the correct password; service runs as root from `/usr/local/bin/auroraops-agent` with `--try-vaapi`; system FFmpeg has `h264_nvenc`, `hevc_nvenc`, and `h264_vaapi`. Reused the local Arch x86_64 `ffmpeg-system,vaapi` build and installed on 2026-05-28 with embedded frontend capture/rotation controls, optional inline macOS-style virtual cursor, pointer-lock drag fixes, and keyboard release hardening; service active; SHA256 `a67d5a38eed00866f2156a3c8a7bef81974444f0708fd6a5dc1d18297bbf2821`. Latest backup: `/usr/local/bin/auroraops-agent.bak-20260528-maccursor-inline`. 2026-06-09 Weylus timeout Linux x86_64 VAAPI build SHA256 available locally: `608b712ee1404afe5a163bd56a0db75077f8ee1b4f02f569a6f2d8841a49f73e`; not deployed because password authentication failed. |
| 9 | gl | gl-H610M-H-V3-DDR4 | 192.168.2.7 | Ubuntu 26.04 LTS | x86_64 | `gl@192.168.2.7:22` | SSH works. Remote shell is fish; use `bash -lc` for maintenance commands. Service runs as root from `/opt/auroraops/auroraops-agent`; system FFmpeg has `h264_nvenc`, `hevc_nvenc`, and `h264_vaapi`. Installed Linux x86_64 `ffmpeg-system,vaapi` build on 2026-06-09 with Weylus tunnel TCP connect timeout; service active; SHA256 `608b712ee1404afe5a163bd56a0db75077f8ee1b4f02f569a6f2d8841a49f73e`. Latest backup: `/opt/auroraops/auroraops-agent.bak-20260609-weylus-timeout`. Previous backup: `/opt/auroraops/auroraops-agent.bak-20260528-maccursor-inline`. Server registered the Weylus tunnel at `2026-06-09 10:18:39`. |
| 10 | Win7-64-PDY202501 | WIN7-64-PDY2025 | 192.168.1.166 | Windows 7 SP1 | x86_64 | `Administrator@192.168.1.166:2222` | SSH works. Service `auroraops-agent` runs as LocalSystem from `C:\AuroraOps\auroraops-agent.exe` with config `C:\ProgramData\AuroraOps\agent-config.json`; connected to `http://192.168.2.222:8000` on 2026-06-02, registered as device ID `10`, status `connected`, TCP address `192.168.2.222:8099`, local management port `127.0.0.1:18765`. Installed Win7-compatible patched Windows x86_64 GNU build on 2026-06-09 with Weylus tunnel TCP connect timeout, plus `auroraops-waitonaddress-shim.dll` and `auroraops-bcprng.dll`; service Running; agent SHA256 `9f897bfe72078020d1b44166c1c5fd8b91e93bb047a2c7f9cd0cf2796f56c8df`, wait shim SHA256 `8fa87d0b9912ccf9d87dfc2f38b55763dc474ee68787fafde67412f6049df8bd`, ProcessPrng shim SHA256 `3e9b2d7ef70143cee5af2030ee072ed3a80056a4caa48b7e9832f9716651ab83`. Latest backup: `C:\AuroraOps\auroraops-agent.exe.bak-20260609-weylus-timeout`. Previous backup: `C:\AuroraOps\auroraops-agent.exe.bak-20260602-encoding-cls`. Old incomplete `api-ms-win-core-synch-l1-2-0.dll` shim was renamed to `C:\AuroraOps\api-ms-win-core-synch-l1-2-0.dll.bak` because it interfered with Win7 DLL search. Server registered the Weylus tunnel at `2026-06-09 10:21:25`. |

## Probe Notes

- Device data came from local MySQL database `hotgo.hg_ops_device`.
- Successful SSH probes were read-only: `id`, `hostname`, `uname`, service/path checks.
- Linux x86_64 local build SHA256 on `192.168.2.222`: `a67d5a38eed00866f2156a3c8a7bef81974444f0708fd6a5dc1d18297bbf2821`.
- Linux x86_64 local Weylus timeout build SHA256 on `192.168.2.222` from 2026-06-09: `608b712ee1404afe5a163bd56a0db75077f8ee1b4f02f569a6f2d8841a49f73e`.
- Kylin aarch64 remote build SHA256 on `192.168.2.52`: `496914e1051d5c3330878374d3d49c3c1182ccbb582a9d4c0a819eb8ee0d0700`.
- Kylin x86_64 remote build SHA256 on `192.168.2.161`: `9841f4d6c1ef2b10a314dbafad6672acdac3a94bd01dc3351b0b6aff76d3e85f`.
- Arch x86_64 remote-local NVENC-capable build SHA256 on `192.168.2.186`: `9fa4aacb341dab09ed76dd87effada6996237ff85ddca6c3eb0f03b284c0cd8c`.
- Arch x86_64 remote-local NVFBC Weylus timeout build SHA256 on `192.168.2.186` from 2026-06-09: `412f573d725ad5d46a8d279cb2150e2d8ccd5007728a7cffa7ceb47dd25d7a47`.
- Updated successfully on 2026-05-28: local `192.168.2.222`, remote-local build on `192.168.2.52`, remote-local build on `192.168.2.161`, remote-local NVENC-capable build on `192.168.2.186`, Linux x86_64 VAAPI build on `192.168.1.141`, reused Arch x86_64 VAAPI build on `192.168.2.101`, Linux x86_64 VAAPI build on `192.168.2.7`, Windows x86_64 build on `192.168.2.72` and `192.168.2.36`.
- Updated successfully on 2026-06-02: Win7 x86_64 client on `192.168.1.166:2222`, installed to `C:\AuroraOps`, connected to service server `http://192.168.2.222:8000`.
- Local service server on `192.168.2.222` runs under `auroraops-server.service` from `/data/Projects/AuroraOps/server/temp/auroraops-server`; 2026-06-02 reconnect/terminal-close build SHA256 `6edf89ee6359082237db470d62ced2845e15cdc1b980eb4576c261227bd95caa`.
- Windows build SHA256 with embedded frontend capture/rotation controls, Windows cursor compositing with Win10 fallback, optional inline macOS-style virtual cursor, pointer-lock drag fixes, keyboard release hardening, and explicit DXGI selection preserved: `18370828ec23bfa716f13a4c812a47c74591c9660e6fc4d3740b5857d6559d49`.
- Windows x86_64 Weylus timeout build SHA256 from 2026-06-09: `a22599a153b9a3b856452c3823897df20c67a468de0c03e5fb7c1e11761d24d1`.
- Win7-compatible Windows build SHA256 with dynamic synthetic pointer API loading, terminal pipe fallback, pipe line-input fallback for `cmd.exe`, Windows OEM codepage decoding, `cls` ANSI clear-screen fallback, and WaitOnAddress/ProcessPrng compatibility shims: `e570995faf32eb1d1b21d8e8a5c491b5177ef24fe96778d56591c1292c35b92c`.
- Win7-compatible patched Weylus timeout build SHA256 from 2026-06-09: `9f897bfe72078020d1b44166c1c5fd8b91e93bb047a2c7f9cd0cf2796f56c8df`.
- aarch64 and Windows clients require matching binaries/packages and should not receive the local Linux x86_64 binary.
- Plaintext SSH passwords are not stored in this repository file.
- Updated successfully on 2026-06-09: local `192.168.2.222` and NVFBC Arch host `192.168.2.186`; both services are active and the server log shows fresh Weylus tunnel registrations.
- Additional clients updated successfully on 2026-06-09 after credential confirmation: Kylin x86_64 `192.168.2.161`, Windows `192.168.2.72`, Ubuntu `192.168.2.7`, and Win7 `192.168.1.166:2222`.
- Not updated on 2026-06-09: device 1 needs valid sudo for remote aarch64 build/install; devices 5 and 8 need the correct SSH password; device 7 needs the correct sudo password.

## Win7 Build Procedure

The Win7 client uses the normal Windows GNU release target plus PE import patching. Do not deploy the `x86_64-win7-windows-gnu` custom-target build without retesting on Win7; on 2026-06-02 it passed local build checks but crashed during Win7 process startup with `0xc0000005`.

Build the regular Windows GNU agent first:

```bash
rtk cargo build --manifest-path new-client/Cargo.toml --release --target x86_64-pc-windows-gnu --bin auroraops-agent
```

Patch the generated PE import strings into Win7-compatible shims and time fallback:

```bash
rtk python3 - <<'PY'
from pathlib import Path
import pefile

src = Path("new-client/target/x86_64-pc-windows-gnu/release/auroraops-agent.exe")
dst = Path("/tmp/auroraops-agent-win7-patched.exe")
data = bytearray(src.read_bytes())
replacements = [
    (b"bcryptprimitives.dll\0", b"auroraops-bcprng.dll\0"),
    (b"api-ms-win-core-synch-l1-2-0.dll\0", b"auroraops-waitonaddress-shim.dll\0"),
    (b"GetSystemTimePreciseAsFileTime\0", b"GetSystemTimeAsFileTime\0"),
]

for old, new in replacements:
    if len(new) > len(old):
        raise SystemExit(f"replacement too long: {old!r} -> {new!r}")
    count = 0
    start = 0
    while True:
        pos = data.find(old, start)
        if pos < 0:
            break
        data[pos:pos + len(old)] = new + b"\0" * (len(old) - len(new))
        count += 1
        start = pos + len(old)
    if count == 0:
        raise SystemExit(f"pattern not found: {old!r}")
    print(old.decode(errors="ignore").rstrip("\0"), count)

dst.write_bytes(data)
pe = pefile.PE(str(dst), fast_load=False)
pe.OPTIONAL_HEADER.CheckSum = pe.generate_checksum()
pe.write(str(dst))
print(dst)
PY
```

Verify the patched import table before upload:

```bash
rtk bash -lc 'x86_64-w64-mingw32-objdump -p /tmp/auroraops-agent-win7-patched.exe | grep -i "DLL Name\|WaitOnAddress\|GetSystemTimePrecise\|GetSystemTimeAsFileTime\|bcryptprimitives\|auroraops-bcprng\|auroraops-waitonaddress"'
rtk bash -lc 'sha256sum /tmp/auroraops-agent-win7-patched.exe new-client/target/x86_64-pc-windows-gnu/release/auroraops-bcprng.dll new-client/target/x86_64-pc-windows-gnu/release/auroraops-waitonaddress-shim.dll'
```

The patched import table must show `auroraops-bcprng.dll`, `auroraops-waitonaddress-shim.dll`, `WaitOnAddress`, and two `GetSystemTimeAsFileTime` entries. It must not show `bcryptprimitives.dll`, `api-ms-win-core-synch-l1-2-0.dll`, or `GetSystemTimePreciseAsFileTime`.

Upload the patched exe and test it before replacing the service binary:

```bash
rtk sshpass -p '<password>' scp -F /dev/null -o StrictHostKeyChecking=no -o UserKnownHostsFile=/tmp/auroraops_known_hosts -P 2222 /tmp/auroraops-agent-win7-patched.exe Administrator@192.168.1.166:C:/AuroraOps/auroraops-agent-new.exe
rtk sshpass -p '<password>' ssh -F /dev/null -o StrictHostKeyChecking=no -o UserKnownHostsFile=/tmp/auroraops_known_hosts -p 2222 Administrator@192.168.1.166 "C:\Windows\System32\cmd.exe /c C:\AuroraOps\auroraops-agent-new.exe --version"
```

Only after `--version` works on Win7 should the service be stopped, the old `C:\AuroraOps\auroraops-agent.exe` backed up, the patched exe copied into place, and `auroraops-agent` restarted.

## Need Confirmation

- Valid sudo password for `fit@192.168.2.52` and `zks@192.168.1.141`.
- Valid SSH passwords for `Administrator@192.168.2.36` and `gg@192.168.2.101`.
