#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
AGENT_DIR="${ROOT_DIR}/new-client"
AGENT_BIN="${AGENT_BIN:-${AGENT_DIR}/target/debug/auroraops-agent}"
AGENT_PORT="${AGENT_PORT:-18766}"
WEYLUS_PORT="${WEYLUS_PORT:-1702}"
CONFIG_FILE="${CONFIG_FILE:-$(mktemp /tmp/auroraops-agent.XXXXXX.json)}"
LOG_FILE="${LOG_FILE:-$(mktemp /tmp/auroraops-agent.XXXXXX.log)}"

cleanup() {
  if [ -n "${AGENT_PID:-}" ]; then
    kill "${AGENT_PID}" >/dev/null 2>&1 || true
    wait "${AGENT_PID}" >/dev/null 2>&1 || true
  fi
  rm -f "${CONFIG_FILE}"
}
trap cleanup EXIT

if [ ! -x "${AGENT_BIN}" ]; then
  (cd "${AGENT_DIR}" && cargo build --features ffmpeg-system --bin auroraops-agent)
fi

cat > "${CONFIG_FILE}" <<JSON
{
  "serverHost": "127.0.0.1:8000",
  "deviceName": "local-smoke",
  "httpBase": "http://127.0.0.1:8000",
  "bindAddress": "127.0.0.1",
  "webPort": ${WEYLUS_PORT}
}
JSON

"${AGENT_BIN}" --service --config "${CONFIG_FILE}" --port "${AGENT_PORT}" >"${LOG_FILE}" 2>&1 &
AGENT_PID="$!"

READY=0
for _ in $(seq 1 80); do
  if curl -fsS "http://127.0.0.1:${AGENT_PORT}/api/status" >/tmp/auroraops-agent-status.json; then
    READY=1
    break
  fi
  if ! kill -0 "${AGENT_PID}" >/dev/null 2>&1; then
    echo "Agent exited early. Log:" >&2
    sed -n '1,160p' "${LOG_FILE}" >&2
    exit 1
  fi
  sleep 0.25
done

if [ "${READY}" != "1" ]; then
  echo "Agent local HTTP did not become ready. Log:" >&2
  sed -n '1,160p' "${LOG_FILE}" >&2
  exit 1
fi

curl -fsS "http://127.0.0.1:${AGENT_PORT}/" >/dev/null
curl -fsS "http://127.0.0.1:${AGENT_PORT}/api/assets/preview" >/dev/null

WEYLUS_READY=0
for _ in $(seq 1 80); do
  if python3 - "${WEYLUS_PORT}" <<'PY'
import base64
import os
import socket
import sys

port = int(sys.argv[1])
key = base64.b64encode(os.urandom(16)).decode("ascii")
request = (
    "GET /ws HTTP/1.1\r\n"
    f"Host: 127.0.0.1:{port}\r\n"
    "Upgrade: websocket\r\n"
    "Connection: Upgrade\r\n"
    f"Sec-WebSocket-Key: {key}\r\n"
    "Sec-WebSocket-Version: 13\r\n"
    "\r\n"
)
with socket.create_connection(("127.0.0.1", port), timeout=1.0) as sock:
    sock.sendall(request.encode("ascii"))
    response = sock.recv(1024).decode("latin1", "replace")
if " 101 " not in response.split("\r\n", 1)[0]:
    raise SystemExit(response.split("\r\n", 1)[0])
PY
  then
    WEYLUS_READY=1
    break
  fi
  sleep 0.25
done

if [ "${WEYLUS_READY}" != "1" ]; then
  echo "Weylus websocket did not become ready. Log:" >&2
  sed -n '1,220p' "${LOG_FILE}" >&2
  exit 1
fi

echo "Agent local HTTP smoke passed on http://127.0.0.1:${AGENT_PORT}/"
echo "Weylus websocket smoke passed on ws://127.0.0.1:${WEYLUS_PORT}/ws"
echo "Status:"
cat /tmp/auroraops-agent-status.json
