#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  scripts/deploy-windows-agent.sh --host HOST --user USER --password PASS [options]

Options:
  --exe PATH            Local agent exe path.
                        Default: new-client/target/x86_64-pc-windows-gnu/release/auroraops-agent.exe
  --target PATH         Remote installed exe path.
                        Default: C:\Program Files\AuroraOps\auroraops-agent.exe
  --service NAME        Windows service name. Default: auroraops-agent
  --port PORT           Local management port. Default: 18765
  --no-capture-test     Skip POST /api/capture-test verification.

Example:
  scripts/deploy-windows-agent.sh --host 192.168.2.36 --user Administrator --password 1234
EOF
}

host=""
user=""
password=""
exe="new-client/target/x86_64-pc-windows-gnu/release/auroraops-agent.exe"
target='C:\Program Files\AuroraOps\auroraops-agent.exe'
service="auroraops-agent"
port="18765"
capture_test="1"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --host)
      host="${2:-}"
      shift 2
      ;;
    --user)
      user="${2:-}"
      shift 2
      ;;
    --password)
      password="${2:-}"
      shift 2
      ;;
    --exe)
      exe="${2:-}"
      shift 2
      ;;
    --target)
      target="${2:-}"
      shift 2
      ;;
    --service)
      service="${2:-}"
      shift 2
      ;;
    --port)
      port="${2:-}"
      shift 2
      ;;
    --no-capture-test)
      capture_test="0"
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

if [[ -z "$host" || -z "$user" || -z "$password" ]]; then
  usage >&2
  exit 2
fi

if [[ ! -f "$exe" ]]; then
  echo "Agent exe not found: $exe" >&2
  exit 1
fi

ssh_opts=(
  -o IdentitiesOnly=yes
  -o PreferredAuthentications=password,keyboard-interactive
  -o PubkeyAuthentication=no
  -o StrictHostKeyChecking=no
)

remote_tmp="C:/Users/${user}/auroraops-agent-deploy.exe"
local_hash="$(sha256sum "$exe" | awk '{print $1}')"

echo "host=$host"
echo "user=$user"
echo "service=$service"
echo "target=$target"
echo "local_hash=$local_hash"

sshpass -p "$password" scp "${ssh_opts[@]}" "$exe" "${user}@${host}:${remote_tmp}"

powershell=$(cat <<EOF
\$ErrorActionPreference = "Stop"
\$ProgressPreference = "SilentlyContinue"
\$WarningPreference = "SilentlyContinue"
\$InformationPreference = "SilentlyContinue"
function Log([string]\$Message) {
  [Console]::Out.WriteLine(\$Message)
}
\$service = "$service"
\$target = "$target"
\$remoteTmp = "C:\\Users\\$user\\auroraops-agent-deploy.exe"
\$backup = "\$target.bak-winlogon-\$(Get-Date -Format yyyyMMddHHmmss)"
Log "stopping_service=\$service"
Stop-Service \$service -WarningAction SilentlyContinue
Start-Sleep -Seconds 2
Log "backup=\$backup"
Copy-Item \$target \$backup -Force
Copy-Item \$remoteTmp \$target -Force
Log "starting_service=\$service"
Start-Service \$service
Start-Sleep -Seconds 5
Get-Service \$service | Format-List Name,Status,StartType
Get-FileHash \$target -Algorithm SHA256 | Format-List
Get-Process auroraops-agent -ErrorAction SilentlyContinue |
  Select-Object Id,SessionId,StartTime,Path |
  Sort-Object SessionId |
  Format-Table -AutoSize
Log "api_status_begin"
Invoke-RestMethod http://127.0.0.1:$port/api/status | ConvertTo-Json -Depth 8
Log "api_status_end"
if ("$capture_test" -eq "1") {
  Log "capture_test_begin"
  Invoke-RestMethod -Method Post http://127.0.0.1:$port/api/capture-test | ConvertTo-Json -Depth 8
  Log "capture_test_end"
}
Log "recent_events_begin"
Get-WinEvent -FilterHashtable @{LogName="Application"; StartTime=(Get-Date).AddMinutes(-30)} -ErrorAction SilentlyContinue |
  Where-Object { \$_.ProviderName -match "Application Error|Windows Error Reporting|auroraops|Rust" -or \$_.Message -match "auroraops|panic|faulting" } |
  Select-Object TimeCreated,ProviderName,Id,LevelDisplayName,Message -First 10 |
  ConvertTo-Json -Depth 4
Log "recent_events_end"
EOF
)

encoded_powershell="$(
  printf '%s' "$powershell" |
    iconv -f UTF-8 -t UTF-16LE |
    base64 -w 0
)"

sshpass -p "$password" ssh "${ssh_opts[@]}" "${user}@${host}" \
  "powershell.exe -NoProfile -ExecutionPolicy Bypass -EncodedCommand $encoded_powershell"
