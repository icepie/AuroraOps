#!/usr/bin/env bash
set -euo pipefail

SERVICE_TARGET="/etc/systemd/system/auroraops-agent.service"
BIN_TARGET="/opt/auroraops/auroraops-agent"

if command -v systemctl >/dev/null 2>&1; then
  systemctl disable --now auroraops-agent.service || true
fi

rm -f "$SERVICE_TARGET"
rm -f "$BIN_TARGET"

if command -v systemctl >/dev/null 2>&1; then
  systemctl daemon-reload || true
fi

exit 0
