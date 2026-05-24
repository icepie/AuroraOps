#!/usr/bin/env bash
set -e

if [ "$1" = "0" ]; then
  if command -v systemctl >/dev/null 2>&1; then
    systemctl stop auroraops-agent.service || true
    systemctl disable auroraops-agent.service || true
  fi
fi

exit 0
