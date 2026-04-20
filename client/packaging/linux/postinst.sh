#!/usr/bin/env bash
set -euo pipefail

map_arch() {
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

ARCH_DIR="$(map_arch)"
BIN_SOURCE=""
for candidate in \
  "/opt/AuroraOps Client/resources/go-agent/linux/$ARCH_DIR/auroraops-agent" \
  "/opt/auroraops-client/resources/go-agent/linux/$ARCH_DIR/auroraops-agent" \
  "/opt/auroraops/resources/go-agent/linux/$ARCH_DIR/auroraops-agent" \
  "$(find /opt -type f -path "*/resources/go-agent/linux/$ARCH_DIR/auroraops-agent" | head -n 1 || true)" \
  "$(find /opt -type f -path '*/resources/go-agent/auroraops-agent' | head -n 1 || true)"; do
  if [[ -n "$candidate" && -f "$candidate" ]]; then
    BIN_SOURCE="$candidate"
    break
  fi
done

if [[ -z "$BIN_SOURCE" ]]; then
  echo "auroraops-agent resource not found for architecture $ARCH_DIR" >&2
  exit 1
fi

RESOURCE_DIR="$(dirname "$BIN_SOURCE")"
SERVICE_SOURCE="$RESOURCE_DIR/auroraops-agent.service"
BIN_TARGET="/opt/auroraops/auroraops-agent"
CONFIG_TARGET="/etc/auroraops/agent-config.json"
SERVICE_TARGET="/etc/systemd/system/auroraops-agent.service"

install -d /opt/auroraops
install -d /etc/auroraops
install -m 0755 "$BIN_SOURCE" "$BIN_TARGET"
install -m 0644 "$SERVICE_SOURCE" "$SERVICE_TARGET"

if [[ ! -f "$CONFIG_TARGET" ]]; then
  cat > "$CONFIG_TARGET" <<'JSON'
{
  "serverHost": "127.0.0.1:8000",
  "deviceName": "linux-node-01",
  "httpBase": "http://127.0.0.1:8000"
}
JSON
  chmod 600 "$CONFIG_TARGET"
fi

if command -v systemctl >/dev/null 2>&1; then
  systemctl daemon-reload || true
  systemctl enable auroraops-agent.service || true
  systemctl restart auroraops-agent.service || systemctl start auroraops-agent.service || true
fi

exit 0
