#!/usr/bin/env bash
set -euo pipefail

detect_arch_dir() {
  case "$(uname -m)" in
    x86_64|amd64)
      echo "x64"
      ;;
    aarch64|arm64)
      echo "arm64"
      ;;
    *)
      echo "$(uname -m)"
      ;;
  esac
}

ARCH_DIR="$(detect_arch_dir)"
BIN_SOURCE="${1:-./bin/linux/${ARCH_DIR}/auroraops-agent}"
BIN_TARGET="/opt/auroraops/auroraops-agent"
CONFIG_TARGET="/etc/auroraops/agent-config.json"
SERVICE_TARGET="/etc/systemd/system/auroraops-agent.service"

install -d /opt/auroraops
install -d /etc/auroraops
install -m 0755 "${BIN_SOURCE}" "${BIN_TARGET}"

if [[ ! -f "${CONFIG_TARGET}" ]]; then
  cat > "${CONFIG_TARGET}" <<'JSON'
{
  "serverHost": "127.0.0.1:8000",
  "deviceName": "linux-node-01",
  "httpBase": "http://127.0.0.1:8000"
}
JSON
fi

install -m 0644 ./auroraops-agent.service "${SERVICE_TARGET}"
systemctl daemon-reload
systemctl enable --now auroraops-agent.service

echo "AuroraOps agent installed."
echo "UI: http://127.0.0.1:18765/"
