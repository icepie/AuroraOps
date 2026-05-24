#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WEB_HOST="${WEB_HOST:-127.0.0.1}"
WEB_PORT="${WEB_PORT:-5180}"
LOG_FILE="${LOG_FILE:-$(mktemp /tmp/auroraops-web.XXXXXX.log)}"

cleanup() {
  if [ -n "${WEB_PID:-}" ]; then
    kill "${WEB_PID}" >/dev/null 2>&1 || true
    wait "${WEB_PID}" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

(cd "${ROOT_DIR}/web" && pnpm exec vite --host "${WEB_HOST}" --port "${WEB_PORT}" --strictPort >"${LOG_FILE}" 2>&1) &
WEB_PID="$!"

WEB_READY=0
for _ in $(seq 1 80); do
  if curl -fsS "http://${WEB_HOST}:${WEB_PORT}/" >/tmp/auroraops-web.html; then
    WEB_READY=1
    break
  fi
  if ! kill -0 "${WEB_PID}" >/dev/null 2>&1; then
    echo "Web dev server exited early. Log:" >&2
    sed -n '1,180p' "${LOG_FILE}" >&2
    exit 1
  fi
  sleep 0.25
done

if [ "${WEB_READY}" != "1" ]; then
  echo "Web dev server did not become ready. Log:" >&2
  sed -n '1,180p' "${LOG_FILE}" >&2
  exit 1
fi

echo "Web dev smoke passed at http://${WEB_HOST}:${WEB_PORT}/"
