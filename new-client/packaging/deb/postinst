#!/usr/bin/env bash
set -e

if command -v systemctl >/dev/null 2>&1; then
  /opt/auroraops/auroraops-uinput-setup || true
  systemctl daemon-reload || true
  systemctl enable auroraops-agent.service || true
  systemctl restart auroraops-agent.service || systemctl start auroraops-agent.service || true
fi

exit 0
