#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LOG_FILE="${LOG_FILE:-$(mktemp /tmp/auroraops-server.XXXXXX.log)}"
BIN_PATH="${BIN_PATH:-$(mktemp /tmp/auroraops-server-bin.XXXXXX)}"
HTTP_URL="${HTTP_URL:-http://127.0.0.1:8000/api.json}"
TCP_HOST="${TCP_HOST:-127.0.0.1}"
TCP_PORT="${TCP_PORT:-8099}"

cleanup() {
  if [ -n "${SERVER_PID:-}" ]; then
    kill "${SERVER_PID}" >/dev/null 2>&1 || true
    wait "${SERVER_PID}" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

(cd "${ROOT_DIR}/server" && go build -o "${BIN_PATH}" main.go)
(cd "${ROOT_DIR}/server" && "${BIN_PATH}" http >"${LOG_FILE}" 2>&1) &
SERVER_PID="$!"

HTTP_READY=0
for _ in $(seq 1 80); do
  if curl -fsS "${HTTP_URL}" >/tmp/auroraops-server-api.json; then
    HTTP_READY=1
    break
  fi
  if ! kill -0 "${SERVER_PID}" >/dev/null 2>&1; then
    echo "Server exited early. Log:" >&2
    sed -n '1,220p' "${LOG_FILE}" >&2
    exit 1
  fi
  sleep 0.25
done

if [ "${HTTP_READY}" != "1" ]; then
  echo "Server HTTP did not become ready. Log:" >&2
  sed -n '1,220p' "${LOG_FILE}" >&2
  exit 1
fi

python3 - "${TCP_HOST}" "${TCP_PORT}" <<'PY'
import socket
import sys

host = sys.argv[1]
port = int(sys.argv[2])
with socket.create_connection((host, port), timeout=2.0):
    pass
PY

echo "Server HTTP smoke passed at ${HTTP_URL}"
echo "Server TCP smoke passed at ${TCP_HOST}:${TCP_PORT}"
