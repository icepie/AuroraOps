#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RUN_WEB="${RUN_WEB:-1}"
RUN_SERVER="${RUN_SERVER:-1}"
RUN_NEW_CLIENT="${RUN_NEW_CLIENT:-1}"

export GOCACHE="${GOCACHE:-${ROOT_DIR}/.gocache}"
export GOMODCACHE="${GOMODCACHE:-${ROOT_DIR}/.gocache/pkg/mod}"

run() {
  local name="$1"
  shift
  echo
  echo "==> ${name}"
  (cd "$1" && shift && "$@")
}

if [ "${RUN_SERVER}" = "1" ]; then
  run "server desktop tests" "${ROOT_DIR}/server" go test ./internal/logic/tcpserver -run 'TestDesktop'
  run "server compile checks" "${ROOT_DIR}/server" go test ./internal/controller/admin/sys ./internal/logic/sys ./internal/service ./internal/library/network/tcp -run '^$'
fi

if [ "${RUN_NEW_CLIENT}" = "1" ]; then
  run "new-client cargo check" "${ROOT_DIR}/new-client" cargo check --features ffmpeg-system --bin auroraops-agent
fi

if [ "${RUN_WEB}" = "1" ]; then
  run "web build" "${ROOT_DIR}/web" pnpm run build
fi

echo
echo "All enabled checks passed."
