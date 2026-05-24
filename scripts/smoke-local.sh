#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SERVER_LOG="${SERVER_LOG:-$(mktemp /tmp/auroraops-server.XXXXXX.log)}"
SERVER_BIN="${SERVER_BIN:-$(mktemp /tmp/auroraops-server-bin.XXXXXX)}"
SERVER_PID=""

cleanup() {
  if [ -n "${SERVER_PID}" ]; then
    kill "${SERVER_PID}" >/dev/null 2>&1 || true
    wait "${SERVER_PID}" >/dev/null 2>&1 || true
  fi
  rm -f "${SERVER_BIN}"
}
trap cleanup EXIT

(cd "${ROOT_DIR}/server" && go build -o "${SERVER_BIN}" main.go)
(cd "${ROOT_DIR}/server" && "${SERVER_BIN}" http >"${SERVER_LOG}" 2>&1) &
SERVER_PID="$!"

SERVER_READY=0
for _ in $(seq 1 80); do
  if curl -fsS "http://127.0.0.1:8000/api.json" >/tmp/auroraops-server-api.json; then
    SERVER_READY=1
    break
  fi
  if ! kill -0 "${SERVER_PID}" >/dev/null 2>&1; then
    echo "Server exited early. Log:" >&2
    sed -n '1,220p' "${SERVER_LOG}" >&2
    exit 1
  fi
  sleep 0.25
done

if [ "${SERVER_READY}" != "1" ]; then
  echo "Server HTTP did not become ready. Log:" >&2
  sed -n '1,220p' "${SERVER_LOG}" >&2
  exit 1
fi

python3 - <<'PY'
import socket
with socket.create_connection(("127.0.0.1", 8099), timeout=2.0):
    pass
PY
echo "Server HTTP smoke passed at http://127.0.0.1:8000/api.json"
echo "Server TCP smoke passed at 127.0.0.1:8099"

"${ROOT_DIR}/scripts/smoke-local-web.sh"
"${ROOT_DIR}/scripts/smoke-local-agent.sh"

echo "All local smoke checks passed."
